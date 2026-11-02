<script lang="ts">
  import { t } from 'svelte-i18n';
  import { invoke, Channel } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { setupState } from '$lib/stores/setupState.svelte.js';
  import { open } from '@tauri-apps/plugin-dialog';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { getCurrentWebview } from '@tauri-apps/api/webview';

  let { onContinue = () => {} } = $props();

  let loading = $state(true);
  let status = $state<any>(null);
  
  // ⭐ États pour l'installateur
  let isDragging = $state(false);
  let isInstalling = $state(false);
  let forceShowDropzone = $state(false); // ⭐ NOUVEAU
  let installProgress = $state(0);
  let currentFile = $state('');
  let unlistenDragDrop: any;

  async function handleFileSelect(filePath: string) {
    if (!filePath) return;
    if (!filePath.match(/\.(zip|rar|7z)$/i)) {
      alert($t('setup.csp.install_error') + " (Format non supporté)");
      return;
    }

    isInstalling = true;
    installProgress = 0;
    currentFile = 'Initialisation...';

    try {
      const channel = new Channel<{ type: string, percent?: number, current_file?: string }>();
      channel.onmessage = (msg) => {
        if (msg.type === 'progress') {
          installProgress = msg.percent || 0;
          currentFile = msg.current_file || '';
        }
      };

      await invoke('install_csp_archive', {
        acPath: appState.acPath || setupState.data.acPath,
        archivePath: filePath,
        onEvent: channel
      });

      await load(); // On rafraîchit le statut
      alert($t('setup.csp.install_success'));
    } catch (e) {
      alert($t('setup.csp.install_error') + e);
    } finally {
      isInstalling = false;
      installProgress = 0;
      currentFile = '';
    }
  }

  async function browseArchive() {
    const selected = await open({
      multiple: false,
      filters: [{ name: 'Archive', extensions: ['zip', 'rar', '7z'] }]
    });
    if (selected) {
      handleFileSelect(selected as string);
    }
  }

  async function load() {
    loading = true;
    try {
      const res = await invoke<any>('detect_setup_status');
      status = res;
      if (res) {
        setupState.data.hasCsp = res.has_csp;
        setupState.data.hasSol = res.has_sol;
        setupState.data.hasPure = res.has_pure;
        setupState.data.cspVersion = res.csp_version;
        setupState.data.solVersion = res.sol_version;
        setupState.data.pureVersion = res.pure_version;
        setupState.data.carsCount = res.cars_count;
        setupState.data.tracksCount = res.tracks_count;
      }
    } finally {
      loading = false;
    }
  }

  function goToCspTab() {
    appState.activeTab = 'tab-settings';
    window.dispatchEvent(new CustomEvent('qarl-open-section', {
      detail: { tab: 'subtab-csp', group: '', cspFile: null }
    }));
    // ⭐ On demande au wizard de se cacher (bandeau flottant)
    window.dispatchEvent(new CustomEvent('qarl-setup-hide'));
  }

  onMount(() => {
    load();

    // ⭐ Écouteur pour le Drag & Drop
    getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'over') {
        isDragging = true;
      } else if (event.payload.type === 'drop') {
        isDragging = false;
        if (event.payload.paths.length > 0 && !isInstalling) {
          handleFileSelect(event.payload.paths[0]);
        }
      } else {
        isDragging = false;
      }
    }).then(unlisten => { unlistenDragDrop = unlisten; });

    return () => {
      if (unlistenDragDrop) unlistenDragDrop();
    };
  });

  let state: 'none' | 'csp_only' | 'all_good' | 'loading' = $derived(
    loading ? 'loading' :
    !status ? 'none' :
    !status.has_csp && !status.has_sol && !status.has_pure ? 'none' :
    status.has_csp && !status.has_sol && !status.has_pure ? 'csp_only' :
    'all_good'
  );
</script>

{#if loading}
  <div style="text-align: center; padding: 40px; color: var(--text-secondary);">
    ⏳ Analyse des modules…
  </div>
{:else}
  {#if state === 'none'}
    <div class="setup-warning">
      <strong>⚠️ {$t('setup.csp.none_installed')}</strong>
      <p style="margin-top: 8px; white-space: pre-line;">{$t('setup.csp.none_desc')}</p>
    </div>
  {:else if state === 'csp_only'}
    <div class="setup-warning">
      <strong>⚠️ {$t('setup.csp.csp_only')}</strong>
      <p style="margin-top: 8px;">{$t('setup.csp.csp_only_desc')}</p>
    </div>
  {:else}
    <div class="setup-success" style="display: flex; justify-content: space-between; align-items: center;">
      <span>✅ {$t('setup.csp.all_good')}</span>
      {#if !forceShowDropzone}
        <button class="btn-secondary" style="font-size: 0.75rem; padding: 4px 8px;" onclick={() => forceShowDropzone = true}>
          🔄 {$t('setup.csp.btn_update')}
        </button>
      {/if}
    </div>
    <p style="color: var(--text-secondary); margin-bottom: 16px;">{$t('setup.csp.all_desc')}</p>
  {/if}

  {#if state === 'none' || state === 'csp_only' || forceShowDropzone}
    <!-- ⭐ CARTES D'INSTALLATION -->
    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 15px; margin-bottom: 20px;">
      <!-- Carte Gratuit (CSP + Sol) -->
      <div style="background: var(--bg-elevated); border: 1px solid #3b82f6; border-radius: var(--r); padding: 16px; text-align: center; display: flex; flex-direction: column;">
        <div style="font-size: 2rem; margin-bottom: 8px;">🟢</div>
        <h3 style="margin: 0; font-size: 1.1rem; color: #3b82f6;">{$t('setup.csp.card_free_title')}</h3>
        <p style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 8px; line-height: 1.4; flex: 1;">{$t('setup.csp.card_free_desc')}</p>
        
        <div style="display: flex; flex-direction: column; gap: 8px; margin-top: 15px;">
          <button class="filter-btn" onclick={() => openUrl('https://acstuff.club/patch/?get=0.1.79')} style="font-size: 0.8rem; padding: 6px; width: 100%;">
            ⬇️ Télécharger CSP (acstuff.club)
          </button>
          <button class="filter-btn" onclick={() => openUrl('https://www.overtake.gg/downloads/sol.24914/')} style="font-size: 0.8rem; padding: 6px; width: 100%;">
            ⬇️ Télécharger Sol (Overtake)
          </button>
        </div>
      </div>

      <!-- Carte Premium (CSP + Pure) -->
      <div style="background: var(--bg-elevated); border: 1px solid #e6a33b; border-radius: var(--r); padding: 16px; text-align: center; display: flex; flex-direction: column;">
        <div style="font-size: 2rem; margin-bottom: 8px;">👑</div>
        <h3 style="margin: 0; font-size: 1.1rem; color: #e6a33b;">{$t('setup.csp.card_premium_title')}</h3>
        <p style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 8px; line-height: 1.4; flex: 1;">{$t('setup.csp.card_premium_desc')}</p>
        
        <div style="display: flex; flex-direction: column; gap: 8px; margin-top: 15px;">
          <button class="filter-btn" onclick={() => openUrl('https://acstuff.club/patch/?get=0.2.11')} style="font-size: 0.8rem; padding: 6px; width: 100%;">
            ⬇️ Télécharger CSP (Dernière version)
          </button>
          <button class="filter-btn" onclick={() => openUrl('https://www.patreon.com/pure')} style="font-size: 0.8rem; padding: 6px; width: 100%;">
            ⬇️ Télécharger Pure (Patreon)
          </button>
        </div>
      </div>
    </div>

    <!-- ⭐ DROP ZONE -->
    <div 
      style="
        border: 2px dashed {isDragging ? 'var(--accent)' : 'var(--border)'}; 
        border-radius: var(--r); 
        padding: 30px 20px; 
        text-align: center; 
        background: {isDragging ? 'rgba(111, 201, 138, 0.1)' : 'var(--bg-panel)'};
        transition: all 0.2s;
        margin-bottom: 20px;
        cursor: pointer;
      "
      onclick={browseArchive}
      role="button"
      tabindex="0"
      onkeydown={(e) => e.key === 'Enter' && browseArchive()}
    >
      {#if isInstalling}
        <div style="font-size: 1.5rem; margin-bottom: 10px;">⏳</div>
        <div style="font-weight: 600; color: var(--accent); margin-bottom: 8px;">{$t('setup.csp.installing')}</div>
        <div style="font-size: 0.8rem; color: var(--text-secondary); margin-bottom: 10px; word-break: break-all;">{currentFile}</div>
        <div style="width: 100%; height: 8px; background: var(--bg-elevated); border-radius: 4px; overflow: hidden;">
          <div style="height: 100%; width: {installProgress}%; background: var(--accent); transition: width 0.2s;"></div>
        </div>
        <div style="font-size: 0.8rem; color: var(--text-secondary); margin-top: 5px;">{Math.round(installProgress)}%</div>
      {:else}
        <div style="font-size: 2rem; margin-bottom: 10px;">📥</div>
        <div style="font-weight: 700; font-size: 1rem; color: var(--text-primary); margin-bottom: 8px;">{$t('setup.csp.drop_title')}</div>
        <div style="font-size: 0.85rem; color: var(--text-secondary); margin-bottom: 16px;">{$t('setup.csp.drop_desc')}</div>
        <button class="filter-btn" style="padding: 8px 16px; font-weight: 600;">
          {$t('setup.csp.btn_browse')}
        </button>
      {/if}
    </div>
  {/if}

  <div style="display: flex; flex-direction: column; gap: 10px; margin-bottom: 20px;">
    <div style="display: flex; justify-content: space-between; padding: 12px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
      <span style="font-weight: 600;">{status?.has_csp ? '✅' : '❌'} {$t('setup.csp.csp_label')}</span>
      <span style="color: var(--text-secondary); font-size: 0.82rem;">
        {#if status?.has_csp}
          {$t('setup.csp.version_prefix')}{status.csp_version}
        {:else}
          {$t('setup.csp.not_installed')}
        {/if}
      </span>
    </div>
    <div style="display: flex; justify-content: space-between; padding: 12px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
      <span style="font-weight: 600;">{status?.has_sol ? '✅' : '❌'} {$t('setup.csp.sol_label')}</span>
      <span style="color: var(--text-secondary); font-size: 0.82rem;">
        {#if status?.has_sol}
          {$t('setup.csp.version_prefix')}{status.sol_version}
        {:else}
          {$t('setup.csp.not_installed')}
        {/if}
      </span>
    </div>
    <div style="display: flex; justify-content: space-between; padding: 12px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
      <span style="font-weight: 600;">{status?.has_pure ? '✅' : '❌'} {$t('setup.csp.pure_label')}</span>
      <span style="color: var(--text-secondary); font-size: 0.82rem;">
        {#if status?.has_pure}
          {$t('setup.csp.version_prefix')}{status.pure_version}
        {:else}
          {$t('setup.csp.not_installed')}
        {/if}
      </span>
    </div>
  </div>
{/if}

<div style="display: flex; gap: 10px; justify-content: flex-end;">
  <button class="filter-btn" onclick={goToCspTab} style="padding: 10px 20px;">
    {$t('setup.csp.go_csp_tab')}
  </button>
  <button class="filter-btn" onclick={onContinue} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent); font-weight: 700;">
    {$t('setup.csp.continue')}
  </button>
</div>