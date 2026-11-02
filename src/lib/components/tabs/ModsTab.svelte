<script lang="ts">
  import { t, locale } from 'svelte-i18n';
  import { alert } from '$lib/stores/dialog.svelte'; // ⭐ AJOUT IMPORT QARL
  import { onMount } from 'svelte';
  import { invoke, Channel } from '@tauri-apps/api/core';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { appState } from '$lib/stores/appState.svelte.js';
  

  let isDragging = $state(false);
  let isInstalling = $state(false);
  let installLogs = $state<string[]>([]);
  let modsHistory = $state<any[]>([]);
  
  // ⭐ Nouveaux états pour la progression
  let progressPercent = $state(0);
  let currentFileName = $state('');
  let currentArchiveIndex = $state(0);
  let totalArchives = $state(0);

  async function loadHistory() {
    try {
      modsHistory = await invoke<any[]>('get_mods_history', { acPath: appState.acPath });
    } catch (e) { console.error(e); }
  }

  onMount(async () => {
    loadHistory();

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

  async function handleDrop(paths: string[]) {
    if (!appState.acPath) {
      alert($t('mods.error_path'));
      return;
    }
    const validArchives = paths.filter(p => /\.(zip|rar|7z)$/i.test(p));
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
          const shortName = message.current_file.split('/').pop();
          currentFileName = shortName;
        }
      };

      try {
        const result = await invoke<string>('install_mod_from_archive', {
          acPath: appState.acPath,
          archivePath: file,
          onEvent: onEvent // ⭐ On passe le canal à Rust
        });
        installLogs = [`✅ ${result}`, ...installLogs];
        await loadHistory();
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
    style="border: 2px dashed {isDragging ? 'var(--accent)' : 'var(--border)'}; 
           background: {isDragging ? 'var(--accent-muted)' : 'var(--bg-elevated)'};
           border-radius: 8px; padding: 40px; text-align: center; 
           transition: all 0.2s; margin-top: 15px;"
  >
    <div style="font-size: 2rem; margin-bottom: 10px;">📦</div>
    <div style="font-weight: 600; color: var(--text-primary); margin-bottom: 5px;">
      {isDragging ? $t('mods.drop_active') : $t('mods.drop_title')}
    </div>
    <div style="font-size: 0.8rem; color: var(--text-tertiary); margin-bottom: 15px;">
      {$t('mods.drop_desc')}
    </div>
    
    {#if isInstalling}
      <div style="margin-top: 15px; display: flex; flex-direction: column; align-items: center; gap: 8px;">
        <div style="color: var(--accent); font-weight: bold;">
          {$t('mods.extracting_archive', { values: { current: currentArchiveIndex, total: totalArchives } })}
        </div>
        
        <!-- La barre de progression -->
        <div style="width: 80%; max-width: 400px; background: var(--bg-base); border: 1px solid var(--border); border-radius: 8px; height: 12px; overflow: hidden; position: relative;">
          <div style="background: var(--accent); height: 100%; width: {progressPercent}%; transition: width 0.1s ease-out;"></div>
        </div>
        
        <div style="font-size: 0.75rem; color: var(--text-secondary); height: 1.2rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; max-width: 90%;">
          {currentFileName}
        </div>
      </div>
    {/if}
  </div>
</div>

<div class="group">
  <div class="group-title">{$t('mods.history_title')}</div>
  {#if modsHistory.length === 0}
    <div style="text-align: center; padding: 20px; color: var(--text-tertiary); font-style: italic; font-size: 0.85rem;">
      {$t('mods.history_empty')}
    </div>
  {:else}
    <div style="display: flex; flex-direction: column; gap: 8px; margin-top: 10px;">
      {#each modsHistory as mod}
        <div style="display: flex; align-items: center; justify-content: space-between; padding: 10px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 6px;">
          <div>
            <div style="font-weight: 600; font-size: 0.9rem;">{mod.mod_name}</div>
            <div style="font-size: 0.75rem; color: var(--text-secondary);">
              {$t('mods.history_type', { values: { type: mod.mod_type, date: formatDate(mod.date) } })}
            </div>
          </div>
          <button class="filter-btn" 
            style="padding: 4px 8px; font-size: 0.75rem; {mod.exists ? '' : 'opacity: 0.4; cursor: not-allowed;'}" 
            title={mod.exists ? $t('mods.history_view') : $t('mods.history_missing')}
            onclick={() => { 
              if (mod.exists) {
                invoke('open_folder', { acPath: appState.acPath, folderType: 'mod', folderId: mod.path_installed })
                  .catch(e => alert($t('mods.open_folder_error') + e)); 
              }
            }}
            disabled={!mod.exists}
          >
            {mod.exists ? $t('mods.history_view') : $t('mods.history_missing')}
          </button>
        </div>
      {/each}
    </div>
  {/if}
</div>

<div class="group">
  <div class="group-title">{$t('mods.log_title')}</div>
  <div style="max-height: 150px; overflow-y: auto; background: #0f1219; border: 1px solid var(--border); border-radius: 6px; padding: 10px; font-family: monospace; font-size: 0.8rem;">
    {#if installLogs.length === 0}
      <div style="color: var(--text-tertiary);">{$t('mods.log_waiting')}</div>
    {:else}
      {#each installLogs as log}
        <div style="margin-bottom: 4px; color: {log.startsWith('✅') ? '#4ade80' : log.startsWith('❌') ? '#f87171' : 'var(--text-secondary)'};">
          {log}
        </div>
      {/each}
    {/if}
  </div>
</div>