/**
 * MapPage — Full-height Leaflet map with member sidebar, SOS overlay,
 * geolocation tracking, and family group management.
 */

import { createElement, clearElement, delegate } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { geoService as geolocationService } from '../services/geolocation.js';
import { CircleManager } from '../components/family-groups.js';
import { SOSButton } from '../components/sos-button.js';

export class MapPage {
  constructor(container) {
    this.container = container;
    this.map = null;
    this.circleManager = null;
    this.sosButton = null;
    this.showTrails = true;
    this.showPlaces = true;
    this.markers = new Map();
    this.trails = new Map();
    this.trailPolylines = new Map();
    this.placeCircles = new Map();
    this._render();
    this._initMap();
    this._initGeolocation();
    this._subscribe();
  }

  /**
   * Render the map page layout.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'map-page';

    // Main map area
    this.mainEl = createElement('div', { class: 'map-page-main' });
    this.mapContainer = createElement('div', { class: 'map-container', id: 'family-map' });
    this.mainEl.appendChild(this.mapContainer);

    // Layer controls
    this.layerControls = createElement('div', { class: 'layer-controls' });
    this.trailsToggle = createElement('button', { class: 'layer-toggle active', onclick: () => this._toggleTrails() }, 'Trails');
    this.placesToggle = createElement('button', { class: 'layer-toggle active', onclick: () => this._togglePlaces() }, 'Places');
    this.layerControls.appendChild(this.trailsToggle);
    this.layerControls.appendChild(this.placesToggle);
    this.mainEl.appendChild(this.layerControls);

    // SOS overlay
    this.sosContainer = createElement('div', { class: 'sos-overlay' });
    this.mainEl.appendChild(this.sosContainer);

    this.container.appendChild(this.mainEl);

    // Sidebar
    this.sidebar = createElement('aside', { class: 'map-page-sidebar' });
    this.sidebarContent = createElement('div', { class: 'map-page-sidebar-content' });
    this.sidebar.appendChild(this.sidebarContent);
    this.container.appendChild(this.sidebar);
  }

  /**
   * Initialize the Leaflet map.
   * @private
   */
  _initMap() {
    // Leaflet is loaded via CDN
    this.map = L.map('family-map', {
      zoomControl: false,
      attributionControl: false,
    });

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; OpenStreetMap contributors',
    }).addTo(this.map);

    L.control.zoom({ position: 'topright' }).addTo(this.map);

    // Initialize CircleManager
    this.circleManager = new CircleManager(this.sidebarContent);
    this.circleManager.init();

    // Add SOS button
    this.sosButton = new SOSButton(this.sosContainer);

    // Try to get user's location for initial center
    geolocationService.getCurrentPosition().then((pos) => {
      this.map.setView([pos.latitude, pos.longitude], 13);
    }).catch(() => {
      // Default to a central location
      this.map.setView([40.7128, -74.0060], 13);
    });
  }

  /**
   * Initialize geolocation tracking.
   * @private
   */
  _initGeolocation() {
    // Listen for position updates from the geolocation service
    window.addEventListener('geolocation:update', (e) => {
      const { latitude, longitude, accuracy } = e.detail;
      this._updateMemberLocation('me', latitude, longitude, accuracy);
    });

    // Start tracking with balanced mode
    const ws = window.__WS__ || null;
    geolocationService.start(ws, 'balanced');

    // Add tracking mode controls to layer controls
    this._addTrackingControls();
  }

  /**
   * Add tracking mode and privacy controls.
   * @private
   */
  _addTrackingControls() {
    const controls = createElement('div', { class: 'tracking-controls' });

    // Tracking mode selector
    const modeSelect = createElement('select', { class: 'tracking-mode-select' });
    const modes = [
      { value: 'active', label: 'Active (5s)' },
      { value: 'balanced', label: 'Balanced (10s)' },
      { value: 'passive', label: 'Passive (30s)' },
    ];
    for (const m of modes) {
      const opt = createElement('option', { value: m.value }, m.label);
      if (m.value === 'balanced') opt.selected = true;
      modeSelect.appendChild(opt);
    }
    modeSelect.addEventListener('change', (e) => {
      geolocationService.setTrackingMode(e.target.value);
      this._showNotification(`Tracking mode: ${e.target.selectedOptions[0].text}`, 'info');
    });
    controls.appendChild(modeSelect);

    // Privacy mode toggle
    const privacyToggle = createElement('button', {
      class: 'privacy-toggle',
      onclick: () => this._togglePrivacyMode(),
    }, 'Privacy: Off');
    controls.appendChild(privacyToggle);

    this.layerControls.appendChild(controls);
  }

  /**
   * Toggle privacy mode on/off.
   * @private
   */
  _togglePrivacyMode() {
    const isPrivate = !geolocationService.privacyMode;
    geolocationService.setPrivacyMode(isPrivate);

    const btn = this.layerControls.querySelector('.privacy-toggle');
    if (btn) {
      btn.textContent = `Privacy: ${isPrivate ? 'On' : 'Off'}`;
      btn.classList.toggle('active', isPrivate);
    }

    this._showNotification(
      isPrivate ? 'Privacy mode enabled — location not shared' : 'Privacy mode disabled — location shared',
      isPrivate ? 'warning' : 'success'
    );
  }

  /**
   * Subscribe to store changes and WebSocket events.
   * @private
   */
  _subscribe() {
    store.on('members', (members) => this._updateMemberList(members));
    store.on('places', (places) => this._updatePlaceCircles(places));

    // WebSocket realtime events
    window.addEventListener('ws:location_ping', (e) => this._onLocationPing(e.detail));
    window.addEventListener('ws:geofence_event', (e) => this._onGeofenceEvent(e.detail));
    window.addEventListener('ws:alert', (e) => this._onAlert(e.detail));
    window.addEventListener('ws:presence', (e) => this._onPresence(e.detail));
    window.addEventListener('ws:fall_detected', (e) => this._onFallDetected(e.detail));
    window.addEventListener('ws:crash_detected', (e) => this._onCrashDetected(e.detail));
  }

  /**
   * Handle incoming location ping from another member.
   * @param {Object} data
   * @private
   */
  _onLocationPing(data) {
    const { user_id, latitude, longitude } = data;
    if (user_id === 'me') return;

    // Find member data for name/color
    const members = store.get('members') || [];
    const member = members.find(m => m.id === user_id);
    const memberData = member ? {
      color: member.color,
      display_name: member.display_name || member.name,
    } : null;

    this._updateMemberLocation(user_id, latitude, longitude, null, memberData);
  }

  /**
   * Handle geofence enter/exit event.
   * @param {Object} data
   * @private
   */
  _onGeofenceEvent(data) {
    const { place_name, user_id, entered, latitude, longitude } = data;
    const members = store.get('members') || [];
    const member = members.find(m => m.id === user_id);
    const name = member ? (member.display_name || member.name) : 'A member';

    const action = entered ? 'arrived at' : 'left';
    this._showNotification(`${name} ${action} ${place_name}`, entered ? 'success' : 'info');

    // Flash the place circle on the map
    if (this.placeCircles) {
      for (const circle of this.placeCircles.values()) {
        circle.setStyle({ weight: 4, fillOpacity: 0.3 });
        setTimeout(() => circle.setStyle({ weight: 2, fillOpacity: 0.15 }), 2000);
      }
    }
  }

  /**
   * Handle SOS/incident alert.
   * @param {Object} data
   * @private
   */
  _onAlert(data) {
    const { id, user_id, incident_type, latitude, longitude, status } = data;
    if (status !== 'active') return;

    const members = store.get('members') || [];
    const member = members.find(m => m.id === user_id);
    const name = member ? (member.display_name || member.name) : 'A member';

    // Show incident alert modal
    this._showIncidentAlert({
      id,
      type: incident_type,
      userName: name,
      location: `${latitude.toFixed(5)}, ${longitude.toFixed(5)}`,
      timestamp: new Date().toISOString(),
      details: incident_type === 'sos' ? 'SOS alert triggered' : `${incident_type} detected`,
    });

    // Fly to the incident location
    if (latitude && longitude) {
      this.map.flyTo([latitude, longitude], 16, { duration: 2 });
    }
  }

  /**
   * Handle presence event (member online/offline).
   * @param {Object} data
   * @private
   */
  _onPresence(data) {
    const { user_id, online } = data;
    const marker = this.markers.get(user_id);
    if (marker) {
      marker.setOpacity(online ? 1.0 : 0.4);
    }
  }

  /**
   * Handle fall detection event.
   * @param {Object} data
   * @private
   */
  _onFallDetected(data) {
    const members = store.get('members') || [];
    const member = members.find(m => m.id === data.user_id);
    const name = member ? (member.display_name || member.name) : 'A member';

    // Show fall alert overlay
    const { FallAlert } = require('../components/fall-alert.js');
    const alertContainer = createElement('div', { class: 'fall-alert-container' });
    document.body.appendChild(alertContainer);

    new FallAlert(alertContainer, {
      alert_id: data.alert_id,
      user_id: data.user_id,
      userName: name,
      timestamp: data.timestamp,
    }, (alertId) => {
      // User acknowledged — they're OK
      if (window.__TAURI__) {
        window.__TAURI__.invoke('acknowledge_fall_alert').catch(console.error);
      }
      this._showNotification(`${name} is OK — fall alert cancelled`, 'success');
    }, (alertId) => {
      // Escalated — trigger SOS
      if (window.__TAURI__) {
        window.__TAURI__.invoke('trigger_sos', { silent: false }).catch(console.error);
      }
      this._showNotification(`Fall alert escalated — SOS sent for ${name}`, 'danger');
    });
  }

  /**
   * Handle crash detection event.
   * @param {Object} data
   * @private
   */
  _onCrashDetected(data) {
    const severity = data.severity || 'unknown';
    const autoSos = data.auto_sos || false;

    this._showNotification(
      `Crash detected! Severity: ${severity}${autoSos ? ' — SOS auto-triggered' : ''}`,
      'danger'
    );

    // Show incident alert
    this._showIncidentAlert({
      id: crypto.randomUUID(),
      type: 'crash',
      userName: 'You',
      location: 'Current location',
      timestamp: new Date().toISOString(),
      details: `Crash severity: ${severity}. ${autoSos ? 'Emergency services have been notified.' : 'Please confirm you are OK.'}`,
    });
  }

  /**
   * Show a toast notification.
   * @param {string} message
   * @param {string} type
   * @private
   */
  _showNotification(message, type = 'info') {
    const container = document.getElementById('notification-container') || this._createNotificationContainer();
    const toast = createElement('div', { class: `toast toast-${type}` }, message);
    container.appendChild(toast);
    setTimeout(() => toast.classList.add('show'), 10);
    setTimeout(() => {
      toast.classList.remove('show');
      setTimeout(() => toast.remove(), 300);
    }, 4000);
  }

  /**
   * Create notification container if it doesn't exist.
   * @returns {HTMLElement}
   * @private
   */
  _createNotificationContainer() {
    const container = createElement('div', { id: 'notification-container', class: 'notification-container' });
    document.body.appendChild(container);
    return container;
  }

  /**
   * Show incident alert modal.
   * @param {Object} incident
   * @private
   */
  _showIncidentAlert(incident) {
    const { IncidentAlert } = require('../components/incident-alert.js');
    const alertContainer = createElement('div', { class: 'incident-alert-container' });
    document.body.appendChild(alertContainer);
    new IncidentAlert(alertContainer, incident, (id) => {
      // Acknowledge via API
      api.post(`/api/incidents/${id}/acknowledge`, { action: 'acknowledged' }).catch(console.error);
    }, () => {
      alertContainer.remove();
    });
  }

  /**
   * Update the member list sidebar.
   * @private
   */
  _updateMemberList(members) {
    // Update the CircleManager's member list
    if (this.circleManager) {
      this.circleManager.members = members;
    }

    // Update markers on the map
    for (const member of members) {
      if (member.lat && member.lng) {
        this._updateMemberLocation(member.id, member.lat, member.lng, null, member);
      }
    }
  }

  /**
   * Update a member's location on the map.
   * @param {string} userId
   * @param {number} lat
   * @param {number} lng
   * @param {number|null} accuracy
   * @param {Object|null} memberData
   * @private
   */
  _updateMemberLocation(userId, lat, lng, accuracy, memberData = null) {
    const isMe = userId === 'me';
    const id = isMe ? 'me' : userId;

    // Update or create marker
    if (this.markers.has(id)) {
      const marker = this.markers.get(id);
      marker.setLatLng([lat, lng]);
      if (accuracy) {
        marker.setRadius(accuracy / 10);
      }
    } else {
      const color = isMe ? '#22c55e' : (memberData?.color || '#6366f1');
      const name = isMe ? 'You' : (memberData?.display_name || memberData?.name || 'Member');

      const marker = L.circleMarker([lat, lng], {
        radius: 8,
        fillColor: color,
        color: '#fff',
        weight: 2,
        opacity: 1,
        fillOpacity: 0.8,
      }).addTo(this.map);

      marker.bindPopup(`<strong>${name}</strong>`);
      this.markers.set(id, marker);

      // Add accuracy circle
      if (accuracy) {
        const accuracyCircle = L.circle([lat, lng], {
          radius: accuracy,
          color: color,
          fillColor: color,
          fillOpacity: 0.1,
          weight: 1,
        }).addTo(this.map);
        marker.accuracyCircle = accuracyCircle;
      }
    }

    // Update trail
    if (this.showTrails) {
      if (!this.trails.has(id)) {
        this.trails.set(id, []);
      }
      const trail = this.trails.get(id);
      trail.push([lat, lng]);
      if (trail.length > 200) trail.shift();

      if (this.trailPolylines.has(id)) {
        this.trailPolylines.get(id).setLatLngs(trail);
      } else {
        const polyline = L.polyline(trail, {
          color: isMe ? '#22c55e' : (memberData?.color || '#6366f1'),
          weight: 2,
          opacity: 0.4,
        }).addTo(this.map);
        this.trailPolylines.set(id, polyline);
      }
    }
  }

  /**
   * Update place circles on the map.
   * @param {Array} places
   * @private
   */
  _updatePlaceCircles(places) {
    if (!this.showPlaces) return;

    // Clear existing place circles
    if (this.placeCircles) {
      for (const circle of this.placeCircles.values()) {
        this.map.removeLayer(circle);
      }
    }
    this.placeCircles = new Map();

    for (const place of places) {
      if (place.latitude && place.longitude) {
        const circle = L.circle([place.latitude, place.longitude], {
          radius: place.radius_meters || 150,
          color: '#8b5cf6',
          fillColor: '#8b5cf6',
          fillOpacity: 0.15,
          weight: 2,
          dashArray: '5, 5',
        }).addTo(this.map);
        circle.bindPopup(`<strong>${place.name}</strong><br>Radius: ${place.radius_meters || 150}m`);
        this.placeCircles.set(place.id, circle);
      }
    }
  }

  /**
   * Toggle trail visibility.
   * @private
   */
  _toggleTrails() {
    this.showTrails = !this.showTrails;
    this.trailsToggle.classList.toggle('active', this.showTrails);

    if (this.trailPolylines) {
      for (const polyline of this.trailPolylines.values()) {
        if (this.showTrails) {
          this.map.addLayer(polyline);
        } else {
          this.map.removeLayer(polyline);
        }
      }
    }
  }

  /**
   * Toggle place circles visibility.
   * @private
   */
  _togglePlaces() {
    this.showPlaces = !this.showPlaces;
    this.placesToggle.classList.toggle('active', this.showPlaces);

    if (this.placeCircles) {
      for (const circle of this.placeCircles.values()) {
        if (this.showPlaces) {
          this.map.addLayer(circle);
        } else {
          this.map.removeLayer(circle);
        }
      }
    }
  }

  /**
   * Destroy the page.
   */
  destroy() {
    geolocationService.stop();
    clearElement(this.container);
  }
}

export default MapPage;
