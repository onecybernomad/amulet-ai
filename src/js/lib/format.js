/**
 * Formatting helpers for dates, times, distances, and more.
 */

/**
 * Format a date/time as a short time string (e.g., "2:30 PM").
 * @param {Date|string|number} date
 * @returns {string}
 */
export function formatTime(date) {
  const d = new Date(date);
  if (isNaN(d.getTime())) return '--:--';
  return d.toLocaleTimeString([], { hour: 'numeric', minute: '2-digit' });
}

/**
 * Format a date as a readable date string (e.g., "Sep 26, 2026").
 * @param {Date|string|number} date
 * @returns {string}
 */
export function formatDate(date) {
  const d = new Date(date);
  if (isNaN(d.getTime())) return '---';
  return d.toLocaleDateString([], { month: 'short', day: 'numeric', year: 'numeric' });
}

/**
 * Format a date with both date and time.
 * @param {Date|string|number} date
 * @returns {string}
 */
export function formatDateTime(date) {
  const d = new Date(date);
  if (isNaN(d.getTime())) return '---';
  return d.toLocaleString([], {
    month: 'short',
    day: 'numeric',
    hour: 'numeric',
    minute: '2-digit',
  });
}

/**
 * Format a distance in kilometers.
 * @param {number} km
 * @returns {string}
 */
export function formatDistance(km) {
  if (km == null || isNaN(km)) return '--';
  if (km < 1) return `${Math.round(km * 1000)} m`;
  if (km < 10) return `${km.toFixed(1)} km`;
  return `${Math.round(km)} km`;
}

/**
 * Format a speed in km/h.
 * @param {number} kmh
 * @returns {string}
 */
export function formatSpeed(kmh) {
  if (kmh == null || isNaN(kmh)) return '--';
  return `${Math.round(kmh)} km/h`;
}

/**
 * Format a duration in milliseconds.
 * @param {number} ms
 * @returns {string}
 */
export function formatDuration(ms) {
  if (ms == null || isNaN(ms)) return '--';
  const seconds = Math.floor(ms / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);

  if (hours > 0) {
    const remainingMin = minutes % 60;
    return `${hours}h ${remainingMin}m`;
  }
  if (minutes > 0) {
    const remainingSec = seconds % 60;
    return `${minutes}m ${remainingSec}s`;
  }
  return `${seconds}s`;
}

/**
 * Format a timestamp as a relative "time ago" string.
 * @param {number|string|Date} ts - Timestamp
 * @returns {string}
 */
export function timeAgo(ts) {
  const date = new Date(ts);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffSec = Math.floor(diffMs / 1000);
  const diffMin = Math.floor(diffSec / 60);
  const diffHr = Math.floor(diffMin / 60);
  const diffDay = Math.floor(diffHr / 24);

  if (diffSec < 10) return 'just now';
  if (diffSec < 60) return `${diffSec}s ago`;
  if (diffMin < 60) return `${diffMin}m ago`;
  if (diffHr < 24) return `${diffHr}h ago`;
  if (diffDay < 7) return `${diffDay}d ago`;
  if (diffDay < 30) return `${Math.floor(diffDay / 7)}w ago`;
  if (diffDay < 365) return `${Math.floor(diffDay / 30)}mo ago`;
  return `${Math.floor(diffDay / 365)}y ago`;
}

/**
 * Format a percentage value.
 * @param {number} value
 * @param {number} [decimals=0]
 * @returns {string}
 */
export function formatPercent(value, decimals = 0) {
  if (value == null || isNaN(value)) return '--';
  return `${value.toFixed(decimals)}%`;
}

/**
 * Format a number with commas.
 * @param {number} num
 * @returns {string}
 */
export function formatNumber(num) {
  if (num == null || isNaN(num)) return '--';
  return num.toLocaleString();
}

/**
 * Get initials from a name string.
 * @param {string} name
 * @returns {string}
 */
export function getInitials(name) {
  if (!name) return '?';
  return name
    .split(' ')
    .map(part => part.charAt(0))
    .join('')
    .toUpperCase()
    .slice(0, 2);
}

/**
 * Format a battery level as a percentage.
 * @param {number} level - 0-100
 * @returns {string}
 */
export function formatBattery(level) {
  if (level == null || isNaN(level)) return '--';
  return `${Math.round(level)}%`;
}
