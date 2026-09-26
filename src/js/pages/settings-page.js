/**
 * SettingsPage — Profile, notifications, subscription, and logout.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { toggleTheme } from '../styles/theme.js';

export class SettingsPage {
  constructor(container) {
    this.container = container;
    this.user = store.get('user');
    this._render();
  }

  /**
   * Render the settings page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'settings-page';

    // Header
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('h1', { class: 'section-title' }, 'Settings'));
    this.container.appendChild(header);

    // Profile section
    const profileSection = createElement('div', { class: 'settings-section' });
    profileSection.appendChild(createElement('h3', { class: 'settings-section-title' }, 'Profile'));

    const profileCard = createElement('div', { class: 'card' });
    const profileHeader = createElement('div', { class: 'flex items-center gap-4' });
    const avatar = createElement('div', { class: 'avatar avatar-xl' });
    avatar.style.background = '#6366f1';
    avatar.textContent = this.user?.name?.charAt(0) || '?';
    profileHeader.appendChild(avatar);

    const profileInfo = createElement('div');
    profileInfo.appendChild(createElement('div', { class: 'font-semibold', style: { fontSize: '18px' } }, this.user?.name || 'User'));
    profileInfo.appendChild(createElement('div', { class: 'text-sm text-muted' }, this.user?.email || ''));
    profileHeader.appendChild(profileInfo);
    profileCard.appendChild(profileHeader);
    profileSection.appendChild(profileCard);
    this.container.appendChild(profileSection);

    // Notifications section
    const notifSection = createElement('div', { class: 'settings-section' });
    notifSection.appendChild(createElement('h3', { class: 'settings-section-title' }, 'Notifications'));

    const notifCard = createElement('div', { class: 'card' });
    this._addToggle(notifCard, 'Push Notifications', 'Receive alerts on your device', true);
    this._addToggle(notifCard, 'Location Alerts', 'Get notified when members arrive/leave places', true);
    this._addToggle(notifCard, 'SOS Alerts', 'Emergency notifications for your family', true);
    this._addToggle(notifCard, 'Medication Reminders', 'Daily medication schedule alerts', false);
    notifSection.appendChild(notifCard);
    this.container.appendChild(notifSection);

    // Appearance section
    const appearanceSection = createElement('div', { class: 'settings-section' });
    appearanceSection.appendChild(createElement('h3', { class: 'settings-section-title' }, 'Appearance'));

    const appearanceCard = createElement('div', { class: 'card' });
    const themeItem = createElement('div', { class: 'settings-item' });
    themeItem.appendChild(createElement('div', {},
      createElement('div', { class: 'settings-item-label' }, 'Dark Mode'),
      createElement('div', { class: 'settings-item-description' }, 'Switch between dark and light themes')
    ));
    const themeToggle = createElement('label', { class: 'toggle' });
    const themeCheckbox = createElement('input', { type: 'checkbox', checked: 'checked' });
    themeCheckbox.addEventListener('change', () => toggleTheme());
    themeToggle.appendChild(themeCheckbox);
    themeToggle.appendChild(createElement('span', { class: 'toggle-track' }));
    themeItem.appendChild(themeToggle);
    appearanceCard.appendChild(themeItem);
    appearanceSection.appendChild(appearanceCard);
    this.container.appendChild(appearanceSection);

    // Subscription section
    const subSection = createElement('div', { class: 'settings-section' });
    subSection.appendChild(createElement('h3', { class: 'settings-section-title' }, 'Subscription'));

    const subCard = createElement('div', { class: 'card' });
    const subInfo = createElement('div', { class: 'flex items-center justify-between' });
    subInfo.appendChild(createElement('div', {},
      createElement('div', { class: 'font-semibold' }, 'Free Plan'),
      createElement('div', { class: 'text-sm text-muted' }, 'Basic features · Up to 3 members')
    ));
    const upgradeBtn = createElement('button', { class: 'btn btn-primary btn-sm' }, 'Upgrade');
    upgradeBtn.addEventListener('click', () => {
      window.location.href = '#/settings';
    });
    subInfo.appendChild(upgradeBtn);
    subCard.appendChild(subInfo);
    subSection.appendChild(subCard);
    this.container.appendChild(subSection);

    // Logout
    const logoutSection = createElement('div', { class: 'settings-section' });
    const logoutBtn = createElement('button', { class: 'btn btn-danger btn-block' }, 'Log Out');
    logoutBtn.addEventListener('click', () => this._logout());
    logoutSection.appendChild(logoutBtn);
    this.container.appendChild(logoutSection);
  }

  /**
   * Add a toggle switch item.
   * @private
   */
  _addToggle(parent, label, description, defaultChecked) {
    const item = createElement('div', { class: 'settings-item' });
    item.appendChild(createElement('div', {},
      createElement('div', { class: 'settings-item-label' }, label),
      createElement('div', { class: 'settings-item-description' }, description)
    ));

    const toggle = createElement('label', { class: 'toggle' });
    const checkbox = createElement('input', { type: 'checkbox' });
    if (defaultChecked) checkbox.checked = true;
    toggle.appendChild(checkbox);
    toggle.appendChild(createElement('span', { class: 'toggle-track' }));
    item.appendChild(toggle);
    parent.appendChild(item);
  }

  /**
   * Handle logout.
   * @private
   */
  async _logout() {
    try {
      await api.post('/auth/logout');
    } catch (err) {
      console.error('[SettingsPage] Logout error:', err);
    }
    api.clearTokens();
    store.set('user', null);
    store.set('circle', null);
    window.location.hash = '#/login';
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default SettingsPage;
