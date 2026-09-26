/**
 * DrivingReport — Card displaying driving session stats and safety score.
 */

import { createElement } from '../lib/dom.js';
import { formatDate, formatDistance, formatSpeed } from '../lib/format.js';

export class DrivingReport {
  constructor(session) {
    this.session = session;
    this.element = this._render();
  }

  /**
   * Render the driving report card.
   * @private
   */
  _render() {
    const s = this.session;
    const score = s.safetyScore != null ? s.safetyScore : this._calculateScore(s);
    const scoreClass = score >= 80 ? 'good' : score >= 50 ? 'moderate' : 'poor';

    const card = createElement('div', { class: 'driving-report-card' });

    // Header
    const header = createElement('div', { class: 'driving-report-header' });
    header.appendChild(createElement('div', { class: 'driving-report-date' }, formatDate(s.date)));

    const scoreEl = createElement('div', { class: `safety-score safety-score-${scoreClass}` }, String(score));
    header.appendChild(scoreEl);
    card.appendChild(header);

    // Stats
    const stats = createElement('div', { class: 'driving-stats' });

    const distanceStat = createElement('div', { class: 'driving-stat' });
    distanceStat.appendChild(createElement('div', { class: 'driving-stat-value' }, formatDistance(s.distance)));
    distanceStat.appendChild(createElement('div', { class: 'driving-stat-label' }, 'Distance'));
    stats.appendChild(distanceStat);

    const maxSpeedStat = createElement('div', { class: 'driving-stat' });
    maxSpeedStat.appendChild(createElement('div', { class: 'driving-stat-value' }, formatSpeed(s.maxSpeed)));
    maxSpeedStat.appendChild(createElement('div', { class: 'driving-stat-label' }, 'Max Speed'));
    stats.appendChild(maxSpeedStat);

    const avgSpeedStat = createElement('div', { class: 'driving-stat' });
    avgSpeedStat.appendChild(createElement('div', { class: 'driving-stat-value' }, formatSpeed(s.avgSpeed)));
    avgSpeedStat.appendChild(createElement('div', { class: 'driving-stat-label' }, 'Avg Speed'));
    stats.appendChild(avgSpeedStat);

    card.appendChild(stats);

    // Events
    if (s.events?.length) {
      const events = createElement('div', { class: 'driving-events' });
      for (const event of s.events) {
        const label = event.type.replace(/_/g, ' ');
        events.appendChild(createElement('span', { class: `driving-event-tag ${event.type}` }, label));
      }
      card.appendChild(events);
    }

    return card;
  }

  /**
   * Calculate a safety score from events.
   * @private
   */
  _calculateScore(session) {
    let score = 100;
    for (const event of session.events || []) {
      if (event.type === 'hard_brake') score -= 10;
      else if (event.type === 'rapid_accel') score -= 5;
      else if (event.type === 'phone_use') score -= 15;
    }
    return Math.max(0, score);
  }

  /**
   * Get the DOM element.
   * @returns {HTMLElement}
   */
  getElement() {
    return this.element;
  }

  /**
   * Destroy the component.
   */
  destroy() {
    this.element.remove();
  }
}

export default DrivingReport;
