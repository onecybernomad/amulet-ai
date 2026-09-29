/**
 * LoginPage — Email/password login with link to signup.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { router } from '../router.js';

export class LoginPage {
  constructor(container) {
    this.container = container;
    this.error = null;
    this.loading = false;
    this._render();
  }

  /**
   * Render the login page.
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
    brand.appendChild(createElement('p', { class: 'auth-subtitle' }, 'Family safety, together'));
    wrapper.appendChild(brand);

    // Card
    const card = createElement('div', { class: 'card auth-card' });

    // Error message
    if (this.error) {
      const errorEl = createElement('div', { class: 'form-error' }, this.error);
      card.appendChild(errorEl);
    }

    // Email
    const emailGroup = createElement('div', { class: 'form-group' });
    emailGroup.appendChild(createElement('label', { class: 'form-label', for: 'login-email' }, 'Email'));
    this.emailInput = createElement('input', {
      class: 'form-input',
      type: 'email',
      id: 'login-email',
      placeholder: 'you@example.com',
      autocomplete: 'email',
    });
    emailGroup.appendChild(this.emailInput);
    card.appendChild(emailGroup);

    // Password
    const passwordGroup = createElement('div', { class: 'form-group' });
    passwordGroup.appendChild(createElement('label', { class: 'form-label', for: 'login-password' }, 'Password'));
    this.passwordInput = createElement('input', {
      class: 'form-input',
      type: 'password',
      id: 'login-password',
      placeholder: 'Enter your password',
      autocomplete: 'current-password',
    });
    passwordGroup.appendChild(this.passwordInput);
    card.appendChild(passwordGroup);

    // Submit
    this.submitBtn = createElement('button', {
      class: 'btn btn-primary btn-block',
      disabled: this.loading ? 'disabled' : null,
    }, this.loading ? 'Signing in...' : 'Sign In');
    this.submitBtn.addEventListener('click', () => this._login());
    card.appendChild(this.submitBtn);

    // Signup link
    const signupLink = createElement('div', { class: 'auth-footer' });
    signupLink.appendChild(createElement('span', {}, 'Don\'t have an account? '));
    const link = createElement('a', { href: '#/signup' }, 'Sign up');
    signupLink.appendChild(link);
    card.appendChild(signupLink);

    wrapper.appendChild(card);
    this.container.appendChild(wrapper);
  }

  /**
   * Handle login submission.
   * @private
   */
  async _login() {
    const email = this.emailInput.value.trim();
    const password = this.passwordInput.value;

    this.error = null;

    if (!email || !password) {
      this.error = 'Please fill in all fields';
      this._render();
      return;
    }

    this.loading = true;
    this._render();

    try {
      const res = await api.post('/api/auth/login', { email, password });
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
      this.error = err.message || 'Login failed. Please check your credentials.';
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

export default LoginPage;
