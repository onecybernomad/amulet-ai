/**
 * MedCard — Medication card with schedule, taken/skipped actions, and adherence streak.
 */

import { createElement } from '../lib/dom.js';
import { formatTime } from '../lib/format.js';

export class MedCard {
  constructor(medication, onAction) {
    this.med = medication;
    this.onAction = onAction;
    this.element = this._render();
  }

  /**
   * Render the medication card.
   * @private
   */
  _render() {
    const card = createElement('div', { class: 'med-card', 'data-med-id': this.med.id });

    // Header
    const header = createElement('div', { class: 'med-card-header' });
    const nameGroup = createElement('div');
    nameGroup.appendChild(createElement('div', { class: 'med-card-name' }, this.med.name));
    nameGroup.appendChild(createElement('div', { class: 'med-card-dosage' }, this.med.dosage));
    header.appendChild(nameGroup);

    if (this.med.nextReminder) {
      header.appendChild(createElement('span', { class: 'badge badge-primary' }, `Next: ${formatTime(this.med.nextReminder)}`));
    }
    card.appendChild(header);

    // Schedule
    const schedule = createElement('div', { class: 'med-card-schedule' });
    for (const slot of this.med.schedule || []) {
      const timeSlot = createElement('span', {
        class: `med-time-slot ${slot.status || ''}`,
      }, `${formatTime(slot.time)}${slot.status === 'taken' ? ' ✓' : slot.status === 'missed' ? ' ✗' : ''}`);
      schedule.appendChild(timeSlot);
    }
    card.appendChild(schedule);

    // Actions
    const actions = createElement('div', { class: 'med-card-actions' });
    const takenBtn = createElement('button', { class: 'btn btn-sm btn-success' }, 'Taken');
    const skipBtn = createElement('button', { class: 'btn btn-sm btn-secondary' }, 'Skip');

    takenBtn.addEventListener('click', () => this._handleAction('taken'));
    skipBtn.addEventListener('click', () => this._handleAction('skipped'));

    actions.appendChild(takenBtn);
    actions.appendChild(skipBtn);
    card.appendChild(actions);

    // Adherence
    if (this.med.streak != null) {
      const adherence = createElement('div', { class: 'med-adherence' });
      adherence.appendChild(createElement('span', { class: 'adherence-streak' }, `🔥 ${this.med.streak} day streak`));
      card.appendChild(adherence);
    }

    return card;
  }

  /**
   * Handle taken/skipped action.
   * @private
   */
  _handleAction(action) {
    if (this.onAction) {
      this.onAction(this.med.id, action);
    }
  }

  /**
   * Get the DOM element.
   * @returns {HTMLElement}
   */
  getElement() {
    return this.element;
  }

  /**
   * Update the card with new data.
   * @param {Object} data
   */
  update(data) {
    Object.assign(this.med, data);
    const newEl = this._render();
    this.element.replaceWith(newEl);
    this.element = newEl;
  }

  /**
   * Destroy the component.
   */
  destroy() {
    this.element.remove();
  }
}

export default MedCard;
