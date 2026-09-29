/**
 * Realtime WebSocket client with auto-reconnection and exponential backoff.
 */

import { store } from './state.js';
import { get } from './lib/storage.js';

export class RealtimeClient {
  constructor(url) {
    this.url = url || window.__WS_URL || import.meta.env.VITE_WS_URL || 'ws://localhost:3000/ws';
    this.ws = null;
    this.handlers = new Map();
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 10;
    this.baseReconnectDelay = 1000;
    this.maxReconnectDelay = 30000;
    this.reconnectTimer = null;
    this.pingInterval = null;
    this.pingFrequency = 30000; // 30 seconds
    this.isIntentionallyClosed = false;
    this.isConnected = false;
  }

  /**
   * Connect to the WebSocket server.
   */
  connect() {
    if (this.ws && (this.ws.readyState === WebSocket.OPEN || this.ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    this.isIntentionallyClosed = false;

    try {
      this.ws = new WebSocket(this.url);

      this.ws.addEventListener('open', () => this._onOpen());
      this.ws.addEventListener('message', (e) => this._onMessage(e));
      this.ws.addEventListener('close', (e) => this._onClose(e));
      this.ws.addEventListener('error', (e) => this._onError(e));
    } catch (err) {
      console.error('[WS] Connection error:', err);
      this._scheduleReconnect();
    }
  }

  /**
   * Disconnect from the WebSocket server.
   */
  disconnect() {
    this.isIntentionallyClosed = true;
    this._clearReconnectTimer();
    this._stopPing();
    if (this.ws) {
      this.ws.close(1000, 'Client disconnect');
      this.ws = null;
    }
    this.isConnected = false;
    store.set('wsConnected', false);
  }

  /**
   * Send a message to the server.
   * @param {string} type - Message type
   * @param {Object} payload - Message payload
   */
  send(type, payload = {}) {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('[WS] Cannot send — not connected');
      return false;
    }

    try {
      this.ws.send(JSON.stringify({ type, ...payload }));
      return true;
    } catch (err) {
      console.error('[WS] Send error:', err);
      return false;
    }
  }

  /**
   * Register a message handler.
   * @param {string} type - Message type
   * @param {Function} handler - (data) => void
   * @returns {Function} Unsubscribe function
   */
  on(type, handler) {
    if (!this.handlers.has(type)) {
      this.handlers.set(type, new Set());
    }
    this.handlers.get(type).add(handler);
    return () => this.off(type, handler);
  }

  /**
   * Remove a message handler.
   * @param {string} type
   * @param {Function} handler
   */
  off(type, handler) {
    if (this.handlers.has(type)) {
      this.handlers.get(type).delete(handler);
    }
  }

  /**
   * Handle connection open.
   * @private
   */
  _onOpen() {
    console.log('[WS] Connected');
    this.isConnected = true;
    this.reconnectAttempts = 0;
    store.set('wsConnected', true);

    // Authenticate
    const token = get('auth_token', null);
    if (token) {
      this.send('auth', { token });
    }

    this._startPing();
  }

  /**
   * Handle incoming message.
   * @private
   */
  _onMessage(event) {
    try {
      const data = JSON.parse(event.data);

      // Handle pong
      if (data.type === 'pong') return;

      // Dispatch to registered handlers
      if (this.handlers.has(data.type)) {
        for (const handler of this.handlers.get(data.type)) {
          try {
            handler(data);
          } catch (err) {
            console.error(`[WS] Handler error for "${data.type}":`, err);
          }
        }
      }

      // Wildcard handlers
      if (this.handlers.has('*')) {
        for (const handler of this.handlers.get('*')) {
          try {
            handler(data);
          } catch (err) {
            console.error('[WS] Wildcard handler error:', err);
          }
        }
      }

      // Emit as window event for non-WS-aware components
      window.dispatchEvent(new CustomEvent(`ws:${data.type}`, { detail: data }));
    } catch (err) {
      console.error('[WS] Message parse error:', err);
    }
  }

  /**
   * Handle connection close.
   * @private
   */
  _onClose(event) {
    console.log(`[WS] Closed: ${event.code} ${event.reason}`);
    this.isConnected = false;
    this._stopPing();
    store.set('wsConnected', false);

    if (!this.isIntentionallyClosed && event.code !== 1000) {
      this._scheduleReconnect();
    }
  }

  /**
   * Handle connection error.
   * @private
   */
  _onError(event) {
    console.error('[WS] Error:', event);
  }

  /**
   * Schedule a reconnection attempt with exponential backoff.
   * @private
   */
  _scheduleReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('[WS] Max reconnection attempts reached');
      return;
    }

    const delay = Math.min(
      this.baseReconnectDelay * Math.pow(2, this.reconnectAttempts),
      this.maxReconnectDelay
    );

    this.reconnectAttempts++;
    console.log(`[WS] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`);

    this._clearReconnectTimer();
    this.reconnectTimer = setTimeout(() => this.connect(), delay);
  }

  /**
   * Clear the reconnection timer.
   * @private
   */
  _clearReconnectTimer() {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
  }

  /**
   * Start ping keepalive.
   * @private
   */
  _startPing() {
    this._stopPing();
    this.pingInterval = setInterval(() => {
      if (this.isConnected) {
        this.send('ping', { timestamp: Date.now() });
      }
    }, this.pingFrequency);
  }

  /**
   * Stop ping keepalive.
   * @private
   */
  _stopPing() {
    if (this.pingInterval) {
      clearInterval(this.pingInterval);
      this.pingInterval = null;
    }
  }
}

/** Global WebSocket client instance */
export const ws = new RealtimeClient();
export default ws;
