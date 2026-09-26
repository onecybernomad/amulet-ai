/**
 * SOSButton — Large pulsing red button with hold-to-activate.
 */

import { createElement } from '../lib/dom.js';

const HOLD_DURATION = 2000; // 2 seconds
const COUNTDOWN_INTERVAL = 50;

export class SOSButton {
  constructor(container, onActivate) {
    this.container = container;
    this.onActivate = onActivate;
    this.holdTimer = null;
    this.countdownTimer = null;
    this.remaining = HOLD_DURATION;
    this.isActivated = false;
    this._render();
  }

  /**
   * Render the SOS button.
   * @private
   */
  _render() {
    this.wrapper = createElement('div', { class: 'sos-button-wrapper' });

    this.button = createElement('button', {
      class: 'sos-button',
      'aria-label': 'Emergency SOS - Hold for 2 seconds',
    }, 'SOS');

    this.button.addEventListener('mousedown', (e) => this._startHold(e));
    this.button.addEventListener('touchstart', (e) => this._startHold(e), { passive: true });
    this.button.addEventListener('mouseup', () => this._cancelHold());
    this.button.addEventListener('mouseleave', () => this._cancelHold());
    this.button.addEventListener('touchend', () => this._cancelHold());

    this.wrapper.appendChild(this.button);
    this.container.appendChild(this.wrapper);
  }

  /**
   * Start the hold timer.
   * @private
   */
  _startHold(e) {
    if (this.isActivated) return;
    e.preventDefault();

    this.button.classList.add('activating');
    this.remaining = HOLD_DURATION;

    // Show countdown overlay
    this.countdownEl = createElement('div', { class: 'sos-countdown' }, '2');
    this.button.appendChild(this.countdownEl);

    const startTime = Date.now();
    this.countdownTimer = setInterval(() => {
      this.remaining = HOLD_DURATION - (Date.now() - startTime);
      const seconds = Math.ceil(this.remaining / 1000);
      if (this.countdownEl) this.countdownEl.textContent = seconds;

      if (this.remaining <= 0) {
        this._activate();
      }
    }, COUNTDOWN_INTERVAL);
  }

  /**
   * Cancel the hold.
   * @private
   */
  _cancelHold() {
    if (this.isActivated) return;
    this.button.classList.remove('activating');
    clearInterval(this.countdownTimer);
    if (this.countdownEl) {
      this.countdownEl.remove();
      this.countdownEl = null;
    }
  }

  /**
   * Activate SOS.
   * @private
   */
  _activate() {
    this.isActivated = true;
    clearInterval(this.countdownTimer);
    this.button.classList.remove('activating');
    this.button.classList.add('activated');
    if (this.countdownEl) {
      this.countdownEl.remove();
      this.countdownEl = null;
    }

    // Show cancel option
    this.cancelBtn = createElement('button', { class: 'sos-cancel' }, 'Cancel');
    this.cancelBtn.addEventListener('click', () => this._deactivate());
    this.wrapper.appendChild(this.cancelBtn);

    // Trigger Tauri invoke or callback
    if (window.__TAURI__) {
      window.__TAURI__.invoke('trigger_sos').catch(err => {
        console.error('[SOS] Tauri invoke failed:', err);
      });
    }

    if (this.onActivate) this.onActivate();

    // Auto-deactivate after 10 seconds
    this.autoDeactivateTimer = setTimeout(() => this._deactivate(), 10000);
  }

  /**
   * Deactivate SOS.
   * @private
   */
  _deactivate() {
    this.isActivated = false;
    clearTimeout(this.autoDeactivateTimer);
    this.button.classList.remove('activated');
    if (this.cancelBtn) {
      this.cancelBtn.remove();
      this.cancelBtn = null;
    }
  }

  /**
   * Destroy the component.
   */
  destroy() {
    this._cancelHold();
    clearTimeout(this.autoDeactivateTimer);
    this.wrapper.remove();
  }
}

export default SOSButton;
