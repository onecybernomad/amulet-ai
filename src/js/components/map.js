/**
 * FamilyMap — Leaflet map with member markers, trails, and place circles.
 * Uses CDN-loaded global L (Leaflet).
 */

import { store } from '../state.js';

const TILE_LAYERS = {
  positron: {
    url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
  },
  dark: {
    url: 'https://tile.openstreetmap.org/{z}/{x}/{y}.png',
    attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
  },
};

const MEMBER_COLORS = ['#6366f1', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899', '#06b6d4', '#84cc16'];
const TRAIL_LENGTH = 200;

export class FamilyMap {
  constructor(containerId) {
    this.containerId = containerId;
    this.map = null;
    this.markers = new Map();
    this.trails = new Map();
    this.placeCircles = new Map();
    this.currentTheme = 'positron';
  }

  /**
   * Initialize the map.
   */
  init() {
    const container = document.getElementById(this.containerId);
    if (!container) {
      console.error(`[FamilyMap] Container #${this.containerId} not found`);
      return;
    }

    this.map = L.map(this.containerId, {
      center: [40.7128, -74.0060],
      zoom: 13,
      zoomControl: false,
    });

    L.control.zoom({ position: 'bottomright' }).addTo(this.map);

    this.tileLayer = L.tileLayer(TILE_LAYERS[this.currentTheme].url, {
      attribution: TILE_LAYERS[this.currentTheme].attribution,
      maxZoom: 19,
    }).addTo(this.map);

    return this;
  }

  /**
   * Update a member's location on the map.
   * @param {string} userId
   * @param {number} lat
   * @param {number} lng
   * @param {Object} metadata - { name, online, color, avatar }
   */
  updateMemberLocation(userId, lat, lng, metadata = {}) {
    if (!this.map) return;

    const color = metadata.color || MEMBER_COLORS[Math.abs(this._hashCode(userId)) % MEMBER_COLORS.length];
    const name = metadata.name || 'Member';
    const initials = name.split(' ').map(w => w[0]).join('').toUpperCase().slice(0, 2);

    let marker = this.markers.get(userId);

    if (!marker) {
      const icon = L.divIcon({
        className: 'member-marker-wrapper',
        html: `<div class="member-marker ${metadata.online ? 'online' : 'offline'}" style="background:${color};width:36px;height:36px;font-size:12px;">${initials}</div>`,
        iconSize: [36, 36],
        iconAnchor: [18, 18],
        popupAnchor: [0, -20],
      });

      marker = L.marker([lat, lng], { icon }).addTo(this.map);
      marker.bindPopup(this._buildPopup(name, metadata));
      this.markers.set(userId, marker);
    } else {
      marker.setLatLng([lat, lng]);
      marker.setIcon(L.divIcon({
        className: 'member-marker-wrapper',
        html: `<div class="member-marker ${metadata.online ? 'online' : 'offline'} pulse" style="background:${color};width:36px;height:36px;font-size:12px;">${initials}</div>`,
        iconSize: [36, 36],
        iconAnchor: [18, 18],
        popupAnchor: [0, -20],
      }));
      marker.setPopupContent(this._buildPopup(name, metadata));
    }

    // Update trail
    this._addTrailPoint(userId, lat, lng, color);
  }

  /**
   * Add a point to a member's trail.
   * @private
   */
  _addTrailPoint(userId, lat, lng, color) {
    let trail = this.trails.get(userId);
    if (!trail) {
      trail = L.polyline([], { color, weight: 3, opacity: 0.7 }).addTo(this.map);
      this.trails.set(userId, trail);
    }
    const latLngs = trail.getLatLngs();
    latLngs.push([lat, lng]);
    if (latLngs.length > TRAIL_LENGTH) latLngs.shift();
    trail.setLatLngs(latLngs);
  }

  /**
   * Add a geofence circle for a place.
   * @param {Object} place - { id, name, lat, lng, radius }
   */
  addPlaceCircle(place) {
    if (!this.map) return;
    this.removePlaceCircle(place.id);
    const circle = L.circle([place.lat, place.lng], {
      radius: place.radius || 100,
      color: '#6366f1',
      fillColor: '#6366f1',
      fillOpacity: 0.1,
      weight: 2,
      dashArray: '5, 10',
    }).addTo(this.map);
    circle.bindPopup(`<strong>${place.name}</strong><br>Radius: ${place.radius || 100}m`);
    this.placeCircles.set(place.id, circle);
  }

  /**
   * Remove a place circle.
   * @param {string} placeId
   */
  removePlaceCircle(placeId) {
    const circle = this.placeCircles.get(placeId);
    if (circle) {
      this.map.removeLayer(circle);
      this.placeCircles.delete(placeId);
    }
  }

  /**
   * Fly to a location.
   * @param {number} lat
   * @param {number} lng
   * @param {number} [zoom]
   */
  flyTo(lat, lng, zoom = 16) {
    if (!this.map) return;
    this.map.flyTo([lat, lng], zoom, { duration: 1.5 });
  }

  /**
   * Clear all trails.
   */
  clearTrails() {
    for (const trail of this.trails.values()) {
      this.map.removeLayer(trail);
    }
    this.trails.clear();
  }

  /**
   * Toggle between light and dark map themes.
   * Uses CSS filters on the tile layer for dark mode (OSM standard tiles only come in light).
   */
  toggleTheme() {
    if (!this.map) return;
    this.currentTheme = this.currentTheme === 'positron' ? 'dark' : 'positron';
    const container = this.map.getContainer();
    if (this.currentTheme === 'dark') {
      container.classList.add('map-dark');
    } else {
      container.classList.remove('map-dark');
    }
  }

  /**
   * Destroy the map instance.
   */
  destroy() {
    if (this.map) {
      this.map.remove();
      this.map = null;
    }
    this.markers.clear();
    this.trails.clear();
    this.placeCircles.clear();
  }

  /**
   * Build popup HTML.
   * @private
   */
  _buildPopup(name, metadata) {
    return `
      <div class="member-marker-popup">
        <div class="name">${name}</div>
        <div class="meta">${metadata.online ? 'Online' : 'Offline'}${metadata.speed ? ` · ${metadata.speed} km/h` : ''}</div>
      </div>
    `;
  }

  /**
   * Simple hash function for consistent colors.
   * @private
   */
  _hashCode(str) {
    let hash = 0;
    for (let i = 0; i < str.length; i++) {
      hash = ((hash << 5) - hash) + str.charCodeAt(i);
      hash |= 0;
    }
    return hash;
  }
}

export default FamilyMap;
