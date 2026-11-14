<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import '../app.css';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import Dialog from '$lib/components/Dialog.svelte';
  import SetupWizard from '$lib/components/SetupWizard.svelte';
  import AppLoadingScreen from '$lib/components/AppLoadingScreen.svelte';
  import { setupI18n } from '$lib/i18n';
  import { waitLocale } from 'svelte-i18n';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';

  onMount(() => {
    // ⭐ On retire le splash screen HTML natif (app.html) dès que Svelte est monté
    const splash = document.getElementById('global-splash');
    if (splash) {
      splash.style.opacity = '0';
      // On attend la fin de la transition CSS (0.4s) avant de le supprimer du DOM
      setTimeout(() => splash.remove(), 400);
    }

    let unlisten: any;
    // ⭐ On écoute la fermeture d'AC pour reprendre le focus
    listen('ac-closed', async () => {
      try {
        const win = getCurrentWindow();
        await win.unminimize();
        await win.setFocus();
      } catch (e) {
        console.warn("Impossible de reprendre le focus :", e);
      }
    }).then(u => unlisten = u);

    return () => { if (unlisten) unlisten(); };
  });

    setupI18n();
  let i18nReady = $state(false);
  let bootDone = $state(false);
  waitLocale().then(() => { i18nReady = true; });

  // ⭐ Appliquer le thème sauvegardé au démarrage
  if (typeof document !== 'undefined') {
    const savedTheme = localStorage.getItem('qarl-theme') || 'dark';
    document.documentElement.setAttribute('data-theme', savedTheme);
  }
  
  let { children } = $props();
</script>

<div style="display: flex; flex-direction: column; height: 100vh; overflow: hidden; background-color: var(--bg-base, #0f1219); position: relative;">
  {#if i18nReady}
    <Titlebar />
    
    <!-- ⭐ L'interface est TOUJOURS montée, mais cachée si bootDone est faux -->
    <div 
      class="app-body" 
      style="flex: 1; display: flex; min-height: 0; transition: opacity 0.3s ease;"
      style:opacity={bootDone ? 1 : 0}
      style:pointer-events={bootDone ? 'auto' : 'none'}
    >
      {@render children()}
    </div>

    <!-- ⭐ L'écran de chargement est superposé par-dessus -->
    {#if !bootDone}
      <div style="position: absolute; top: 34px; left: 0; right: 0; bottom: 0; z-index: 9999; display: flex; flex-direction: column;">
        <AppLoadingScreen onComplete={() => bootDone = true} />
      </div>
    {/if}
  {:else}
    <!-- ⭐ Fallback pendant le chargement de la langue pour éviter le white screen -->
    <div style="flex: 1; display: flex; align-items: center; justify-content: center; background-color: var(--bg-base, #0f1219);">
      <div class="spinner"></div>
    </div>
  {/if}
  
  <!-- ⭐ Ajout de la modale globale ici (en dehors de l'app-body pour passer par-dessus tout) -->
  <Dialog />

  <!-- ⭐ Setup Wizard (1er lancement) -->
  {#if i18nReady && bootDone}
    <SetupWizard />
  {/if}
</div>

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