<script lang="ts">
  import { t, locale } from 'svelte-i18n';
  import { alert } from '$lib/stores/dialog.svelte'; // ⭐ AJOUT IMPORT QARL
  import { onMount } from 'svelte';
  import { invoke, Channel } from '@tauri-apps/api/core';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { open } from '@tauri-apps/plugin-dialog';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import ModAppModal from '$lib/components/ModAppModal.svelte';
  import ModInfoModal from '$lib/components/ModInfoModal.svelte';


  let isDragging = $state(false);
  let isInstalling = $state(false);
  let installLogs = $state<string[]>([]);
  let modsHistory = $state<any[]>([]);

  // ⭐ Nouveaux états pour la progression
  let progressPercent = $state(0);
  let currentFileName = $state('');
  let currentArchiveIndex = $state(0);
  let totalArchives = $state(0);

  // ⭐ Nouveaux états pour l'UI
  let isHistoryOpen = $state(true);

  type CrashFxData = {
  installed: boolean;
  enabled: boolean;
  volume_percent: number;
  };

  let showCrashFxModal = $state(false);
  let crashFxError = $state('');
  let crashFxData = $state<CrashFxData>({
    installed: false,
    enabled: false,
    volume_percent: 100
  });

  type ModAppsStatus = {
    cmrt_installed: boolean;
    map_display_installed: boolean;
  };

  let modAppsStatus = $state<ModAppsStatus>({
    cmrt_installed: false,
    map_display_installed: false
  });
  let modAppsError = $state('');
  let showCmrtModal = $state(false);
  let showMapDisplayModal = $state(false);

  async function loadModAppsStatus() {
    if (!appState.acPath) return;

    try {
      modAppsStatus = await invoke<ModAppsStatus>('read_mod_apps_status', {
        acPath: appState.acPath
      });
      modAppsError = '';
    } catch (e) {
      modAppsError = String(e);
    }
  }

  async function openModDownload(url: string) {
    modAppsError = '';
    try {
      await openUrl(url);
    } catch (e) {
      modAppsError = String(e);
    }
  }

  async function loadCrashFxData() {
    if (!appState.acPath) return;

    try {
      crashFxData = await invoke<CrashFxData>('read_crashfx_data', {
        acPath: appState.acPath,
        documentsPath: appState.documentsPath
      });
      crashFxError = '';
    } catch (e) {
  crashFxData = { ...crashFxData, installed: true };
  crashFxError = String(e);
    }
  }

  async function saveCrashFxData(enabled: boolean, volumePercent: number) {
    if (!appState.acPath || !crashFxData.installed) return;

    try {
      crashFxError = '';
      await invoke('save_crashfx_data', {
        acPath: appState.acPath,
        documentsPath: appState.documentsPath,
        enabled,
        volumePercent
      });
      crashFxData = { ...crashFxData, enabled, volume_percent: volumePercent };
    } catch (e) {
      crashFxError = String(e);
    }
  }

  async function openCrashFxPage() {
    try {
      await openUrl('https://www.overtake.gg/');
    } catch (e) {
      crashFxError = String(e);
    }
  }

    const archiveExtensions = [
    '7z', 'zip', 'rar', 'tar', 'gz', 'bz2', 'xz', 'wim',
    'ar', 'arj', 'cab', 'chm', 'cpio', 'cramfs', 'dmg',
    'ext', 'fat', 'gpt', 'hfs', 'ihex', 'iso', 'lzh',
    'lzma', 'mbr', 'msi', 'nsis', 'ntfs', 'qcow2',
    'rpm', 'squashfs', 'udf', 'vdi', 'vhd', 'vhdx',
    'vmdk', 'xar', 'z'
  ] as const;

  async function loadHistory() {
    try {
      modsHistory = await invoke<any[]>('get_mods_history', {
        acPath: appState.acPath,
        documentsPath: appState.documentsPath
      });
    } catch (e) { console.error(e); }
  }

  onMount(async () => {
    loadHistory();
    void loadCrashFxData();
    void loadModAppsStatus();

    // ⭐ Écoute du Drag & Drop natif de Tauri 2
    const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
      if (event.payload.type === 'enter') {
        isDragging = true;
      } else if (event.payload.type === 'leave') {
        isDragging = false;
      } else if (event.payload.type === 'drop') {
        isDragging = false;
        handleDrop(event.payload.paths);
      }
    });

    return () => unlisten();
  });

    async function browseArchives() {
    const selected = await open({
      multiple: true,
      directory: false,
      filters: [{
        name: 'Archives compatibles 7-Zip',
        extensions: [...archiveExtensions]
      }]
    });

    if (selected) {
      await handleDrop(Array.isArray(selected) ? selected : [selected]);
    }
  }

  async function handleDrop(paths: string[]) {
    if (!appState.acPath) {
      alert($t('mods.error_path'));
      return;
    }
    const validArchives = paths.filter(path =>
  archiveExtensions.some(extension => path.toLowerCase().endsWith(`.${extension}`))
);
    if (validArchives.length === 0) {
      alert($t('mods.error_archive'));
      return;
    }

    isInstalling = true;
    totalArchives = validArchives.length;
    currentArchiveIndex = 0;

    for (const file of validArchives) {
      currentArchiveIndex++;
      const fileName = file.split(/[\/\\]/).pop() || file;
      installLogs = [$t('mods.log_analyzing', { values: { name: fileName } }), ...installLogs];
      progressPercent = 0;
      currentFileName = $t('mods.log_archive_analysis');

      // ⭐ Création du canal d'écoute
      const onEvent = new Channel<any>();
      onEvent.onmessage = (message) => {
        if (message.type === 'start') {
          installLogs = [$t('mods.log_extracting', { values: { name: fileName } }), ...installLogs];
          currentFileName = $t('mods.log_starting_extraction');
        } else if (message.type === 'progress') {
          progressPercent = message.percent;
          // ⭐ Sécurité : on vérifie que current_file existe avant de split
          const shortName = message.current_file && message.current_file.trim() !== ''
            ? message.current_file.split(/[\/\\]/).pop()
            : $t('mods.log_extracting', { values: { name: fileName } });
          currentFileName = shortName || 'Extraction...';
        }
      };

      try {
        const result = await invoke<string>('install_mod_from_archive', {
          acPath: appState.acPath,
          documentsPath: appState.documentsPath,
          archivePath: file,
          onEvent: onEvent // ⭐ On passe le canal à Rust
        });
        installLogs = [`✅ ${result}`, ...installLogs];
        await loadHistory();
        await loadCrashFxData();
        await loadModAppsStatus();
      } catch (e) {
        installLogs = [`❌ Erreur pour ${fileName}: ${e}`, ...installLogs];
      }
    }

    isInstalling = false;
    progressPercent = 0;
    currentFileName = '';
  }

  function formatDate(ts: string) {
    return new Date(parseInt(ts) * 1000).toLocaleString($locale);
  }
</script>

<div class="group">
  <div class="group-title">{$t('mods.install_title')}</div>

  <div
    style="display: grid; grid-template-columns: minmax(0, 1fr) minmax(0, 1fr); gap: 12px; margin-top: 12px;"
  >
    <div
      style="border: 2px dashed {isDragging ? 'var(--accent)' : 'var(--border)'};
             background: {isDragging ? 'var(--accent-muted)' : 'var(--bg-elevated)'};
             border-radius: 8px; padding: 18px; min-height: 150px;
             display: flex; flex-direction: column; justify-content: center; align-items: flex-start;"
    >
      <div style="font-size: 1.5rem; margin-bottom: 6px;">📦</div>

      <div style="font-weight: 600; color: var(--text-primary); margin-bottom: 4px;">
        {isDragging ? $t('mods.drop_active') : $t('mods.drop_title')}
      </div>

      <div style="font-size: 0.8rem; color: var(--text-tertiary); line-height: 1.4;">
        {$t('mods.drop_desc')}
      </div>

      <button
        class="filter-btn"
        style="margin-top: 10px;"
        onclick={browseArchives}
        disabled={isInstalling}
      >
        {$t('mods.browse')}
      </button>

      {#if isInstalling}
        <div style="margin-top: 12px; width: 100%;">
          <div style="color: var(--accent); font-weight: bold; font-size: 0.85rem;">
            {$t('mods.extracting_archive', { values: { current: currentArchiveIndex, total: totalArchives } })}
          </div>

          <div style="margin-top: 6px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; height: 10px; overflow: hidden;">
            <div style="background: var(--accent); height: 100%; width: {progressPercent}%; transition: width 0.1s ease-out;"></div>
          </div>

          <div style="font-size: 0.75rem; color: var(--text-secondary); margin-top: 5px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
            {currentFileName}
          </div>
        </div>
      {/if}
    </div>

    <div
      style="min-width: 0; min-height: 150px; max-height: 210px; overflow-y: auto;
             background: #0f1219; border: 1px solid var(--border); border-radius: 8px;
             padding: 12px; font-family: monospace; font-size: 0.8rem;"
    >
      <div style="font-weight: 600; color: var(--text-secondary); margin-bottom: 8px;">
        {$t('mods.log_title')}
      </div>

      {#if installLogs.length === 0}
        <div style="color: var(--text-tertiary);">{$t('mods.log_waiting')}</div>
      {:else}
        {#each installLogs as log}
          <div style="margin-bottom: 4px; overflow-wrap: anywhere; color: {log.startsWith('✅') ? '#4ade80' : log.startsWith('❌') ? '#f87171' : 'var(--text-secondary)'};">
            {log}
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<div class="group">
  <div class="group-title">{$t('mods.apps_mods_title')}</div>

  <div class="mods-app-grid">
    <section class="mods-app-column">
      <h4 class="mods-column-title">{$t('mods.apps_title')}</h4>

      <button
        class="mod-app-card"
        class:active={crashFxData.installed && crashFxData.enabled}
        onclick={() => showCrashFxModal = true}
        aria-label={$t('mods.crashfx_open')}
      >
        <img src="/assets/CrashFX/logo.png" alt="CrashFX" />
        <span>by JWL</span>
        <span class="mod-app-add-icon" aria-hidden="true">+</span>
      </button>

      <button
        class="mod-app-card"
        class:active={modAppsStatus.cmrt_installed}
        onclick={() => {
          modAppsError = '';
          showCmrtModal = true;
        }}
        aria-label={$t('mods.cmrt_open')}
      >
        <img src="/assets/CMRT/logo.png" alt="" />
        <span>CMRT</span>
        <span class="mod-app-add-icon" aria-hidden="true">+</span>
      </button>

      <button
        class="mod-app-card"
        class:active={modAppsStatus.map_display_installed}
        onclick={() => {
          modAppsError = '';
          showMapDisplayModal = true;
        }}
        aria-label={$t('mods.mapdisplay_open')}
      >
        <img src="/assets/MapDisplay/logo.png" alt="" />
        <span>by AW939</span>
        <span class="mod-app-add-icon" aria-hidden="true">+</span>
      </button>
    </section>
  </div>
  {#if modAppsError}
    <p class="app-status-error">{$t('common.error')}: {modAppsError}</p>
  {/if}
</div>

<div class="group">
  <button
    type="button"
    class="mods-history-toggle"
    aria-expanded={isHistoryOpen}
    onclick={() => isHistoryOpen = !isHistoryOpen}
  >
    <span class="accordion-arrow" class:open={isHistoryOpen}>▸</span>
    <span class="mods-column-title">{$t('mods.mods_title')}</span>
    <span class="accordion-line"></span>
  </button>

  {#if isHistoryOpen}
    {#if modsHistory.filter(m => m.exists).length === 0}
      <div class="mods-history-empty">{$t('mods.history_empty')}</div>
    {:else}
      <div class="mods-history-list">
        {#each modsHistory.filter(m => m.exists) as mod}
          <div class="mods-history-item">
            <div>
              <div style="font-weight: 600; font-size: 0.9rem;">{mod.mod_name}</div>
              <div style="font-size: 0.75rem; color: var(--text-secondary);">
                {$t('mods.history_type', { values: { type: mod.mod_type, date: formatDate(mod.date) } })}
              </div>
            </div>

            <button
              class="filter-btn"
              style="padding: 4px 8px; font-size: 0.75rem;"
              title={$t('mods.history_view')}
              onclick={() => {
                invoke('open_folder', {
                  acPath: appState.acPath,
                  documentsPath: appState.documentsPath,
                  folderType: 'mod',
                  folderId: mod.path_installed
                }).catch(e => alert($t('mods.open_folder_error') + e));
              }}
            >
              {$t('mods.history_view')}
            </button>
          </div>
        {/each}
      </div>
    {/if}
  {/if}
  </div>


<ModAppModal
  show={showCrashFxModal}
  installed={crashFxData.installed}
  enabled={crashFxData.enabled}
  volumePercent={crashFxData.volume_percent}
  error={crashFxError}
  onClose={() => showCrashFxModal = false}
  onDownload={openCrashFxPage}
  onEnabledChange={(enabled) => saveCrashFxData(enabled, crashFxData.volume_percent)}
  onVolumeChange={(volume) => saveCrashFxData(crashFxData.enabled, volume)}
/>

<ModInfoModal
  show={showCmrtModal}
  title="CMRT"
  logo="/assets/CMRT/logo.png"
  images={[
    '/assets/CMRT/CMRT_1.png',
    '/assets/CMRT/CMRT_2.png',
    '/assets/CMRT/CMRT_3.png'
  ]}
  description={$t('mods.cmrt_description')}
  installed={modAppsStatus.cmrt_installed}
  downloadLinks={[
    { label: $t('mods.cmrt_free_download'), url: 'https://www.overtake.gg/downloads/cmrt-essential-hud.69475/' },
    { label: $t('mods.cmrt_premium_download'), url: 'https://www.patreon.com/cw/CMRT/home' }
  ]}
  error={modAppsError}
  onClose={() => showCmrtModal = false}
  onDownload={openModDownload}
/>

<ModInfoModal
  show={showMapDisplayModal}
  title="Map Display"
  credit="by AW939"
  logo="/assets/MapDisplay/logo.png"
  images={[
    '/assets/MapDisplay/MAPDisplay_1.png',
    '/assets/MapDisplay/MAPDisplay_2.png',
    '/assets/MapDisplay/MAPDisplay_3.png'
  ]}
  description={$t('mods.mapdisplay_description')}
  installed={modAppsStatus.map_display_installed}
  downloadLinks={[
    { label: $t('mods.mapdisplay_download'), url: 'https://www.overtake.gg/downloads/track-map-display-extended.51962/' }
  ]}
  error={modAppsError}
  onClose={() => showMapDisplayModal = false}
  onDownload={openModDownload}
/>

<style>
  /* ⭐ NOUVEAU STYLE POUR LES ACCORDÉONS (Style CSP) */
  .accordion-header {
    display: flex;
    align-items: center;
    cursor: pointer;
    user-select: none;
    padding-top: 5px;
    padding-bottom: 5px;
  }

  .accordion-arrow {
    color: var(--accent); /* Couleur de ton thème (cyan/vert) */
    font-size: 1rem;
    margin-right: 8px;
    transition: transform 0.2s ease;
    display: inline-block;
    width: 16px;
    text-align: center;
  }

  .accordion-arrow.open {
    transform: rotate(90deg);
  }

  .accordion-title {
    color: var(--accent); /* La même couleur que tes titres CSP */
    font-weight: 700;
    font-size: 0.8rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    white-space: nowrap;
  }

  .accordion-line {
    flex-grow: 1;
    height: 1px;
    background: var(--border);
    margin-left: 12px;
    opacity: 0.6;
  }

  .mods-app-grid {
    display: grid;
    grid-template-columns: 1fr;
    gap: 12px;
    margin-top: 12px;
  }

  .mods-app-column {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    min-width: 0;
    padding: 12px;
    background: var(--bg-elevated);
    border: 1px solid var(--border);
    border-radius: 8px;
  }

  .mods-column-title {
    flex: 0 0 100%;
    margin: 0 0 4px;
    color: var(--accent);
    font-size: 0.8rem;
    font-weight: 700;
    letter-spacing: 0.5px;
    text-transform: uppercase;
  }

  .mod-app-card {
    position: relative;
    display: flex;
    align-items: center;
    gap: 10px;
    flex: 1 1 240px;
    min-width: 0;
    padding: 10px;
    color: var(--text-primary);
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 8px;
    cursor: pointer;
    transition: box-shadow 0.2s ease, border-color 0.2s ease;
  }

  .mod-app-card img {
    width: 110px;
    max-height: 36px;
    object-fit: contain;
  }

  .mod-app-card.active {
    border-color: var(--accent);
    box-shadow: 0 0 5px var(--accent);
  }

  .mod-app-add-icon {
    margin-left: auto;
    color: var(--accent);
    font-size: 1.4rem;
    font-weight: 700;
  }

  .mods-history-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .mods-history-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px;
    background: var(--bg-base);
    border: 1px solid var(--border);
    border-radius: 6px;
  }

  .mods-history-empty {
    padding: 16px;
    color: var(--text-tertiary);
    font-size: 0.85rem;
    font-style: italic;
    text-align: center;
  }

  .app-status-error {
    color: var(--danger, #f87171);
    overflow-wrap: anywhere;
  }

  @media (max-width: 700px) {
    .mods-app-grid {
      grid-template-columns: 1fr;
    }
  }

  .mods-history-toggle {
  display: flex;
  align-items: center;
  width: 100%;
  padding: 0;
  margin: 0 0 12px;
  background: none;
  border: 0;
  text-align: left;
  cursor: pointer;
}

.mods-history-toggle .mods-column-title {
  margin: 0;
}
</style>