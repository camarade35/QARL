<script lang="ts">
  import { invoke, Channel } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { t } from 'svelte-i18n';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { CATEGORIES, MODULES } from '$lib/utils/cspModules.js';
  import CspModuleModal from './CspModuleModal.svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import * as dialog from '$lib/stores/dialog.svelte.js';

  // ⭐ État pour l'import CM
  let showCmModal = $state(false);
  let cmPresets = $state<Array<{name: string, path: string}>>([]);
  let loadingCm = $state(false);

  let showModal = $state(false);
  let modalFilename = $state('');
  let modalTitle = $state('');

  // ⭐ Écouteur pour la recherche globale
  $effect(() => {
    const handler = (e: CustomEvent) => {
      if (e.detail.tab !== 'subtab-csp') return;
      const cspFile = e.detail.cspFile;
      if (cspFile) {
        const mod = MODULES.find((m: any) => `${m.id}.ini` === cspFile);
        if (mod) {
          if (mod.category) openCategories[mod.category] = true;
          setTimeout(() => {
            modalFilename = `${mod.id}.ini`;
            modalTitle = mod.label;
            showModal = true;
          }, 120);
        }
      }
    };
    window.addEventListener('qarl-open-section', handler as EventListener);
    return () => window.removeEventListener('qarl-open-section', handler as EventListener);
  });

  let loading = $state(true);
  let isDownloadingConfigs = $state(false);
  let downloadStatus = $state(''); // ⭐ Pour afficher le message en cours
  let errorMsg = $state('');
  let moduleInfos = $state<Record<string, boolean>>({}); // ⭐ Déplacé ici
  
  // État des fichiers sur le disque { filename: { exists, enabled, size } }
  let modulesState = $state<Record<string, { exists: boolean; enabled: boolean; size: number }>>({});
  
  // État des accordéons (repliés par défaut, mémorisés)
  let openCategories = $state<Record<string, boolean>>({});

  // ⭐ État des presets
  let presets = $state<string[]>([]);
  let selectedPreset = $state('');
  let isPresetMenuOpen = $state(false);

  // ⭐ Système de notification (Toast)
  let toastMessage = $state('');
  let toastTimeout: ReturnType<typeof setTimeout>;

  function showToast(msg: string) {
    toastMessage = msg;
    clearTimeout(toastTimeout);
    toastTimeout = setTimeout(() => {
      toastMessage = '';
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

  // ⭐ NOUVEAU : Écraser le preset sélectionné sans demander de nom
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

  // ⭐ Enregistrer sous un nouveau nom
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
      
      // ⭐ On force le rafraîchissement
      modulesState = {}; 
      await loadModules(); 
      
      // ⭐ On prévient l'onglet Vidéo de se mettre à jour
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

  // ⭐ Importation depuis un fichier local
  async function handleImportFile() {
    try {
      const selected = await open({
        multiple: false,
        directory: false,
        filters: [{ name: 'Preset CSP', extensions: ['ini'] }],
        title: $t('csp.btn_import_file_title')
      });
      
      if (!selected) return;
      
      await invoke('import_csp_preset_from_path', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        sourcePath: selected as string, // ⭐ CORRECTION ICI
        saveToQarl: true
      });
      
      showCmModal = false;
      await loadPresets();
      
      // ⭐ On force un rechargement complet pour mettre à jour les cases à cocher
      modulesState = {}; 
      await loadModules();
      
      // ⭐ On prévient l'onglet Vidéo de se mettre à jour
      window.dispatchEvent(new CustomEvent('qarl-csp-preset-loaded'));
      
      showToast($t('csp.toast_imported'));
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), 'error');
    }
  }

  // ⭐ Importation depuis Content Manager
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
      
      // ⭐ On force un rechargement complet de la liste des modules
      await loadModules(); 
      
      // ⭐ On prévient l'onglet Vidéo de se mettre à jour
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
    if (!appState.documentsPath || !appState.acPath) return; // ⭐ On vérifie aussi acPath
    loading = true;
    errorMsg = '';
    
    try {
      // 1. On demande à Rust quels templates existent et leur état par défaut
      const existing = await invoke<Array<{ filename: string; default_enabled: boolean }>>('list_csp_templates', {
        acPath: appState.acPath || ''
      });

      // ⭐ On stocke les infos pour les utiliser plus tard
      const newInfos: Record<string, boolean> = {};
      for (const info of existing) {
        newInfos[info.filename] = info.default_enabled;
      }
      moduleInfos = newInfos;

      // 2. Pour chaque fichier existant, on lit son état (ENABLED ou non)
      const newState: Record<string, any> = {};
      
      for (const mod of MODULES) {
        const filename = `${mod.id}.ini`;
        const templateExists = existing.find(e => e.filename === filename);
        
        if (templateExists) {
          try {
            const data = await invoke<any>('read_csp_module_data', {
              documentsPath: appState.documentsPath,
              acPath: appState.acPath || '',
              filename: filename
            });
            
            // On regarde si la section BASIC est activée
            const basicSection = data.sections.find((s: any) => s.name === 'BASIC');
            const isEnabled = basicSection ? basicSection.is_enabled : true;

            newState[filename] = { exists: true, enabled: isEnabled, size: 0 };
          } catch (e) {
            console.warn(`Erreur lecture ${filename}`, e);
            newState[filename] = { exists: true, enabled: true, size: 0 };
          }
        } else {
          newState[filename] = { exists: false, enabled: false, size: 0 };
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
    const newState = !currentlyEnabled;
    
    // Optimistic UI update
    modulesState[filename] = { ...modulesState[filename], enabled: newState };
    
    // On détermine le type du module depuis les infos de Rust
    const isDefaultEnabled = moduleInfos[filename] ?? true;
    
    try {
      await invoke('set_csp_module_enabled', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        filename: filename,
        section: 'BASIC', // L'activation globale passe par la section BASIC
        enabled: newState,
        defaultEnabled: isDefaultEnabled
      });
    } catch (e) {
      dialog.alert($t('csp.error_modify', { values: { filename } }) + e, $t('common.error'), "error");
      // Revert on failure
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

    // ⭐ Création du canal pour recevoir les logs
    const onEvent = new Channel<any>();
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

  // ⭐ NOUVEAU : On recharge les données dès que les chemins deviennent disponibles
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
          // Par défaut, on ouvre la première catégorie pour montrer qu'il y a du contenu
          openCategories = { graphics: true };
        }
      } catch (e) {}
    }
  });
</script>

<div class="csp-toolbar" style="display: flex; flex-direction: column; gap: 10px; align-items: stretch;">
  <div style="display: flex; justify-content: space-between; align-items: center;">
    <div class="csp-title">{$t('csp.title')}</div>
    <button class="filter-btn" onclick={loadModules} disabled={loading}>
      {loading ? $t('csp.refreshing') : $t('csp.refresh')}
    </button>
  </div>
  
  <!-- ⭐ Gestionnaire de presets -->
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
        <div class="csp-category-header" onclick={() => toggleCategory(category.id)}>
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

<!-- ⭐ La Modale des options -->
<CspModuleModal 
  bind:show={showModal} 
  filename={modalFilename} 
  title={modalTitle} 
  documentsPath={appState.documentsPath} 
  acPath={appState.acPath} 
  onClose={() => showModal = false} 
  onSaved={() => loadModules()} 
/>

<!-- ⭐ Notification Toast -->
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

<!-- ⭐ Modale d'import Content Manager -->
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