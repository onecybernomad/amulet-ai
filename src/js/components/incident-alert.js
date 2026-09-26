/**
 * IncidentAlert — Modal overlay for crash/fall/SOS incident notifications.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { formatDateTime } from '../lib/format.js';

const AUTO_CLOSE_SECONDS = 30;

export class IncidentAlert {
  constructor(container, incident, onAcknowledge, onDismiss) {
    this.container = container;
    this.incident = incident;
    this.onAcknowledge = onAcknowledge;
    this.onDismiss = onDismiss;
    this.timer = null;
    this._render();
  }

  /**
   * Render the incident alert.
   * @private
   */
  _render() {
    clearElement(this.container);

    const typeLabels = { crash: 'Crash Detected', fall: 'Fall Detected', sos: 'SOS Alert' };
    const type = this.incident.type || 'sos';
    const isWarning = type === 'fall';

    this.el = createElement('div', {
      class: `incident-alert ${isWarning ? 'warning' : ''}`,
      role: 'alert',
      'aria-live': 'assertive',
    });

    // Header
    const header = createElement('div', { class: 'incident-alert-header' });
    const icon = createElement('div', { class: 'incident-alert-icon' }, '⚠');
    const titleGroup = createElement('div');
    titleGroup.appendChild(createElement('div', { class: 'incident-alert-title' }, typeLabels[type] || 'Incident'));
    titleGroup.appendChild(createElement('div', { class: 'incident-alert-subtitle' }, this.incident.userName || 'Unknown'));
    header.appendChild(icon);
    header.appendChild(titleGroup);
    this.el.appendChild(header);

    // Body
    const body = createElement('div', { class: 'incident-alert-body' });

    const locationRow = createElement('div', { class: 'incident-alert-detail' });
    locationRow.appendChild(createElement('span', {}, 'Location'));
    locationRow.appendChild(createElement('span', {}, this.incident.location || 'Unknown'));
    body.appendChild(locationRow);

    const timeRow = createElement('div', { class: 'incident-alert-detail' });
    timeRow.appendChild(createElement('span', {}, 'Time'));
    timeRow.appendChild(createElement('span', {}, formatDateTime(this.incident.timestamp)));
    body.appendChild(timeRow);

    if (this.incident.details) {
      const detailsRow = createElement('div', { class: 'incident-alert-detail' });
      detailsRow.appendChild(createElement('span', {}, 'Details'));
      detailsRow.appendChild(createElement('span', {}, this.incident.details));
      body.appendChild(detailsRow);
    }

    this.el.appendChild(body);

    // Actions
    const actions = createElement('div', { class: 'incident-alert-actions' });
    const ackBtn = createElement('button', { class: 'btn btn-primary btn-block' }, 'Acknowledge');
    ackBtn.addEventListener('click', () => this._acknowledge());
    actions.appendChild(ackBtn);

    const dismissBtn = createElement('button', { class: 'btn btn-ghost' }, 'Dismiss');
    dismissBtn.addEventListener('click', () => this._dismiss());
    actions.appendChild(dismissBtn);

    this.el.appendChild(actions);

    // Auto-close timer
    this.timerBar = createElement('div', { class: 'incident-alert-timer' });
    this.timerBarFill = createElement('div', { class: 'incident-alert-timer-bar' });
    this.timerBar.appendChild(this.timerBarFill);
    this.el.appendChild(this.timerBar);

    this.container.appendChild(this.el);

    // Start auto-close countdown
    this._startAutoClose();
  }

  /**
   * Start the auto-close countdown timer.
   * @private
   */
  _startAutoClose() {
    const startTime = Date.now();
    const duration = AUTO_CLOSE_SECONDS * 1000;

    this.timer = setInterval(() => {
      const elapsed = Date.now() - startTime;
      const remaining = Math.max(0, 1 - elapsed / duration);
      this.timerBarFill.style.width = `${remaining * 100}%`;

      if (remaining <= 0) {
        this._dismiss();
      }
    }, 100);
  }

  /**
   * Acknowledge the incident.
   * @private
   */
  _acknowledge() {
    clearInterval(this.timer);
    if (this.onAcknowledge) this.onAcknowledge(this.incident.id);
    this.destroy();
  }

  /**
   * Dismiss the incident.
   * @private
   */
  _dismiss() {
    clearInterval(this.timer);
    if (this.onDismiss) this.onDismiss(this.incident.id);
    this.destroy();
  }

  /**
   * Destroy the component.
   */
  destroy() {
    clearInterval(this.timer);
    clearElement(this.container);
  }
}

export default IncidentAlert;
