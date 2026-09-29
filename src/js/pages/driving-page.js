/**
 * DrivingPage — Session list, report cards, and safety score trend chart.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { api } from '../api.js';
import { DrivingReport } from '../components/driving-report.js';

export class DrivingPage {
  constructor(container) {
    this.container = container;
    this.sessions = [];
    this.viewMode = 'weekly'; // 'weekly' | 'monthly'
    this._render();
    this._loadSessions();
  }

  /**
   * Render the driving page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'driving-page';

    // Header
    const header = createElement('div', { class: 'section-header' });
    header.appendChild(createElement('div', {},
      createElement('h1', { class: 'section-title' }, 'Driving Reports'),
      createElement('p', { class: 'section-subtitle' }, 'Safety scores, events, and trends')
    ));

    // Toggle
    this.toggle = createElement('div', { class: 'driving-toggle' });
    this.weeklyBtn = createElement('button', { class: 'driving-toggle-btn active' }, 'Weekly');
    this.monthlyBtn = createElement('button', { class: 'driving-toggle-btn' }, 'Monthly');
    this.weeklyBtn.addEventListener('click', () => this._setView('weekly'));
    this.monthlyBtn.addEventListener('click', () => this._setView('monthly'));
    this.toggle.appendChild(this.weeklyBtn);
    this.toggle.appendChild(this.monthlyBtn);
    header.appendChild(this.toggle);

    this.container.appendChild(header);

    // Session controls
    this.sessionControls = createElement('div', { class: 'session-controls' });
    this.startSessionBtn = createElement('button', { class: 'btn btn-primary' }, 'Start Driving');
    this.startSessionBtn.addEventListener('click', () => this._startDrivingSession());
    this.stopSessionBtn = createElement('button', { class: 'btn btn-danger', style: { display: 'none' } }, 'Stop Driving');
    this.stopSessionBtn.addEventListener('click', () => this._stopDrivingSession());
    this.sessionControls.appendChild(this.startSessionBtn);
    this.sessionControls.appendChild(this.stopSessionBtn);
    this.container.appendChild(this.sessionControls);

    // Stats
    this.statsEl = createElement('div', { class: 'driving-stats' });
    this.container.appendChild(this.statsEl);

    // Chart
    this.chartContainer = createElement('div', { class: 'chart-container', style: { marginBottom: '24px' } });
    this.container.appendChild(this.chartContainer);

    // Reports list
    this.reportsList = createElement('div', { class: 'driving-reports' });
    this.container.appendChild(this.reportsList);
  }

  /**
   * Set view mode and reload.
   * @private
   */
  _setView(mode) {
    this.viewMode = mode;
    this.weeklyBtn.classList.toggle('active', mode === 'weekly');
    this.monthlyBtn.classList.toggle('active', mode === 'monthly');
    this._loadSessions();
  }

  /**
   * Load driving sessions.
   * @private
   */
  async _loadSessions() {
    try {
      this.sessions = await api.get('/driving/sessions', { period: this.viewMode });
      this._renderReports();
      this._renderChart();
      this._loadDrivingStats();
    } catch (err) {
      console.error('[DrivingPage] Failed to load sessions:', err);
    }
  }

  /**
   * Load driving statistics.
   * @private
   */
  async _loadDrivingStats() {
    try {
      const response = await api.get('/api/driving/stats');
      this.drivingStats = response.data;
      this._renderDrivingStats();
    } catch (err) {
      console.error('[DrivingPage] Failed to load driving stats:', err);
    }
  }

  /**
   * Render driving statistics summary.
   * @private
   */
  _renderDrivingStats() {
    if (!this.statsEl) return;
    clearElement(this.statsEl);

    if (!this.drivingStats) return;

    const stats = this.drivingStats;
    const grid = createElement('div', { class: 'driving-stats-grid' });

    const items = [
      { label: 'Total Trips', value: stats.total_sessions },
      { label: 'Distance', value: `${stats.total_distance.toFixed(1)} km` },
      { label: 'Max Speed', value: `${stats.max_speed.toFixed(0)} km/h` },
      { label: 'Avg Speed', value: `${stats.avg_speed.toFixed(0)} km/h` },
      { label: 'Hard Braking', value: stats.total_harsh_braking },
      { label: 'Rapid Accel', value: stats.total_rapid_accel },
      { label: 'Phone Use', value: stats.total_phone_use },
    ];

    for (const item of items) {
      const statEl = createElement('div', { class: 'driving-stat-card' });
      statEl.appendChild(createElement('div', { class: 'driving-stat-value' }, String(item.value)));
      statEl.appendChild(createElement('div', { class: 'driving-stat-label' }, item.label));
      grid.appendChild(statEl);
    }

    this.statsEl.appendChild(grid);
  }

  /**
   * Start a driving session via Tauri.
   * @private
   */
  async _startDrivingSession() {
    if (window.__TAURI__) {
      try {
        const session = await window.__TAURI__.invoke('start_driving_session');
        this._showNotification('Driving session started', 'success');
        this.activeSession = session;
      } catch (err) {
        console.error('[DrivingPage] Failed to start session:', err);
        this._showNotification('Failed to start driving session', 'danger');
      }
    } else {
      this._showNotification('Driving mode requires the desktop app', 'warning');
    }
  }

  /**
   * Stop the current driving session.
   * @private
   */
  async _stopDrivingSession() {
    if (window.__TAURI__ && this.activeSession) {
      try {
        const summary = await window.__TAURI__.invoke('stop_driving_session');
        this._showNotification(`Drive complete! Safety score: ${summary.safety_score}`, 'success');
        this.activeSession = null;
        this._loadSessions();
      } catch (err) {
        console.error('[DrivingPage] Failed to stop session:', err);
      }
    }
  }

  /**
   * Show a toast notification.
   * @param {string} message
   * @param {string} type
   * @private
   */
  _showNotification(message, type = 'info') {
    const container = document.getElementById('notification-container') || this._createNotificationContainer();
    const toast = createElement('div', { class: `toast toast-${type}` }, message);
    container.appendChild(toast);
    setTimeout(() => toast.classList.add('show'), 10);
    setTimeout(() => {
      toast.classList.remove('show');
      setTimeout(() => toast.remove(), 300);
    }, 4000);
  }

  /**
   * Create notification container if it doesn't exist.
   * @returns {HTMLElement}
   * @private
   */
  _createNotificationContainer() {
    const container = createElement('div', { id: 'notification-container', class: 'notification-container' });
    document.body.appendChild(container);
    return container;
  }

  /**
   * Render report cards.
   * @private
   */
  _renderReports() {
    clearElement(this.reportsList);

    if (this.sessions.length === 0) {
      const empty = createElement('div', { class: 'empty-state' });
      empty.appendChild(createElement('div', { class: 'empty-state-icon' }, '🚗'));
      empty.appendChild(createElement('div', { class: 'empty-state-title' }, 'No driving sessions'));
      empty.appendChild(createElement('div', { class: 'empty-state-text' }, 'Driving reports will appear here after trips'));
      this.reportsList.appendChild(empty);
      return;
    }

    for (const session of this.sessions) {
      const report = new DrivingReport(session);
      this.reportsList.appendChild(report.getElement());
    }
  }

  /**
   * Render safety score bar chart on canvas.
   * @private
   */
  _renderChart() {
    clearElement(this.chartContainer);

    const canvas = createElement('canvas', { class: 'chart-canvas' });
    this.chartContainer.appendChild(canvas);

    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;
    const rect = this.chartContainer.getBoundingClientRect();

    canvas.width = rect.width * dpr;
    canvas.height = rect.height * dpr;
    ctx.scale(dpr, dpr);

    const padding = 30;
    const chartWidth = rect.width - padding * 2;
    const chartHeight = rect.height - padding * 2;

    // Draw axes
    ctx.strokeStyle = getComputedStyle(document.documentElement).getPropertyValue('--color-border');
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(padding, padding);
    ctx.lineTo(padding, rect.height - padding);
    ctx.lineTo(rect.width - padding, rect.height - padding);
    ctx.stroke();

    if (this.sessions.length === 0) return;

    const barWidth = Math.min(40, (chartWidth / this.sessions.length) - 10);
    const barGap = (chartWidth - barWidth * this.sessions.length) / (this.sessions.length + 1);

    this.sessions.forEach((session, i) => {
      const score = session.safetyScore != null ? session.safetyScore : 80;
      const barHeight = (score / 100) * chartHeight;
      const x = padding + barGap + i * (barWidth + barGap);
      const y = rect.height - padding - barHeight;

      // Bar color based on score
      ctx.fillStyle = score >= 80 ? '#10b981' : score >= 50 ? '#f59e0b' : '#ef4444';
      ctx.fillRect(x, y, barWidth, barHeight);

      // Score label
      ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--color-text');
      ctx.font = 'bold 11px system-ui';
      ctx.textAlign = 'center';
      ctx.fillText(String(score), x + barWidth / 2, y - 5);

      // Date label
      ctx.fillStyle = getComputedStyle(document.documentElement).getPropertyValue('--color-text-muted');
      ctx.font = '10px system-ui';
      const date = new Date(session.date);
      ctx.fillText(date.toLocaleDateString([], { month: 'short', day: 'numeric' }), x + barWidth / 2, rect.height - padding + 14);
    });
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default DrivingPage;
