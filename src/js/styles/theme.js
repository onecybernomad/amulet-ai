/**
 * Theme manager for dark/light mode toggling.
 */

import { get, set } from '../lib/storage.js';

const THEME_KEY = 'theme';
const DARK = 'dark';
const LIGHT = 'light';

/**
 * Get the current theme.
 * @returns {string}
 */
export function getTheme() {
  return get(THEME_KEY, DARK);
}

/**
 * Apply a theme to the document.
 * @param {string} theme - 'dark' or 'light'
 */
export function applyTheme(theme) {
  document.documentElement.setAttribute('data-theme', theme);
  set(THEME_KEY, theme);

  // Update meta theme-color
  const meta = document.querySelector('meta[name="theme-color"]');
  if (meta) {
    meta.setAttribute('content', theme === DARK ? '#0f172a' : '#f8fafc');
  }
}

/**
 * Toggle between dark and light themes.
 * @returns {string} New theme
 */
export function toggleTheme() {
  const current = getTheme();
  const next = current === DARK ? LIGHT : DARK;
  applyTheme(next);
  return next;
}

/**
 * Detect system color scheme preference.
 * @returns {string}
 */
export function detectSystemTheme() {
  if (window.matchMedia && window.matchMedia('(prefers-color-scheme: light)').matches) {
    return LIGHT;
  }
  return DARK;
}

/**
 * Initialize theme from storage or system preference.
 */
export function initTheme() {
  const saved = get(THEME_KEY);
  const theme = saved || detectSystemTheme();
  applyTheme(theme);

  // Listen for system theme changes
  if (window.matchMedia) {
    window.matchMedia('(prefers-color-scheme: dark)').addEventListener('change', (e) => {
      if (!get(THEME_KEY)) {
        applyTheme(e.matches ? DARK : LIGHT);
      }
    });
  }
}
