/**
 * TilePanel — Tile tracker panel with ring, locate, battery, and last seen.
 */

import { createElement } from '../lib/dom.js';
import { timeAgo, formatBattery } from '../lib/format.js';

export class TilePanel {
  constructor(container, tiles, onRing, onLocate) {
    this.container = container;
    this.tiles = tiles;
    this.onRing = onRing;
    this.onLocate = onLocate;
    this._render();
  }

  /**
   * Render the tile panel.
   * @private
   */
  _render() {
    this.container.innerHTML = '';
    this.container.className = 'tile-panel';

    for (const tile of this.tiles) {
      const card = this._renderTileCard(tile);
      this.container.appendChild(card);
    }
  }

  /**
   * Render a single tile card.
   * @private
   */
  _renderTileCard(tile) {
    const card = createElement('div', { class: 'tile-card', 'data-tile-id': tile.id });

    // Header
    const header = createElement('div', { class: 'tile-card-header' });
    header.appendChild(createElement('div', { class: 'tile-card-name' }, tile.name || 'Tile'));

    const status = createElement('div', { class: 'tile-card-status' });
    status.appendChild(createElement('span', {}, tile.lastSeen ? timeAgo(tile.lastSeen) : 'Never'));
    header.appendChild(status);
    card.appendChild(header);

    // Battery
    if (tile.battery != null) {
      const battery = createElement('div', { class: 'tile-battery' });
      battery.appendChild(createElement('span', { class: 'tile-battery-icon' }, this._batteryIcon(tile.battery)));
      battery.appendChild(createElement('span', {}, formatBattery(tile.battery)));
      card.appendChild(battery);
    }

    // Actions
    const actions = createElement('div', { class: 'tile-card-actions' });

    const ringBtn = createElement('button', { class: 'btn btn-sm btn-secondary' }, 'Ring');
    ringBtn.addEventListener('click', () => this.onRing?.(tile.id));
    actions.appendChild(ringBtn);

    const locateBtn = createElement('button', { class: 'btn btn-sm btn-primary' }, 'Locate');
    locateBtn.addEventListener('click', () => this.onLocate?.(tile.id));
    actions.appendChild(locateBtn);

    card.appendChild(actions);
    return card;
  }

  /**
   * Generate battery level indicator.
   * @private
   */
  _batteryIcon(level) {
    const bars = [];
    const activeCount = level > 75 ? 4 : level > 50 ? 3 : level > 25 ? 2 : 1;
    const levelClass = level > 50 ? '' : level > 25 ? 'medium' : 'low';

    for (let i = 0; i < 4; i++) {
      const cls = i < activeCount ? `active ${levelClass}` : '';
      bars.push(`<span class="tile-battery-level ${cls}"></span>`);
    }
    return bars.join('');
  }

  /**
   * Update tiles data.
   * @param {Array} tiles
   */
  update(tiles) {
    this.tiles = tiles;
    this._render();
  }

  /**
   * Destroy the component.
   */
  destroy() {
    this.container.innerHTML = '';
  }
}

export default TilePanel;
