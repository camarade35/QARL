<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import * as dialog from '$lib/stores/dialog.svelte.js';
  import { t } from 'svelte-i18n';
  import { searchState } from '$lib/stores/searchState.svelte.js';
  import AutosaveBanner from '$lib/components/AutosaveBanner.svelte';

  let { show = false, filename = '', title = '', documentsPath = '', acPath = '', onClose = () => {}, onSaved = () => {} } = $props();

  let loading = $state(true);
  let saving = $state(false);
  let errorMsg = $state('');
  let searchQuery = $state('');
  
  // ⭐ Types correspondant à la structure Rust
  type Choice = { kind: 'separator' } | { kind: 'value', value: string, label: string };
  
  type ConditionExpr = 
    | { type: 'eq', key: string, value: string }
    | { type: 'truthy', key: string }
    | { type: 'not', expr: ConditionExpr }
    | { type: 'and', left: ConditionExpr, right: ConditionExpr }
    | { type: 'or', left: ConditionExpr, right: ConditionExpr };

  type CspOption = {
    key: string;
    value: string;
    label: string;
    option_type: string;
    min?: number;
    max?: number;
    step?: number;
    choices: Choice[];
    condition?: ConditionExpr;
    is_hidden: boolean;
    is_percentage: boolean;
    file_filter?: string; // ⭐ NOUVEAU
  };

  type CspSection = {
    name: string;
    is_enabled: boolean;
    has_enabled_toggle: boolean;
    options: CspOption[];
  };

  type CspModuleData = {
    filename: string;
    fullname: string;
    description: string;
    sections: CspSection[];
  };

  let moduleData = $state<CspModuleData | null>(null);

  // ⭐ Auto-save
  let hasChanges = $state(false);
  let lastSavedHash = $state('');
  let autosaveTimer: ReturnType<typeof setTimeout> | null = null;

  function hashModuleData(): string {
    if (!moduleData) return '';
    let h = '';
    for (const sec of moduleData.sections) {
      for (const opt of sec.options) {
        h += `${sec.name}::${opt.key}=${opt.value}|`;
      }
    }
    return h;
  }

  // ⭐ Évaluation récursive de l'AST de condition
  function evaluateCondition(expr: ConditionExpr | undefined, allSections: CspSection[], currentSectionName?: string): boolean {
    if (!expr) return true;

    const findValue = (key: string) => {
      let targetSection = null;
      let targetKey = key;
      
      if (key.includes('/')) {
        const parts = key.split('/');
        targetSection = parts[0].trim();
        targetKey = parts[1].trim();
      }

      for (const sec of allSections) {
        if (targetSection && sec.name !== targetSection) continue;
        
        // ⭐ CORRECTION : Si on cherche "ENABLED", on regarde l'état de la section
        if (targetKey === 'ENABLED') {
          if (targetSection && sec.name === targetSection) return sec.is_enabled ? '1' : '0';
          if (!targetSection) return sec.is_enabled ? '1' : '0';
        }

        const target = sec.options.find(o => o.key === targetKey);
        if (target) return target.value;
      }
      return null;
    };

    if (expr.type === 'eq') {
      return findValue(expr.key) === expr.value;
    }

    if (expr.type === 'truthy') {
      // ⭐ CORRECTION : Gestion spéciale pour ENABLED
      if (expr.key === 'ENABLED' && currentSectionName) {
        const sec = allSections.find(s => s.name === currentSectionName);
        return sec?.is_enabled ?? false;
      }
      const v = findValue(expr.key);
      return v === '1' || v?.toLowerCase() === 'true';
    }

    if (expr.type === 'not') return !evaluateCondition(expr.expr, allSections, currentSectionName);
    if (expr.type === 'and') return evaluateCondition(expr.left, allSections, currentSectionName) && evaluateCondition(expr.right, allSections, currentSectionName);
    if (expr.type === 'or') return evaluateCondition(expr.left, allSections, currentSectionName) || evaluateCondition(expr.right, allSections, currentSectionName);

    return true;
  }

  function isOptionVisible(opt: CspOption, allSections: CspSection[], currentSectionName: string): boolean {
    if (opt.is_hidden) return false;
    return evaluateCondition(opt.condition, allSections, currentSectionName);
  }

  // ⭐ NOUVEAU : Vérifie si la valeur est hors limites
  function isOutOfBounds(entry: CspOption): boolean {
    if (entry.option_type !== 'number') return false;
    if (entry.min === undefined || entry.max === undefined) return false;

    const val = parseFloat(entry.value);
    if (isNaN(val)) return true; // Si ce n'est pas un nombre valide

    return val < entry.min || val > entry.max;
  }

  async function loadModule() {
    if (!documentsPath) return;
    loading = true;
    errorMsg = '';
    
    try {
      const data = await invoke<CspModuleData>('read_csp_module_data', {
        documentsPath: documentsPath,
        acPath: acPath || '',
        filename: filename
      });
      moduleData = data;
      lastSavedHash = hashModuleData();
      hasChanges = false;
    } catch (e) {
      errorMsg = String(e);
    } finally {
      loading = false;
    }
  }

  // ⭐ Auto-save : détecte les changements
  $effect(() => {
    if (!moduleData || !lastSavedHash) return;
    const current = hashModuleData();
    if (current === lastSavedHash) return;
    hasChanges = true;
    if (searchState.autosaveDelay > 0) {
      if (autosaveTimer) clearTimeout(autosaveTimer);
      autosaveTimer = setTimeout(() => {
        if (hasChanges) saveModule(true);
      }, searchState.autosaveDelay * 1000);
    }
  });

  async function handleSectionToggle(sectionName: string, isChecked: boolean) {
    if (!moduleData) return;
    
    try {
      // On garde la logique de QARL (elle est robuste)
      const section = moduleData.sections.find(s => s.name === sectionName);
      const defaultEnabled = section?.is_enabled ?? true; 

      await invoke('set_csp_module_enabled', {
        documentsPath: documentsPath,
        acPath: acPath || '',
        filename: filename,
        section: sectionName,
        enabled: isChecked,
        defaultEnabled: defaultEnabled
      });
      
      if (section) section.is_enabled = isChecked;
      
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), "error");
    }
  }

  async function saveModule(silent = false) {
    if (!documentsPath || !moduleData) return;
    saving = true;
    
    try {
      const updates: Array<{ section: string; key: string; value: string }> = [];
      
      // On parcourt toutes les sections et options pour sauvegarder
      for (const sec of moduleData.sections) {
        for (const entry of sec.options) {
          // On sauvegarde même les options cachées pour ne pas perdre leur état
          // ⭐ On force la conversion en String pour Rust
          updates.push({ section: sec.name, key: entry.key, value: String(entry.value) });
        }
      }

      await invoke('update_ini_values', {
        documentsPath: documentsPath,
        acPath: acPath || '',
        filename: filename,
        updates: updates
      });
      
      lastSavedHash = hashModuleData();
      hasChanges = false;
      if (autosaveTimer) { clearTimeout(autosaveTimer); autosaveTimer = null; }

      if (!silent) {
        onSaved();
        onClose();
      } else {
        // Auto-save silencieux : on rafraîchit quand même le parent mais on garde la modale ouverte
        onSaved();
      }
    } catch (e) {
      dialog.alert($t('csp.error') + e, $t('common.error'), "error");
    } finally {
      saving = false;
    }
  }

  $effect(() => {
    if (show && filename) {
      loadModule();
    }
  });

  // Filtrage pour la recherche
  let filteredSections = $derived(() => {
    if (!moduleData) return [];
    if (!searchQuery.trim()) return moduleData.sections;
    
    const q = searchQuery.toLowerCase();
    return moduleData.sections.map(sec => ({
      ...sec,
      options: sec.options.filter(e => e.label.toLowerCase().includes(q) || e.key.toLowerCase().includes(q))
    })).filter(sec => sec.options.length > 0 || sec.name.toLowerCase().includes(q));
  });

  // ⭐ Transforme un label brut en clé i18n valide (minuscules, sans accents, sans points)
  function labelToKey(label: string): string {
    return label
      .toLowerCase()
      .normalize('NFD').replace(/[\u0300-\u036f]/g, '')
      .replace(/[^a-z0-9]+/g, '_')
      .replace(/^_+|_+$/g, '');
  }

  // ⭐ Résolution d'un label d'option : 4 niveaux (key → label brut → label sanitisé → fallback)
  function trOptionLabel(key: string, label: string): string {
    const byKey = $t(`csp.option_labels.${key}`, { default: '' });
    if (byKey) return byKey;
    const byLabel = $t(`csp.option_labels.${label}`, { default: '' });
    if (byLabel) return byLabel;
    const bySanitized = $t(`csp.option_labels.${labelToKey(label)}`, { default: '' });
    if (bySanitized) return bySanitized;
    return label || key;
  }

  // ⭐ Résolution de la description du module via le filename (stable) au lieu du texte brut
  function trModuleDescription(data: CspModuleData | null): string {
    if (!data) return '';
    const fileKey = data.filename.replace(/\.ini$/i, '');
    
    // ⭐ Priorité 1 : On utilise la description courte (qui est déjà traduite dans csp.modules)
    const byModule = $t(`csp.modules.${fileKey}.desc`, { default: '' });
    if (byModule) return byModule;

    // Priorité 2 : On cherche une éventuelle description longue spécifique (ex: extra_fx)
    const byFile = $t(`csp.module_descriptions.${fileKey}`, { default: '' });
    return byFile || data.description;
  }
</script>

{#if show}
  <div class="modal visible" id="csp-module-modal" style="display: flex;">
    <div class="modal-content csp-module-modal-content">
      <AutosaveBanner hasChanges={hasChanges} onSave={() => saveModule(true)} />
      <div class="modal-header">
        <div>
          <h3 class="modal-title" id="csp-module-modal-title">{$t(title, { default: moduleData?.fullname || title })} — {filename}</h3>
          {#if moduleData?.description}
            <p style="font-size: 0.8rem; color: var(--text-tertiary); margin-top: 4px; max-width: 600px;">
              {trModuleDescription(moduleData)}
            </p>
          {/if}
        </div>
        <button class="modal-close" onclick={onClose}>&times;</button>
      </div>

      <div class="csp-module-modal-search">
        <input type="text" placeholder={$t('csp.modal_search_placeholder')} bind:value={searchQuery}>
      </div>

      <div id="csp-module-modal-body" class="csp-module-modal-body">
        {#if loading}
          <div class="servers-loading">{$t('csp.modal_loading')}</div>
        {:else if errorMsg}
          <div style="color: #e57373; padding: 20px;">{$t('common.error')} : {errorMsg}</div>
        {:else if !moduleData || filteredSections().length === 0}
          <div class="csp-module-search-empty">{$t('csp.modal_no_options')}</div>
        {:else}
          {#each filteredSections() as section}
            <div class="csp-opt-section">
              <div class="csp-opt-section-title" style="display: flex; align-items: center; gap: 10px; border-bottom: 1px solid #333; padding-bottom: 8px; margin-bottom: 12px;">
                {#if section.has_enabled_toggle}
                  <input 
                    type="checkbox" 
                    class="ini-checkbox" 
                    checked={section.is_enabled}
                    onchange={(e) => handleSectionToggle(section.name, e.currentTarget.checked)}
                    style="transform: scale(1.2); cursor: pointer;"
                  />
                {/if}
                <span style="font-size: 1.1em; font-weight: bold;">{section.name}</span>
              </div>
              
              {#each section.options as entry}
                {#if isOptionVisible(entry, moduleData?.sections || [], section.name)}
                  <div class="csp-opt-row" class:has-error={isOutOfBounds(entry)} style="display: flex; align-items: center; justify-content: space-between; padding: 8px 0; border-bottom: 1px solid #2a2a2a;">
                    <span class="csp-opt-label" title="{entry.key}" style="flex: 1; margin-right: 15px; font-size: 0.85rem; line-height: 1.4; color: var(--text-primary);">
                      {trOptionLabel(entry.key, entry.label)}
                      {#if isOutOfBounds(entry)}
                        <span style="color: var(--danger, #ff4c4c); margin-left: 6px; font-size: 0.75rem; font-weight: 700; cursor: help; display: inline-flex; align-items: center; gap: 4px;" title="Min: {entry.min}, Max: {entry.max}">
                          ⚠️ {$t('csp.invalid_value', { default: 'Valeur invalide' })}
                        </span>
                      {/if}
                    </span>
                    
                    <div class="csp-opt-widget" style="flex-shrink: 0; min-width: 150px; display: flex; justify-content: flex-end;">
                      {#if entry.option_type === 'bool'}
                        <input 
                          type="checkbox" 
                          class="ini-checkbox" 
                          checked={entry.value === '1'}
                          onchange={(e) => entry.value = e.currentTarget.checked ? '1' : '0'}
                        />
                      {:else if entry.option_type === 'number'}
                        <!-- Si pas de range défini, on affiche juste un champ texte pour éviter les bugs -->
                                                {#if entry.min !== undefined && entry.max !== undefined}
                          {@const pct = entry.is_percentage ? 100 : 1}
                          {@const displayMin = entry.min * pct}
                          {@const displayMax = entry.max * pct}
                          <!-- On limite à 3 décimales pour éviter les bugs d'affichage flottant -->
                          {@const displayVal = Number((parseFloat(entry.value) * pct).toFixed(3))}
                          
                          <!-- ⭐ Si c'est un pourcentage, on IGNORE le step du Rust et on applique la règle visuelle -->
                          {@const baseStep = (
                            entry.is_percentage 
                              ? (entry.max <= 1.0 ? 0.001 : (entry.max <= 10 ? 0.01 : 1.0))
                              : (entry.step ?? (Number.isInteger(entry.min) && Number.isInteger(entry.max) ? 1.0 : 0.01))
                          )}
                          {@const visualStep = baseStep * pct}
                          
                          <div style="display: flex; align-items: center; gap: 10px; width: 100%;">
                            <!-- Le curseur garde "oninput" pour être fluide -->
                            <input 
                              type="range" 
                              min={displayMin} 
                              max={displayMax} 
                              step={visualStep}
                              class="ini-range"
                              value={displayVal}
                              oninput={(e) => entry.value = String(parseFloat(e.currentTarget.value) / pct)}
                              style="flex: 1;"
                            />
                            <div style="display: flex; align-items: center; gap: 2px;">
                              <!-- Le champ texte utilise "onchange" pour ne valider qu'à la sortie du champ -->
                              <input 
                                type="number" 
                                class="ini-number" 
                                min={displayMin}
                                max={displayMax}
                                step="any" 
                                value={displayVal}
                                onchange={(e) => entry.value = String(parseFloat(e.currentTarget.value) / pct)}
                                style="width: 80px;"
                              />
                              {#if entry.is_percentage}
                                <span style="font-size: 0.8rem; color: var(--text-secondary);">%</span>
                              {/if}
                            </div>
                          </div>
                        {:else if entry.option_type === 'number'}
                          <!-- ⭐ Saisie libre pour les nombres sans limites -->
                          <input type="number" class="ini-text" step="any" bind:value={entry.value} />
                        {:else}
                          <input type="text" class="ini-text" bind:value={entry.value} />
                        {/if}
                      {:else if entry.option_type === 'select' && entry.choices.length > 0}
                        <select class="ini-select" bind:value={entry.value}>
                          {#each entry.choices as choice}
                            {#if choice.kind === 'separator'}
                              <hr style="margin: 4px 0; border-color: #444;" />
                            {:else}
                              <option value={choice.value}>{choice.label}</option>
                            {/if}
                          {/each}
                        </select>
                      {:else if entry.option_type === 'file'}
                        <div style="display: flex; gap: 4px; width: 100%;">
                          <input type="text" class="ini-text" bind:value={entry.value} style="flex: 1;" readonly />
                          <button 
                            class="filter-btn" 
                            style="padding: 4px 8px;" 
                            onclick={async () => {
                              let selectedPath: string | null = null;
                              let isValid = false;
                              
                              while (!isValid) {
                                const ext = entry.file_filter ? entry.file_filter.split('.').pop() : '*';
                                selectedPath = await open({
                                  defaultPath: acPath || undefined, // Ouvre dans le dossier AC
                                  multiple: false,
                                  directory: false,
                                  title: $t('csp.choose_file') + (entry.file_filter ? ` (${entry.file_filter})` : ''),
                                  filters: [{ name: entry.file_filter || 'File', extensions: [ext || '*'] }]
                                }) as string | null;
                                
                                if (!selectedPath) break; // L'utilisateur a annulé
                                
                                // On extrait juste le nom du fichier
                                const fileName = selectedPath.replace(/^.*[\\\/]/, '');
                                
                                // Si un nom est imposé, on vérifie
                                if (entry.file_filter && fileName !== entry.file_filter) {
                                  await dialog.alert(
                                    $t('csp.file_name_error', { values: { name: entry.file_filter, chosen: fileName } }), 
                                    $t('common.warning'), 
                                    'warning'
                                  );
                                } else {
                                  isValid = true;
                                }
                              }
                              
                              if (isValid && selectedPath) {
                                // On sauvegarde UNIQUEMENT le nom du fichier, pas tout le chemin
                                entry.value = selectedPath.replace(/^.*[\\\/]/, '');
                              }
                            }}
                          >...</button>
                        </div>
                      {:else}
                        <input 
                          type="text" 
                          class="ini-text" 
                          bind:value={entry.value}
                        />
                      {/if}
                    </div>
                  </div>
                {/if}
              {/each}
            </div>
          {/each}
        {/if}
      </div>

      <div class="csp-module-modal-footer">
        <span class="csp-module-modal-status"></span>
        <div class="csp-module-modal-actions" style="display: flex; gap: 8px;">
          <button 
            class="filter-btn" 
            onclick={async () => {
              if (await dialog.confirm($t('csp.confirm_module_reset_all'), $t('csp.btn_reset'), "warning")) {
                await invoke('reset_single_csp_module', { documentsPath, acPath: acPath || '', filename });
                await loadModule();
                onSaved(); // On prévient le parent
              }
            }} 
            disabled={saving}
            style="border-color: #a83a3a; color: #e57373;"
          >
            {$t('csp.btn_reset')}
          </button>
          <button class="filter-btn" onclick={loadModule} disabled={saving}>{$t('csp.refresh')}</button>
          <button class="filter-btn ini-btn-save" onclick={saveModule} disabled={saving || loading}>
            {saving ? $t('csp.saving') : $t('common.save')}
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}

<style>
  /* ⭐ NOUVEAU : Style d'alerte pour les valeurs hors limites */
  .csp-opt-row.has-error {
    background-color: rgba(255, 76, 76, 0.08) !important;
    border-left: 3px solid var(--danger, #ff4c4c) !important;
    animation: pulse-error 2s infinite;
  }

  @keyframes pulse-error {
    0% { box-shadow: inset 0 0 0 0 rgba(255, 76, 76, 0.2); }
    50% { box-shadow: inset 0 0 10px 0 rgba(255, 76, 76, 0.4); }
    100% { box-shadow: inset 0 0 0 0 rgba(255, 76, 76, 0.2); }
  }
</style>