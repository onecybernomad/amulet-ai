/**
 * SignupPage — Account registration with email/password.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { router } from '../router.js';

export class SignupPage {
  constructor(container) {
    this.container = container;
    this.error = null;
    this.loading = false;
    this._render();
  }

  /**
   * Render the signup page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'auth-page';

    const wrapper = createElement('div', { class: 'auth-wrapper' });

    // Logo / Brand
    const brand = createElement('div', { class: 'auth-brand' });
    brand.appendChild(createElement('div', { class: 'auth-logo' }, 'A'));
    brand.appendChild(createElement('h1', { class: 'auth-title' }, 'Amulet AI'));
    brand.appendChild(createElement('p', { class: 'auth-subtitle' }, 'Create your account'));
    wrapper.appendChild(brand);

    // Card
    const card = createElement('div', { class: 'card auth-card' });

    // Error message
    if (this.error) {
      const errorEl = createElement('div', { class: 'form-error' }, this.error);
      card.appendChild(errorEl);
    }

    // Display name
    const nameGroup = createElement('div', { class: 'form-group' });
    nameGroup.appendChild(createElement('label', { class: 'form-label', for: 'signup-name' }, 'Display Name'));
    this.nameInput = createElement('input', {
      class: 'form-input',
      type: 'text',
      id: 'signup-name',
      placeholder: 'Your name',
      autocomplete: 'name',
    });
    nameGroup.appendChild(this.nameInput);
    card.appendChild(nameGroup);

    // Email
    const emailGroup = createElement('div', { class: 'form-group' });
    emailGroup.appendChild(createElement('label', { class: 'form-label', for: 'signup-email' }, 'Email'));
    this.emailInput = createElement('input', {
      class: 'form-input',
      type: 'email',
      id: 'signup-email',
      placeholder: 'you@example.com',
      autocomplete: 'email',
    });
    emailGroup.appendChild(this.emailInput);
    card.appendChild(emailGroup);

    // Password
    const passwordGroup = createElement('div', { class: 'form-group' });
    passwordGroup.appendChild(createElement('label', { class: 'form-label', for: 'signup-password' }, 'Password'));
    this.passwordInput = createElement('input', {
      class: 'form-input',
      type: 'password',
      id: 'signup-password',
      placeholder: 'At least 8 characters',
      autocomplete: 'new-password',
    });
    passwordGroup.appendChild(this.passwordInput);
    card.appendChild(passwordGroup);

    // Confirm password
    const confirmGroup = createElement('div', { class: 'form-group' });
    confirmGroup.appendChild(createElement('label', { class: 'form-label', for: 'signup-confirm' }, 'Confirm Password'));
    this.confirmInput = createElement('input', {
      class: 'form-input',
      type: 'password',
      id: 'signup-confirm',
      placeholder: 'Re-enter your password',
      autocomplete: 'new-password',
    });
    confirmGroup.appendChild(this.confirmInput);
    card.appendChild(confirmGroup);

    // Submit
    this.submitBtn = createElement('button', {
      class: 'btn btn-primary btn-block',
      disabled: this.loading ? 'disabled' : null,
    }, this.loading ? 'Creating account...' : 'Create Account');
    this.submitBtn.addEventListener('click', () => this._signup());
    card.appendChild(this.submitBtn);

    // Login link
    const loginLink = createElement('div', { class: 'auth-footer' });
    loginLink.appendChild(createElement('span', {}, 'Already have an account? '));
    const link = createElement('a', { href: '#/login' }, 'Sign in');
    loginLink.appendChild(link);
    card.appendChild(loginLink);

    wrapper.appendChild(card);
    this.container.appendChild(wrapper);
  }

  /**
   * Handle signup submission.
   * @private
   */
  async _signup() {
    const name = this.nameInput.value.trim();
    const email = this.emailInput.value.trim();
    const password = this.passwordInput.value;
    const confirm = this.confirmInput.value;

    this.error = null;

    // Validation
    if (!name || !email || !password || !confirm) {
      this.error = 'Please fill in all fields';
      this._render();
      return;
    }

    if (!email.includes('@')) {
      this.error = 'Please enter a valid email address';
      this._render();
      return;
    }

    if (password.length < 8) {
      this.error = 'Password must be at least 8 characters';
      this._render();
      return;
    }

    if (password !== confirm) {
      this.error = 'Passwords do not match';
      this._render();
      return;
    }

    this.loading = true;
    this._render();

    try {
      const res = await api.post('/api/auth/register', { display_name: name, email, password });
      api.setTokens(res.data);

      // Store user
      store.set('user', res.data.user);

      // Connect WebSocket
      const { ws } = await import('../ws.js');
      ws.connect();

      // Navigate to map
      router.navigate('/map');
    } catch (err) {
      this.loading = false;
      this.error = err.message || 'Signup failed. Please try again.';
      this._render();
    }
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default SignupPage;
