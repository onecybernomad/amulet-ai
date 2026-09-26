/**
 * MapPage — Full-height map with member sidebar, SOS overlay, and layer controls.
 */

import { createElement, clearElement, delegate } from '../lib/dom.js';
import { store } from '../state.js';
import { FamilyMap } from '../components/map.js';
import { SOSButton } from '../components/sos-button.js';
import { timeAgo } from '../lib/format.js';

export class MapPage {
  constructor(container) {
    this.container = container;
    this.map = null;
    this.sosButton = null;
    this.showTrails = true;
    this.showPlaces = true;
    this._render();
    this._initMap();
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
    this.trailsToggle = createElement('button', { class: 'layer-toggle active' }, 'Trails');
    this.placesToggle = createElement('button', { class: 'layer-toggle active' }, 'Places');
    this.themeToggle = createElement('button', { class: 'layer-toggle' }, 'Dark');

    this.trailsToggle.addEventListener('click', () => this._toggleTrails());
    this.placesToggle.addEventListener('click', () => this._togglePlaces());
    this.themeToggle.addEventListener('click', () => this._toggleTheme());

    this.layerControls.appendChild(this.trailsToggle);
    this.layerControls.appendChild(this.placesToggle);
    this.layerControls.appendChild(this.themeToggle);
    this.mainEl.appendChild(this.layerControls);

    // SOS overlay
    this.sosContainer = createElement('div', { class: 'sos-overlay' });
    this.mainEl.appendChild(this.sosContainer);

    this.container.appendChild(this.mainEl);

    // Sidebar
    this.sidebar = createElement('aside', { class: 'map-page-sidebar' });
    const sidebarHeader = createElement('div', { class: 'map-page-sidebar-header' });
    sidebarHeader.appendChild(createElement('h2', { class: 'section-title' }, 'Family Members'));
    this.sidebar.appendChild(sidebarHeader);

    this.memberList = createElement('div', { class: 'member-list' });
    this.sidebarContent = createElement('div', { class: 'map-page-sidebar-content' });
    this.sidebarContent.appendChild(this.memberList);
    this.sidebar.appendChild(this.sidebarContent);

    this.container.appendChild(this.sidebar);
  }

  /**
   * Initialize the Leaflet map.
   * @private
   */
  _initMap() {
    this.map = new FamilyMap('family-map');
    this.map.init();

    // Add existing members
    const members = store.get('members') || [];
    for (const m of members) {
      if (m.lat && m.lng) {
        this.map.updateMemberLocation(m.id, m.lat, m.lng, m);
      }
    }

    // Add place circles
    const places = store.get('places') || [];
    for (const p of places) {
      this.map.addPlaceCircle(p);
    }
  }

  /**
   * Subscribe to store changes.
   * @private
   */
  _subscribe() {
    store.on('members', (members) => this._updateMemberList(members));
    store.on('places', (places) => this._updatePlaceCircles(places));
  }

  /**
   * Update the member list sidebar.
   * @private
   */
  _updateMemberList(members) {
    clearElement(this.memberList);
    for (const m of members) {
      const item = createElement('div', { class: 'member-item', 'data-user-id': m.id });
      const avatar = createElement('div', { class: `avatar ${m.online ? '' : 'avatar-sm'}` });
      avatar.style.background = m.color || '#6366f1';
      avatar.textContent = m.name?.charAt(0) || '?';

      const info = createElement('div', { class: 'member-item-info' });
      info.appendChild(createElement('div', { class: 'member-item-name' }, m.name));
      const status = createElement('div', { class: `member-item-status ${m.online ? 'online' : ''}` });
      status.textContent = m.online ? `Online · ${timeAgo(m.lastSeen)}` : `Last seen ${timeAgo(m.lastSeen)}`;
      info.appendChild(status);

      item.appendChild(avatar);
      item.appendChild(info);

      item.addEventListener('click', () => {
        if (m.lat && m.lng) this.map.flyTo(m.lat, m.lng, 16);
      });

      this.memberList.appendChild(item);
    }
  }

  /**
   * Update place circles on the map.
   * @private
   */
  _updatePlaceCircles(places) {
    if (!this.map) return;
    // Clear and re-add
    for (const p of places) {
      this.map.addPlaceCircle(p);
    }
  }

  /**
   * Toggle trail visibility.
   * @private
   */
  _toggleTrails() {
    this.showTrails = !this.showTrails;
    this.trailsToggle.classList.toggle('active', this.showTrails);
    if (!this.showTrails) {
      this.map?.clearTrails();
    }
  }

  /**
   * Toggle place circles visibility.
   * @private
   */
  _togglePlaces() {
    this.showPlaces = !this.showPlaces;
    this.placesToggle.classList.toggle('active', this.showPlaces);
    // Re-render circles
    const places = store.get('places') || [];
    for (const p of places) {
      if (this.showPlaces) {
        this.map?.addPlaceCircle(p);
      } else {
        this.map?.removePlaceCircle(p.id);
      }
    }
  }

  /**
   * Toggle map theme.
   * @private
   */
  _toggleTheme() {
    this.map?.toggleTheme();
    const isDark = this.themeToggle.textContent === 'Dark';
    this.themeToggle.textContent = isDark ? 'Light' : 'Dark';
  }

  /**
   * Initialize SOS button.
   */
  initSOS() {
    this.sosButton = new SOSButton(this.sosContainer, () => {
      console.log('[MapPage] SOS activated');
    });
  }

  /**
   * Destroy the page.
   */
  destroy() {
    this.sosButton?.destroy();
    this.map?.destroy();
    clearElement(this.container);
  }
}

export default MapPage;
