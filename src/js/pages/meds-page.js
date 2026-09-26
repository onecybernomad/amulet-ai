/**
 * MedsPage — Medication list grouped by member with add modal and adherence calendar.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { MedCard } from '../components/med-card.js';
import { formatDate } from '../lib/format.js';

export class MedsPage {
  constructor(container) {
    this.container = container;
    this.medications = store.get('medications') || [];
    this.members = store.get('members') || [];
    this.filterMember = 'all';
    this._render();
    this._subscribe();
  }

  /**
   * Render the medications page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'meds-page';

    // Header
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('div', {},
      createElement('h1', { class: 'section-title' }, 'Medications'),
      createElement('p', { class: 'section-subtitle' }, 'Track doses, schedules, and adherence')
    ));
    const addBtn = createElement('button', { class: 'btn btn-primary' }, '+ Add Medication');
    addBtn.addEventListener('click', () => this._openAddModal());
    header.appendChild(addBtn);
    this.container.appendChild(header);

    // Filter bar
    this.filterBar = createElement('div', { class: 'filter-bar' });
    this.container.appendChild(this.filterBar);
    this._renderFilters();

    // Medications grid
    this.grid = createElement('div', { class: 'meds-grid' });
    this.container.appendChild(this.grid);

    this._renderMedications();
  }

  /**
   * Render member filter chips.
   * @private
   */
  _renderFilters() {
    clearElement(this.filterBar);

    const allChip = createElement('button', {
      class: `filter-chip ${this.filterMember === 'all' ? 'active' : ''}`,
    }, 'All Members');
    allChip.addEventListener('click', () => {
      this.filterMember = 'all';
      this._renderFilters();
      this._renderMedications();
    });
    this.filterBar.appendChild(allChip);

    for (const member of this.members) {
      const chip = createElement('button', {
        class: `filter-chip ${this.filterMember === member.id ? 'active' : ''}`,
      }, member.name);
      chip.addEventListener('click', () => {
        this.filterMember = member.id;
        this._renderFilters();
        this._renderMedications();
      });
      this.filterBar.appendChild(chip);
    }
  }

  /**
   * Render medication cards.
   * @private
   */
  _renderMedications() {
    clearElement(this.grid);

    const filtered = this.filterMember === 'all'
      ? this.medications
      : this.medications.filter(m => m.memberId === this.filterMember);

    if (filtered.length === 0) {
      const empty = createElement('div', { class: 'empty-state' });
      empty.appendChild(createElement('div', { class: 'empty-state-icon' }, '💊'));
      empty.appendChild(createElement('div', { class: 'empty-state-title' }, 'No medications'));
      empty.appendChild(createElement('div', { class: 'empty-state-text' }, 'Add medications to track schedules and adherence'));
      this.grid.appendChild(empty);
      return;
    }

    for (const med of filtered) {
      const card = new MedCard(med, (medId, action) => this._handleMedAction(medId, action));
      this.grid.appendChild(card.getElement());
    }
  }

  /**
   * Handle medication taken/skipped action.
   * @private
   */
  async _handleMedAction(medId, action) {
    try {
      await api.post(`/medications/${medId}/log`, { action, timestamp: Date.now() });
      // Update local state
      const med = this.medications.find(m => m.id === medId);
      if (med) {
        med.streak = action === 'taken' ? (med.streak || 0) + 1 : 0;
        this._renderMedications();
      }
    } catch (err) {
      console.error('[MedsPage] Failed to log action:', err);
    }
  }

  /**
   * Open add medication modal.
   * @private
   */
  _openAddModal() {
    const modalContainer = createElement('div');
    document.body.appendChild(modalContainer);

    const backdrop = createElement('div', { class: 'modal-backdrop' });
    const modal = createElement('div', { class: 'modal' });

    const header = createElement('div', { class: 'modal-header' });
    header.appendChild(createElement('h2', { class: 'modal-title' }, 'Add Medication'));
    const closeBtn = createElement('button', { class: 'modal-close' }, '✕');
    closeBtn.addEventListener('click', () => { backdrop.remove(); });
    header.appendChild(closeBtn);
    modal.appendChild(header);

    const body = createElement('div', { class: 'modal-body' });

    // Name
    const nameGroup = createElement('div', { class: 'form-group' });
    nameGroup.appendChild(createElement('label', { class: 'form-label' }, 'Medication Name'));
    const nameInput = createElement('input', { class: 'form-input', type: 'text', placeholder: 'e.g., Ibuprofen' });
    nameGroup.appendChild(nameInput);
    body.appendChild(nameGroup);

    // Dosage
    const dosageGroup = createElement('div', { class: 'form-group' });
    dosageGroup.appendChild(createElement('label', { class: 'form-label' }, 'Dosage'));
    const dosageInput = createElement('input', { class: 'form-input', type: 'text', placeholder: 'e.g., 200mg' });
    dosageGroup.appendChild(dosageInput);
    body.appendChild(dosageGroup);

    // Member
    const memberGroup = createElement('div', { class: 'form-group' });
    memberGroup.appendChild(createElement('label', { class: 'form-label' }, 'Member'));
    const memberSelect = createElement('select', { class: 'form-select' });
    for (const m of this.members) {
      memberSelect.appendChild(createElement('option', { value: m.id }, m.name));
    }
    memberGroup.appendChild(memberSelect);
    body.appendChild(memberGroup);

    // Schedule
    const scheduleGroup = createElement('div', { class: 'form-group' });
    scheduleGroup.appendChild(createElement('label', { class: 'form-label' }, 'Schedule (comma-separated times)'));
    const scheduleInput = createElement('input', { class: 'form-input', type: 'text', placeholder: 'e.g., 08:00, 14:00, 20:00' });
    scheduleGroup.appendChild(scheduleInput);
    body.appendChild(scheduleGroup);

    modal.appendChild(body);

    const footer = createElement('div', { class: 'modal-footer' });
    const cancelBtn = createElement('button', { class: 'btn btn-secondary' }, 'Cancel');
    cancelBtn.addEventListener('click', () => { backdrop.remove(); });
    const saveBtn = createElement('button', { class: 'btn btn-primary' }, 'Add');
    saveBtn.addEventListener('click', async () => {
      try {
        const schedule = scheduleInput.value.split(',').map(t => ({
          time: t.trim(),
          status: 'pending',
        }));
        const med = await api.post('/medications', {
          name: nameInput.value.trim(),
          dosage: dosageInput.value.trim(),
          memberId: memberSelect.value,
          schedule,
        });
        this.medications.push(med);
        store.set('medications', this.medications);
        this._renderMedications();
        backdrop.remove();
      } catch (err) {
        console.error('[MedsPage] Failed to add medication:', err);
      }
    });
    footer.appendChild(cancelBtn);
    footer.appendChild(saveBtn);
    modal.appendChild(footer);

    backdrop.appendChild(modal);
    modalContainer.appendChild(backdrop);

    backdrop.addEventListener('click', (e) => {
      if (e.target === backdrop) backdrop.remove();
    });
  }

  /**
   * Subscribe to store changes.
   * @private
   */
  _subscribe() {
    store.on('medications', (meds) => {
      this.medications = meds;
      this._renderMedications();
    });
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default MedsPage;
