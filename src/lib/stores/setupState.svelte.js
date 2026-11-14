// src/lib/stores/setupState.svelte.js
const SAVED_SETUP_COMPLETED = typeof localStorage !== 'undefined'
  ? localStorage.getItem('qarl_setup_completed') === '1' : false;
const SAVED_TUTORIAL_DISABLED = typeof localStorage !== 'undefined'
  ? localStorage.getItem('qarl_tutorial_disabled') === '1' : false;

export const setupState = $state({
  // Persistance
  setupCompleted: SAVED_SETUP_COMPLETED,
  tutorialDisabled: SAVED_TUTORIAL_DISABLED,

  // Runtime
  isOpen: !SAVED_SETUP_COMPLETED,
  currentStep: 0,
  hiddenForRedirect: false,

  // Données partagées entre étapes
  data: {
    acPath: '',
    acPathSource: '',
    documentsPath: '',
    documentsSource: '',
    isAcInitialized: false,
    hasControllers: false,
    controllersCount: 0,
    steamId: '',
    driverName: 'Driver',
    onlineName: '',
    nationality: 'France',
    hasCsp: false,
    hasSol: false,
    hasPure: false,
    cspVersion: '',
    solVersion: '',
    pureVersion: '',
    carsCount: 0,
    tracksCount: 0,
  }
});

export function openSetup() {
  setupState.currentStep = 0;
  setupState.isOpen = true;
  setupState.hiddenForRedirect = false;
}

export function closeSetup() {
  setupState.isOpen = false;
  setupState.hiddenForRedirect = false;
}

export function markSetupCompleted() {
  setupState.setupCompleted = true;
  setupState.isOpen = false;
  setupState.hiddenForRedirect = false;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_setup_completed', '1');
  }
}

export function resetSetup() {
  setupState.setupCompleted = false;
  setupState.isOpen = true;
  setupState.currentStep = 0;
  setupState.hiddenForRedirect = false;
  if (typeof localStorage !== 'undefined') {
    localStorage.removeItem('qarl_setup_completed');
  }
}

export function setTutorialDisabled(disabled) {
  setupState.tutorialDisabled = disabled;
  if (typeof localStorage !== 'undefined') {
    if (disabled) localStorage.setItem('qarl_tutorial_disabled', '1');
    else localStorage.removeItem('qarl_tutorial_disabled');
  }
}

export function goToStep(step) {
  setupState.currentStep = Math.max(0, step);
  setupState.hiddenForRedirect = false;
}

export function hideForRedirect() {
  setupState.hiddenForRedirect = true;
}

export function showAfterRedirect() {
  setupState.hiddenForRedirect = false;
}