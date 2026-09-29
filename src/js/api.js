/**
 * API client with token management, error handling, and auto-retry.
 */

import { get, set, remove } from './lib/storage.js';

export class ApiError extends Error {
  constructor(message, status, data) {
    super(message);
    this.name = 'ApiError';
    this.status = status;
    this.data = data;
  }
}

export class ApiClient {
  constructor(baseURL) {
    this.baseURL = baseURL || window.__API_URL || import.meta.env.VITE_API_URL || 'http://localhost:3000';
    this.token = get('auth_token', null);
    this.refreshToken = get('refresh_token', null);
    this.tokenExpiresAt = get('auth_token_expires_at', null);
    this._refreshPromise = null;
    this._refreshTimer = null;
  }

  /**
   * Get the current auth token.
   * @returns {string|null}
   */
  getToken() {
    return this.token;
  }

  /**
   * Set the auth token.
   * @param {string} token
   */
  setToken(token) {
    this.token = token;
    set('auth_token', token);
  }

  /**
   * Set the refresh token.
   * @param {string} token
   */
  setRefreshToken(token) {
    this.refreshToken = token;
    set('refresh_token', token);
  }

  /**
   * Set both access and refresh tokens from an auth response.
   * @param {Object} data - Response containing token and refresh_token
   */
  setTokens(data) {
    if (data.token) this.setToken(data.token);
    if (data.refresh_token) this.setRefreshToken(data.refresh_token);
    this._scheduleProactiveRefresh();
  }

  /**
   * Decode JWT token to get expiration time.
   * @param {string} token - JWT token
   * @returns {number|null} Expiration timestamp in seconds, or null if invalid
   */
  _getTokenExpiry(token) {
    try {
      const payload = JSON.parse(atob(token.split('.')[1]));
      return payload.exp || null;
    } catch {
      return null;
    }
  }

  /**
   * Schedule proactive token refresh before expiry.
   * Refreshes 1 minute before the token expires.
   * @private
   */
  _scheduleProactiveRefresh() {
    this._clearProactiveRefresh();

    if (!this.token) return;

    const expiry = this._getTokenExpiry(this.token);
    if (!expiry) return;

    const now = Math.floor(Date.now() / 1000);
    const refreshAt = expiry - now - 60; // Refresh 1 minute before expiry

    if (refreshAt <= 0) {
      // Token already expired or about to expire, refresh immediately
      this._refreshAccessToken().catch(() => {});
      return;
    }

    console.log(`[API] Scheduling proactive token refresh in ${refreshAt}s`);
    this._refreshTimer = setTimeout(() => {
      this._refreshAccessToken().catch(() => {});
    }, refreshAt * 1000);
  }

  /**
   * Clear the proactive refresh timer.
   * @private
   */
  _clearProactiveRefresh() {
    if (this._refreshTimer) {
      clearTimeout(this._refreshTimer);
      this._refreshTimer = null;
    }
  }

  /**
   * Clear all auth tokens.
   */
  clearTokens() {
    this.token = null;
    this.refreshToken = null;
    this.tokenExpiresAt = null;
    this._clearProactiveRefresh();
    remove('auth_token');
    remove('refresh_token');
    remove('auth_token_expires_at');
  }

  /**
   * Build request headers.
   * @private
   */
  _headers(contentType = 'application/json') {
    const headers = {};
    if (contentType) headers['Content-Type'] = contentType;
    if (this.token) headers['Authorization'] = `Bearer ${this.token}`;
    return headers;
  }

  /**
   * Handle API response.
   * @private
   */
  async _handleResponse(response) {
    const contentType = response.headers.get('content-type');
    let data = null;

    if (contentType && contentType.includes('application/json')) {
      data = await response.json();
    } else {
      data = await response.text();
    }

    if (!response.ok) {
      throw new ApiError(
        data?.message || `Request failed with status ${response.status}`,
        response.status,
        data
      );
    }

    return data;
  }

  /**
   * Attempt to refresh the access token.
   * @private
   */
  async _refreshAccessToken() {
    if (this._refreshPromise) return this._refreshPromise;

    this._refreshPromise = (async () => {
      try {
        const response = await fetch(`${this.baseURL}/api/auth/refresh`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json' },
          body: JSON.stringify({ refreshToken: this.refreshToken }),
        });

        if (!response.ok) {
          this.clearTokens();
          throw new ApiError('Session expired. Please log in again.', 401, null);
        }

        const data = await response.json();
        this.setToken(data.token);
        if (data.refreshToken) this.setRefreshToken(data.refreshToken);
        return data.token;
      } finally {
        this._refreshPromise = null;
      }
    })();

    return this._refreshPromise;
  }

  /**
   * Make an API request with automatic retry on 401.
   * @private
   */
  async _request(method, path, options = {}, retryCount = 0) {
    const url = `${this.baseURL}${path}`;
    const config = {
      method,
      headers: this._headers(options.contentType),
      ...options,
    };

    if (options.body && typeof options.body === 'object' && !(options.body instanceof FormData)) {
      config.body = JSON.stringify(options.body);
    } else if (options.body) {
      config.body = options.body;
    }

    const response = await fetch(url, config);

    // Auto-retry on 401 with token refresh
    if (response.status === 401 && retryCount < 1 && this.refreshToken) {
      try {
        await this._refreshAccessToken();
        return this._request(method, path, options, retryCount + 1);
      } catch (refreshErr) {
        this.clearTokens();
        window.dispatchEvent(new CustomEvent('auth:expired'));
        throw refreshErr;
      }
    }

    return this._handleResponse(response);
  }

  /**
   * GET request.
   * @param {string} path
   * @param {Object} [params] - Query parameters
   */
  async get(path, params = {}) {
    const query = new URLSearchParams(params).toString();
    const url = query ? `${path}?${query}` : path;
    return this._request('GET', url);
  }

  /**
   * POST request.
   * @param {string} path
   * @param {Object} [body]
   */
  async post(path, body = {}) {
    return this._request('POST', path, { body });
  }

  /**
   * PATCH request.
   * @param {string} path
   * @param {Object} [body]
   */
  async patch(path, body = {}) {
    return this._request('PATCH', path, { body });
  }

  /**
   * DELETE request.
   * @param {string} path
   */
  async delete(path) {
    return this._request('DELETE', path);
  }

  /**
   * Upload a file.
   * @param {string} path
   * @param {FormData} formData
   */
  async upload(path, formData) {
    return this._request('POST', path, {
      body: formData,
      contentType: null, // Let browser set multipart boundary
    });
  }
}

/** Global API client instance */
export const api = new ApiClient();
export default api;
