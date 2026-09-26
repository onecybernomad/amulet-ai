/**
 * Component base class with lifecycle management and store subscription.
 */

export class Component {
  constructor(rootElement) {
    this.root = rootElement;
    this._subscriptions = [];
    this._destroyed = false;
  }

  /**
   * Subscribe to store changes with auto-cleanup on destroy.
   * @param {string} key
   * @param {Function} callback
   */
  subscribe(key, callback) {
    const unsub = this.root.__store.on(key, callback);
    this._subscriptions.push(unsub);
    return unsub;
  }

  /**
   * Virtual render method — must be overridden.
   */
  render() {
    throw new Error('Component.render() must be implemented');
  }

  /**
   * Mount the component to a parent element.
   * @param {HTMLElement} parent
   */
  mount(parent) {
    parent.appendChild(this.root);
    this.afterMount();
  }

  /**
   * Set inner HTML and mount.
   * @param {HTMLElement} parent
   * @param {string} html
   */
  mountHTML(parent, html) {
    this.root.innerHTML = html;
    this.mount(parent);
  }

  /**
   * Virtual afterMount method — called after mount.
   */
  afterMount() {}

  /**
   * Destroy the component and clean up subscriptions.
   */
  destroy() {
    this._destroyed = true;
    this._subscriptions.forEach(unsub => unsub());
    this._subscriptions = [];
    if (this.root.parentNode) {
      this.root.parentNode.removeChild(this.root);
    }
  }
}

export default Component;
