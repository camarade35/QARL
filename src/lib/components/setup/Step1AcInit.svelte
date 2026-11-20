<script lang="ts">
  import { t } from 'svelte-i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { setupState } from '$lib/stores/setupState.svelte.js';

  type AcInitStatus = {
    initialized: boolean;
    hasRace: boolean;
    hasControls: boolean;
    hasControllers: boolean;
    controllersCount: number;
  };

  let { onContinue = () => {} } = $props();

  let status = $state<AcInitStatus | null>(null);
  let checking = $state(false);
  let isLaunching = $state(false);

  async function checkStatus() {
    checking = true;
    try {
      const res = await invoke<AcInitStatus>('check_ac_initialized', {
        documentsPath: setupState.data.documentsPath || null 
      });
      status = res;
      setupState.data.isAcInitialized = res.initialized;
      setupState.data.hasControllers = res.hasControllers;
      setupState.data.controllersCount = res.controllersCount;
    } catch (e) {
      console.warn(e);
    } finally {
      checking = false;
    }
  }

  async function launchAc() {
    isLaunching = true;
    try {
      await invoke('launch_ac_only');
      status = null;
    } catch (e) {
      alert("Erreur lors du lancement d'AC via Steam : " + e);
    } finally {
      isLaunching = false;
    }
  }

  // AC crée race.ini au premier lancement ; le contrôle continue jusqu'à son apparition.
  onMount(() => {
    checkStatus();
    const autoPoll = setInterval(() => {
      if (!status?.initialized) checkStatus();
    }, 3000);
    return () => clearInterval(autoPoll);
  });

  // La configuration des contrôleurs se fait à l'étape 3 et ne bloque pas celle-ci.
  let ready = $derived(status && status.initialized);
</script>

{#if ready}
  <div class="setup-success">
    ✅ {$t('setup.ac_init.ready')}
  </div>
  <div style="display: flex; justify-content: flex-end; margin-top: 20px;">
    <button class="filter-btn" onclick={onContinue} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent); font-weight: 700;">
      {$t('setup.ac_init.continue')}
    </button>
  </div>
{:else}
  <div class="setup-warning">
    <strong>⚠️ {$t('setup.ac_init.title')}</strong>
    <p style="margin-top: 8px;">{$t('setup.ac_init.intro')}</p>
    {#if status}
      <ul style="margin-top: 10px; padding-left: 20px;">
        {#if !status.hasRace}
          <li>❌ {$t('setup.ac_init.missing_race')}</li>
        {/if}
        {#if status.hasControls && !status.hasControllers}
          <li>ℹ️ {$t('setup.ac_init.missing_controllers')}</li>
        {/if}
      </ul>
    {/if}
  </div>

  <div class="setup-hint" style="white-space: pre-line; line-height: 1.8;">
    {$t('setup.ac_init.explain')}
  </div>

  <div class="setup-hint" style="margin-top: 12px; font-style: italic;">
    {$t('setup.ac_init.auto_polling')}
  </div>

  <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px;">
    <button class="filter-btn" onclick={launchAc} disabled={isLaunching} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent); {isLaunching ? 'opacity: 0.5; cursor: wait;' : ''}">
      {isLaunching ? '⏳ Lancement...' : $t('setup.ac_init.launch_ac')}
    </button>
    <button class="filter-btn" onclick={checkStatus} disabled={checking}>
      {checking ? '⏳' : $t('setup.ac_init.recheck')}
    </button>
  </div>
{/if}