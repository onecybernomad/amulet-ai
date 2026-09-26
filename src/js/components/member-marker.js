/**
 * MemberMarker — Creates and manages a Leaflet marker for a family member.
 */

import { getInitials } from '../lib/format.js';

const COLORS = ['#6366f1', '#10b981', '#f59e0b', '#ef4444', '#8b5cf6', '#ec4899', '#06b6d4', '#84cc16'];

export class MemberMarker {
  constructor(map, member) {
    this.map = map;
    this.member = member;
    this.marker = null;
    this.color = member.color || COLORS[Math.abs(this._hash(member.id)) % COLORS.length];
    this._create();
  }

  /**
   * Create the marker on the map.
   * @private
   */
  _create() {
    const initials = getInitials(this.member.name);
    const icon = L.divIcon({
      className: 'member-marker-wrapper',
      html: `<div class="member-marker ${this.member.online ? 'online' : 'offline'}" style="background:${this.color};width:36px;height:36px;font-size:12px;">${initials}</div>`,
      iconSize: [36, 36],
      iconAnchor: [18, 18],
      popupAnchor: [0, -20],
    });

    this.marker = L.marker([this.member.lat, this.member.lng], { icon }).addTo(this.map);
    this.marker.bindPopup(this._popupContent());
  }

  /**
   * Update marker position and metadata.
   * @param {Object} data - { lat, lng, online, speed }
   */
  update(data) {
    if (data.lat != null && data.lng != null) {
      this.marker.setLatLng([data.lat, data.lng]);
    }
    if (data.online !== undefined) {
      this.member.online = data.online;
    }
    if (data.speed !== undefined) {
      this.member.speed = data.speed;
    }
    this.marker.setIcon(L.divIcon({
      className: 'member-marker-wrapper',
      html: `<div class="member-marker ${this.member.online ? 'online' : 'offline'} pulse" style="background:${this.color};width:36px;height:36px;font-size:12px;">${getInitials(this.member.name)}</div>`,
      iconSize: [36, 36],
      iconAnchor: [18, 18],
      popupAnchor: [0, -20],
    }));
    this.marker.setPopupContent(this._popupContent());
  }

  /**
   * Remove marker from map.
   */
  remove() {
    if (this.marker) {
      this.marker.remove();
      this.marker = null;
    }
  }

  /**
   * Build popup HTML.
   * @private
   */
  _popupContent() {
    return `
      <div class="member-marker-popup">
        <div class="name">${this.member.name}</div>
        <div class="meta">${this.member.online ? 'Online' : 'Offline'}${this.member.speed ? ` · ${this.member.speed} km/h` : ''}</div>
      </div>
    `;
  }

  /**
   * Simple hash for color assignment.
   * @private
   */
  _hash(str) {
    let h = 0;
    for (let i = 0; i < str.length; i++) h = ((h << 5) - h) + str.charCodeAt(i);
    return h | 0;
  }
}

export default MemberMarker;
