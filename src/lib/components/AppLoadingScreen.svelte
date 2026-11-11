<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { t } from 'svelte-i18n';
  import { fade } from 'svelte/transition';

  let { onComplete } = $props();

  let step = $state(0); // 0 = voitures, 1 = circuits, 2 = prêt
  let visible = $state(true);

  async function run() {
    // Pas de chemin AC configuré → on saute le chargement
    if (!appState.acPath) {
      done();
      return;
    }

    try {
      // 1. Voitures
      step = 0;
      const cars = await invoke<any[]>('get_ac_cars', { acPath: appState.acPath });
      appState.cars = cars;

      // 2. Circuits
      step = 1;
      const tracks = await invoke<any[]>('get_ac_tracks', { acPath: appState.acPath });
      appState.tracks = tracks;

      // 3. Prêt
      step = 2;
      appState.dataLoadedForPath = appState.acPath;

      await new Promise((r) => setTimeout(r, 800)); // laisse voir "Tout est prêt !"
      done();
    } catch (e) {
      console.error('AppLoadingScreen error:', e);
      done(); // on ne bloque pas l'utilisateur, les tabs ont leurs propres fallbacks
    }
  }

  function done() {
    visible = false;
    setTimeout(() => onComplete?.(), 400); // laisse le fade out
  }

  $effect(() => {
    run();
  });
</script>

{#if visible}
  <div
    class="loading-screen"
    transition:fade={{ duration: 400 }}
    style="flex: 1; display: flex; align-items: center; justify-content: center; background: var(--bg-base, #0f1219);"
  >
    <div style="text-align: center; max-width: 420px; padding: 20px;">
      <!-- ⭐ Le Logo -->
      <img src="/assets/logo.png" alt="QARL" style="width: 160px; height: auto; margin: 0 auto 24px; display: block; user-select: none; -webkit-user-drag: none;" />
      
      <div class="spinner" style="margin: 0 auto 24px;"></div>

      {#if step === 0}
        <p style="color: var(--text-primary); font-size: 1.1rem;">{$t('boot.loading_cars')}</p>
      {:else if step === 1}
        <p style="color: var(--text-primary); font-size: 1.1rem;">{$t('boot.loading_tracks')}</p>
      {:else}
        <p style="color: var(--accent, #e6a33b); font-size: 1.1rem; font-weight: 600;">{$t('boot.ready')}</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .spinner {
    width: 48px;
    height: 48px;
    border: 4px solid rgba(255, 255, 255, 0.12);
    border-top-color: var(--accent, #e6a33b);
    border-radius: 50%;
    animation: spin 0.9s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>