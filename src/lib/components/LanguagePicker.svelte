<script lang="ts">
  import { locale, setLocale } from '$lib/i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';

  const LANGUAGES = [
    { code: 'fr', name: 'Français', flag: 'france'  },
    { code: 'en', name: 'English',  flag: 'uk'      },
    { code: 'es', name: 'Español',  flag: 'spain'   },
    { code: 'it', name: 'Italiano', flag: 'italy'   },
    { code: 'de', name: 'Deutsch',  flag: 'germany' }
  ];

  async function handleLanguageChange(code: string) {
    setLocale(code);
    
    // ⭐ Scénario 3 : Cliquer sur un drapeau ré-active la synchro
    localStorage.setItem('qarl_sync_game_lang', 'true');

    // ⭐ Synchronisation automatique avec Assetto Corsa
    if (appState.documentsPath) {
      try {
        const acLang = code.toLowerCase(); // fr, en, it, de, es
        await invoke('set_ac_language', { 
          documentsPath: appState.documentsPath, 
          languageCode: acLang 
        });
      } catch (e) {
        console.warn('Impossible de synchroniser la langue du jeu AC :', e);
      }
    }
  }
</script>

<div class="lang-controls">
  {#each LANGUAGES as lang}
    <button
      class="lang-btn"
      class:active={$locale === lang.code}
      onclick={() => handleLanguageChange(lang.code)}
      title={lang.name}
      type="button"
    >
      <img src={`/assets/flags/${lang.flag}23.png`} alt={lang.name} />
    </button>
  {/each}
</div>

<style>
  .lang-controls {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }
</style>