<script lang="ts">
  import { t } from 'svelte-i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { setupState } from '$lib/stores/setupState.svelte.js';

  let { onContinue = () => {} } = $props();

  let status = $state<any>(null);
  let checking = $state(false);
  let autoPoll: ReturnType<typeof setInterval> | null = null;

  async function checkStatus() {
    checking = true;
    try {
      // ⭐ On passe le chemin du dossier Documents trouvé à l'étape 1
      const res = await invoke<any>('check_ac_initialized', { 
        documentsPath: setupState.data.documentsPath || null 
      });
      status = res;
      if (res) {
        setupState.data.isAcInitialized = res.initialized;
        setupState.data.hasControllers = res.hasControllers;
        setupState.data.controllersCount = res.controllersCount;
      }
    } catch (e) {
      console.warn(e);
    } finally {
      checking = false;
    }
  }

  async function launchAc() {
    try {
      await invoke('launch_ac_only');
    } catch (e) {
      console.warn('Launch AC failed', e);
    }
  }

  onMount(() => {
    checkStatus();
    autoPoll = setInterval(() => {
      if (!status?.initialized) checkStatus();
    }, 3000);
    return () => { if (autoPoll) clearInterval(autoPoll); };
  });

  let ready = $derived(status && status.initialized && status.hasControllers);
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
        {#if !status.hasControls}
          <li>❌ {$t('setup.ac_init.missing_controls')}</li>
        {:else if !status.hasControllers}
          <li>❌ {$t('setup.ac_init.missing_controllers')}</li>
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
    <button class="filter-btn" onclick={launchAc} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent);">
      {$t('setup.ac_init.launch_ac')}
    </button>
    <button class="filter-btn" onclick={checkStatus} disabled={checking}>
      {checking ? '⏳' : $t('setup.ac_init.recheck')}
    </button>
  </div>
{/if}