/**
 * PlacesPage — List of places with edit/delete and add new place functionality.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { PlaceEditor } from '../components/place-editor.js';
import { formatDistance } from '../lib/format.js';

export class PlacesPage {
  constructor(container) {
    this.container = container;
    this.places = store.get('places') || [];
    this._render();
    this._subscribe();
  }

  /**
   * Render the places page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'places-page';

    // Header
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('div', {}, 
      createElement('h1', { class: 'section-title' }, 'Places'),
      createElement('p', { class: 'section-subtitle' }, 'Manage safe zones and geofences for your family')
    ));
    const addBtn = createElement('button', { class: 'btn btn-primary' }, '+ Add Place');
    addBtn.addEventListener('click', () => this._openEditor());
    header.appendChild(addBtn);
    this.container.appendChild(header);

    // Places grid
    this.grid = createElement('div', { class: 'places-grid' });
    this.container.appendChild(this.grid);

    this._renderPlaces();
  }

  /**
   * Render place cards.
   * @private
   */
  _renderPlaces() {
    clearElement(this.grid);

    if (this.places.length === 0) {
      const empty = createElement('div', { class: 'empty-state' });
      empty.appendChild(createElement('div', { class: 'empty-state-icon' }, '📍'));
      empty.appendChild(createElement('div', { class: 'empty-state-title' }, 'No places yet'));
      empty.appendChild(createElement('div', { class: 'empty-state-text' }, 'Add places like home, school, or work to get geofence alerts'));
      this.grid.appendChild(empty);
      return;
    }

    for (const place of this.places) {
      const card = this._renderPlaceCard(place);
      this.grid.appendChild(card);
    }
  }

  /**
   * Render a single place card.
   * @private
   */
  _renderPlaceCard(place) {
    const card = createElement('div', { class: 'place-card' });

    const header = createElement('div', { class: 'place-card-header' });
    header.appendChild(createElement('div', { class: 'place-card-name' }, place.name));
    header.appendChild(createElement('span', { class: 'badge badge-primary' }, `${place.radius || 100}m`));
    card.appendChild(header);

    card.appendChild(createElement('div', { class: 'place-card-address' }, place.address || 'No address'));

    const meta = createElement('div', { class: 'place-card-meta' });
    meta.appendChild(createElement('span', {}, `📍 ${place.lat?.toFixed(4)}, ${place.lng?.toFixed(4)}`));
    card.appendChild(meta);

    const actions = createElement('div', { class: 'place-card-actions' });
    const editBtn = createElement('button', { class: 'btn btn-sm btn-secondary' }, 'Edit');
    const deleteBtn = createElement('button', { class: 'btn btn-sm btn-danger' }, 'Delete');

    editBtn.addEventListener('click', () => this._openEditor(place));
    deleteBtn.addEventListener('click', () => this._deletePlace(place.id));

    actions.appendChild(editBtn);
    actions.appendChild(deleteBtn);
    card.appendChild(actions);

    return card;
  }

  /**
   * Open the place editor modal.
   * @private
   */
  _openEditor(place = null) {
    const modalContainer = createElement('div');
    document.body.appendChild(modalContainer);

    const editor = new PlaceEditor(
      modalContainer,
      place,
      async (savedPlace) => {
        try {
          if (savedPlace.id) {
            await api.patch(`/places/${savedPlace.id}`, savedPlace);
          } else {
            await api.post('/places', savedPlace);
          }
          await this._loadPlaces();
        } catch (err) {
          console.error('[PlacesPage] Save failed:', err);
        }
        editor.destroy();
        modalContainer.remove();
      },
      async (placeId) => {
        if (!confirm('Delete this place?')) return;
        try {
          await api.delete(`/places/${placeId}`);
          await this._loadPlaces();
        } catch (err) {
          console.error('[PlacesPage] Delete failed:', err);
        }
        editor.destroy();
        modalContainer.remove();
      }
    );
  }

  /**
   * Delete a place.
   * @private
   */
  async _deletePlace(placeId) {
    if (!confirm('Are you sure you want to delete this place?')) return;
    try {
      await api.delete(`/places/${placeId}`);
      await this._loadPlaces();
    } catch (err) {
      console.error('[PlacesPage] Delete failed:', err);
    }
  }

  /**
   * Load places from API.
   * @private
   */
  async _loadPlaces() {
    try {
      const data = await api.get('/places');
      this.places = data;
      store.set('places', data);
      this._renderPlaces();
    } catch (err) {
      console.error('[PlacesPage] Failed to load places:', err);
    }
  }

  /**
   * Subscribe to store changes.
   * @private
   */
  _subscribe() {
    store.on('places', (places) => {
      this.places = places;
      this._renderPlaces();
    });
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default PlacesPage;
