/**
 * Amulet AI — Application entry point.
 * Initializes store, router, API, WebSocket, and global event listeners.
 */

import { store } from './state.js';
import { api } from './api.js';
import { ws } from './ws.js';
import { router } from './router.js';
import { initTheme } from './styles/theme.js';
import { ready } from './lib/dom.js';
import { get } from './lib/storage.js';

/**
 * Initialize the application.
 */
async function init() {
  // Initialize theme
  initTheme();

  // Check for existing session
  const token = api.getToken();
  const user = store.get('user');

  if (token && user) {
    // Restore session
    store.set('user', user);

    // Connect WebSocket
    ws.connect();

    // Register WebSocket handlers
    setupWSHandlers();
  }

  // Set up navigation guard
  router.beforeEach((to) => {
    // Allow access to settings without auth for login purposes
    if (to === '/settings') return true;

    // Redirect to login if not authenticated
    if (!api.getToken() && to !== '/map') {
      return '/map';
    }

    return true;
  });

  // Initialize router
  router.init();

  // Set up global event listeners
  setupGlobalListeners();

  // Set up Tauri integration
  setupTauri();

  // Hide loading screen
  const loadingScreen = document.getElementById('loading-screen');
  if (loadingScreen) {
    loadingScreen.style.display = 'none';
  }

  console.log('[App] Amulet AI initialized');
}

/**
 * Set up WebSocket message handlers.
 */
function setupWSHandlers() {
  ws.on('member_location', (data) => {
    const members = store.get('members') || [];
    const idx = members.findIndex(m => m.id === data.userId);
    if (idx !== -1) {
      members[idx] = { ...members[idx], lat: data.lat, lng: data.lng, lastSeen: Date.now(), online: true };
      store.set('members', members);
    }
  });

  ws.on('incident', (data) => {
    store.set('activeIncident', data);
    store.set('incidents', [...(store.get('incidents') || []), data]);
  });

  ws.on('chat_message', (data) => {
    window.dispatchEvent(new CustomEvent('chat:message', { detail: data }));
  });

  ws.on('typing', (data) => {
    window.dispatchEvent(new CustomEvent('chat:typing', { detail: data }));
  });
}

/**
 * Set up global event listeners.
 */
function setupGlobalListeners() {
  // Handle auth expiration
  window.addEventListener('auth:expired', () => {
    api.clearTokens();
    store.set('user', null);
    store.set('circle', null);
    window.location.hash = '#/map';
  });

  // Handle online/offline
  window.addEventListener('online', () => {
    console.log('[App] Back online');
    ws.connect();
  });

  window.addEventListener('offline', () => {
    console.log('[App] Gone offline');
    ws.disconnect();
  });

  // Handle visibility change (reconnect WS when tab becomes visible)
  document.addEventListener('visibilitychange', () => {
    if (document.visibilityState === 'visible' && api.getToken()) {
      ws.connect();
    }
  });

  // Global error handler
  window.addEventListener('error', (event) => {
    console.error('[App] Unhandled error:', event.error);
  });

  window.addEventListener('unhandledrejection', (event) => {
    console.error('[App] Unhandled promise rejection:', event.reason);
  });
}

/**
 * Set up Tauri native integration.
 */
function setupTauri() {
  if (window.__TAURI__) {
    console.log('[App] Tauri environment detected');

    // Expose Tauri invoke for SOS and other native features
    window.__TAURI__.invoke('get_app_info').then(info => {
      console.log('[App] App info:', info);
    }).catch(err => {
      console.warn('[App] Failed to get app info:', err);
    });
  }
}

// Start the app when DOM is ready
ready(init);
