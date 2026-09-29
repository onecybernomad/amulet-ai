/**
 * GeolocationService — Tracks user location and syncs to backend.
 */

import { api } from '../api.js';
import { store } from '../state.js';

export class GeolocationService {
  constructor() {
    this.watchId = null;
    this.isTracking = false;
    this.lastPosition = null;
    this.syncInterval = null;
    this.syncFrequency = 15000; // 15 seconds
  }

  /**
   * Request permission and start tracking.
   */
  async start() {
    if (this.isTracking) return;

    if (!navigator.geolocation) {
      console.error('[Geo] Geolocation not supported');
      store.set('locationError', 'Geolocation not supported by this browser');
      return;
    }

    try {
      // Request permission by getting initial position
      const position = await this._getCurrentPosition();
      this.lastPosition = position;
      store.set('userLocation', position);
      store.set('locationError', null);

      // Start watching for updates
      this.watchId = navigator.geolocation.watchPosition(
        (pos) => this._onPositionUpdate(pos),
        (err) => this._onError(err),
        {
          enableHighAccuracy: true,
          maximumAge: 10000,
          timeout: 10000,
        },
      );

      // Start periodic sync to backend
      this.syncInterval = setInterval(() => this._syncToBackend(), this.syncFrequency);

      this.isTracking = true;
      console.log('[Geo] Tracking started');
    } catch (err) {
      console.error('[Geo] Failed to start tracking:', err);
      store.set('locationError', this._getErrorMessage(err));
    }
  }

  /**
   * Stop tracking and clean up.
   */
  stop() {
    if (this.watchId !== null) {
      navigator.geolocation.clearWatch(this.watchId);
      this.watchId = null;
    }

    if (this.syncInterval) {
      clearInterval(this.syncInterval);
      this.syncInterval = null;
    }

    this.isTracking = false;
    console.log('[Geo] Tracking stopped');
  }

  /**
   * Get the current position once.
   */
  getCurrentPosition() {
    return this._getCurrentPosition();
  }

  /**
   * Get the last known position.
   */
  getLastPosition() {
    return this.lastPosition;
  }

  /**
   * Check if tracking is active.
   */
  get tracking() {
    return this.isTracking;
  }

  // ── Private ────────────────────────────────────────────────────────

  _getCurrentPosition() {
    return new Promise((resolve, reject) => {
      navigator.geolocation.getCurrentPosition(resolve, reject, {
        enableHighAccuracy: true,
        maximumAge: 10000,
        timeout: 10000,
      });
    });
  }

  _onPositionUpdate(position) {
    const locationData = {
      latitude: position.coords.latitude,
      longitude: position.coords.longitude,
      accuracy: position.coords.accuracy,
      speed: position.coords.speed,
      heading: position.coords.heading,
      timestamp: new Date(position.timestamp).toISOString(),
    };

    this.lastPosition = locationData;
    store.set('userLocation', locationData);
    store.set('locationError', null);

    // Immediate sync if significant movement detected
    if (this._shouldSyncImmediately(locationData)) {
      this._syncToBackend();
    }
  }

  _onError(error) {
    const message = this._getErrorMessage(error);
    console.error('[Geo] Error:', message);
    store.set('locationError', message);
  }

  _shouldSyncImmediately(newPos) {
    if (!this.lastPosition) return true;

    const latDiff = Math.abs(newPos.latitude - this.lastPosition.latitude);
    const lngDiff = Math.abs(newPos.longitude - this.lastPosition.longitude);

    // Sync if moved more than ~10 meters (roughly 0.0001 degrees)
    return latDiff > 0.0001 || lngDiff > 0.0001;
  }

  async _syncToBackend() {
    if (!this.lastPosition) return;

    try {
      const circleId = store.get('circle');
      if (!circleId) return;

      await api.post(`/api/circles/${circleId}/locations`, {
        latitude: this.lastPosition.latitude,
        longitude: this.lastPosition.longitude,
        accuracy: this.lastPosition.accuracy,
        speed: this.lastPosition.speed,
        heading: this.lastPosition.heading,
        timestamp: this.lastPosition.timestamp,
      });
    } catch (err) {
      // Silently fail — don't spam errors for background sync
      console.debug('[Geo] Sync failed:', err.message);
    }
  }

  _getErrorMessage(error) {
    switch (error.code) {
      case error.PERMISSION_DENIED:
        return 'Location permission denied. Please enable location access.';
      case error.POSITION_UNAVAILABLE:
        return 'Location information unavailable.';
      case error.TIMEOUT:
        return 'Location request timed out.';
      default:
        return 'An unknown location error occurred.';
    }
  }
}

export const geoService = new GeolocationService();
export default geoService;
