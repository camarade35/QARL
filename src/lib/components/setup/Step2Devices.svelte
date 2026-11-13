<script lang="ts">
  import { t } from 'svelte-i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { setupState } from '$lib/stores/setupState.svelte.js';

  let { onContinue = () => {} } = $props();

  let devices = $state<any[]>([]);
  let presets = $state<any[]>([]);
  let mappedGuids = $state<string[]>([]);
  let loading = $state(true);
  
  // ⭐ Nouveau : Choix du périphérique principal
  let inputMethod = $state<'X360' | 'KEYBOARD' | null>(null);

  async function setMethod(method: 'X360' | 'KEYBOARD') {
    inputMethod = method;
    // On appelle la future fonction Rust (on la fera juste après)
    try {
      await invoke('set_input_method', { 
        documentsPath: setupState.data.documentsPath, 
        method: method 
      });
    } catch (e) {
      console.error("Erreur lors de la modification du header:", e);
    }
  }

  async function load() {
    loading = true;
    try {
      const [devs, pres, entries] = await Promise.all([
        invoke<any[]>('get_directinput_devices').catch(() => []),
        invoke<any[]>('list_controller_presets', { acPath: setupState.data.acPath }).catch(() => []),
        invoke<any[]>('parse_controllers_section', { documentsPath: setupState.data.documentsPath }).catch(() => []),
      ]);
      devices = devs;
      presets = pres;
      
      // ⭐ On aplatit la liste pour avoir tous les IGUID et PGUID valides (en minuscules pour comparer sans souci de casse)
      mappedGuids = entries.flatMap((e: any) => [
        (e.iguid || '').toLowerCase(), 
        (e.pguid || '').toLowerCase()
      ]).filter(Boolean);
      
    } finally {
      loading = false;
    }
  }

  function findPresetForDevice(deviceName: string): any | null {
    if (!deviceName) return null;
    const lower = deviceName.toLowerCase();
    // Match partiel : "Fanatec ClubSport V2.5" matche "Fanatec ClubSport V2.5.ini"
    for (const p of presets) {
      const pn = (p.name || '').toLowerCase();
      const pf = (p.filename || '').toLowerCase().replace('.ini', '');
      if (lower.includes(pn) || pn.includes(lower) || lower.includes(pf) || pf.includes(lower)) {
        return p;
      }
    }
    return null;
  }

  function openControls() {
    // Redirige vers Paramètres > Contrôles
    appState.activeTab = 'tab-settings';
    window.dispatchEvent(new CustomEvent('qarl-open-section', {
      detail: { tab: 'subtab-controls', group: '', cspFile: null }
    }));
    // Demande au wizard de se cacher (bandeau flottant)
    window.dispatchEvent(new CustomEvent('qarl-setup-hide'));
  }

  onMount(load);
</script>

{#if loading}
  <div style="text-align: center; padding: 40px; color: var(--text-secondary);">
    {$t('setup.devices.loading')}
  </div>
{:else if devices.length === 0}
  <div class="setup-warning">
    <strong>{$t('setup.devices.no_devices_title', { values: { count: devices.length } })}</strong>
    <p style="margin-top: 8px; white-space: pre-line;">
      {$t('setup.devices.no_devices_desc')}
    </p>
  </div>
{:else}
  <p style="color: var(--text-secondary); margin-bottom: 16px;">
    {$t('setup.devices.intro', { values: { count: devices.length } })}
  </p>

  <div style="display: flex; flex-direction: column; gap: 8px; margin-bottom: 16px;">
    {#each devices as dev}
      {@const isMapped = mappedGuids.includes((dev.iguid || '').toLowerCase()) || mappedGuids.includes((dev.pguid || '').toLowerCase())}
      {@const preset = findPresetForDevice(dev.name)}
      <div style="display: flex; align-items: center; gap: 12px; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
        <span style="font-size: 1.4rem;">{isMapped ? '✅' : (preset ? 'ℹ️' : '⚠️')}</span>
        <div style="flex: 1; min-width: 0;">
          <div style="font-weight: 600; font-size: 0.88rem; color: var(--text-primary);">{dev.name}</div>
          {#if isMapped}
            <div style="font-size: 0.75rem; color: #6fc98a; font-weight: 600;">
              {$t('setup.devices.status_mapped')}
            </div>
          {:else if preset}
            <div style="font-size: 0.75rem; color: var(--accent);">
              {$t('setup.devices.status_template')}
            </div>
          {:else}
            <div style="font-size: 0.75rem; color: #e6a33b;">
              {$t('setup.devices.status_missing')}
            </div>
          {/if}
        </div>
      </div>
    {/each}
  </div>

  <div class="setup-hint" style="font-size: 0.8rem; line-height: 1.6;">
    {$t('setup.devices.hint')}
  </div>
{/if}

  <!-- ⭐ NOUVEAU : Choix du périphérique principal -->
  <div style="background: var(--bg-elevated); border: 1px solid var(--border); padding: 16px; border-radius: var(--r-sm); margin-top: 20px;">
    <div style="font-weight: 700; font-size: 0.9rem; margin-bottom: 12px; color: var(--text-primary); text-align: center;">
      {$t('setup.devices.input_method_title')}
    </div>
    <div style="display: flex; gap: 12px; justify-content: center;">
      <button 
        class="filter-btn" 
        class:active={inputMethod === 'X360'}
        onclick={() => setMethod('X360')} 
        style="flex: 1; padding: 15px; display: flex; flex-direction: column; align-items: center; gap: 8px; font-size: 0.9rem; font-weight: 700; border: 2px solid {inputMethod === 'X360' ? 'var(--accent)' : 'var(--border)'}; background: {inputMethod === 'X360' ? 'var(--accent-muted)' : 'var(--bg-panel)'}; color: {inputMethod === 'X360' ? 'var(--accent)' : 'var(--text-primary)'};"
      >
        <span style="font-size: 2rem;">🎮</span>
        {$t('setup.devices.btn_gamepad')}
      </button>
      <button 
        class="filter-btn" 
        class:active={inputMethod === 'KEYBOARD'}
        onclick={() => setMethod('KEYBOARD')} 
        style="flex: 1; padding: 15px; display: flex; flex-direction: column; align-items: center; gap: 8px; font-size: 0.9rem; font-weight: 700; border: 2px solid {inputMethod === 'KEYBOARD' ? 'var(--accent)' : 'var(--border)'}; background: {inputMethod === 'KEYBOARD' ? 'var(--accent-muted)' : 'var(--bg-panel)'}; color: {inputMethod === 'KEYBOARD' ? 'var(--accent)' : 'var(--text-primary)'};"
      >
        <span style="font-size: 2rem;">⌨️</span>
        {$t('setup.devices.btn_keyboard')}
      </button>
    </div>
  </div>

  <div style="display: flex; gap: 10px; justify-content: flex-end; margin-top: 20px;">
    <button class="filter-btn" onclick={load} disabled={loading} style="padding: 10px 20px; margin-right: auto;">
      {loading ? '⏳' : $t('setup.ac_init.recheck')}
    </button>
    <button class="filter-btn" onclick={openControls} style="padding: 10px 20px;">
      {$t('setup.devices.open_controls')}
    </button>
    <button class="filter-btn" onclick={onContinue} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent); font-weight: 700;">
      {$t('setup.devices.continue')}
    </button>
  </div>