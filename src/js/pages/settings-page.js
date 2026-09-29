/**
 * SettingsPage — Profile, notifications, appearance, and logout.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { router } from '../router.js';

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

    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('h1', { class: 'section-title' }, 'Account'));
    this.container.appendChild(header);

    if (!this.user) {
      const empty = createElement('div', { class: 'empty-state' });
      empty.appendChild(createElement('div', { class: 'empty-state-title' }, 'Not logged in'));
      empty.appendChild(createElement('div', { class: 'empty-state-text' }, 'Please sign in to view your account.'));
      const loginBtn = createElement('button', { class: 'btn btn-primary mt-4' }, 'Go to Login');
      loginBtn.addEventListener('click', () => router.navigate('/login'));
      empty.appendChild(loginBtn);
      this.container.appendChild(empty);
      return;
    }

    // Profile section
    const profileSection = createElement('div', { class: 'settings-section' });
    profileSection.appendChild(createElement('h3', { class: 'settings-section-title' }, 'Profile'));

    const profileCard = createElement('div', { class: 'card' });
    const profileHeader = createElement('div', { class: 'flex items-center gap-4' });
    const avatar = createElement('div', { class: 'avatar avatar-xl' });
    avatar.style.background = '#6366f1';
    avatar.textContent = (this.user.display_name || this.user.email || '?').charAt(0).toUpperCase();
    profileHeader.appendChild(avatar);

    const profileInfo = createElement('div');
    profileInfo.appendChild(createElement('div', { class: 'font-semibold', style: { fontSize: '18px' } }, this.user.display_name || 'User'));
    profileInfo.appendChild(createElement('div', { class: 'text-sm text-muted' }, this.user.email || ''));
    profileHeader.appendChild(profileInfo);
    profileCard.appendChild(profileHeader);

    // Edit profile button
    const editBtn = createElement('button', { class: 'btn btn-secondary btn-sm', style: { marginTop: '12px' } }, 'Edit Profile');
    editBtn.addEventListener('click', () => this._showEditProfile());
    profileCard.appendChild(editBtn);

    profileSection.appendChild(profileCard);
    this.container.appendChild(profileSection);

    // Two-Factor Authentication section
    const totpSection = createElement('div', { class: 'settings-section' });
    totpSection.appendChild(createElement('h3', { class: 'settings-section-title' }, 'Two-Factor Authentication'));

    const totpCard = createElement('div', { class: 'card' });
    const totpItem = createElement('div', { class: 'settings-item' });
    totpItem.appendChild(createElement('div', {},
      createElement('div', { class: 'settings-item-label' }, '2FA Status'),
      createElement('div', { class: 'settings-item-description' },
        this.user.totp_enabled ? 'Enabled' : 'Disabled')
    ));

    const totpToggle = createElement('label', { class: 'toggle' });
    const totpCheckbox = createElement('input', { type: 'checkbox' });
    if (this.user.totp_enabled) totpCheckbox.checked = true;
    totpCheckbox.addEventListener('change', () => {
      if (totpCheckbox.checked) {
        this._setupTotp();
      } else {
        this._disableTotp();
      }
    });
    totpToggle.appendChild(totpCheckbox);
    totpToggle.appendChild(createElement('span', { class: 'toggle-track' }));
    totpItem.appendChild(totpToggle);
    totpCard.appendChild(totpItem);
    totpSection.appendChild(totpCard);
    this.container.appendChild(totpSection);

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
    themeCheckbox.addEventListener('change', () => {
      import('../styles/theme.js').then(m => m.toggleTheme());
    });
    themeToggle.appendChild(themeCheckbox);
    themeToggle.appendChild(createElement('span', { class: 'toggle-track' }));
    themeItem.appendChild(themeToggle);
    appearanceCard.appendChild(themeItem);
    appearanceSection.appendChild(appearanceCard);
    this.container.appendChild(appearanceSection);

    // Account section
    const accountSection = createElement('div', { class: 'settings-section' });
    accountSection.appendChild(createElement('h3', { class: 'settings-section-title' }, 'Account'));

    const accountCard = createElement('div', { class: 'card' });
    accountCard.appendChild(createElement('div', { class: 'settings-item' },
      createElement('div', {},
        createElement('div', { class: 'settings-item-label' }, 'User ID'),
        createElement('div', { class: 'settings-item-description' }, this.user.id || '')
      )
    ));
    if (this.user.phone) {
      accountCard.appendChild(createElement('div', { class: 'settings-item' },
        createElement('div', {},
          createElement('div', { class: 'settings-item-label' }, 'Phone'),
          createElement('div', { class: 'settings-item-description' }, this.user.phone)
        )
      ));
    }
    if (this.user.created_at) {
      accountCard.appendChild(createElement('div', { class: 'settings-item' },
        createElement('div', {},
          createElement('div', { class: 'settings-item-label' }, 'Member Since'),
          createElement('div', { class: 'settings-item-description' }, new Date(this.user.created_at).toLocaleDateString())
        )
      ));
    }
    accountSection.appendChild(accountCard);
    this.container.appendChild(accountSection);

    // Logout
    const logoutSection = createElement('div', { class: 'settings-section' });
    const logoutBtn = createElement('button', { class: 'btn btn-danger btn-block' }, 'Log Out');
    logoutBtn.addEventListener('click', () => this._logout());
    logoutSection.appendChild(logoutBtn);
    this.container.appendChild(logoutSection);
  }

  /**
   * Show edit profile modal.
   * @private
   */
  _showEditProfile() {
    const modal = createElement('div', { class: 'modal-backdrop' });
    const dialog = createElement('div', { class: 'modal' });

    const header = createElement('div', { class: 'modal-header' });
    header.appendChild(createElement('h3', { class: 'modal-title' }, 'Edit Profile'));
    const closeBtn = createElement('button', { class: 'btn btn-ghost btn-sm' }, '×');
    closeBtn.addEventListener('click', () => modal.remove());
    header.appendChild(closeBtn);
    dialog.appendChild(header);

    const body = createElement('div', { class: 'modal-body' });

    const nameGroup = createElement('div', { class: 'form-group' });
    nameGroup.appendChild(createElement('label', { class: 'form-label' }, 'Display Name'));
    const nameInput = createElement('input', { class: 'form-input', type: 'text', value: this.user?.display_name || '' });
    nameGroup.appendChild(nameInput);
    body.appendChild(nameGroup);

    const phoneGroup = createElement('div', { class: 'form-group' });
    phoneGroup.appendChild(createElement('label', { class: 'form-label' }, 'Phone'));
    const phoneInput = createElement('input', { class: 'form-input', type: 'tel', value: this.user?.phone || '' });
    phoneGroup.appendChild(phoneInput);
    body.appendChild(phoneGroup);

    dialog.appendChild(body);

    const footer = createElement('div', { class: 'modal-footer' });
    const cancelBtn = createElement('button', { class: 'btn btn-secondary' }, 'Cancel');
    cancelBtn.addEventListener('click', () => modal.remove());
    footer.appendChild(cancelBtn);

    const saveBtn = createElement('button', { class: 'btn btn-primary' }, 'Save');
    saveBtn.addEventListener('click', async () => {
      try {
        const res = await api.patch('/api/users/me', {
          display_name: nameInput.value.trim(),
          phone: phoneInput.value.trim(),
        });
        this.user = res.data;
        store.set('user', this.user);
        modal.remove();
        this._render();
      } catch (e) {
        alert(e.message || 'Failed to update profile');
      }
    });
    footer.appendChild(saveBtn);
    dialog.appendChild(footer);

    modal.appendChild(dialog);
    document.body.appendChild(modal);
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
   * Setup TOTP for the current user.
   * @private
   */
  async _setupTotp() {
    try {
      const res = await api.post('/api/auth/totp/setup', { user_id: this.user.id });
      const { secret, qr_code_uri } = res.data;

      const modal = createElement('div', { class: 'modal-backdrop' });
      const dialog = createElement('div', { class: 'modal' });

      const header = createElement('div', { class: 'modal-header' });
      header.appendChild(createElement('h3', { class: 'modal-title' }, 'Setup 2FA'));
      const closeBtn = createElement('button', { class: 'btn btn-ghost btn-sm' }, '×');
      closeBtn.addEventListener('click', () => modal.remove());
      header.appendChild(closeBtn);
      dialog.appendChild(header);

      const body = createElement('div', { class: 'modal-body' });

      body.appendChild(createElement('p', { class: 'text-sm text-muted mb-4' },
        'Scan this QR code with your authenticator app (Google Authenticator, Authy, etc.)'));

      // QR Code placeholder
      const qrContainer = createElement('div', { class: 'flex justify-center mb-4' });
      const qrImg = createElement('img', {
        src: `https://api.qrserver.com/v1/create-qr-code/?size=200x200&data=${encodeURIComponent(qr_code_uri)}`,
        alt: 'QR Code',
        style: { width: '200px', height: '200px' }
      });
      qrContainer.appendChild(qrImg);
      body.appendChild(qrContainer);

      // Secret display
      const secretGroup = createElement('div', { class: 'form-group' });
      secretGroup.appendChild(createElement('label', { class: 'form-label' }, 'Manual Entry Secret'));
      const secretInput = createElement('input', { class: 'form-input', type: 'text', value: secret, readonly: 'readonly' });
      secretGroup.appendChild(secretInput);
      body.appendChild(secretGroup);

      // OTP verification
      const otpGroup = createElement('div', { class: 'form-group' });
      otpGroup.appendChild(createElement('label', { class: 'form-label' }, 'Enter 6-digit code to verify'));
      const otpInput = createElement('input', { class: 'form-input', type: 'text', maxlength: '6', placeholder: '000000' });
      otpGroup.appendChild(otpInput);
      body.appendChild(otpGroup);

      dialog.appendChild(body);

      const footer = createElement('div', { class: 'modal-footer' });
      const cancelBtn = createElement('button', { class: 'btn btn-secondary' }, 'Cancel');
      cancelBtn.addEventListener('click', () => modal.remove());
      footer.appendChild(cancelBtn);

      const verifyBtn = createElement('button', { class: 'btn btn-primary' }, 'Verify & Enable');
      verifyBtn.addEventListener('click', async () => {
        try {
          await api.post('/api/auth/totp/verify', { user_id: this.user.id, otp: otpInput.value.trim() });
          this.user.totp_enabled = true;
          store.set('user', this.user);
          modal.remove();
          this._render();
        } catch (e) {
          alert(e.message || 'Verification failed');
        }
      });
      footer.appendChild(verifyBtn);
      dialog.appendChild(footer);

      modal.appendChild(dialog);
      document.body.appendChild(modal);
    } catch (e) {
      alert(e.message || 'Failed to setup 2FA');
    }
  }

  /**
   * Disable TOTP for the current user.
   * @private
   */
  async _disableTotp() {
    const otp = prompt('Enter your 6-digit code to disable 2FA:');
    if (!otp) return;

    try {
      await api.post('/api/auth/totp/disable', { user_id: this.user.id, otp });
      this.user.totp_enabled = false;
      store.set('user', this.user);
      this._render();
    } catch (e) {
      alert(e.message || 'Failed to disable 2FA');
    }
  }

  /**
   * Handle logout.
   * @private
   */
  async _logout() {
    try {
      await api.post('/api/auth/logout');
    } catch (err) {
      console.error('[SettingsPage] Logout error:', err);
    }
    api.clearTokens();
    store.set('user', null);
    store.set('circle', null);
    router.navigate('/login');
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default SettingsPage;
