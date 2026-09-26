/**
 * Storage wrapper with JSON serialization and namespacing.
 */

const NAMESPACE = 'amulet:';

/**
 * Get a value from storage.
 * @param {string} key - Storage key
 * @param {*} [defaultValue] - Default value if key not found
 * @returns {*}
 */
export function get(key, defaultValue = null) {
  try {
    const fullKey = NAMESPACE + key;
    const raw = localStorage.getItem(fullKey);
    if (raw === null) return defaultValue;
    return JSON.parse(raw);
  } catch (err) {
    console.error(`[Storage] Error reading key "${key}":`, err);
    return defaultValue;
  }
}

/**
 * Set a value in storage.
 * @param {string} key - Storage key
 * @param {*} value - Value to store (will be JSON serialized)
 * @returns {boolean} Success
 */
export function set(key, value) {
  try {
    const fullKey = NAMESPACE + key;
    localStorage.setItem(fullKey, JSON.stringify(value));
    return true;
  } catch (err) {
    console.error(`[Storage] Error writing key "${key}":`, err);
    return false;
  }
}

/**
 * Remove a key from storage.
 * @param {string} key - Storage key
 */
export function remove(key) {
  try {
    const fullKey = NAMESPACE + key;
    localStorage.removeItem(fullKey);
  } catch (err) {
    console.error(`[Storage] Error removing key "${key}":`, err);
  }
}

/**
 * Clear all namespaced keys from storage.
 */
export function clear() {
  try {
    const keysToRemove = [];
    for (let i = 0; i < localStorage.length; i++) {
      const key = localStorage.key(i);
      if (key && key.startsWith(NAMESPACE)) {
        keysToRemove.push(key);
      }
    }
    keysToRemove.forEach(key => localStorage.removeItem(key));
  } catch (err) {
    console.error('[Storage] Error clearing storage:', err);
  }
}

/**
 * Check if a key exists in storage.
 * @param {string} key
 * @returns {boolean}
 */
export function has(key) {
  try {
    return localStorage.getItem(NAMESPACE + key) !== null;
  } catch {
    return false;
  }
}

/**
 * Get all keys in the namespace.
 * @returns {string[]}
 */
export function keys() {
  const result = [];
  for (let i = 0; i < localStorage.length; i++) {
    const key = localStorage.key(i);
    if (key && key.startsWith(NAMESPACE)) {
      result.push(key.slice(NAMESPACE.length));
    }
  }
  return result;
}
