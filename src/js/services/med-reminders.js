/**
 * MedReminder — Local medication reminder scheduler with push notifications.
 *
 * Checks medication schedules every minute and triggers push notifications
 * when a dose is due. Uses the Notification API for local alerts.
 */

import { store } from '../state.js';
import { api } from '../api.js';

const CHECK_INTERVAL = 30_000; // Check every 30 seconds
const REMINDER_WINDOW_MINUTES = 15; // Remind 15 min before scheduled time

export class MedReminder {
  constructor() {
    this.timer = null;
    this.medications = [];
    this.lastNotified = new Map(); // med_id -> timestamp
  }

  /**
   * Start the reminder scheduler.
   */
  start() {
    if (this.timer) return;

    // Request notification permission
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission();
    }

    this.medications = store.get('medications') || [];
    this.timer = setInterval(() => this._checkReminders(), CHECK_INTERVAL);
    console.log('[MedReminder] Started');
  }

  /**
   * Stop the reminder scheduler.
   */
  stop() {
    clearInterval(this.timer);
    this.timer = null;
    console.log('[MedReminder] Stopped');
  }

  /**
   * Update the medications list.
   * @param {Array} medications
   */
  updateMedications(medications) {
    this.medications = medications;
  }

  /**
   * Check for due reminders.
   * @private
   */
  _checkReminders() {
    const now = new Date();
    const currentTime = now.getHours() * 60 + now.getMinutes(); // minutes since midnight

    for (const med of this.medications) {
      if (!med.schedule || !Array.isArray(med.schedule)) continue;

      for (const slot of med.schedule) {
        if (slot.status === 'taken') continue;

        const [hours, minutes] = slot.time.split(':').map(Number);
        const scheduledMinutes = hours * 60 + minutes;

        // Check if within reminder window
        const diff = scheduledMinutes - currentTime;
        if (diff > 0 && diff <= REMINDER_WINDOW_MINUTES) {
          const notifyKey = `${med.id}_${slot.time}_${now.toDateString()}`;
          if (!this.lastNotified.has(notifyKey)) {
            this._sendNotification(med, slot);
            this.lastNotified.set(notifyKey, Date.now());
          }
        }
      }
    }

    // Clean up old notifications (older than 24 hours)
    const cutoff = Date.now() - 24 * 60 * 60 * 1000;
    for (const [key, timestamp] of this.lastNotified) {
      if (timestamp < cutoff) {
        this.lastNotified.delete(key);
      }
    }
  }

  /**
   * Send a push notification for a medication reminder.
   * @param {Object} med
   * @param {Object} slot
   * @private
   */
  _sendNotification(med, slot) {
    const title = `Medication Reminder: ${med.name}`;
    const body = `Time to take ${med.dosage || 'your dose'} — scheduled for ${slot.time}`;

    // Use Notification API if available
    if ('Notification' in window && Notification.permission === 'granted') {
      const notification = new Notification(title, {
        body,
        icon: '/assets/pill-icon.png',
        tag: `med-${med.id}-${slot.time}`,
        requireInteraction: true,
      });

      notification.onclick = () => {
        window.focus();
        notification.close();
        // Navigate to medications page
        window.dispatchEvent(new CustomEvent('navigate', { detail: '/meds' }));
      };
    }

    // Also show in-app toast
    window.dispatchEvent(new CustomEvent('med:reminder', {
      detail: { med, slot, title, body },
    }));

    console.log(`[MedReminder] ${title}: ${body}`);
  }

  /**
   * Get adherence statistics for a medication.
   * @param {string} medId
   * @returns {Object} { taken, missed, skipped, streak, rate }
   */
  async getAdherenceStats(medId) {
    try {
      const response = await api.get(`/api/medications/${medId}/adherence`);
      const records = response.data || [];

      let taken = 0;
      let missed = 0;
      let skipped = 0;
      let currentStreak = 0;
      let maxStreak = 0;

      for (const record of records) {
        if (record.taken) {
          taken++;
          currentStreak++;
          if (currentStreak > maxStreak) maxStreak = currentStreak;
        } else {
          if (record.notes === 'skipped') skipped++;
          else missed++;
          currentStreak = 0;
        }
      }

      const total = taken + missed + skipped;
      const rate = total > 0 ? Math.round((taken / total) * 100) : 0;

      return { taken, missed, skipped, streak: currentStreak, maxStreak, rate };
    } catch (e) {
      console.error('[MedReminder] Failed to get adherence stats:', e);
      return { taken: 0, missed: 0, skipped: 0, streak: 0, maxStreak: 0, rate: 0 };
    }
  }

  /**
   * Calculate refill date based on remaining doses.
   * @param {Object} med
   * @param {number} remainingPills
   * @returns {Date|null}
   */
  calculateRefillDate(med, remainingPills) {
    if (!med.schedule || !Array.isArray(med.schedule)) return null;

    const dosesPerDay = med.schedule.length;
    if (dosesPerDay === 0) return null;

    const daysRemaining = Math.floor(remainingPills / dosesPerDay);
    const refillDate = new Date();
    refillDate.setDate(refillDate.getDate() + daysRemaining);
    return refillDate;
  }
}

export const medReminder = new MedReminder();
export default medReminder;
