/**
 * Web Crypto API helpers for encryption/decryption.
 * Uses AES-GCM with 256-bit keys.
 */

const encoder = new TextEncoder();
const decoder = new TextDecoder();

/**
 * Generate a new AES-GCM 256-bit key.
 * @returns {Promise<CryptoKey>}
 */
export async function generateKey() {
  return crypto.subtle.generateKey(
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

/**
 * Export a CryptoKey as JWK (JSON Web Key).
 * @param {CryptoKey} key
 * @returns {Promise<string>} JSON string
 */
export async function exportKey(key) {
  const jwk = await crypto.subtle.exportKey('jwk', key);
  return JSON.stringify(jwk);
}

/**
 * Import a CryptoKey from a JWK string.
 * @param {string} jwkString - JSON Web Key string
 * @returns {Promise<CryptoKey>}
 */
export async function importKey(jwkString) {
  const jwk = JSON.parse(jwkString);
  return crypto.subtle.importKey(
    'jwk',
    jwk,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

/**
 * Encrypt a message string with AES-GCM.
 * @param {string} message - Plain text to encrypt
 * @param {CryptoKey} key - Encryption key
 * @returns {Promise<string>} Base64-encoded ciphertext (includes IV)
 */
export async function encrypt(message, key) {
  const iv = crypto.getRandomValues(new Uint8Array(12));
  const encoded = encoder.encode(message);

  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    encoded
  );

  // Prepend IV to ciphertext for storage
  const combined = new Uint8Array(iv.length + ciphertext.byteLength);
  combined.set(iv, 0);
  combined.set(new Uint8Array(ciphertext), iv.length);

  return arrayBufferToBase64(combined);
}

/**
 * Decrypt a ciphertext string with AES-GCM.
 * @param {string} ciphertext - Base64-encoded ciphertext (includes IV)
 * @param {CryptoKey} key - Decryption key
 * @returns {Promise<string>} Decrypted plain text
 */
export async function decrypt(ciphertext, key) {
  const combined = base64ToArrayBuffer(ciphertext);
  const iv = combined.slice(0, 12);
  const data = combined.slice(12);

  const decrypted = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv },
    key,
    data
  );

  return decoder.decode(decrypted);
}

/**
 * Derive a key from a password using PBKDF2.
 * @param {string} password
 * @param {Uint8Array} salt
 * @param {number} [iterations=100000]
 * @returns {Promise<CryptoKey>}
 */
export async function deriveKeyFromPassword(password, salt, iterations = 100000) {
  const keyMaterial = await crypto.subtle.importKey(
    'raw',
    encoder.encode(password),
    'PBKDF2',
    false,
    ['deriveKey']
  );

  return crypto.subtle.deriveKey(
    {
      name: 'PBKDF2',
      salt,
      iterations,
      hash: 'SHA-256',
    },
    keyMaterial,
    { name: 'AES-GCM', length: 256 },
    true,
    ['encrypt', 'decrypt']
  );
}

/**
 * Generate a random salt.
 * @param {number} [length=16]
 * @returns {Uint8Array}
 */
export function generateSalt(length = 16) {
  return crypto.getRandomValues(new Uint8Array(length));
}

/**
 * Convert ArrayBuffer to Base64 string.
 * @param {ArrayBuffer|Uint8Array} buffer
 * @returns {string}
 */
function arrayBufferToBase64(buffer) {
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return btoa(binary);
}

/**
 * Convert Base64 string to Uint8Array.
 * @param {string} base64
 * @returns {Uint8Array}
 */
function base64ToArrayBuffer(base64) {
  const binary = atob(base64);
  const bytes = new Uint8Array(binary.length);
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i);
  }
  return bytes;
}
