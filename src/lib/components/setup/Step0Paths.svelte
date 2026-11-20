<script lang="ts">
  import { t } from 'svelte-i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { alert } from '$lib/stores/dialog.svelte';
  import { appState, setAcPath, setDocumentsPath } from '$lib/stores/appState.svelte.js';
  import { setupState } from '$lib/stores/setupState.svelte.js';

  type DetectedSetupPaths = {
    ac_path: string | null;
    ac_path_source: string;
    documents_path: string | null;
    documents_source: string;
  };

  let { onContinue = () => {} } = $props();

  let acPath = $state(appState.acPath || setupState.data.acPath || '');
  let documentsPath = $state(appState.documentsPath || setupState.data.documentsPath || '');
  let detecting = $state(false);
  let isLaunching = $state(false);

  async function autoDetect() {
    detecting = true;
    try {
      const status = await invoke<DetectedSetupPaths>('detect_setup_status');
      if (status.ac_path) {
        acPath = status.ac_path;
        setupState.data.acPathSource = status.ac_path_source;
      }
      if (status.documents_path) {
        documentsPath = status.documents_path;
        setupState.data.documentsSource = status.documents_source;
      }
    } catch (e) {
      console.warn('Auto-detect failed', e);
    } finally {
      detecting = false;
    }
  }

  // Lance AC pour permettre la création des fichiers utilisateur manquants.
  async function launchAc() {
    isLaunching = true;
    try {
      await invoke('launch_ac_only');
    } catch (e) {
      alert($t('setup.paths.ac_launch_error') + e);
    } finally {
      isLaunching = false;
    }
  }

  async function browseAc() {
    const selected = await open({ directory: true, multiple: false, title: $t('setup.paths.ac_label') });
    if (typeof selected === 'string') {
      acPath = selected;
    }
  }

  async function browseDocs() {
    const selected = await open({ directory: true, multiple: false, title: $t('setup.paths.docs_label') });
    if (typeof selected === 'string') {
      documentsPath = selected;
    }
  }

  function handleContinue() {
    if (!acPath) {
      alert($t('setup.errors.no_ac_path'));
      return;
    }
    if (!documentsPath) {
      alert($t('setup.errors.no_docs_path'));
      return;
    }
    setAcPath(acPath);
    setDocumentsPath(documentsPath);
    setupState.data.acPath = acPath;
    setupState.data.documentsPath = documentsPath;
    onContinue();
  }
</script>

<div class="setup-field">
  <label for="setup-ac-path">{$t('setup.paths.ac_label')}</label>
  <div class="setup-input-row">
    <input id="setup-ac-path" type="text" readonly value={acPath} placeholder={$t('setup.paths.ac_placeholder')} />
    <button class="filter-btn" onclick={autoDetect} disabled={detecting}>
      {detecting ? '⏳' : $t('setup.paths.auto_detect')}
    </button>
    <button class="filter-btn" onclick={browseAc}>{$t('setup.paths.browse')}</button>
  </div>
  {#if !acPath}
    <div class="setup-hint" style="color: #e6a33b;">{$t('setup.paths.ac_not_found')}</div>
  {/if}
</div>

<div class="setup-field">
  <label for="setup-documents-path">{$t('setup.paths.docs_label')}</label>
  <div class="setup-input-row">
    <input id="setup-documents-path" type="text" readonly value={documentsPath} placeholder={$t('setup.paths.docs_placeholder')} />
    <button class="filter-btn" onclick={browseDocs}>{$t('setup.paths.browse')}</button>
  </div>
  {#if !documentsPath}
    <div class="setup-hint" style="color: #e6a33b;">{$t('setup.paths.docs_not_found')}</div>
  {/if}
</div>

<!-- Aide pour créer les fichiers utilisateur manquants -->
{#if !documentsPath}
  <div class="setup-warning" style="margin-top: 15px; background: rgba(230, 163, 59, 0.1); border: 1px solid rgba(230, 163, 59, 0.3);">
    <strong style="color: #e6a33b;">{$t('setup.paths.help_generate_title')}</strong>
    <p style="margin-top: 8px; font-size: 0.85rem;">{$t('setup.paths.help_generate_desc')}</p>
    <button class="filter-btn" onclick={launchAc} disabled={isLaunching} style="margin-top: 10px; background: var(--accent); color: #fff; border-color: var(--accent);">
      {isLaunching ? $t('setup.paths.launching') : $t('setup.ac_init.launch_ac')}
    </button>
  </div>
{/if}

<div style="display: flex; justify-content: flex-end; margin-top: 20px;">
  <button class="filter-btn" onclick={handleContinue} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent); font-weight: 700;">
    {$t('setup.paths.continue')}
  </button>
</div>