import { register, init, locale } from 'svelte-i18n';

// Enregistrement des langues disponibles
// Le chargement du JSON est lazy (fait au changement de locale)
register('fr', () => import('./locales/fr.json'));
register('en', () => import('./locales/en.json'));
register('es', () => import('./locales/es.json'));
register('it', () => import('./locales/it.json')); // ⭐ NOUVEAU
register('de', () => import('./locales/de.json')); // ⭐ NOUVEAU

const AVAILABLE = ['fr', 'en', 'es', 'it', 'de']; // ⭐ Ajout de it et de

function getInitialLocale(): string {
  if (typeof localStorage === 'undefined') return 'fr';
  const saved = localStorage.getItem('qarl_locale');
  if (saved && AVAILABLE.includes(saved)) return saved;
  return 'fr';
}

export function setupI18n() {
  init({
    fallbackLocale: 'fr',
    initialLocale: getInitialLocale(),
  });
}

// Helper pour changer de langue + persist
export function setLocale(code: string) {
  locale.set(code);
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_locale', code);
  }
}

export { locale };