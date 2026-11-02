<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import '../app.css';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import Dialog from '$lib/components/Dialog.svelte';
  import SetupWizard from '$lib/components/SetupWizard.svelte';
  import { setupI18n } from '$lib/i18n';
  import { waitLocale } from 'svelte-i18n';
  import { listen } from '@tauri-apps/api/event';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';

  onMount(() => {
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
  waitLocale().then(() => { i18nReady = true; });

  // ⭐ Appliquer le thème sauvegardé au démarrage
  if (typeof document !== 'undefined') {
    const savedTheme = localStorage.getItem('qarl-theme') || 'dark';
    document.documentElement.setAttribute('data-theme', savedTheme);
  }

  // ⭐ Appliquer le thème sauvegardé au démarrage
  if (typeof document !== 'undefined') {
    const savedTheme = localStorage.getItem('qarl-theme') || 'dark';
    document.documentElement.setAttribute('data-theme', savedTheme);
  }
  
  let { children } = $props();
</script>

<div style="display: flex; flex-direction: column; height: 100vh; overflow: hidden;">
  <Titlebar />
  <div class="app-body" style="flex: 1; display: flex; min-height: 0;">
    {#if i18nReady}
      {@render children()}
    {/if}
  </div>
  
  <!-- ⭐ Ajout de la modale globale ici (en dehors de l'app-body pour passer par-dessus tout) -->
  <Dialog />

  <!-- ⭐ Setup Wizard (1er lancement) -->
  {#if i18nReady}
    <SetupWizard />
  {/if}
</div>