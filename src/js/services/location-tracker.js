/**
 * LocationTracker — GPS tracking with WebSocket sync and offline caching.
 */

import { store } from '../state.js';
import { api } from '../api.js';

const OFFLINE_CACHE_KEY = 'amulet:location_cache';
const MAX_CACHE_SIZE = 500;
const UPDATE_INTERVAL = 10000; // 10 seconds

export class LocationTracker {
  constructor() {
    this.watchId = null;
    this.isTracking = false;
    this.lastPosition = null;
    this.lastUpdateTime = 0;
    this.ws = null;
    this.trackingMode = 'balanced'; // 'active' | 'balanced' | 'passive'
    this.privacyMode = false; // When true, don't share location with circle
    this.updateIntervals = {
      active: 5000,    // 5 seconds — high accuracy, frequent updates
      balanced: 10000,  // 10 seconds — default
      passive: 30000,   // 30 seconds — battery saver
    };
  }

  /**
   * Start tracking the user's location.
   * @param {Object} ws - WebSocket instance
   * @param {string} mode - Tracking mode: 'active', 'balanced', or 'passive'
   */
  start(ws, mode = 'balanced') {
    if (this.isTracking) return;
    if (!navigator.geolocation) {
      console.error('[LocationTracker] Geolocation not supported');
      return;
    }

    this.ws = ws;
    this.trackingMode = mode;
    this.isTracking = true;

    const options = this._getGeolocationOptions(mode);

    this.watchId = navigator.geolocation.watchPosition(
      (position) => this._onPositionUpdate(position),
      (error) => this._onError(error),
      options
    );

    console.log(`[LocationTracker] Started tracking in ${mode} mode`);
  }

  /**
   * Get geolocation options based on tracking mode.
   * @param {string} mode
   * @returns {Object}
   * @private
   */
  _getGeolocationOptions(mode) {
    switch (mode) {
      case 'active':
        return { enableHighAccuracy: true, timeout: 5000, maximumAge: 0 };
      case 'passive':
        return { enableHighAccuracy: false, timeout: 30000, maximumAge: 15000 };
      case 'balanced':
      default:
        return { enableHighAccuracy: true, timeout: 10000, maximumAge: 5000 };
    }
  }

  /**
   * Set the tracking mode dynamically.
   * @param {string} mode - 'active', 'balanced', or 'passive'
   */
  setTrackingMode(mode) {
    this.trackingMode = mode;
    if (this.isTracking) {
      // Restart with new options
      this.stop();
      this.start(this.ws, mode);
    }
  }

  /**
   * Set privacy mode — when enabled, location is not shared with circle.
   * @param {boolean} enabled
   */
  setPrivacyMode(enabled) {
    this.privacyMode = enabled;
    console.log(`[LocationTracker] Privacy mode ${enabled ? 'enabled' : 'disabled'}`);
  }

  /**
   * Stop tracking the user's location.
   */
  stop() {
    if (this.watchId !== null) {
      navigator.geolocation.clearWatch(this.watchId);
      this.watchId = null;
    }
    this.isTracking = false;
    this.lastPosition = null;
    console.log('[LocationTracker] Stopped tracking');
  }

  /**
   * Get the current cached position.
   * @returns {Object|null}
   */
  getCurrentPosition() {
    return this.lastPosition;
  }

  /**
   * Handle position update.
   * @private
   */
  _onPositionUpdate(position) {
    const now = Date.now();
    const interval = this.updateIntervals[this.trackingMode] || UPDATE_INTERVAL;
    if (now - this.lastUpdateTime < interval) return;
    this.lastUpdateTime = now;

    const locationData = {
      latitude: position.coords.latitude,
      longitude: position.coords.longitude,
      accuracy: position.coords.accuracy,
      speed: position.coords.speed,
      heading: position.coords.heading,
      timestamp: new Date().toISOString(),
    };

    this.lastPosition = locationData;
    store.set('userLocation', locationData);

    // Don't share location if privacy mode is on
    if (this.privacyMode) {
      this._cacheLocation(locationData);
      return;
    }

    // Send via WebSocket if connected
    if (this.ws && this.ws.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify({
        type: 'location_ping',
        latitude: locationData.latitude,
        longitude: locationData.longitude,
        accuracy: locationData.accuracy,
        speed: locationData.speed,
        heading: locationData.heading,
        timestamp: locationData.timestamp,
      }));
    } else {
      // Cache for later sync
      this._cacheLocation(locationData);
    }
  }

  /**
   * Handle geolocation error.
   * @private
   */
  _onError(error) {
    console.error('[LocationTracker] Error:', error.message);
    const errors = {
      1: 'Permission denied',
      2: 'Position unavailable',
      3: 'Timeout',
    };
    store.set('locationError', errors[error.code] || 'Unknown error');
  }

  /**
   * Cache location for offline sync.
   * @private
   */
  _cacheLocation(locationData) {
    try {
      const cache = JSON.parse(localStorage.getItem(OFFLINE_CACHE_KEY) || '[]');
      cache.push(locationData);
      if (cache.length > MAX_CACHE_SIZE) {
        cache.shift();
      }
      localStorage.setItem(OFFLINE_CACHE_KEY, JSON.stringify(cache));
    } catch (e) {
      console.error('[LocationTracker] Failed to cache location:', e);
    }
  }

  /**
   * Sync cached locations when back online.
   */
  async syncCachedLocations() {
    try {
      const cache = JSON.parse(localStorage.getItem(OFFLINE_CACHE_KEY) || '[]');
      if (cache.length === 0) return;

      const circleId = store.get('circle');
      if (!circleId) {
        localStorage.removeItem(OFFLINE_CACHE_KEY);
        return;
      }

      // Send cached locations in batch
      for (const loc of cache) {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
          this.ws.send(JSON.stringify({
            type: 'location_ping',
            ...loc,
          }));
        }
      }

      localStorage.removeItem(OFFLINE_CACHE_KEY);
      console.log(`[LocationTracker] Synced ${cache.length} cached locations`);
    } catch (e) {
      console.error('[LocationTracker] Failed to sync cache:', e);
    }
  }

  /**
   * Get location history from the server.
   * @param {string} circleId
   * @param {string} userId
   * @param {number} days
   */
  async getLocationHistory(circleId, userId, days = 7) {
    try {
      const response = await api.get(`/api/circles/${circleId}/locations/history/${userId}`, { days });
      return response.data || [];
    } catch (e) {
      console.error('[LocationTracker] Failed to get location history:', e);
      return [];
    }
  }
}

export const locationTracker = new LocationTracker();
export default locationTracker;
