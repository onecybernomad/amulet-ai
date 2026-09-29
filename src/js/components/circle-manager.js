/**
 * CircleManager — Create, join, and manage family groups (circles).
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';

export class CircleManager {
  constructor(container) {
    this.container = container;
    this.circles = [];
    this.currentCircle = store.get('circle');
    this.members = [];
    this._render();
  }

  /**
   * Render the circle manager.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'circle-manager';

    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('h2', {}, 'Family Groups'));
    this.container.appendChild(header);

    // Current circle section
    if (this.currentCircle) {
      this._renderCurrentCircle();
    } else {
      this._renderNoCircle();
    }

    // Circle list
    this._renderCircleList();
  }

  /**
   * Render the current active circle.
   * @private
   */
  _renderCurrentCircle() {
    const section = createElement('div', { class: 'current-circle card' });
    section.style.marginBottom = '16px';

    const header = createElement('div', { class: 'card-header' });
    header.appendChild(createElement('h3', {}, this.currentCircle.name || 'Family Circle'));
    section.appendChild(header);

    const body = createElement('div', { class: 'card-body' });

    // Invite code
    const inviteRow = createElement('div', { class: 'flex items-center justify-between', style: { marginBottom: '12px' } });
    const inviteLabel = createElement('span', { class: 'text-sm text-muted' }, 'Invite Code:');
    const inviteCode = createElement('code', { class: 'badge badge-primary' }, this.currentCircle.invite_code || '—');
    inviteRow.appendChild(inviteLabel);
    inviteRow.appendChild(inviteCode);
    body.appendChild(inviteRow);

    // Members count
    const membersRow = createElement('div', { class: 'flex items-center justify-between' });
    const membersLabel = createElement('span', { class: 'text-sm text-muted' }, 'Members:');
    const membersCount = createElement('span', {}, `${this.members.length || 0} / ${this.currentCircle.max_members || 4}`);
    membersRow.appendChild(membersLabel);
    membersRow.appendChild(membersCount);
    body.appendChild(membersRow);

    // Actions
    const actions = createElement('div', { class: 'flex gap-2', style: { marginTop: '12px' } });

    const copyBtn = createElement('button', { class: 'btn btn-secondary btn-sm' }, 'Copy Invite');
    copyBtn.addEventListener('click', () => this._copyInviteCode());
    actions.appendChild(copyBtn);

    const leaveBtn = createElement('button', { class: 'btn btn-danger btn-sm' }, 'Leave');
    leaveBtn.addEventListener('click', () => this._leaveCircle());
    actions.appendChild(leaveBtn);

    body.appendChild(actions);
    section.appendChild(body);
    this.container.appendChild(section);
  }

  /**
   * Render when no circle is active.
   * @private
   */
  _renderNoCircle() {
    const section = createElement('div', { class: 'no-circle card' });
    section.style.marginBottom = '16px';

    const body = createElement('div', { class: 'card-body' });
    body.appendChild(createElement('p', { class: 'text-muted' }, 'You are not in any family group yet.'));

    const actions = createElement('div', { class: 'flex gap-2' });

    const createBtn = createElement('button', { class: 'btn btn-primary btn-sm' }, 'Create Group');
    createBtn.addEventListener('click', () => this._showCreateModal());
    actions.appendChild(createBtn);

    const joinBtn = createElement('button', { class: 'btn btn-secondary btn-sm' }, 'Join with Code');
    joinBtn.addEventListener('click', () => this._showJoinModal());
    actions.appendChild(joinBtn);

    body.appendChild(actions);
    section.appendChild(body);
    this.container.appendChild(section);
  }

  /**
   * Render the list of available circles.
   * @private
   */
  _renderCircleList() {
    if (this.circles.length === 0) return;

    const section = createElement('div', { class: 'circle-list' });
    section.appendChild(createElement('h3', { class: 'section-title' }, 'Your Groups'));

    const grid = createElement('div', { class: 'places-grid' });

    this.circles.forEach(circle => {
      const card = createElement('div', { class: 'card place-card' });
      const cardBody = createElement('div', { class: 'card-body' });
      cardBody.appendChild(createElement('h4', {}, circle.name));
      cardBody.appendChild(createElement('p', { class: 'text-sm text-muted' }, `${circle.member_count || 0} members`));

      const switchBtn = createElement('button', { class: 'btn btn-primary btn-sm btn-block' }, 'Switch to this group');
      switchBtn.addEventListener('click', () => this._switchCircle(circle));
      cardBody.appendChild(switchBtn);

      card.appendChild(cardBody);
      grid.appendChild(card);
    });

    section.appendChild(grid);
    this.container.appendChild(section);
  }

  /**
   * Show create circle modal.
   * @private
   */
  _showCreateModal() {
    const modal = createElement('div', { class: 'modal-backdrop' });
    const dialog = createElement('div', { class: 'modal' });

    const header = createElement('div', { class: 'modal-header' });
    header.appendChild(createElement('h3', {}, 'Create Family Group'));
    const closeBtn = createElement('button', { class: 'btn btn-ghost btn-sm' }, '×');
    closeBtn.addEventListener('click', () => modal.remove());
    header.appendChild(closeBtn);
    dialog.appendChild(header);

    const body = createElement('div', { class: 'modal-body' });

    const nameGroup = createElement('div', { class: 'form-group' });
    nameGroup.appendChild(createElement('label', { class: 'form-label' }, 'Group Name'));
    const nameInput = createElement('input', { class: 'form-input', type: 'text', placeholder: 'e.g. The Smith Family' });
    nameGroup.appendChild(nameInput);
    body.appendChild(nameGroup);

    const descGroup = createElement('div', { class: 'form-group' });
    descGroup.appendChild(createElement('label', { class: 'form-label' }, 'Description (optional)'));
    const descInput = createElement('textarea', { class: 'form-textarea', placeholder: 'What is this group for?' });
    descGroup.appendChild(descInput);
    body.appendChild(descGroup);

    dialog.appendChild(body);

    const footer = createElement('div', { class: 'modal-footer' });
    const cancelBtn = createElement('button', { class: 'btn btn-secondary' }, 'Cancel');
    cancelBtn.addEventListener('click', () => modal.remove());
    footer.appendChild(cancelBtn);

    const createBtn = createElement('button', { class: 'btn btn-primary' }, 'Create');
    createBtn.addEventListener('click', async () => {
      const name = nameInput.value.trim();
      if (!name) {
        alert('Please enter a group name');
        return;
      }
      try {
        const res = await api.post('/api/circles', { name });
        store.set('circle', res.data);
        modal.remove();
        this._render();
      } catch (e) {
        alert(e.message || 'Failed to create group');
      }
    });
    footer.appendChild(createBtn);
    dialog.appendChild(footer);

    modal.appendChild(dialog);
    document.body.appendChild(modal);
  }

  /**
   * Show join circle modal.
   * @private
   */
  _showJoinModal() {
    const modal = createElement('div', { class: 'modal-backdrop' });
    const dialog = createElement('div', { class: 'modal' });

    const header = createElement('div', { class: 'modal-header' });
    header.appendChild(createElement('h3', {}, 'Join Family Group'));
    const closeBtn = createElement('button', { class: 'btn btn-ghost btn-sm' }, '×');
    closeBtn.addEventListener('click', () => modal.remove());
    header.appendChild(closeBtn);
    dialog.appendChild(header);

    const body = createElement('div', { class: 'modal-body' });

    const codeGroup = createElement('div', { class: 'form-group' });
    codeGroup.appendChild(createElement('label', { class: 'form-label' }, 'Invite Code'));
    const codeInput = createElement('input', { class: 'form-input', type: 'text', placeholder: 'Enter invite code' });
    codeGroup.appendChild(codeInput);
    body.appendChild(codeGroup);

    dialog.appendChild(body);

    const footer = createElement('div', { class: 'modal-footer' });
    const cancelBtn = createElement('button', { class: 'btn btn-secondary' }, 'Cancel');
    cancelBtn.addEventListener('click', () => modal.remove());
    footer.appendChild(cancelBtn);

    const joinBtn = createElement('button', { class: 'btn btn-primary' }, 'Join');
    joinBtn.addEventListener('click', async () => {
      const code = codeInput.value.trim();
      if (!code) {
        alert('Please enter an invite code');
        return;
      }
      try {
        const res = await api.post(`/api/circles/join/${code}`, {});
        store.set('circle', res.data);
        modal.remove();
        this._render();
      } catch (e) {
        alert(e.message || 'Failed to join group');
      }
    });
    footer.appendChild(joinBtn);
    dialog.appendChild(footer);

    modal.appendChild(dialog);
    document.body.appendChild(modal);
  }

  /**
   * Copy invite code to clipboard.
   * @private
   */
  _copyInviteCode() {
    const code = this.currentCircle?.invite_code;
    if (code) {
      navigator.clipboard.writeText(code).then(() => {
        alert('Invite code copied to clipboard');
      });
    }
  }

  /**
   * Leave the current circle.
   * @private
   */
  async _leaveCircle() {
    if (!this.currentCircle) return;
    if (!confirm('Are you sure you want to leave this group?')) return;
    try {
      await api.post(`/api/circles/${this.currentCircle.id}/leave`, {});
      store.set('circle', null);
      this.currentCircle = null;
      this._render();
    } catch (e) {
      alert(e.message || 'Failed to leave group');
    }
  }

  /**
   * Switch to a different circle.
   * @private
   */
  async _switchCircle(circle) {
    try {
      store.set('circle', circle);
      this.currentCircle = circle;
      this._render();
    } catch (e) {
      alert(e.message || 'Failed to switch group');
    }
  }

  /**
   * Load circles from the server.
   */
  async loadCircles() {
    try {
      const res = await api.get('/api/circles');
      this.circles = res.data || [];
    } catch (e) {
      console.error('[CircleManager] Failed to load circles:', e);
      this.circles = [];
    }
  }

  /**
   * Load members for the current circle.
   */
  async loadMembers() {
    if (!this.currentCircle) return;
    try {
      const res = await api.get(`/api/circles/${this.currentCircle.id}/members`);
      this.members = res.data || [];
    } catch (e) {
      console.error('[CircleManager] Failed to load members:', e);
      this.members = [];
    }
  }

  /**
   * Destroy the component.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default CircleManager;
