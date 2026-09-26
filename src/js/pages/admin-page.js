/**
 * AdminPage — Circle management, member roles, invite codes, and data export.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { getInitials } from '../lib/format.js';

export class AdminPage {
  constructor(container) {
    this.container = container;
    this.circle = store.get('circle');
    this.members = store.get('members') || [];
    this._render();
  }

  /**
   * Render the admin page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'admin-page';

    // Header
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('h1', { class: 'section-title' }, 'Admin Panel'));
    this.container.appendChild(header);

    const grid = createElement('div', { class: 'admin-grid' });

    // Circle management
    grid.appendChild(this._renderCircleCard());

    // Member management
    grid.appendChild(this._renderMembersCard());

    // Subscription
    grid.appendChild(this._renderSubscriptionCard());

    // Data export
    grid.appendChild(this._renderDataCard());

    this.container.appendChild(grid);
  }

  /**
   * Render circle management card.
   * @private
   */
  _renderCircleCard() {
    const card = createElement('div', { class: 'card' });
    card.appendChild(createElement('h3', { class: 'card-title' }, 'Circle Management'));

    const body = createElement('div', { class: 'card-body' });

    if (this.circle) {
      body.appendChild(createElement('div', { class: 'mb-2' },
        createElement('strong', {}, 'Name: '), this.circle.name
      ));
      body.appendChild(createElement('div', { class: 'mb-2' },
        createElement('strong', {}, 'Members: '), String(this.members.length)
      ));
    }

    // Invite code
    const inviteGroup = createElement('div', { class: 'form-group' });
    inviteGroup.appendChild(createElement('label', { class: 'form-label' }, 'Invite Code'));
    const inviteRow = createElement('div', { class: 'flex gap-2' });
    const inviteInput = createElement('input', {
      class: 'form-input',
      type: 'text',
      value: this.circle?.inviteCode || '------',
      readonly: 'readonly',
    });
    const copyBtn = createElement('button', { class: 'btn btn-sm btn-secondary' }, 'Copy');
    copyBtn.addEventListener('click', () => {
      navigator.clipboard.writeText(this.circle?.inviteCode || '');
      copyBtn.textContent = 'Copied!';
      setTimeout(() => { copyBtn.textContent = 'Copy'; }, 2000);
    });
    inviteRow.appendChild(inviteInput);
    inviteRow.appendChild(copyBtn);
    inviteGroup.appendChild(inviteRow);
    body.appendChild(inviteGroup);

    card.appendChild(body);
    return card;
  }

  /**
   * Render member management card.
   * @private
   */
  _renderMembersCard() {
    const card = createElement('div', { class: 'card' });
    card.appendChild(createElement('h3', { class: 'card-title' }, 'Members'));

    const body = createElement('div', { class: 'card-body' });

    for (const member of this.members) {
      const item = createElement('div', { class: 'flex items-center gap-3', style: { padding: '8px 0', borderBottom: '1px solid var(--color-border)' } });

      const avatar = createElement('div', { class: 'avatar avatar-sm' });
      avatar.style.background = member.color || '#6366f1';
      avatar.textContent = getInitials(member.name);

      const info = createElement('div', { style: { flex: 1 } });
      info.appendChild(createElement('div', { class: 'font-medium', style: { fontSize: '14px' } }, member.name));
      info.appendChild(createElement('div', { class: 'text-xs text-muted' }, member.email || ''));

      const roleBadge = createElement('span', {
        class: `badge ${member.role === 'admin' ? 'badge-primary' : 'badge-neutral'}`,
      }, member.role || 'member');

      item.appendChild(avatar);
      item.appendChild(info);
      item.appendChild(roleBadge);
      body.appendChild(item);
    }

    card.appendChild(body);
    return card;
  }

  /**
   * Render subscription card.
   * @private
   */
  _renderSubscriptionCard() {
    const card = createElement('div', { class: 'card' });
    card.appendChild(createElement('h3', { class: 'card-title' }, 'Subscription'));

    const body = createElement('div', { class: 'card-body' });
    body.appendChild(createElement('div', { class: 'mb-2' },
      createElement('strong', {}, 'Plan: '), 'Free'
    ));
    body.appendChild(createElement('div', { class: 'mb-2' },
      createElement('strong', {}, 'Status: '), 'Active'
    ));
    body.appendChild(createElement('div', {},
      createElement('strong', {}, 'Members: '), `${this.members.length} / 3`
    ));

    const footer = createElement('div', { class: 'card-footer' });
    const upgradeBtn = createElement('button', { class: 'btn btn-primary btn-sm' }, 'Upgrade Plan');
    upgradeBtn.addEventListener('click', () => {
      window.location.hash = '#/settings';
    });
    footer.appendChild(upgradeBtn);
    card.appendChild(footer);

    return card;
  }

  /**
   * Render data export card.
   * @private
   */
  _renderDataCard() {
    const card = createElement('div', { class: 'card' });
    card.appendChild(createElement('h3', { class: 'card-title' }, 'Data'));

    const body = createElement('div', { class: 'card-body' });
    body.appendChild(createElement('p', { class: 'text-sm text-muted', style: { marginBottom: '12px' } },
      'Export all your family data including locations, messages, and settings.'
    ));

    const exportBtn = createElement('button', { class: 'btn btn-secondary btn-sm' }, 'Export Data (JSON)');
    exportBtn.addEventListener('click', () => this._exportData());
    body.appendChild(exportBtn);

    card.appendChild(body);
    return card;
  }

  /**
   * Export all data as JSON.
   * @private
   */
  async _exportData() {
    try {
      const data = await api.get('/export');
      const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json' });
      const url = URL.createObjectURL(blob);
      const a = createElement('a', { href: url, download: `amulet-export-${Date.now()}.json` });
      a.click();
      URL.revokeObjectURL(url);
    } catch (err) {
      console.error('[AdminPage] Export failed:', err);
    }
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default AdminPage;
