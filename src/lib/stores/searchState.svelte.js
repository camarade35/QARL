// src/lib/stores/searchState.svelte.js
export const searchState = $state({
  query: '',
  autosaveDelay: 0
});

// Restauration de l'autosave
if (typeof localStorage !== 'undefined') {
  const saved = localStorage.getItem('qarl_autosave_delay');
  if (saved !== null) {
    searchState.autosaveDelay = parseInt(saved, 10) || 0;
  }
}

export function setSearchQuery(q) {
  searchState.query = q;
}

export function clearSearch() {
  searchState.query = '';
}

export function setAutosaveDelay(v) {
  searchState.autosaveDelay = Number(v) || 0;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_autosave_delay', String(searchState.autosaveDelay));
  }
}