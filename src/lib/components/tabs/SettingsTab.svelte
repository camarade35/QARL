<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { appState, setAcPath, setSteamId, setDriverName, setOnlineName, setDocumentsPath } from '$lib/stores/appState.svelte.js';
  import { resetSetup } from '$lib/stores/setupState.svelte.js';
  import ControlsTab from './ControlsTab.svelte';
  import VideoTab from './VideoTab.svelte';
  import AudioTab from './AudioTab.svelte';
  import CspTab from './CspTab.svelte';
  import HelpTab from './HelpTab.svelte';
  import SettingsSearch from '$lib/components/SettingsSearch.svelte';
  import { searchState } from '$lib/stores/searchState.svelte.js';
  import { onMount } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { alert, confirm } from '$lib/stores/dialog.svelte';
  import { t } from 'svelte-i18n';

  let activeSubTab = $state('subtab-general');
  let isDetectingSteam = $state(false);
  let selectedPath = $state(appState.acPath || '');
  let capabilities = $state<{ has_csp: boolean; has_pure: boolean; has_pure_weather: boolean; has_sol: boolean; has_acs: boolean; csp_version: string; sol_version: string; pure_version: string; pure_planner_version: string } | null>(null);
  let isChecking = $state(false);

  // ⭐ Écouteurs pour basculer automatiquement sur les bons sous-onglets
  onMount(() => {
    const handleOpenHelp = () => { activeSubTab = 'subtab-help'; };
    const handleOpenGeneral = () => { activeSubTab = 'subtab-general'; };
    
    // ⭐ Écouteur pour les redirections depuis le Wizard (Contrôles, CSP...)
    const handleOpenSection = (e: any) => {
      if (e.detail && e.detail.tab) {
        activeSubTab = e.detail.tab;
      }
    };
    
    window.addEventListener('open-help-tab', handleOpenHelp);
    window.addEventListener('open-general-tab', handleOpenGeneral);
    window.addEventListener('qarl-open-section', handleOpenSection);
    
    return () => {
      window.removeEventListener('open-help-tab', handleOpenHelp);
      window.removeEventListener('open-general-tab', handleOpenGeneral);
      window.removeEventListener('qarl-open-section', handleOpenSection);
    };
  });

    // Constante pour la version (à modifier lors du push Github)
  const APP_VERSION = "v1.0.0";

  async function detectSteamId() {
    isDetectingSteam = true;
    try {
      const id = await invoke<string>('detect_steam_id');
      setSteamId(id);
    } catch (e) {
      alert($t('settings.general.steam_detect_error', { values: { error: e } }));
    } finally {
      isDetectingSteam = false;
    }
  }

  async function choisirDossier() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: $t('settings.general.path_label')
    });
    if (selected) {
      selectedPath = selected as string;
    }
  }

  async function choisirDossierDocuments() {
    const selected = await open({
      directory: true,
      multiple: false,
      title: $t('settings.general.docs_select_title')
    });
    if (selected) {
      setDocumentsPath(selected as string);
    }
  }

  async function sauvegarder() {
    if (!selectedPath) {
      alert($t('settings.general.path_select_error'));
      return;
    }
    setAcPath(selectedPath);
    await verifierCapacites();
  }

  async function verifierCapacites() {
    if (!appState.acPath) return;
    isChecking = true;
    try {
      capabilities = await invoke<{ has_csp: boolean; has_pure: boolean; has_pure_weather: boolean; has_sol: boolean; has_acs: boolean; csp_version: string; sol_version: string; pure_version: string; pure_planner_version: string }>(
        'check_ac_capabilities',
        { acPath: appState.acPath }
      );
    } catch (e) {
      console.error(e);
      capabilities = null;
    } finally {
      isChecking = false;
    }
  }

  async function resetDonnees() {
    if (await confirm($t('settings.general.data_confirm'), $t('settings.general.data_confirm_title'), "warning")) {
      localStorage.removeItem('qarl_favorites');
      localStorage.removeItem('qarl_used');
      localStorage.removeItem('qarl_selections');
      localStorage.removeItem('qarl_session');
      localStorage.removeItem('qarl_custom_servers');
      localStorage.removeItem('qarl_recent_servers');
      localStorage.removeItem('qarl_favorite_servers');
      localStorage.removeItem('qarl_server_profiles');
      localStorage.removeItem('qarl_server_passwords');
      
        alert($t('settings.general.data_success'));
      window.location.reload();
    }
  }

  $effect(() => {
    if (appState.acPath) {
      verifierCapacites();
    }
  });

  // ⭐ Navigation depuis la recherche globale
  function handleSearchNavigate(tabId: string, groupKey: string, cspFile: string | null) {
    activeSubTab = tabId;
    if (groupKey) {
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent('qarl-open-section', {
          detail: { tab: tabId, group: groupKey, cspFile }
        }));
      }, 80);
    }
  }
</script>

<!-- ⭐ Barre de recherche globale -->
<SettingsSearch onNavigate={handleSearchNavigate} />

<!-- ⭐ Barre des sous-onglets -->
<div class="subtabs" style:display={searchState.query.trim() ? 'none' : 'flex'}>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-general'} onclick={() => activeSubTab = 'subtab-general'}>{$t('settings.subtabs.general')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-controls'} onclick={() => activeSubTab = 'subtab-controls'}>{$t('settings.subtabs.controls')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-video'} onclick={() => activeSubTab = 'subtab-video'}>{$t('settings.subtabs.video')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-audio'} onclick={() => activeSubTab = 'subtab-audio'}>{$t('settings.subtabs.audio')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-csp'} onclick={() => activeSubTab = 'subtab-csp'}>{$t('settings.subtabs.csp')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-help'} onclick={() => activeSubTab = 'subtab-help'}>{$t('settings.subtabs.help')}</button>
</div>

<!-- ⭐ SOUS-ONGLET GÉNÉRAL (Tout ton contenu actuel) -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-general'}>
  <div class="group">
    <div class="group-title">{$t('settings.general.path_title')}</div>

  {#if capabilities && !capabilities.has_acs}
    <div style="background: rgba(230, 163, 59, 0.1); border: 1px solid rgba(230, 163, 59, 0.4); color: #e6a33b; padding: 10px 14px; border-radius: 6px; font-size: 0.85rem; display: flex; align-items: center; gap: 10px; margin-bottom: 10px;">
      <span style="font-size: 1.2rem;">⚠️</span>
      <span>{$t('settings.general.path_invalid')}</span>
    </div>
  {/if}

  <!-- Ligne 1 : Dossier du jeu -->
  <div style="display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap;">
    <div class="field" style="flex: 1; min-width: 300px;">
      <label>{$t('settings.general.path_label')}</label>
      <input type="text" readonly bind:value={selectedPath} placeholder="{$t('settings.general.path_placeholder')}" />
    </div>
    <button class="btn-secondary" onclick={choisirDossier} style="padding: 10px 20px; height: 40px;">{$t('settings.general.path_browse')}</button>
    <button class="btn-launch" onclick={sauvegarder} style="padding: 10px 20px; height: 40px;">{$t('settings.general.path_save')}</button>
  </div>

  <!-- Ligne 2 : Dossier Documents -->
  <div style="display: flex; gap: 12px; align-items: flex-end; flex-wrap: wrap; margin-top: 8px;">
    <div class="field" style="flex: 1; min-width: 300px;">
      <label>{$t('settings.general.docs_label')}</label>
      <input type="text" readonly value={appState.documentsPath || $t('settings.general.docs_auto')} placeholder={$t('settings.general.docs_placeholder')} />
    </div>
    <button class="btn-secondary" onclick={choisirDossierDocuments} style="padding: 10px 20px; height: 40px;">{$t('settings.general.path_browse')}</button>
  </div>
</div>

<div class="group">
  <div class="group-title">{$t('settings.general.identity_title')}</div>
  <div class="row">
    <div class="field">
      <label>{$t('settings.general.name_solo')}</label>
      <input type="text" value={appState.driverName} oninput={(e) => setDriverName(e.currentTarget.value)} placeholder={$t('settings.general.name_solo_placeholder')} />
    </div>
    <div class="field">
      <label>{$t('settings.general.name_online')}</label>
      <input type="text" value={appState.onlineName} oninput={(e) => setOnlineName(e.currentTarget.value)} placeholder={$t('settings.general.name_online_placeholder')} />
    </div>
  </div>
  <div class="row">
    <div class="field" style="flex: 2;">
      <label>{$t('settings.general.steam_id')}</label>
      <input type="text" value={appState.steamId} oninput={(e) => setSteamId(e.currentTarget.value)} placeholder={$t('settings.general.steam_id_placeholder')} />
    </div>
    <div class="field" style="flex: 0 0 auto; justify-content: flex-end;">
      <button class="btn-secondary" onclick={detectSteamId} disabled={isDetectingSteam}>
        {isDetectingSteam ? $t('settings.general.steam_detecting') : $t('settings.general.steam_detect')}
      </button>
    </div>
  </div>
</div>

{#if capabilities}
  <div class="group">
    <div class="group-title">{$t('settings.general.modules_title')}</div>
    <div class="row">
      <div class="field">
        <label>Content Shaders Patch (CSP) {#if capabilities.has_csp}<span style="color: var(--accent); font-weight: normal;">{capabilities.csp_version}</span>{/if}</label>
        <div style="display: flex; align-items: center; gap: 8px; padding: 8px 0;">
          <span style="font-size: 1.2rem;">{capabilities.has_csp ? '✅' : '❌'}</span>
          <span>{capabilities.has_csp ? $t('settings.general.module_installed') : $t('settings.general.module_not_installed')}</span>
        </div>
      </div>
      <div class="field">
        <label>Pure Weather {#if capabilities.has_pure_weather && capabilities.pure_version !== 'Inconnue'}<span style="color: var(--accent); font-weight: normal;">{capabilities.pure_version}</span>{/if}</label>
        <div style="display: flex; align-items: center; gap: 8px; padding: 8px 0;">
          <span style="font-size: 1.2rem;">{capabilities.has_pure_weather ? '✅' : '❌'}</span>
          <span>{capabilities.has_pure_weather ? $t('settings.general.module_installed') : $t('settings.general.module_not_installed')}</span>
        </div>
      </div>
      <div class="field">
        <label>Sol Weather {#if capabilities.has_sol && capabilities.sol_version !== 'Inconnue'}<span style="color: var(--accent); font-weight: normal;">{capabilities.sol_version}</span>{/if}</label>
        <div style="display: flex; align-items: center; gap: 8px; padding: 8px 0;">
          <span style="font-size: 1.2rem;">{capabilities.has_sol ? '✅' : '❌'}</span>
          <span>{capabilities.has_sol ? $t('settings.general.module_installed') : $t('settings.general.module_not_installed')}</span>
        </div>
      </div>
      <div class="field">
        <label>Pure Planner {#if capabilities.has_pure && capabilities.pure_planner_version !== 'Inconnue'}<span style="color: var(--accent); font-weight: normal;">{capabilities.pure_planner_version}</span>{/if}</label>
        <div style="display: flex; align-items: center; gap: 8px; padding: 8px 0;">
          <span style="font-size: 1.2rem;">{capabilities.has_pure ? '✅' : '❌'}</span>
          <span>{capabilities.has_pure ? $t('settings.general.module_installed') : $t('settings.general.module_not_installed')}</span>
        </div>
      </div>
    </div>
  </div>
{/if}

{#if isChecking}
  <div style="text-align: center; padding: 20px; color: var(--text-secondary);">{$t('settings.general.modules_checking')}</div>
{/if}

<div class="group">
  <div class="group-title">Assistant d'installation</div>
  <p style="font-size: 0.85rem; color: var(--text-secondary); line-height: 1.6; margin-bottom: 10px;">
    Relance l'assistant de configuration initiale pour revoir les étapes de détection, identité et modules météo.
  </p>
  <button class="btn-secondary" onclick={() => resetSetup()} style="align-self: flex-start;">
    🏗️ Relancer l'assistant
  </button>
</div>

<div class="group">
  <div class="group-title">{$t('settings.general.data_title')}</div>
  <p style="font-size: 0.85rem; color: var(--text-secondary); line-height: 1.6; margin-bottom: 10px;">
    {$t('settings.general.data_desc')} <br> <em>{$t('settings.general.data_desc_small')}</em>
  </p>
  <button class="btn-secondary" onclick={resetDonnees} style="align-self: flex-start; border-color: #a83a3a; color: #e57373;">
    {$t('settings.general.data_reset_btn')}
  </button>
</div>

<div class="group">
  <div class="group-title">{$t('settings.general.config_title')}</div>
  <p style="font-size: 0.85rem; color: var(--text-secondary); line-height: 1.6;">
    {$t('settings.general.config_desc')}
  </p>
  <button class="btn-secondary" onclick={() => invoke('open_folder', { acPath: appState.acPath, folderType: 'cfg', folderId: '' })} style="align-self: flex-start; margin-top: 8px;">
    {$t('settings.general.config_open_btn')}
  </button>
</div>

<div class="group">
  <div class="group-title">{$t('settings.general.about_title')}</div>
  <div class="row" style="align-items: center; justify-content: space-between;">
    <div style="font-size: 0.9rem; font-weight: 600; color: var(--text-primary);">
      QACRL Custom Launcher <span style="color: var(--accent); margin-left: 8px;">{APP_VERSION}</span>
    </div>
  </div>
  
  <div class="row" style="margin-top: 15px; gap: 10px;">
    <button class="filter-btn" onclick={() => openUrl('https://acstuff.ru/app/')}>🌐 Content Manager</button>
    <button class="filter-btn" onclick={() => openUrl('https://www.patreon.com/pure')}>🌤️ Pure</button>
    <button class="filter-btn" onclick={() => openUrl('https://discord.gg/assettocorsa')}>💬 Discord AC</button>
    <button class="filter-btn" onclick={() => openUrl('https://www.overtake.gg/')}>🏁 Overtake.gg</button>
  </div>
</div>
</div> <!-- ⭐ Fermeture du sous-onglet Général -->

<!-- ⭐ SOUS-ONGLET CONTRÔLES -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-controls'}>
  <ControlsTab active={activeSubTab === 'subtab-controls'} />
</div>

<!-- ⭐ SOUS-ONGLET VIDÉO -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-video'}>
  <VideoTab />
</div>

<!-- ⭐ SOUS-ONGLET AUDIO -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-audio'}>
  <AudioTab />
</div>

<!-- ⭐ SOUS-ONGLET CSP -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-csp'}>
  <CspTab />
</div>

<!-- ⭐ SOUS-ONGLET AIDE -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-help'}>
  <HelpTab />
</div>