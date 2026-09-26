/**
 * Lightweight pub/sub Store for application state management.
 */

import { get, set } from './lib/storage.js';

export class Store {
  constructor(initialState = {}) {
    this.state = { ...initialState };
    this.listeners = new Map();
  }

  /**
   * Get a value from state.
   * @param {string} key
   * @returns {*}
   */
  get(key) {
    return this.state[key];
  }

  /**
   * Set a value in state and notify listeners.
   * @param {string} key
   * @param {*} value
   */
  set(key, value) {
    const prev = this.state[key];
    this.state[key] = value;
    this._notify(key, value, prev);
  }

  /**
   * Merge an object into state.
   * @param {Object} partial
   */
  merge(partial) {
    Object.entries(partial).forEach(([key, value]) => {
      this.set(key, value);
    });
  }

  /**
   * Subscribe to state changes.
   * @param {string} key - State key to watch
   * @param {Function} callback - (newValue, oldValue) => void
   * @returns {Function} Unsubscribe function
   */
  on(key, callback) {
    if (!this.listeners.has(key)) {
      this.listeners.set(key, new Set());
    }
    this.listeners.get(key).add(callback);
    return () => this.off(key, callback);
  }

  /**
   * Unsubscribe from state changes.
   * @param {string} key
   * @param {Function} callback
   */
  off(key, callback) {
    if (this.listeners.has(key)) {
      this.listeners.get(key).delete(callback);
    }
  }

  /**
   * Notify listeners of a state change.
   * @private
   */
  _notify(key, newValue, oldValue) {
    if (this.listeners.has(key)) {
      for (const cb of this.listeners.get(key)) {
        try {
          cb(newValue, oldValue);
        } catch (err) {
          console.error(`[Store] Listener error for "${key}":`, err);
        }
      }
    }
    // Also notify wildcard listeners
    if (this.listeners.has('*')) {
      for (const cb of this.listeners.get('*')) {
        try {
          cb(key, newValue, oldValue);
        } catch (err) {
          console.error('[Store] Wildcard listener error:', err);
        }
      }
    }
  }
}

/**
 * Create the global store with initial state.
 */
export function createStore() {
  return new Store({
    user: get('user', null),
    circle: get('circle', null),
    members: [],
    places: [],
    incidents: [],
    medications: [],
    activeIncident: null,
    wsConnected: false,
    isTracking: false,
  });
}

/** Global store instance */
export const store = createStore();
export default store;
