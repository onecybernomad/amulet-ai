/**
 * Hash-based SPA router with dynamic segments and navigation guards.
 */

import { store } from './state.js';
import { api } from './api.js';

const ROUTES = {
  '/login': () => import('./pages/login-page.js'),
  '/signup': () => import('./pages/signup-page.js'),
  '/onboarding': () => import('./pages/onboarding-page.js'),
  '/map': () => import('./pages/map-page.js'),
  '/places': () => import('./pages/places-page.js'),
  '/chat': () => import('./pages/chat-page.js'),
  '/meds': () => import('./pages/meds-page.js'),
  '/driving': () => import('./pages/driving-page.js'),
  '/history': () => import('./pages/history-page.js'),
  '/assistance': () => import('./pages/assistance-page.js'),
  '/settings': () => import('./pages/settings-page.js'),
  '/admin': () => import('./pages/admin-page.js'),
};

export class Router {
  constructor() {
    this.currentPage = null;
    this.currentRoute = null;
    this.guards = [];
    this._onHashChange = this._handleHashChange.bind(this);
  }

  /**
   * Initialize the router.
   */
  init() {
    window.addEventListener('hashchange', this._onHashChange);
    this._handleHashChange();
  }

  /**
   * Add a navigation guard.
   * @param {Function} guard - (to, from) => boolean | string | Promise
   */
  beforeEach(guard) {
    this.guards.push(guard);
  }

  /**
   * Navigate to a route.
   * @param {string} path
   */
  navigate(path) {
    window.location.hash = `#${path}`;
  }

  /**
   * Handle hash change events.
   * @private
   */
  async _handleHashChange() {
    const hash = window.location.hash.slice(1) || '/map';
    const path = hash.split('?')[0];

    // Run navigation guards
    for (const guard of this.guards) {
      const result = await guard(path, this.currentRoute);
      if (result === false) return;
      if (typeof result === 'string') {
        this.navigate(result);
        return;
      }
    }

    await this._render(path);
  }

  /**
   * Render the matched page.
   * @private
   */
  async _render(path) {
    // Destroy current page
    if (this.currentPage) {
      this.currentPage.destroy();
      this.currentPage = null;
    }

    // Find matching route
    const routeKey = Object.keys(ROUTES).find(key => {
      if (key === path) return true;
      // Dynamic segments: e.g., /places/:id
      const keyParts = key.split('/');
      const pathParts = path.split('/');
      if (keyParts.length !== pathParts.length) return false;
      return keyParts.every((part, i) => part.startsWith(':') || part === pathParts[i]);
    });

    const appContainer = document.getElementById('app');
    if (!appContainer) return;

    // Show loading state
    appContainer.innerHTML = '<div class="loading-screen"><div class="loading-spinner"></div></div>';

    try {
      if (routeKey) {
        const module = await ROUTES[routeKey]();
        const PageClass = module.default;
        this.currentPage = new PageClass(appContainer);
        this.currentRoute = path;

        // Update nav active state
        this._updateNav(path);
      } else {
        // 404 fallback
        appContainer.innerHTML = `
          <div class="empty-state" style="min-height: 100vh;">
            <div class="empty-state-icon">🔍</div>
            <div class="empty-state-title">Page Not Found</div>
            <div class="empty-state-text">The page you're looking for doesn't exist.</div>
            <button class="btn btn-primary mt-4" onclick="window.location.hash='#/map'">Go to Map</button>
          </div>
        `;
      }
    } catch (err) {
      console.error('[Router] Failed to render page:', err);
      appContainer.innerHTML = `
        <div class="empty-state" style="min-height: 100vh;">
          <div class="empty-state-icon">⚠️</div>
          <div class="empty-state-title">Something went wrong</div>
          <div class="empty-state-text">Failed to load the page. Please try again.</div>
          <button class="btn btn-primary mt-4" onclick="window.location.reload()">Reload</button>
        </div>
      `;
    }
  }

  /**
   * Update navigation active state.
   * @private
   */
  _updateNav(path) {
    document.querySelectorAll('.nav-item').forEach(item => {
      const route = item.getAttribute('data-route');
      item.classList.toggle('active', path.startsWith(route));
    });
  }

  /**
   * Destroy the router.
   */
  destroy() {
    window.removeEventListener('hashchange', this._onHashChange);
    if (this.currentPage) {
      this.currentPage.destroy();
    }
  }
}

/** Global router instance */
export const router = new Router();
export default router;
