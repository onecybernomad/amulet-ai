/**
 * OnboardingPage — Welcome flow for new users with feature tour.
 */

import { createElement, clearElement } from '../lib/dom.js';
import { store } from '../state.js';
import { router } from '../router.js';

const STEPS = [
  {
    icon: '🛡️',
    title: 'Welcome to Amulet AI',
    description: 'Your family safety and assistance app. Keep your loved ones connected, protected, and supported — all in one place.',
  },
  {
    icon: '📍',
    title: 'Real-Time Location',
    description: 'See where your family members are on a live map. Share your location privately and only with your Circle.',
  },
  {
    icon: '🏠',
    title: 'Place Alerts',
    description: 'Get notified when family members arrive at or leave important places like home, school, or work.',
  },
  {
    icon: '🆘',
    title: 'SOS & Emergency',
    description: 'Send instant emergency alerts to your Circle with your location. Help is always just a tap away.',
  },
  {
    icon: '💊',
    title: 'Medication Tracking',
    description: 'Never miss a dose. Track medications, set reminders, and monitor adherence for your whole family.',
  },
  {
    icon: '🚗',
    title: 'Driving Safety',
    description: 'Monitor driving habits, get safety scores, and detect crashes automatically. Available on premium plans.',
  },
];

export class OnboardingPage {
  constructor(container) {
    this.container = container;
    this.currentStep = 0;
    this._render();
  }

  /**
   * Render the onboarding page.
   * @private
   */
  _render() {
    clearElement(this.container);
    this.container.className = 'onboarding-page';

    const step = STEPS[this.currentStep];
    const isLast = this.currentStep === STEPS.length - 1;
    const isFirst = this.currentStep === 0;

    // Progress dots
    const progress = createElement('div', { class: 'onboarding-progress' });
    for (let i = 0; i < STEPS.length; i++) {
      const dot = createElement('div', {
        class: `onboarding-dot ${i === this.currentStep ? 'active' : ''} ${i < this.currentStep ? 'completed' : ''}`,
      });
      progress.appendChild(dot);
    }
    this.container.appendChild(progress);

    // Content card
    const card = createElement('div', { class: 'onboarding-card' });

    const icon = createElement('div', { class: 'onboarding-icon' }, step.icon);
    const title = createElement('h1', { class: 'onboarding-title' }, step.title);
    const desc = createElement('p', { class: 'onboarding-description' }, step.description);

    card.appendChild(icon);
    card.appendChild(title);
    card.appendChild(desc);

    // Actions
    const actions = createElement('div', { class: 'onboarding-actions' });

    if (!isFirst) {
      const backBtn = createElement('button', { class: 'btn btn-secondary' }, 'Back');
      backBtn.addEventListener('click', () => this._prevStep());
      actions.appendChild(backBtn);
    }

    const nextBtn = createElement('button', { class: 'btn btn-primary' }, isLast ? 'Get Started' : 'Next');
    nextBtn.addEventListener('click', () => this._nextStep());
    actions.appendChild(nextBtn);

    card.appendChild(actions);
    this.container.appendChild(card);

    // Skip button
    if (!isLast) {
      const skipBtn = createElement('button', { class: 'onboarding-skip' }, 'Skip');
      skipBtn.addEventListener('click', () => this._finish());
      this.container.appendChild(skipBtn);
    }
  }

  /**
   * Go to the next step.
   * @private
   */
  _nextStep() {
    if (this.currentStep < STEPS.length - 1) {
      this.currentStep++;
      this._render();
    } else {
      this._finish();
    }
  }

  /**
   * Go to the previous step.
   * @private
   */
  _prevStep() {
    if (this.currentStep > 0) {
      this.currentStep--;
      this._render();
    }
  }

  /**
   * Finish onboarding and navigate to the map.
   * @private
   */
  _finish() {
    // Mark onboarding as completed
    localStorage.setItem('amulet:onboarding_completed', 'true');
    router.navigate('/map');
  }

  /**
   * Destroy the page.
   */
  destroy() {
    clearElement(this.container);
  }
}

export default OnboardingPage;
