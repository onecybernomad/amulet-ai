/**
 * AssistancePage — Roadside and medical assistance request flow.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';

const ASSISTANCE_TYPES = [
  { id: 'towing', label: 'Towing', icon: '🚗', description: 'Get towed to a nearby shop' },
  { id: 'flat_tire', label: 'Flat Tire', icon: '🛞', description: 'Tire change or repair' },
  { id: 'jump_start', label: 'Jump Start', icon: '🔋', description: 'Dead battery jump' },
  { id: 'lockout', label: 'Lockout', icon: '🔑', description: 'Locked out of vehicle' },
  { id: 'fuel_delivery', label: 'Fuel Delivery', icon: '⛽', description: 'Out of gas' },
  { id: 'medical', label: 'Medical Advice', icon: '🏥', description: 'Speak to a medical professional' },
];

export class AssistancePage {
  constructor(container) {
    this.container = container;
    this.selectedType = null;
    this.activeRequest = null;
    this._render();
  }

  /**
   * Render the assistance page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'assistance-page';

    // Header
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('div', {},
      createElement('h1', { class: 'section-title' }, 'Assistance'),
      createElement('p', { class: 'section-subtitle' }, 'Roadside and medical help when you need it')
    ));
    this.container.appendChild(header);

    // Active request status
    this.statusEl = createElement('div', { class: 'assistance-status', style: { display: 'none' } });
    this.container.appendChild(this.statusEl);

    // Assistance type grid
    this.grid = createElement('div', { class: 'assistance-grid' });
    this.container.appendChild(this.grid);
    this._renderGrid();
  }

  /**
   * Render the assistance type grid.
   * @private
   */
  _renderGrid() {
    clearElement(this.grid);

    for (const type of ASSISTANCE_TYPES) {
      const card = createElement('div', {
        class: 'assistance-card',
        'data-type': type.id,
      });

      const icon = createElement('div', { class: 'assistance-card-icon' }, type.icon);
      const label = createElement('div', { class: 'assistance-card-label' }, type.label);
      const desc = createElement('div', { class: 'assistance-card-desc' }, type.description);

      card.appendChild(icon);
      card.appendChild(label);
      card.appendChild(desc);

      card.addEventListener('click', () => this._selectType(type));
      this.grid.appendChild(card);
    }
  }

  /**
   * Select an assistance type.
   * @param {Object} type
   * @private
   */
  _selectType(type) {
    this.selectedType = type;

    // Show confirmation dialog
    const confirmed = confirm(`Request ${type.label} assistance?\n\nYour current location will be shared with the service provider.`);
    if (confirmed) {
      this._requestAssistance(type);
    }
  }

  /**
   * Request assistance.
   * @param {Object} type
   * @private
   */
  async _requestAssistance(type) {
    try {
      // Get current location
      const position = await this._getCurrentLocation();
      const latitude = position.latitude;
      const longitude = position.longitude;

      let response;
      if (type.id === 'medical') {
        response = await api.post('/api/assistance/medical', {
          latitude,
          longitude,
          notes: type.description,
        });
      } else {
        response = await api.post('/api/assistance/roadside', {
          assistance_type: type.id,
          latitude,
          longitude,
          notes: type.description,
        });
      }

      this.activeRequest = response.data;
      this._showStatus(response.data);
    } catch (err) {
      console.error('[AssistancePage] Failed to request assistance:', err);
      alert('Failed to request assistance. Please try again.');
    }
  }

  /**
   * Show the active request status.
   * @param {Object} request
   * @private
   */
  _showStatus(request) {
    this.statusEl.style.display = 'block';
    clearElement(this.statusEl);

    const statusCard = createElement('div', { class: 'assistance-status-card' });

    // Status header
    const statusHeader = createElement('div', { class: 'assistance-status-header' });
    statusHeader.appendChild(createElement('div', { class: 'assistance-status-title' },
      `${this._getTypeLabel(request.assistance_type || 'medical')} Request`));
    statusHeader.appendChild(createElement('div', { class: 'assistance-status-badge' },
      request.status.toUpperCase()));
    statusCard.appendChild(statusHeader);

    // Details
    const details = createElement('div', { class: 'assistance-status-details' });

    if (request.provider) {
      details.appendChild(createElement('div', { class: 'assistance-status-row' },
        createElement('span', {}, 'Provider: '),
        createElement('span', {}, request.provider),
      ));
    }
    if (request.estimated_arrival) {
      details.appendChild(createElement('div', { class: 'assistance-status-row' },
        createElement('span', {}, 'ETA: '),
        createElement('span', {}, request.estimated_arrival),
      ));
    }
    if (request.cost_estimate) {
      details.appendChild(createElement('div', { class: 'assistance-status-row' },
        createElement('span', {}, 'Cost: '),
        createElement('span', {}, request.cost_estimate),
      ));
    }
    if (request.provider_phone) {
      details.appendChild(createElement('div', { class: 'assistance-status-row' },
        createElement('span', {}, 'Phone: '),
        createElement('a', { href: `tel:${request.provider_phone}` }, request.provider_phone),
      ));
    }

    statusCard.appendChild(details);

    // Cancel button
    const cancelBtn = createElement('button', { class: 'btn btn-danger' }, 'Cancel Request');
    cancelBtn.addEventListener('click', () => this._cancelRequest(request.request_id));
    statusCard.appendChild(cancelBtn);

    this.statusEl.appendChild(statusCard);

    // Start polling for status updates
    this._startStatusPolling(request.request_id);
  }

  /**
   * Start polling for status updates.
   * @param {string} requestId
   * @private
   */
  _startStatusPolling(requestId) {
    if (this.pollTimer) clearInterval(this.pollTimer);

    this.pollTimer = setInterval(async () => {
      try {
        const response = await api.get(`/api/assistance/${requestId}/status`);
        this._updateStatus(response.data);

        // Stop polling if completed or cancelled
        if (['completed', 'cancelled'].includes(response.data.status)) {
          clearInterval(this.pollTimer);
        }
      } catch (err) {
        console.error('[AssistancePage] Failed to poll status:', err);
      }
    }, 10_000);
  }

  /**
   * Update the status display.
   * @param {Object} status
   * @private
   */
  _updateStatus(status) {
    const badge = this.statusEl.querySelector('.assistance-status-badge');
    if (badge) {
      badge.textContent = status.status.toUpperCase();
      badge.className = `assistance-status-badge assistance-status-${status.status}`;
    }
  }

  /**
   * Cancel an assistance request.
   * @param {string} requestId
   * @private
   */
  async _cancelRequest(requestId) {
    try {
      await api.post(`/api/assistance/${requestId}/cancel`);
      clearInterval(this.pollTimer);
      this.statusEl.style.display = 'none';
      this.activeRequest = null;
    } catch (err) {
      console.error('[AssistancePage] Failed to cancel request:', err);
    }
  }

  /**
   * Get the current device location.
   * @returns {Promise<{latitude: number, longitude: number}>}
   * @private
   */
  _getCurrentLocation() {
    return new Promise((resolve, reject) => {
      if (!navigator.geolocation) {
        reject(new Error('Geolocation not supported'));
        return;
      }

      navigator.geolocation.getCurrentPosition(
        (position) => resolve({
          latitude: position.coords.latitude,
          longitude: position.coords.longitude,
        }),
        (error) => reject(error),
        { enableHighAccuracy: true, timeout: 10000 },
      );
    });
  }

  /**
   * Get the label for an assistance type.
   * @param {string} type
   * @returns {string}
   * @private
   */
  _getTypeLabel(type) {
    const found = ASSISTANCE_TYPES.find(t => t.id === type);
    return found ? found.label : type;
  }

  /**
   * Destroy the page.
   */
  destroy() {
    if (this.pollTimer) clearInterval(this.pollTimer);
    clearElement(this.container);
  }
}

export default AssistancePage;
