/**
 * FamilyGroups — UI for creating, joining, and managing family groups.
 */

import { createElement, clearElement, delegate } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';

export class FamilyGroups {
  /**
   * @param {HTMLElement} container
   * @param {Object} [options]
   * @param {boolean} [options.compact] — Render compact list without create/join forms
   * @param {(group: Object) => void} [options.onSelect] — Callback when a group is selected
   */
  constructor(container, options = {}) {
    this.container = container;
    this.options = options;
    this.groups = [];
    this.selectedId = store.get('circle');
    this.loading = false;
    this.error = null;
    this.mode = 'list'; // 'list' | 'create' | 'join'
  }

  /**
   * Initialize and render.
   */
  async init() {
    await this.fetchGroups();
    this.render();
    this._subscribe();
  }

  /**
   * Subscribe to store changes.
   * @private
   */
  _subscribe() {
    store.on('circle', (val) => {
      this.selectedId = val;
      this._renderList();
    });
  }

  /**
   * Fetch groups from the API.
   */
  async fetchGroups() {
    this.loading = true;
    this.error = null;
    this.render();

    try {
      const res = await api.get('/api/circles');
      this.groups = res.data || [];
    } catch (err) {
      this.error = err.message || 'Failed to load groups';
    } finally {
      this.loading = false;
      this.render();
    }
  }

  /**
   * Set render mode.
   * @param {'list'|'create'|'join'} mode
   */
  setMode(mode) {
    this.mode = mode;
    this.render();
  }

  /**
   * Render the full component.
   */
  render() {
    clearElement(this.container);
    this.container.className = 'family-groups';

    if (this.loading) {
      this.container.appendChild(createElement('div', { class: 'loading-state' }, 'Loading groups…'));
      return;
    }

    if (this.error) {
      this.container.appendChild(createElement('div', { class: 'form-error' }, this.error));
      this.container.appendChild(
        createElement('button', { class: 'btn btn-secondary btn-sm', onclick: () => this.fetchGroups() }, 'Retry'),
      );
      return;
    }

    // Header with toggle buttons
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('h3', { class: 'section-title' }, 'Family Groups'));

    if (!this.options.compact) {
      const toggle = createElement('div', { class: 'flex gap-2' });

      if (this.mode === 'list') {
        const createBtn = createElement('button', { class: 'btn btn-primary btn-sm', onclick: () => this.setMode('create') }, 'Create');
        const joinBtn = createElement('button', { class: 'btn btn-secondary btn-sm', onclick: () => this.setMode('join') }, 'Join');
        toggle.appendChild(createBtn);
        toggle.appendChild(joinBtn);
      } else {
        const backBtn = createElement('button', { class: 'btn btn-ghost btn-sm', onclick: () => this.setMode('list') }, '← Back');
        toggle.appendChild(backBtn);
      }

      header.appendChild(toggle);
    }

    this.container.appendChild(header);

    // Body
    if (this.mode === 'create') {
      this.container.appendChild(this._renderCreateForm());
    } else if (this.mode === 'join') {
      this.container.appendChild(this._renderJoinForm());
    } else {
      this.container.appendChild(this._renderList());
    }
  }

  /**
   * Render the group list.
   * @private
   */
  _renderList() {
    const existing = this.container.querySelector('.groups-list');
    const list = existing || createElement('div', { class: 'groups-list' });

    if (!existing) {
      clearElement(list);
    } else {
      clearElement(list);
    }

    if (this.groups.length === 0) {
      list.appendChild(
        createElement('div', { class: 'empty-state' },
          createElement('div', { class: 'empty-state-icon' }, '👨‍👩‍👧‍👦'),
          createElement('div', { class: 'empty-state-title' }, 'No family groups yet'),
          createElement('div', { class: 'empty-state-text' }, 'Create a group or join one with an invite code'),
        ),
      );
      if (!existing) this.container.appendChild(list);
      return list;
    }

    for (const group of this.groups) {
      const item = createElement('div', {
        class: `group-item card ${group.id === this.selectedId ? 'selected' : ''}`,
        'data-group-id': group.id,
        onclick: () => this._selectGroup(group),
      });

      const header = createElement('div', { class: 'flex items-center gap-3' });
      const avatar = createElement('div', { class: 'avatar avatar-lg' });
      avatar.style.background = '#6366f1';
      avatar.textContent = (group.name || '?').charAt(0).toUpperCase();
      header.appendChild(avatar);

      const info = createElement('div');
      info.appendChild(createElement('div', { class: 'font-semibold' }, group.name || 'Unnamed Group'));
      info.appendChild(createElement('div', { class: 'text-sm text-muted' }, `Code: ${group.invite_code || '—'}`));
      header.appendChild(info);

      item.appendChild(header);

      if (group.id === this.selectedId) {
        item.appendChild(createElement('div', { class: 'text-sm text-muted', style: { marginTop: '8px' } }, '✓ Active group'));
      }

      list.appendChild(item);
    }

    if (!existing) this.container.appendChild(list);
    return list;
  }

  /**
   * Render the create group form.
   * @private
   */
  _renderCreateForm() {
    const form = createElement('div', { class: 'card', style: { padding: '16px' } });

    const nameLabel = createElement('label', { class: 'form-label' }, 'Group Name');
    const nameInput = createElement('input', { class: 'form-input', type: 'text', placeholder: 'e.g. The Smith Family' });
    form.appendChild(nameLabel);
    form.appendChild(nameInput);

    const submitBtn = createElement('button', { class: 'btn btn-primary btn-block', style: { marginTop: '12px' } }, 'Create Group');

    submitBtn.addEventListener('click', async () => {
      const name = nameInput.value.trim();
      if (!name) {
        alert('Please enter a group name');
        return;
      }

      submitBtn.disabled = true;
      submitBtn.textContent = 'Creating...';

      try {
        const res = await api.post('/api/circles', { name });
        const newGroup = res.data;
        this.groups.push(newGroup);
        this.selectedId = newGroup.id;
        store.set('circle', newGroup);
        this.mode = 'list';
        this.render();
        // Refresh page data if needed
        window.location.reload();
      } catch (err) {
        alert(err.message || 'Failed to create group');
        submitBtn.disabled = false;
        submitBtn.textContent = 'Create Group';
      }
    });

    form.appendChild(submitBtn);
    return form;
  }

  /**
   * Render the join group form.
   * @private
   */
  _renderJoinForm() {
    const form = createElement('div', { class: 'card', style: { padding: '16px' } });

    const codeLabel = createElement('label', { class: 'form-label' }, 'Invite Code');
    const codeInput = createElement('input', { class: 'form-input', type: 'text', placeholder: 'Enter invite code' });
    form.appendChild(codeLabel);
    form.appendChild(codeInput);

    const submitBtn = createElement('button', { class: 'btn btn-primary btn-block', style: { marginTop: '12px' } }, 'Join Group');

    submitBtn.addEventListener('click', async () => {
      const code = codeInput.value.trim();
      if (!code) {
        alert('Please enter an invite code');
        return;
      }

      submitBtn.disabled = true;
      submitBtn.textContent = 'Joining...';

      try {
        const res = await api.post(`/api/circles/join/${code}`, {});
        const joinedGroup = res.data;
        this.groups.push(joinedGroup);
        this.selectedId = joinedGroup.id;
        store.set('circle', joinedGroup);
        this.mode = 'list';
        this.render();
        window.location.reload();
      } catch (err) {
        alert(err.message || 'Failed to join group');
        submitBtn.disabled = false;
        submitBtn.textContent = 'Join Group';
      }
    });

    form.appendChild(submitBtn);
    return form;
  }

  /**
   * Select a group.
   * @private
   */
  _selectGroup(group) {
    this.selectedId = group.id;
    store.set('circle', group);
    this._renderList();
  }

  /**
   * Destroy the component.
   */
  destroy() {
    this.stopTracking();
    clearElement(this.container);
  }

  /**
   * Check if the user has a circle.
   * @returns {boolean}
   */
  hasCircle() {
    return !!this.currentCircle;
  }

  /**
   * Get the current circle.
   * @returns {Object|null}
   */
  getCircle() {
    return this.currentCircle;
  }

  /**
   * Get the circle members.
   * @returns {Array}
   */
  getMembers() {
    return this.members;
  }
}

export { FamilyGroups as CircleManager };
export default FamilyGroups;
