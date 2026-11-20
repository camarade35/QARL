<script lang="ts">
  import { invoke, Channel } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { t } from 'svelte-i18n';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { CATEGORIES, MODULES } from '$lib/utils/cspModules.js';
  import CspModuleModal from './CspModuleModal.svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import * as dialog from '$lib/stores/dialog.svelte.js';

  type ModuleState = { exists: boolean; enabled: boolean };
  type CspModuleStatus = { sections: { name: string; is_enabled: boolean }[] };
  type ConfigDownloadEvent = { type: 'info' | 'progress'; msg: string };
  type OpenSectionDetail = { tab: string; cspFile?: string | null };

  let showCmModal = $state(false);
  let cmPresets = $state<Array<{name: string, path: string}>>([]);
  let loadingCm = $state(false);

  let showModal = $state(false);
  let modalFilename = $state('');
  let modalTitle = $state('');
  let sectionOpenTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    const handler = (event: Event) => {
      const { tab, cspFile } = (event as CustomEvent<OpenSectionDetail>).detail;
      if (tab !== 'subtab-csp') return;
      if (cspFile) {
        const mod = MODULES.find(m => `${m.id}.ini` === cspFile);
        if (mod) {
          if (mod.category) openCategories[mod.category] = true;
          if (sectionOpenTimer !== null) clearTimeout(sectionOpenTimer);
          sectionOpenTimer = setTimeout(() => {
            modalFilename = `${mod.id}.ini`;
            modalTitle = mod.label;
            showModal = true;
            sectionOpenTimer = null;
          }, 120);
        }
      }
    };
    window.addEventListener('qarl-open-section', handler);
    return () => {
      window.removeEventListener('qarl-open-section', handler);
      if (sectionOpenTimer !== null) {
        clearTimeout(sectionOpenTimer);
        sectionOpenTimer = null;
      }
    };
  });

  let loading = $state(true);
  let isDownloadingConfigs = $state(false);
  let downloadStatus = $state('');
  let errorMsg = $state('');
  let moduleInfos = $state<Record<string, boolean>>({});
  
  let modulesState = $state<Record<string, ModuleState>>({});
  
  let openCategories = $state<Record<string, boolean>>({});

  let presets = $state<string[]>([]);
  let selectedPreset = $state('');

  let toastMessage = $state('');
  let toastTimeout: ReturnType<typeof setTimeout> | null = null;

  function showToast(msg: string) {
    toastMessage = msg;
    if (toastTimeout !== null) clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
      toastMessage = '';
      toastTimeout = null;
    }, 3000); // Disparaît après 3 secondes
  }

  async function loadPresets() {
    if (!appState.documentsPath) return;
    try {
      presets = await invoke<string[]>('list_csp_presets', { documentsPath: appState.documentsPath });
    } catch (e) {
      console.warn("Erreur chargement presets", e);
    }
  }

  async function handleOverwritePreset() {
    if (!selectedPreset) return;
    const ok = await dialog.confirm(
      $t('csp.confirm_overwrite', { values: { name: selectedPreset } }),
      $t('csp.dialog_title_overwrite'),
      'warning'
    );
    if (!ok) return;
    
    try {
      await invoke('save_csp_preset', { 
        documentsPath: appState.documentsPath, 
        acPath: appState.acPath || '', 
        presetName: selectedPreset 
      });
      showToast($t('csp.toast_overwrite'));
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    }
  }

  async function handleSaveAsPreset() {
    const name = await dialog.prompt($t('csp.prompt_save_as'), '', $t('csp.dialog_title_save_as'));
    if (!name || !name.trim()) return;
    
    try {
      await invoke('save_csp_preset', { 
        documentsPath: appState.documentsPath, 
        acPath: appState.acPath || '', 
        presetName: name.trim() 
      });
      await loadPresets();
      selectedPreset = name.trim();
      showToast($t('csp.toast_saved'));
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    }
  }

  async function handleLoadPreset() {
    if (!selectedPreset) return;
    const ok = await dialog.confirm(
      $t('csp.confirm_load', { values: { name: selectedPreset } }),
      $t('csp.dialog_title_load')
    );
    if (!ok) return;
    
    try {
      await invoke('load_csp_preset', { 
        documentsPath: appState.documentsPath, 
        acPath: appState.acPath || '', 
        presetName: selectedPreset 
      });
      
      modulesState = {}; 
      await loadModules(); 
      
      window.dispatchEvent(new CustomEvent('qarl-csp-preset-loaded'));
      
      showToast($t('csp.toast_loaded'));
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    }
  }

  async function handleResetDefaults() {
    const ok = await dialog.confirm(
      $t('csp.confirm_reset'),
      $t('csp.dialog_title_reset'),
      'warning'
    );
    if (!ok) return;
    
    try {
      await invoke('reset_csp_to_defaults', { 
        documentsPath: appState.documentsPath, 
        acPath: appState.acPath || '' 
      });
      await loadModules();
      showToast($t('csp.toast_reset'));
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    }
  }

  async function handleImportFile() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: 'Preset CSP', extensions: ['ini'] }],
        title: $t('csp.btn_import_file_title')
      });
      
      if (typeof selected !== 'string') return;
      
      await invoke('import_csp_preset_from_path', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        sourcePath: selected,
        saveToQarl: true
      });
      
      showCmModal = false;
      await loadPresets();
      
      modulesState = {}; 
      await loadModules();
      
      window.dispatchEvent(new CustomEvent('qarl-csp-preset-loaded'));
      
      showToast($t('csp.toast_imported'));
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    }
  }

  async function handleOpenCmImport() {
    loadingCm = true;
    try {
      const results = await invoke<Array<{name: string, path: string}>>('list_cm_presets');
      cmPresets = results;
      showCmModal = true;
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    } finally {
      loadingCm = false;
    }
  }

  async function handleSelectCmPreset(presetPath: string) {
    try {
      await invoke('import_csp_preset_from_path', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        sourcePath: presetPath,
        saveToQarl: true
      });
      
      showCmModal = false;
      await loadPresets();
      
      await loadModules(); 
      
      window.dispatchEvent(new CustomEvent('qarl-csp-preset-loaded'));
      
      showToast($t('csp.toast_cm_imported'));
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    }
  }

  function toggleCategory(catId: string) {
    openCategories[catId] = !openCategories[catId];
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('qarl_csp_categories', JSON.stringify(openCategories));
    }
  }

  async function loadModules() {
    if (!appState.documentsPath || !appState.acPath) return;
    loading = true;
    errorMsg = '';
    
    try {
      const existing = await invoke<Array<{ filename: string; default_enabled: boolean }>>('list_csp_templates', {
        acPath: appState.acPath || ''
      });

      const newInfos: Record<string, boolean> = {};
      for (const info of existing) {
        newInfos[info.filename] = info.default_enabled;
      }
      moduleInfos = newInfos;

      const newState: Record<string, ModuleState> = {};
      
      for (const mod of MODULES) {
        const filename = `${mod.id}.ini`;
        const templateExists = existing.find(e => e.filename === filename);
        
        if (templateExists) {
          try {
            const data = await invoke<CspModuleStatus>('read_csp_module_data', {
              documentsPath: appState.documentsPath,
              acPath: appState.acPath || '',
              filename: filename
            });
            
            const basicSection = data.sections.find(section => section.name === 'BASIC');
            const isEnabled = basicSection ? basicSection.is_enabled : true;

            newState[filename] = { exists: true, enabled: isEnabled };
          } catch (e) {
            console.warn(`Erreur lecture ${filename}`, e);
            newState[filename] = { exists: true, enabled: true };
          }
        } else {
          newState[filename] = { exists: false, enabled: false };
        }
      }
      
      modulesState = newState;
    } catch (e) {
      console.error(e);
      errorMsg = String(e);
    } finally {
      loading = false;
    }
  }

  async function toggleModule(modId: string, currentlyEnabled: boolean) {
    if (!appState.documentsPath) return;
    const filename = `${modId}.ini`;
    const nextEnabled = !currentlyEnabled;
    
    modulesState[filename] = { ...modulesState[filename], enabled: nextEnabled };
    
    const isDefaultEnabled = moduleInfos[filename] ?? true;
    
    try {
      await invoke('set_csp_module_enabled', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        filename: filename,
        section: 'BASIC',
        enabled: nextEnabled,
        defaultEnabled: isDefaultEnabled
      });
    } catch (e) {
      dialog.alert($t('csp.error_modify', { values: { filename } }) + e, $t('common.error'), "error");
      modulesState[filename] = { ...modulesState[filename], enabled: currentlyEnabled };
    }
  }

  async function handleDownloadConfigs() {
    if (!appState.acPath) {
      dialog.alert("Veuillez d'abord configurer le chemin d'Assetto Corsa.", "Erreur", "error");
      return;
    }
    isDownloadingConfigs = true;
    downloadStatus = "Démarrage...";

    const onEvent = new Channel<ConfigDownloadEvent>();
    onEvent.onmessage = (message) => {
      if (message.type === 'info' || message.type === 'progress') {
        downloadStatus = message.msg;
      }
    };

    try {
      const result = await invoke<string>('download_csp_configs', { 
        acPath: appState.acPath,
        onEvent: onEvent 
      });
      showToast(result);
      downloadStatus = '';
    } catch (e) {
      dialog.alert("Erreur lors du téléchargement : " + e, "Erreur", "error");
      downloadStatus = '';
    } finally {
      isDownloadingConfigs = false;
    }
  }

  function openOptions(modId: string, label: string) {
    modalFilename = `${modId}.ini`;
    modalTitle = label;
    showModal = true;
  }

  $effect(() => {
    if (appState.documentsPath && appState.acPath) {
      loadModules();
      loadPresets();
    }
  });

  onMount(() => {
    // Restauration de l'état des accordéons
    if (typeof localStorage !== 'undefined') {
      try {
        const saved = localStorage.getItem('qarl_csp_categories');
        if (saved) {
          openCategories = JSON.parse(saved);
        } else {
          openCategories = { graphics: true };
        }
      } catch (e) {
        console.warn("Impossible de restaurer l'état des catégories CSP:", e);
      }
    }

    return () => {
      if (toastTimeout !== null) clearTimeout(toastTimeout);
    };
  });
</script>

<div class="csp-toolbar" style="display: flex; flex-direction: column; gap: 10px; align-items: stretch;">
  <div style="display: flex; justify-content: space-between; align-items: center;">
    <div class="csp-title">{$t('csp.title')}</div>
    <button class="filter-btn" onclick={loadModules} disabled={loading}>
      {loading ? $t('csp.refreshing') : $t('csp.refresh')}
    </button>
  </div>
  
  <div style="display: flex; gap: 8px; align-items: center; background: var(--bg-elevated); padding: 8px 12px; border-radius: 6px; border: 1px solid var(--border);">
    <span style="font-size: 0.75rem; font-weight: 700; color: var(--text-secondary); text-transform: uppercase;">{$t('csp.presets_title')}</span>
    
    <select 
      class="ini-select" 
      style="flex: 1; min-width: 150px; height: 32px;" 
      bind:value={selectedPreset}
      onchange={() => { if (selectedPreset) handleLoadPreset(); }}
    >
      <option value="">{$t('csp.presets_placeholder')}</option>
      {#each presets as p}
        <option value={p}>{p}</option>
      {/each}
    </select>
    
    <button class="filter-btn" onclick={handleOverwritePreset} disabled={!selectedPreset} title={$t('csp.btn_overwrite_title')} style="padding: 6px 12px; height: 32px;">{$t('csp.btn_overwrite')}</button>
    <button class="filter-btn" onclick={handleSaveAsPreset} title={$t('csp.btn_save_as_title')} style="padding: 6px 12px; height: 32px;">{$t('csp.btn_save_as')}</button>
    
    <div style="width: 1px; height: 20px; background: var(--border); margin: 0 4px;"></div>
    
    <button class="filter-btn" onclick={handleImportFile} title={$t('csp.btn_import_file_title')} style="padding: 6px 12px; height: 32px;">{$t('csp.btn_import_file')}</button>
    <button class="filter-btn" onclick={handleOpenCmImport} disabled={loadingCm} title={$t('csp.btn_import_cm_title')} style="padding: 6px 12px; height: 32px;">
      {loadingCm ? $t('csp.btn_importing') : $t('csp.btn_import_cm')}
    </button>
    
    <div style="flex: 1;"></div>
    
    <div style="display: flex; align-items: center; gap: 10px;">
      {#if isDownloadingConfigs}
        <span style="font-size: 0.75rem; color: var(--success); font-weight: 600; font-style: italic;">
          {downloadStatus}
        </span>
      {/if}
      <button class="filter-btn" onclick={handleDownloadConfigs} disabled={isDownloadingConfigs} title="Mettre à jour les configs (clignotants, feux, etc.)" style="padding: 6px 12px; height: 32px; min-width: 130px; justify-content: center; background: rgba(76, 175, 80, 0.1); border-color: var(--success); color: var(--success);">
        {isDownloadingConfigs ? '⏳ Chargement...' : '📥 Configs CSP'}
      </button>
    </div>

    <button class="filter-btn" onclick={handleResetDefaults} title={$t('csp.btn_reset_title')} style="padding: 6px 12px; height: 32px; border-color: #a83a3a; color: #e57373;">{$t('csp.btn_reset')}</button>
  </div>
</div>

{#if errorMsg}
  <div style="color: #e57373; padding: 20px;">{$t('csp.error')}{errorMsg}</div>
{:else if loading && Object.keys(modulesState).length === 0}
  <div class="servers-loading">{$t('csp.loading')}</div>
{:else}
  <div class="csp-container">
    {#each CATEGORIES as category}
      <div class="csp-category" class:collapsed={!openCategories[category.id]}>
        <div
          class="csp-category-header"
          role="button"
          tabindex="0"
          aria-expanded={openCategories[category.id] ?? false}
          onclick={() => toggleCategory(category.id)}
          onkeydown={(event) => {
            if (event.key === 'Enter' || event.key === ' ') {
              event.preventDefault();
              toggleCategory(category.id);
            }
          }}
        >
          <span class="csp-category-arrow">▸</span>
          <span class="csp-category-label">{$t(category.label)}</span>
        </div>
        
        {#if openCategories[category.id]}
          <div class="csp-category-content">
            {#each MODULES.filter(m => m.category === category.id) as mod}
              {@const filename = `${mod.id}.ini`}
              {@const state = modulesState[filename]}
              
              {#if state && state.exists}
                <div class="csp-module-row" style="width: 100%; box-sizing: border-box; display: flex; justify-content: space-between; align-items: center; padding-right: 10px;">
                  <div class="csp-module-label" style="flex: 1; min-width: 0;">
                    {#if mod.disableable === false}
                      <div class="csp-module-nodisable" title={$t('csp.module_nodisable')}>🚫</div>
                    {:else}
                      <input 
                        type="checkbox" 
                        class="csp-module-checkbox" 
                        checked={state.enabled}
                        onchange={() => toggleModule(mod.id, state.enabled)}
                      />
                    {/if}
                    <span class="csp-module-name">{$t(mod.label)}</span>
                    <span class="csp-module-desc">{$t(mod.description)}</span>
                  </div>
                  
                  <div style="display: flex; gap: 8px; align-items: center; flex-shrink: 0;">
                    <button 
                      class="csp-module-open" 
                      onclick={async () => {
                        if (await dialog.confirm(
                          $t('csp.confirm_module_reset', { values: { name: mod.label } }),
                          $t('csp.dialog_title_module_reset'),
                          'warning'
                        )) {
                          await invoke('reset_single_csp_module', { documentsPath: appState.documentsPath, acPath: appState.acPath || '', filename: `${mod.id}.ini` });
                          await loadModules();
                          showToast($t('csp.toast_module_reset'));
                        }
                      }} 
                      title={$t('csp.btn_module_reset_title')}
                      style="color: #e57373;"
                    >
                      🔄
                    </button>
                    <button class="csp-module-open" onclick={() => openOptions(mod.id, mod.label)} title={$t('csp.btn_module_options_title')}>
                      ⚙️
                    </button>
                  </div>
                </div>
              {/if}
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
{/if}

<CspModuleModal
  show={showModal}
  filename={modalFilename} 
  title={modalTitle} 
  documentsPath={appState.documentsPath || ''}
  acPath={appState.acPath || ''}
  onClose={() => showModal = false} 
  onSaved={() => loadModules()} 
/>

{#if toastMessage}
  <div style="
    position: fixed; 
    bottom: 24px; 
    left: 50%; 
    transform: translateX(-50%); 
    background: var(--bg-panel); 
    border: 1px solid var(--success); 
    color: var(--text-primary); 
    padding: 12px 24px; 
    border-radius: var(--r-md); 
    box-shadow: var(--shadow-lg); 
    z-index: 9999; 
    font-weight: 600; 
    font-size: 0.9rem;
    display: flex;
    align-items: center;
    gap: 8px;
    animation: slideUp 0.3s ease;
  ">
    {toastMessage}
  </div>
{/if}

<style>
  @keyframes slideUp {
    from { opacity: 0; transform: translate(-50%, 20px); }
    to { opacity: 1; transform: translate(-50%, 0); }
  }
</style>

{#if showCmModal}
  <div class="modal visible" style="display: flex;">
    <div class="modal-content" style="width: 600px; max-width: 90vw; max-height: 80vh;">
      <div class="modal-header">
        <h3 class="modal-title">{$t('csp.modal_cm_title')}</h3>
        <button class="modal-close" onclick={() => showCmModal = false}>&times;</button>
      </div>
      
      <div style="flex: 1; overflow-y: auto; padding: 10px 0;">
        {#if cmPresets.length === 0}
          <div style="text-align: center; padding: 30px; color: var(--text-tertiary); font-style: italic;">
            {$t('csp.modal_cm_empty')}<br>
            {$t('csp.modal_cm_empty_hint')}
          </div>
        {:else}
          <div style="display: flex; flex-direction: column; gap: 6px;">
            {#each cmPresets as preset}
              <button 
                class="filter-btn" 
                style="text-align: left; padding: 12px 16px; height: auto; display: flex; justify-content: space-between; align-items: center;"
                onclick={() => handleSelectCmPreset(preset.path)}
              >
                <span>{preset.name}</span>
                <span style="color: var(--accent); font-size: 0.8rem;">{$t('csp.modal_cm_import')}</span>
              </button>
            {/each}
          </div>
        {/if}
      </div>
      
      <div class="modal-footer" style="display: flex; justify-content: flex-end; margin-top: 15px;">
        <button class="filter-btn" onclick={() => showCmModal = false}>{$t('csp.modal_close')}</button>
      </div>
    </div>
  </div>
{/if}