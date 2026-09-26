/**
 * DOM utility functions for creating and manipulating elements.
 */

/**
 * Create a new DOM element with attributes and children.
 * @param {string} tag - HTML tag name
 * @param {Object} attrs - Attributes to set
 * @param {Array|string|Node} children - Child nodes or HTML string
 * @returns {HTMLElement}
 */
export function createElement(tag, attrs = {}, children = []) {
  const el = document.createElement(tag);

  for (const [key, value] of Object.entries(attrs)) {
    if (key === 'class') {
      el.className = value;
    } else if (key === 'dataset') {
      Object.assign(el.dataset, value);
    } else if (key === 'style') {
      Object.assign(el.style, value);
    } else if (key.startsWith('on') && typeof value === 'function') {
      el.addEventListener(key.slice(2).toLowerCase(), value);
    } else if (key === 'html') {
      el.innerHTML = value;
    } else if (key === 'text') {
      el.textContent = value;
    } else if (value !== null && value !== undefined) {
      el.setAttribute(key, value);
    }
  }

  const childArray = Array.isArray(children) ? children : [children];
  for (const child of childArray) {
    if (child === null || child === undefined) continue;
    if (typeof child === 'string' || typeof child === 'number') {
      el.appendChild(document.createTextNode(String(child)));
    } else if (child instanceof Node) {
      el.appendChild(child);
    }
  }

  return el;
}

/**
 * Clear all children from an element.
 * @param {HTMLElement} el
 */
export function clearElement(el) {
  while (el.firstChild) {
    el.removeChild(el.firstChild);
  }
}

/**
 * Add a class to an element.
 * @param {HTMLElement} el
 * @param {string} className
 */
export function addClass(el, className) {
  if (el && className) {
    el.classList.add(...className.split(' ').filter(Boolean));
  }
}

/**
 * Remove a class from an element.
 * @param {HTMLElement} el
 * @param {string} className
 */
export function removeClass(el, className) {
  if (el && className) {
    el.classList.remove(...className.split(' ').filter(Boolean));
  }
}

/**
 * Toggle a class on an element.
 * @param {HTMLElement} el
 * @param {string} className
 * @param {boolean} force - Force add or remove
 */
export function toggleClass(el, className, force) {
  if (el && className) {
    const classes = className.split(' ').filter(Boolean);
    classes.forEach(cls => el.classList.toggle(cls, force));
  }
}

/**
 * Event delegation helper.
 * @param {HTMLElement} container - Container element
 * @param {string} event - Event name
 * @param {string} selector - CSS selector to match
 * @param {Function} handler - Event handler
 */
export function delegate(container, event, selector, handler) {
  container.addEventListener(event, (e) => {
    const target = e.target.closest(selector);
    if (target && container.contains(target)) {
      handler(e, target);
    }
  });
}

/**
 * Execute callback when DOM is ready.
 * @param {Function} callback
 */
export function ready(callback) {
  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', callback);
  } else {
    callback();
  }
}

/**
 * Query selector shorthand.
 * @param {string} selector
 * @param {HTMLElement} parent
 * @returns {HTMLElement|null}
 */
export function $(selector, parent = document) {
  return parent.querySelector(selector);
}

/**
 * Query selector all shorthand.
 * @param {string} selector
 * @param {HTMLElement} parent
 * @returns {NodeList}
 */
export function $$(selector, parent = document) {
  return parent.querySelectorAll(selector);
}

/**
 * Check if element is in viewport.
 * @param {HTMLElement} el
 * @returns {boolean}
 */
export function isInViewport(el) {
  const rect = el.getBoundingClientRect();
  return (
    rect.top >= 0 &&
    rect.left >= 0 &&
    rect.bottom <= window.innerHeight &&
    rect.right <= window.innerWidth
  );
}

/**
 * Debounce function.
 * @param {Function} fn
 * @param {number} delay
 * @returns {Function}
 */
export function debounce(fn, delay = 200) {
  let timeoutId;
  return function (...args) {
    clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn.apply(this, args), delay);
  };
}

/**
 * Throttle function.
 * @param {Function} fn
 * @param {number} limit
 * @returns {Function}
 */
export function throttle(fn, limit = 100) {
  let inThrottle = false;
  return function (...args) {
    if (!inThrottle) {
      fn.apply(this, args);
      inThrottle = true;
      setTimeout(() => { inThrottle = false; }, limit);
    }
  };
}
