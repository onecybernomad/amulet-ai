/**
 * ChatWindow — Message list with bubbles, input, typing indicator, and read receipts.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { formatTime } from '../lib/format.js';
import { store } from '../state.js';

export class ChatWindow {
  constructor(container, roomId) {
    this.container = container;
    this.roomId = roomId;
    this.messages = [];
    this.currentUser = store.get('user');
    this.typingUsers = new Set();
    this._render();
  }

  /**
   * Render the chat window.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'chat-main';

    // Messages area
    this.messagesEl = createElement('div', { class: 'chat-messages', 'data-room': this.roomId });

    // Typing indicator
    this.typingEl = createElement('div', { class: 'typing-indicator', style: { display: 'none' } });
    this.typingEl.innerHTML = '<div class="typing-dot"></div><div class="typing-dot"></div><div class="typing-dot"></div>';

    // Input area
    this.inputArea = createElement('div', { class: 'chat-input-area' });

    this.input = createElement('textarea', {
      class: 'chat-input',
      placeholder: 'Type a message…',
      rows: '1',
    });

    this.sendBtn = createElement('button', {
      class: 'chat-send-btn',
      'aria-label': 'Send message',
      disabled: 'disabled',
    }, '➤');

    this.input.addEventListener('input', () => {
      this.sendBtn.disabled = !this.input.value.trim();
      this.input.style.height = 'auto';
      this.input.style.height = Math.min(this.input.scrollHeight, 120) + 'px';
    });

    this.input.addEventListener('keydown', (e) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        this.sendMessage();
      }
    });

    this.sendBtn.addEventListener('click', () => this.sendMessage());

    this.inputArea.appendChild(this.input);
    this.inputArea.appendChild(this.sendBtn);

    this.container.appendChild(this.messagesEl);
    this.container.appendChild(this.typingEl);
    this.container.appendChild(this.inputArea);
  }

  /**
   * Add a message to the chat.
   * @param {Object} message - { id, senderId, senderName, text, timestamp, mediaUrl, readBy }
   */
  addMessage(message) {
    this.messages.push(message);
    const isOwn = message.senderId === this.currentUser?.id;

    const bubble = createElement('div', {
      class: `chat-bubble ${isOwn ? 'chat-bubble-own' : 'chat-bubble-other'}`,
    });

    if (message.mediaUrl) {
      const media = createElement('div', { class: 'chat-bubble-media' });
      media.appendChild(createElement('img', { src: message.mediaUrl, alt: 'Shared media' }));
      bubble.appendChild(media);
    }

    if (message.text) {
      bubble.appendChild(createElement('div', { class: 'chat-bubble-text' }, message.text));
    }

    const meta = createElement('div', { class: 'chat-bubble-meta' });
    meta.appendChild(createElement('span', {}, formatTime(message.timestamp)));
    if (isOwn && message.readBy?.length) {
      meta.appendChild(createElement('span', { style: { marginLeft: '4px' } }, '✓✓'));
    }
    bubble.appendChild(meta);

    this.messagesEl.appendChild(bubble);
    this._scrollToBottom();
  }

  /**
   * Show typing indicator for a user.
   * @param {string} userId
   * @param {string} userName
   */
  showTyping(userId, userName) {
    this.typingUsers.add(userId);
    this.typingEl.style.display = 'flex';
    this.typingEl.innerHTML = `<span style="font-size:12px;color:var(--color-text-muted);">${userName} is typing…</span>`;
  }

  /**
   * Hide typing indicator.
   * @param {string} userId
   */
  hideTyping(userId) {
    this.typingUsers.delete(userId);
    if (this.typingUsers.size === 0) {
      this.typingEl.style.display = 'none';
    }
  }

  /**
   * Send the current message.
   */
  sendMessage() {
    const text = this.input.value.trim();
    if (!text) return;

    const message = {
      id: crypto.randomUUID(),
      roomId: this.roomId,
      senderId: this.currentUser?.id,
      senderName: this.currentUser?.name,
      text,
      timestamp: Date.now(),
    };

    this.addMessage(message);
    this.input.value = '';
    this.input.style.height = 'auto';
    this.sendBtn.disabled = true;

    // Send via WebSocket if connected
    if (window.__WS__ && window.__WS__.readyState === WebSocket.OPEN) {
      window.__WS__.send('chat_message', {
        room_id: this.roomId,
        body: text,
      });
    } else {
      // Fallback: send via API
      api.post(`/api/rooms/${this.roomId}/messages`, { body: text }).catch(console.error);
    }
  }

  /**
   * Handle incoming realtime message from WebSocket.
   * @param {Object} data
   */
  handleRealtimeMessage(data) {
    if (data.room_id !== this.roomId) return;

    const message = {
      id: data.id || crypto.randomUUID(),
      senderId: data.sender_id,
      senderName: data.sender_name || 'Member',
      text: data.body,
      timestamp: data.created_at || Date.now(),
      readBy: data.read_by || [],
    };

    this.addMessage(message);
  }

  /**
   * Update read receipts for messages.
   * @param {string} userId
   * @param {string} messageId
   */
  updateReadReceipt(userId, messageId) {
    // Update UI to show read receipt
    const messages = this.messages.filter(m => m.id === messageId);
    for (const msg of messages) {
      if (!msg.readBy) msg.readBy = [];
      if (!msg.readBy.includes(userId)) {
        msg.readBy.push(userId);
      }
    }
  }

  /**
   * Scroll to the bottom of messages.
   * @private
   */
  _scrollToBottom() {
    this.messagesEl.scrollTop = this.messagesEl.scrollHeight;
  }

  /**
   * Clear all messages.
   */
  clear() {
    this.messages = [];
    clearElement(this.messagesEl);
  }

  /**
   * Destroy the component.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default ChatWindow;
