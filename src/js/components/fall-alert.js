/**
 * FallAlert — Full-screen fall detection alert with audible prompt and countdown.
 *
 * When a fall is detected, this component:
 * 1. Plays an audible "Are you OK?" prompt
 * 2. Shows a 30-second countdown
 * 3. If no response, escalates to emergency contacts
 */

import { createElement, clearElement } from '../lib/dom.js';

const ESCALATION_SECONDS = 30;
const AUDIO_FREQUENCY = 440; // Hz — audible tone

export class FallAlert {
  constructor(container, alertData, onAcknowledge, onEscalate) {
    this.container = container;
    this.alertData = alertData;
    this.onAcknowledge = onAcknowledge;
    this.onEscalate = onEscalate;
    this.countdown = ESCALATION_SECONDS;
    this.timer = null;
    this.audioCtx = null;
    this._render();
    this._startAudio();
    this._startCountdown();
  }

  /**
   * Render the fall alert overlay.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'fall-alert-overlay';

    this.el = createElement('div', {
      class: 'fall-alert',
      role: 'alert',
      'aria-live': 'assertive',
    });

    // Icon
    const icon = createElement('div', { class: 'fall-alert-icon' }, '⚠');

    // Title
    const title = createElement('div', { class: 'fall-alert-title' }, 'Fall Detected');

    // Message
    const message = createElement('div', { class: 'fall-alert-message' },
      `${this.alertData.userName || 'A family member'} may have fallen. Are they OK?`);

    // Countdown
    this.countdownEl = createElement('div', { class: 'fall-alert-countdown' },
      `${this.countdown}s`);

    // Progress bar
    this.progressBar = createElement('div', { class: 'fall-alert-progress' });
    this.progressFill = createElement('div', { class: 'fall-alert-progress-fill' });
    this.progressBar.appendChild(this.progressFill);

    // Actions
    const actions = createElement('div', { class: 'fall-alert-actions' });

    this.okBtn = createElement('button', { class: 'btn btn-success btn-lg' }, "I'm OK");
    this.okBtn.addEventListener('click', () => this._acknowledge());

    this.helpBtn = createElement('button', { class: 'btn btn-danger btn-lg' }, 'Need Help');
    this.helpBtn.addEventListener('click', () => this._escalate());

    actions.appendChild(this.okBtn);
    actions.appendChild(this.helpBtn);

    this.el.appendChild(icon);
    this.el.appendChild(title);
    this.el.appendChild(message);
    this.el.appendChild(this.countdownEl);
    this.el.appendChild(this.progressBar);
    this.el.appendChild(actions);

    this.container.appendChild(this.el);
  }

  /**
   * Start the audible alert tone.
   * @private
   */
  _startAudio() {
    try {
      this.audioCtx = new (window.AudioContext || window.webkitAudioContext)();
      this._playAlertTone();
    } catch (e) {
      console.error('[FallAlert] Audio not supported:', e);
    }
  }

  /**
   * Play an audible alert tone.
   * @private
   */
  _playAlertTone() {
    if (!this.audioCtx) return;

    const oscillator = this.audioCtx.createOscillator();
    const gainNode = this.audioCtx.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(this.audioCtx.destination);

    oscillator.frequency.value = AUDIO_FREQUENCY;
    oscillator.type = 'sine';
    gainNode.gain.value = 0.3;

    // Beep pattern: 3 short beeps
    const now = this.audioCtx.currentTime;
    for (let i = 0; i < 3; i++) {
      oscillator.start(now + i * 0.4);
      oscillator.stop(now + i * 0.4 + 0.2);
    }

    // Repeat every 3 seconds
    this.audioInterval = setInterval(() => {
      if (this.audioCtx) {
        const osc = this.audioCtx.createOscillator();
        const gain = this.audioCtx.createGain();
        osc.connect(gain);
        gain.connect(this.audioCtx.destination);
        osc.frequency.value = AUDIO_FREQUENCY;
        osc.type = 'sine';
        gain.gain.value = 0.3;
        const t = this.audioCtx.currentTime;
        osc.start(t);
        osc.stop(t + 0.2);
      }
    }, 3000);
  }

  /**
   * Start the escalation countdown.
   * @private
   */
  _startCountdown() {
    const startTime = Date.now();
    const duration = ESCALATION_SECONDS * 1000;

    this.timer = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const remaining = Math.max(0, Math.ceil((duration - elapsed) / 1000));
      this.countdown = remaining;

      if (this.countdownEl) {
        this.countdownEl.textContent = `${remaining}s`;
      }
      if (this.progressFill) {
        const pct = Math.max(0, (1 - elapsed / duration) * 100);
        this.progressFill.style.width = `${pct}%`;
      }

      if (remaining <= 0) {
        this._escalate();
      }
    }, 100);
  }

  /**
   * User acknowledges they are OK.
   * @private
   */
  _acknowledge() {
    this._cleanup();
    if (this.onAcknowledge) {
      this.onAcknowledge(this.alertData.alert_id);
    }
    this.destroy();
  }

  /**
   * Escalate — user needs help or countdown expired.
   * @private
   */
  _escalate() {
    this._cleanup();
    if (this.onEscalate) {
      this.onEscalate(this.alertData.alert_id);
    }
    this.destroy();
  }

  /**
   * Clean up audio and timers.
   * @private
   */
  _cleanup() {
    clearInterval(this.timer);
    clearInterval(this.audioInterval);
    if (this.audioCtx) {
      this.audioCtx.close().catch(() => {});
      this.audioCtx = null;
    }
  }

  /**
   * Destroy the component.
   */
  destroy() {
    this._cleanup();
    clearElement(this.container);
  }
}

export default FallAlert;
