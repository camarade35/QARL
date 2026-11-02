<script lang="ts">
  import { locale, setLocale } from '$lib/i18n';

  const LANGUAGES = [
    { code: 'fr', name: 'Français', flag: 'france' },
    { code: 'en', name: 'English',  flag: 'uk'     },
    { code: 'es', name: 'Español',  flag: 'spain'  }
  ];
</script>

<div class="lang-controls">
  {#each LANGUAGES as lang}
    <button
      class="lang-btn"
      class:active={$locale === lang.code}
      onclick={() => setLocale(lang.code)}
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