/**
 * PlaceEditor — Modal form for creating/editing places with map picker and radius.
 */

import { createElement, clearElement } from '../lib/dom.js';

export class PlaceEditor {
  constructor(container, place, onSave, onDelete) {
    this.container = container;
    this.place = place || { name: '', address: '', lat: null, lng: null, radius: 100 };
    this.onSave = onSave;
    this.onDelete = onDelete;
    this.map = null;
    this.marker = null;
    this._render();
  }

  /**
   * Render the place editor modal.
   * @private
   */
  _render() {
    clearElement(this.container);

    this.backdrop = createElement('div', { class: 'modal-backdrop' });
    this.modal = createElement('div', { class: 'modal modal-lg' });

    // Header
    const header = createElement('div', { class: 'modal-header' });
    header.appendChild(createElement('h2', { class: 'modal-title' }, this.place.id ? 'Edit Place' : 'New Place'));
    const closeBtn = createElement('button', { class: 'modal-close', 'aria-label': 'Close' }, '✕');
    closeBtn.addEventListener('click', () => this.close());
    header.appendChild(closeBtn);
    this.modal.appendChild(header);

    // Body
    const body = createElement('div', { class: 'modal-body' });

    // Name
    const nameGroup = createElement('div', { class: 'form-group' });
    nameGroup.appendChild(createElement('label', { class: 'form-label', for: 'place-name' }, 'Name'));
    this.nameInput = createElement('input', {
      class: 'form-input',
      id: 'place-name',
      type: 'text',
      placeholder: 'e.g., Home, School, Grandma\'s',
      value: this.place.name || '',
    });
    nameGroup.appendChild(this.nameInput);
    body.appendChild(nameGroup);

    // Address search
    const addrGroup = createElement('div', { class: 'form-group' });
    addrGroup.appendChild(createElement('label', { class: 'form-label', for: 'place-address' }, 'Address'));
    this.addrInput = createElement('input', {
      class: 'form-input',
      id: 'place-address',
      type: 'text',
      placeholder: 'Search address…',
      value: this.place.address || '',
    });
    addrGroup.appendChild(this.addrInput);
    addrGroup.appendChild(createElement('div', { class: 'form-hint' }, 'Type an address to search, or click the map to set location'));
    body.appendChild(addrGroup);

    // Map picker
    const mapGroup = createElement('div', { class: 'form-group' });
    this.mapContainer = createElement('div', {
      class: 'map-container',
      id: 'place-editor-map',
      style: { height: '250px' },
    });
    mapGroup.appendChild(this.mapContainer);
    body.appendChild(mapGroup);

    // Radius
    const radiusGroup = createElement('div', { class: 'form-group' });
    radiusGroup.appendChild(createElement('label', { class: 'form-label' }, `Geofence Radius: ${this.place.radius}m`));
    this.radiusSlider = createElement('input', {
      type: 'range',
      class: 'form-input',
      min: '50',
      max: '1000',
      step: '50',
      value: this.place.radius || 100,
    });
    this.radiusSlider.addEventListener('input', (e) => {
      radiusGroup.querySelector('.form-label').textContent = `Geofence Radius: ${e.target.value}m`;
    });
    radiusGroup.appendChild(this.radiusSlider);
    body.appendChild(radiusGroup);

    this.modal.appendChild(body);

    // Footer
    const footer = createElement('div', { class: 'modal-footer' });
    if (this.place.id && this.onDelete) {
      const deleteBtn = createElement('button', { class: 'btn btn-danger' }, 'Delete');
      deleteBtn.addEventListener('click', () => this.onDelete(this.place.id));
      footer.appendChild(deleteBtn);
    }
    const cancelBtn = createElement('button', { class: 'btn btn-secondary' }, 'Cancel');
    cancelBtn.addEventListener('click', () => this.close());
    const saveBtn = createElement('button', { class: 'btn btn-primary' }, 'Save Place');
    saveBtn.addEventListener('click', () => this._save());
    footer.appendChild(cancelBtn);
    footer.appendChild(saveBtn);
    this.modal.appendChild(footer);

    this.backdrop.appendChild(this.modal);
    this.container.appendChild(this.backdrop);

    // Close on backdrop click
    this.backdrop.addEventListener('click', (e) => {
      if (e.target === this.backdrop) this.close();
    });

    // Initialize map after DOM is ready
    setTimeout(() => this._initMap(), 50);
  }

  /**
   * Initialize the map picker.
   * @private
   */
  _initMap() {
    if (typeof L === 'undefined') return;

    const center = this.place.lat && this.place.lng
      ? [this.place.lat, this.place.lng]
      : [40.7128, -74.0060];

    this.map = L.map('place-editor-map').setView(center, 13);
    L.tileLayer('https://{s}.basemaps.cartocdn.com/light_all/{z}/{x}/{y}{r}.png', {
      attribution: '&copy; <a href="https://www.openstreetmap.org/copyright">OSM</a>',
      maxZoom: 19,
    }).addTo(this.map);

    if (this.place.lat && this.place.lng) {
      this.marker = L.marker([this.place.lat, this.place.lng], { draggable: true }).addTo(this.map);
    }

    this.map.on('click', (e) => {
      const { lat, lng } = e.latlng;
      if (this.marker) {
        this.marker.setLatLng([lat, lng]);
      } else {
        this.marker = L.marker([lat, lng], { draggable: true }).addTo(this.map);
      }
      this.place.lat = lat;
      this.place.lng = lng;
    });
  }

  /**
   * Save the place.
   * @private
   */
  _save() {
    this.place.name = this.nameInput.value.trim();
    this.place.address = this.addrInput.value.trim();
    this.place.radius = parseInt(this.radiusSlider.value, 10);

    if (!this.place.name) {
      alert('Please enter a name for this place.');
      return;
    }
    if (this.place.lat == null || this.place.lng == null) {
      alert('Please set a location on the map.');
      return;
    }

    if (this.onSave) this.onSave(this.place);
    this.close();
  }

  /**
   * Search address using Nominatim.
   * @param {string} query
   */
  async searchAddress(query) {
    if (!query || query.length < 3) return;
    try {
      const res = await fetch(`https://nominatim.openstreetmap.org/search?format=json&q=${encodeURIComponent(query)}&limit=1`);
      const data = await res.json();
      if (data.length > 0) {
        const { lat, lon, display_name } = data[0];
        this.place.lat = parseFloat(lat);
        this.place.lng = parseFloat(lon);
        this.addrInput.value = display_name;
        if (this.map) {
          this.map.setView([this.place.lat, this.place.lng], 15);
          if (this.marker) {
            this.marker.setLatLng([this.place.lat, this.place.lng]);
          } else {
            this.marker = L.marker([this.place.lat, this.place.lng], { draggable: true }).addTo(this.map);
          }
        }
      }
    } catch (err) {
      console.error('[PlaceEditor] Address search failed:', err);
    }
  }

  /**
   * Close the editor.
   */
  close() {
    if (this.map) {
      this.map.remove();
      this.map = null;
    }
    clearElement(this.container);
  }

  /**
   * Destroy the component.
   */
  destroy() {
    this.close();
  }
}

export default PlaceEditor;
