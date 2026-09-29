/**
 * Amulet AI — Application entry point.
 * Initializes store, router, API, WebSocket, and global event listeners.
 */

import { store } from './state.js';
import { api } from './api.js';
import { ws } from './ws.js';
import { router } from './router.js';
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
  }

  // Set up navigation guard
  router.beforeEach((to) => {
    // Allow public routes without auth
    if (to === '/login' || to === '/signup' || to === '/onboarding') return true;

    // Redirect to login if not authenticated
    if (!api.getToken()) {
      return '/login';
    }

    // Redirect authenticated users away from auth pages
    if ((to === '/login' || to === '/signup') && api.getToken()) {
      return '/map';
    }

    // Redirect to onboarding if not completed
    const onboardingCompleted = localStorage.getItem('amulet:onboarding_completed');
    if (!onboardingCompleted && to !== '/onboarding') {
      return '/onboarding';
    }

    return true;
  });

  // Initialize router
  router.init();

  // Set up global event listeners
  setupGlobalListeners();

  // Set up WebSocket message handlers
  setupWebSocketHandlers();

  // Start medication reminders
  setupMedicationReminders();

  console.log('[App] Amulet AI initialized');
}

/**
 * Set up WebSocket message handlers for realtime features.
 */
function setupWebSocketHandlers() {
  // Chat messages
  ws.on('chat_message', (data) => {
    window.dispatchEvent(new CustomEvent('ws:chat_message', { detail: data }));
  });

  // Fall detection alerts
  ws.on('fall_detected', (data) => {
    window.dispatchEvent(new CustomEvent('ws:fall_detected', { detail: data }));
  });

  // Crash detection alerts
  ws.on('crash_detected', (data) => {
    window.dispatchEvent(new CustomEvent('ws:crash_detected', { detail: data }));
  });

  // Geofence events
  ws.on('geofence_event', (data) => {
    window.dispatchEvent(new CustomEvent('ws:geofence_event', { detail: data }));
  });

  // SOS alerts
  ws.on('alert', (data) => {
    window.dispatchEvent(new CustomEvent('ws:alert', { detail: data }));
  });
}

/**
 * Set up medication reminder service.
 */
function setupMedicationReminders() {
  // Start the reminder service
  if ('Notification' in window) {
    import('./services/med-reminders.js').then(({ medReminder }) => {
      medReminder.start();

      // Update medications when store changes
      store.on('medications', (meds) => {
        medReminder.updateMedications(meds);
      });
    });
  }
}

/**
 * Initialize theme from localStorage or system preference.
 */
function initTheme() {
  const savedTheme = get('theme', null);
  if (savedTheme) {
    document.documentElement.setAttribute('data-theme', savedTheme);
  } else if (window.matchMedia('(prefers-color-scheme: dark)').matches) {
    document.documentElement.setAttribute('data-theme', 'dark');
  }
}

/**
 * Set up global event listeners.
 */
function setupGlobalListeners() {
  // Handle auth expiration
  window.addEventListener('auth:expired', () => {
    api.clearTokens();
    store.set('user', null);
    router.navigate('/login');
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
}

// Start the app when DOM is ready
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', init);
} else {
  init();
}
