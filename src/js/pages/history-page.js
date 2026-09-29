/**
 * HistoryPage — Location history timeline with map trail and export.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { formatDateTime, formatTime } from '../lib/format.js';

export class HistoryPage {
  constructor(container) {
    this.container = container;
    this.members = store.get('members') || [];
    this.selectedMember = 'all';
    this.historyData = [];
    this.map = null;
    this.trailLayer = null;
    this._render();
    this._loadHistory();
  }

  /**
   * Render the history page layout.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'history-page';

    // Header
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('div', {},
      createElement('h1', { class: 'section-title' }, 'Location History'),
      createElement('p', { class: 'section-subtitle' }, 'Review where members have been')
    ));

    // Export button
    const exportBtn = createElement('button', { class: 'btn btn-secondary' }, 'Export GPX');
    exportBtn.addEventListener('click', () => this._exportGPX());
    header.appendChild(exportBtn);

    this.container.appendChild(header);

    // Filter bar
    this.filterBar = createElement('div', { class: 'filter-bar' });
    this.container.appendChild(this.filterBar);
    this._renderFilters();

    // Content area
    this.contentEl = createElement('div', { class: 'history-content' });

    // Map
    this.mapContainer = createElement('div', { class: 'history-map', id: 'history-map' });
    this.contentEl.appendChild(this.mapContainer);

    // Timeline
    this.timelineEl = createElement('div', { class: 'history-timeline' });
    this.contentEl.appendChild(this.timelineEl);

    this.container.appendChild(this.contentEl);

    // Initialize map after DOM is ready
    setTimeout(() => this._initMap(), 50);
  }

  /**
   * Render member filter chips.
   * @private
   */
  _renderFilters() {
    clearElement(this.filterBar);

    const allChip = createElement('button', {
      class: `filter-chip ${this.selectedMember === 'all' ? 'active' : ''}`,
    }, 'All Members');
    allChip.addEventListener('click', () => {
      this.selectedMember = 'all';
      this._renderFilters();
      this._loadHistory();
    });
    this.filterBar.appendChild(allChip);

    for (const member of this.members) {
      const chip = createElement('button', {
        class: `filter-chip ${this.selectedMember === member.id ? 'active' : ''}`,
      }, member.name);
      chip.addEventListener('click', () => {
        this.selectedMember = member.id;
        this._renderFilters();
        this._loadHistory();
      });
      this.filterBar.appendChild(chip);
    }
  }

  /**
   * Initialize the Leaflet map.
   * @private
   */
  _initMap() {
    if (typeof L === 'undefined') return;

    this.map = L.map('history-map', {
      zoomControl: false,
      attributionControl: false,
    });

    L.tileLayer('https://{s}.tile.openstreetmap.org/{z}/{x}/{y}.png', {
      maxZoom: 19,
      attribution: '&copy; OpenStreetMap contributors',
    }).addTo(this.map);

    L.control.zoom({ position: 'topright' }).addTo(this.map);
  }

  /**
   * Load location history from the server.
   * @private
   */
  async _loadHistory() {
    try {
      const circleId = store.get('circle');
      if (!circleId) return;

      let url = `/api/circles/${circleId}/locations/history`;
      if (this.selectedMember !== 'all') {
        url += `/${this.selectedMember}`;
      }

      const response = await api.get(url);
      this.historyData = response.data || [];
      this._renderTimeline();
      this._renderMapTrail();
    } catch (err) {
      console.error('[HistoryPage] Failed to load history:', err);
    }
  }

  /**
   * Render the timeline view.
   * @private
   */
  _renderTimeline() {
    clearElement(this.timelineEl);

    if (this.historyData.length === 0) {
      const empty = createElement('div', { class: 'empty-state' });
      empty.appendChild(createElement('div', { class: 'empty-state-icon' }, '📍'));
      empty.appendChild(createElement('div', { class: 'empty-state-title' }, 'No location history'));
      empty.appendChild(createElement('div', { class: 'empty-state-text' }, 'Location history will appear here once members start sharing'));
      this.timelineEl.appendChild(empty);
      return;
    }

    for (const entry of this.historyData) {
      const item = createElement('div', { class: 'timeline-item' });

      const time = createElement('div', { class: 'timeline-time' }, formatTime(entry.timestamp));
      const details = createElement('div', { class: 'timeline-details' });
      details.appendChild(createElement('div', { class: 'timeline-location' },
        `${entry.latitude.toFixed(5)}, ${entry.longitude.toFixed(5)}`));
      details.appendChild(createElement('div', { class: 'timeline-date' },
        formatDateTime(entry.timestamp)));

      item.appendChild(time);
      item.appendChild(details);
      this.timelineEl.appendChild(item);
    }
  }

  /**
   * Render the map trail.
   * @private
   */
  _renderMapTrail() {
    if (!this.map || this.historyData.length === 0) return;

    // Clear existing trail
    if (this.trailLayer) {
      this.map.removeLayer(this.trailLayer);
    }

    // Create polyline from history data
    const points = this.historyData.map(entry => [entry.latitude, entry.longitude]);
    this.trailLayer = L.polyline(points, {
      color: '#6366f1',
      weight: 3,
      opacity: 0.7,
      dashArray: '5, 10',
    }).addTo(this.map);

    // Add markers for start and end
    if (points.length > 0) {
      const startMarker = L.circleMarker(points[0], {
        radius: 6,
        fillColor: '#22c55e',
        color: '#fff',
        weight: 2,
        fillOpacity: 1,
      }).addTo(this.map);
      startMarker.bindPopup('Start');

      const endMarker = L.circleMarker(points[points.length - 1], {
        radius: 6,
        fillColor: '#ef4444',
        color: '#fff',
        weight: 2,
        fillOpacity: 1,
      }).addTo(this.map);
      endMarker.bindPopup('End');
    }

    // Fit map to trail bounds
    if (points.length > 1) {
      this.map.fitBounds(this.trailLayer.getBounds(), { padding: [50, 50] });
    }
  }

  /**
   * Export history as GPX file.
   * @private
   */
  _exportGPX() {
    if (this.historyData.length === 0) {
      alert('No location data to export');
      return;
    }

    const gpx = `<?xml version="1.0" encoding="UTF-8"?>
<gpx version="1.1" creator="Amulet AI">
  <trk>
    <name>Location History</name>
    <trkseg>
${this.historyData.map(entry => `      <trkpt lat="${entry.latitude}" lon="${entry.longitude}"><time>${entry.timestamp}</time></trkpt>`).join('\n')}
    </trkseg>
  </trk>
</gpx>`;

    const blob = new Blob([gpx], { type: 'application/gpx+xml' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `amulet-history-${new Date().toISOString().split('T')[0]}.gpx`;
    a.click();
    URL.revokeObjectURL(url);
  }

  /**
   * Destroy the page.
   */
  destroy() {
    if (this.map) {
      this.map.remove();
      this.map = null;
    }
    clearElement(this.container);
  }
}

export default HistoryPage;
