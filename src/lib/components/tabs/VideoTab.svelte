<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { t } from 'svelte-i18n';
  import { alert } from '$lib/stores/dialog.svelte'; // ⭐ AJOUT IMPORT QARL
  import { searchState } from '$lib/stores/searchState.svelte.js';
  import AutosaveBanner from '$lib/components/AutosaveBanner.svelte';

  // ⭐ État pour gérer l'ouverture des sections Vidéo internes
  let openVideoSections = $state<Record<string, boolean>>({
    display: true, refresh: true, camera: true, post_process: true,
    saturation: true, effects: true, mirror: true, cubemap: true, assetto: true
  });

  function toggleVideoSection(section: string) {
    openVideoSections[section] = !openVideoSections[section];
  }

  // ⭐ État pour réduire les grands blocs (Groupes)
  let openGroups = $state<Record<string, boolean>>({
    rendu: true, cameralibre: true, affichage: true, effets: true, ingame: true
  });

  function toggleGroup(group: string) {
    openGroups[group] = !openGroups[group];
  }

  // ⭐ Liste des filtres PP disponibles
  let availableFilters = $state<string[]>([]);
  let hasChanges = $state(false);
  
  // ⭐ États pour les presets Vidéo
  let videoPresets = $state<string[]>([]);
  let selectedPreset = $state('');
  let showSaveAsVideo = $state(false);
  let newPresetName = $state('');
  let lastSavedJson = '';
  let autosaveTimer: ReturnType<typeof setTimeout> | null = null;

  // ⭐ État des paramètres vidéo
  let videoData = $state({
    weatherFx: { style: 'default' }, // ⭐ Ajout du state pour Weather FX
    video: {
      aaquality: 0, aasamples: 4, anisotropic: 16, disableLegacyHdr: true,
      limitFps: true, fpsLimit: 80, fpsCapMs: 12.5, // ⭐ NOUVEAU : limitFps et fpsLimit
      fullscreen: true, index: 272, refresh: 100,
      shadowMapSize: 2048, vsync: false, width: 3440, height: 1440
    },
    refresh: { value: 100 },
    camera: { mode: 'DEFAULT' },
    post_process: {
      dof: 5, fxaa: true, glare: 5, heatShimmer: true, quality: 5,
      raysOfGod: true, enabled: true, filter: 'pure'
    },
    saturation: { level: 100 },
    effects: { fxaa: false, motionBlur: false, smokeInMirror: true, smoke: 3 },
    mirror: { hq: true, size: 2048 },
    cubemap: { facesPerFrame: 2, farplane: 1200, size: 1024 },
    assetto: { hideArms: true, hideSteer: true, lockSteer: false, worldDetail: 5 }
  });

  // Charger video.ini
  async function loadVideoIni() {
    if (!appState.documentsPath) return;
    try {
      const content = await invoke<string>('read_ini_file', { 
        documentsPath: appState.documentsPath, 
        filename: 'video.ini' 
      });
      
      if (content) {
        const lines = content.split('\n');
        let currentSec = '';
        for (const line of lines) {
          const l = line.trim();
          if (l.startsWith('[') && l.endsWith(']')) { currentSec = l.slice(1, -1); continue; }
          const parts = l.split('=');
          if (parts.length >= 2) {
            const k = parts[0].trim().toUpperCase();
            const v = parts[1].trim();
            if (currentSec === 'VIDEO') {
              if (k === 'AAQUALITY') videoData.video.aaquality = parseInt(v);
              if (k === 'AASAMPLES') videoData.video.aasamples = parseInt(v);
              if (k === 'ANISOTROPIC') videoData.video.anisotropic = parseInt(v);
              if (k === 'DISABLE_LEGACY_HDR') videoData.video.disableLegacyHdr = v === '1';
              if (k === 'FPS_CAP_MS') {
                const ms = parseFloat(v);
                videoData.video.fpsCapMs = ms;
                if (ms > 0) {
                  videoData.video.limitFps = true;
                  videoData.video.fpsLimit = Math.round(1000 / ms); // Calcul inverse pour l'affichage
                } else {
                  videoData.video.limitFps = false;
                  videoData.video.fpsLimit = 60; // Valeur par défaut si désactivé
                }
              }
              if (k === 'FULLSCREEN') videoData.video.fullscreen = v === '1';
              if (k === 'INDEX') videoData.video.index = parseInt(v);
              if (k === 'REFRESH') videoData.video.refresh = parseInt(v);
              if (k === 'SHADOW_MAP_SIZE') videoData.video.shadowMapSize = parseInt(v);
              if (k === 'VSYNC') videoData.video.vsync = v === '1';
              if (k === 'WIDTH') videoData.video.width = parseInt(v);
              if (k === 'HEIGHT') videoData.video.height = parseInt(v);
            }
            if (currentSec === 'POST_PROCESS') {
              if (k === 'DOF') videoData.post_process.dof = parseInt(v);
              if (k === 'ENABLED') videoData.post_process.enabled = v === '1';
              if (k === 'FILTER') videoData.post_process.filter = v;
              if (k === 'FXAA') videoData.post_process.fxaa = v === '1';
              if (k === 'GLARE') videoData.post_process.glare = parseInt(v);
              if (k === 'HEAT_SHIMMER') videoData.post_process.heatShimmer = v === '1';
              if (k === 'QUALITY') videoData.post_process.quality = parseInt(v);
              if (k === 'RAYS_OF_GOD') videoData.post_process.raysOfGod = v === '1';
            }
          }
        }
      }
    } catch (e) {
      console.warn("Error loading video.ini:", e);
    }

    // ⭐ Lecture de Weather FX Implementation
    try {
      const wfxData = await invoke<any>('read_csp_module_data', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        filename: 'weather_fx.ini'
      });
      
      // On cherche la section BASIC et l'option IMPLEMENTATION
      const basicSection = wfxData.sections?.find((s: any) => s.name === 'BASIC');
      const implOption = basicSection?.options?.find((o: any) => o.key === 'IMPLEMENTATION');
      
      if (implOption) {
        const impl = String(implOption.value).trim().toLowerCase();
        if (impl === 'pure') videoData.weatherFx.style = 'pure';
        else if (impl === 'pure lcs') videoData.weatherFx.style = 'lcs';
        else videoData.weatherFx.style = 'default';
      } else {
        console.warn("⚠️ IMPLEMENTATION option not found in weather_fx.ini");
      }
    } catch (e) {
      console.warn("Error loading weather_fx.ini:", e);
    } finally {
      lastSavedJson = JSON.stringify(videoData);
      hasChanges = false;
    }
  }

  // Charger la liste des filtres PP
  async function loadPpfilters() {
    if (!appState.acPath) return;
    try {
      const filters = await invoke<string[]>('list_ppfilters', { acPath: appState.acPath });
      availableFilters = filters;
    } catch (e) {
      console.warn("Unable to load PPFilters list:", e);
    }
  }

  // Sauvegarder video.ini
  async function saveVideoIni(silent = false) {
    try {
      const v = videoData;
      const content = `[ASSETTOCORSA]
HIDE_ARMS=${v.assetto.hideArms ? 1 : 0}
HIDE_STEER=${v.assetto.hideSteer ? 1 : 0}
LOCK_STEER=${v.assetto.lockSteer ? 1 : 0}
WORLD_DETAIL=${v.assetto.worldDetail}

[CAMERA]
MODE=${v.camera.mode}

[CUBEMAP]
FACES_PER_FRAME=${v.cubemap.facesPerFrame}
FARPLANE=${v.cubemap.farplane}
SIZE=${v.cubemap.size}

[EFFECTS]
FXAA=${v.effects.fxaa ? 1 : 0}
MOTION_BLUR=${v.effects.motionBlur ? 1 : 0}
RENDER_SMOKE_IN_MIRROR=${v.effects.smokeInMirror ? 1 : 0}
SMOKE=${v.effects.smoke}

[MIRROR]
HQ=${v.mirror.hq ? 1 : 0}
SIZE=${v.mirror.size}

[POST_PROCESS]
DOF=${v.post_process.dof}
ENABLED=${v.post_process.enabled ? 1 : 0}
FILTER=${v.post_process.filter}
FXAA=${v.post_process.fxaa ? 1 : 0}
GLARE=${v.post_process.glare}
HEAT_SHIMMER=${v.post_process.heatShimmer ? 1 : 0}
QUALITY=${v.post_process.quality}
RAYS_OF_GOD=${v.post_process.raysOfGod ? 1 : 0}

[REFRESH]
VALUE=${v.refresh.value}

[SATURATION]
LEVEL=${v.saturation.level}

[VIDEO]
AAQUALITY=${v.video.aaquality}
AASAMPLES=${v.video.aasamples}
ANISOTROPIC=${v.video.anisotropic}
DISABLE_LEGACY_HDR=${v.video.disableLegacyHdr ? 1 : 0}
FPS_CAP_MS=${v.video.limitFps ? (1000 / v.video.fpsLimit).toFixed(13) : 0}
FULLSCREEN=${v.video.fullscreen ? 1 : 0}
HEIGHT=${v.video.height}
INDEX=${v.video.index}
REFRESH=${v.video.refresh}
SHADOW_MAP_SIZE=${v.video.shadowMapSize}
VSYNC=${v.video.vsync ? 1 : 0}
WIDTH=${v.video.width}
`;
      await invoke('write_ini_file', { 
        documentsPath: appState.documentsPath, 
        filename: 'video.ini', 
        content 
      });

      // ⭐ Sauvegarde de Weather FX Implementation
      // ⚠️ CSP est très sensible à la casse : "pure" doit être en minuscules, "Pure LCS" avec des majuscules.
      const implValue = videoData.weatherFx.style === 'pure' ? 'pure' : (videoData.weatherFx.style === 'lcs' ? 'Pure LCS' : 'Default');
      
      await invoke('set_weather_fx_implementation', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        implementation: implValue
      });
      
      lastSavedJson = JSON.stringify(videoData);
      hasChanges = false;
      if (autosaveTimer) { clearTimeout(autosaveTimer); autosaveTimer = null; }
      if (!silent) alert($t('video.save_success'));
    } catch (e) {
      if (!silent) alert($t('video.save_error') + e);
    }
  }

  // ⭐ Fonctions des Presets Vidéo
  async function loadVideoPresets() {
    try {
      videoPresets = await invoke<string[]>('list_video_presets');
    } catch (e) {
      console.warn("Erreur chargement presets vidéo:", e);
    }
  }

  async function handleSaveVideoPreset() {
    if (!newPresetName.trim()) return;
    try {
      // 1. On s'assure d'abord que le video.ini sur le disque est à jour avec l'UI
      await saveVideoIni(true);
      
      // 2. On lit le fichier généré
      const content = await invoke<string>('read_ini_file', {
        documentsPath: appState.documentsPath,
        filename: 'video.ini'
      });
      
      // 3. ⭐ On injecte le style Weather FX à la fin (avec la bonne casse pour CSP)
      const implValue = videoData.weatherFx.style === 'pure' ? 'pure' : (videoData.weatherFx.style === 'lcs' ? 'Pure LCS' : 'Default');
      const fullContent = content + `\n[WEATHER_FX]\nIMPLEMENTATION=${implValue}\n`;
      
      // 4. On sauvegarde le tout
      await invoke('save_video_preset', { name: newPresetName.trim(), content: fullContent });
      
      showSaveAsVideo = false;
      newPresetName = '';
      await loadVideoPresets();
      alert($t('video.presets.save_success'));
    } catch (e) {
      alert($t('video.presets.save_error') + e);
    }
  }

  async function handleLoadVideoPreset() {
    if (!selectedPreset) return;
    try {
      // 1. ⭐ Le backend restaure le video.ini propre et nous renvoie le style Weather FX
      const weatherFxValue = await invoke<string>('load_video_preset', {
        documentsPath: appState.documentsPath,
        name: selectedPreset
      });
      
      // 2. On applique le Weather FX dans son fichier dédié
      await invoke('set_weather_fx_implementation', {
        documentsPath: appState.documentsPath,
        acPath: appState.acPath || '',
        implementation: weatherFxValue
      });
      
      // 3. On rafraîchit l'UI
      await loadVideoIni();
      alert($t('video.presets.load_success'));
    } catch (e) {
      alert($t('video.presets.load_error') + e);
    }
  }

  async function handleDeleteVideoPreset() {
    if (!selectedPreset) return;
    if (confirm($t('video.presets.delete_confirm'))) {
      try {
        await invoke('delete_video_preset', { name: selectedPreset });
        selectedPreset = '';
        await loadVideoPresets();
      } catch (e) {
        alert(e);
      }
    }
  }

  // ⭐ Auto-save
  $effect(() => {
    const current = JSON.stringify(videoData);
    if (!lastSavedJson) return;
    if (current === lastSavedJson) return;
    hasChanges = true;
    if (searchState.autosaveDelay > 0) {
      if (autosaveTimer) clearTimeout(autosaveTimer);
      autosaveTimer = setTimeout(() => {
        if (hasChanges) saveVideoIni(true);
      }, searchState.autosaveDelay * 1000);
    }
  });

  onMount(() => {
    loadVideoIni();
    loadPpfilters();
    loadVideoPresets(); // ⭐ Chargement des presets

    // ⭐ Écouteur pour recharger l'onglet Vidéo quand un profil CSP est importé
    const handlePresetLoaded = () => {
      loadVideoIni();
    };
    window.addEventListener('qarl-csp-preset-loaded', handlePresetLoaded);

    // ⭐ Écouteur pour la recherche globale
    const handler = (e: CustomEvent) => {
      if (e.detail.tab !== 'subtab-video') return;
      const g = e.detail.group;
      if (g in openGroups) openGroups[g] = true;
      if (g in openVideoSections) openVideoSections[g] = true;
    };
    window.addEventListener('qarl-open-section', handler as EventListener);

    return () => {
      window.removeEventListener('qarl-csp-preset-loaded', handlePresetLoaded);
      window.removeEventListener('qarl-open-section', handler as EventListener);
    };
  });
</script>

<AutosaveBanner hasChanges={hasChanges} onSave={() => saveVideoIni(false)} />

<!-- ⭐ BARRE DES PRESETS VIDÉO -->
<div class="ini-toolbar" style="flex-wrap: wrap; gap: 10px; margin-bottom: 20px; background: var(--bg-panel); padding: 10px; border-radius: var(--r-sm); border: 1px solid var(--border);">
  <span style="font-weight: 700; font-size: 0.85rem; color: var(--accent); display: flex; align-items: center; gap: 5px;">📁 {$t('video.presets.title')}</span>
  
  <select class="ini-select" style="min-width: 200px; flex: 1;" bind:value={selectedPreset}>
    <option value="">-- {$t('video.presets.placeholder')} --</option>
    {#each videoPresets as p}
      <option value={p}>{p}</option>
    {/each}
  </select>
  
  <button class="btn-secondary" onclick={handleLoadVideoPreset} disabled={!selectedPreset}>
    {$t('video.presets.load_btn')}
  </button>

  {#if showSaveAsVideo}
    <input type="text" class="ini-text" style="width: 150px;" placeholder={$t('video.presets.prompt_name')} bind:value={newPresetName} />
    <button class="btn-secondary ini-btn-save" onclick={handleSaveVideoPreset} disabled={!newPresetName.trim()}>
      {$t('video.presets.save_btn')}
    </button>
    <button class="btn-secondary" onclick={() => showSaveAsVideo = false}>
      {$t('common.cancel')}
    </button>
  {:else}
    <button class="btn-secondary" onclick={() => showSaveAsVideo = true}>
      {$t('video.presets.save_as_btn')}
    </button>
  {/if}

  <button class="btn-secondary" onclick={handleDeleteVideoPreset} disabled={!selectedPreset} style="color: #e57373; border-color: #a83a3a;">
    {$t('video.presets.delete_btn')}
  </button>
</div>

<div style="display: flex; justify-content: flex-end; margin-bottom: 10px;">
  <button class="btn-launch" onclick={() => saveVideoIni(false)} style="padding: 10px 20px; font-size: 0.85rem; display: flex; align-items: center; gap: 8px;">
    💾 {$t('video.save_btn')}
  </button>
</div>

<!-- ⭐ RENDU — STYLE & FILTRE -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('rendu')}>
    <span style="transform: rotate({openGroups.rendu ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🎥 {$t('video.group_rendering')}
  </div>
  
  {#if openGroups.rendu}
    <div style="display: flex; gap: 20px; flex-wrap: wrap;">
      <div style="flex: 1; min-width: 250px; background: var(--bg-elevated); padding: 12px; border-radius: var(--r-sm); border: 1px solid var(--border-subtle);">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
          <span style="font-size: 0.85rem; font-weight: 600;">{$t('video.style_weather_fx')}</span>
          <label class="check-inline"><input type="checkbox" checked /> {$t('video.enable')}</label>
        </div>
        <select class="ini-select" style="width: 100%; font-weight: 700;" bind:value={videoData.weatherFx.style}>
          <option value="pure">Pure Gamma</option>
          <option value="lcs">Pure LCS</option>
          <option value="default">Default</option>
        </select>
      </div>

      <div style="flex: 1; min-width: 250px; background: var(--bg-elevated); padding: 12px; border-radius: var(--r-sm); border: 1px solid var(--border-subtle);">
        <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 8px;">
          <span style="font-size: 0.85rem; font-weight: 600;">{$t('video.pp_filter')}</span>
          <label class="check-inline">
            <input type="checkbox" bind:checked={videoData.post_process.enabled} /> {$t('video.enable')}
          </label>
        </div>
        <select class="ini-select" style="width: 100%; font-weight: 700;" bind:value={videoData.post_process.filter}>
          {#each availableFilters as filter}
            <option value={filter}>{filter}</option>
          {/each}
          {#if videoData.post_process.filter && !availableFilters.includes(videoData.post_process.filter)}
            <option value={videoData.post_process.filter}>{videoData.post_process.filter} ({$t('audio.not_found')})</option>
          {/if}
        </select>
      </div>
    </div>
    
    <div style="font-size: 0.75rem; color: var(--text-tertiary); margin-top: 10px; line-height: 1.6; background: var(--bg-elevated); padding: 10px; border-radius: var(--r-sm);">
      💡 {$t('video.help_weather_fx')}<br>
      • <strong>Default</strong> : {$t('video.help_weather_default')}<br>
      • <strong>Pure Gamma</strong> : {$t('video.help_weather_gamma')}<br>
      • <strong>Pure LCS</strong> : {$t('video.help_weather_lcs')}
    </div>
  {/if}
</div>

<!-- ⭐ CAMÉRA LIBRE & VUE ORBITE -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('cameralibre')}>
    <span style="transform: rotate({openGroups.cameralibre ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🎥 {$t('video.group_camera')}
  </div>
  
  {#if openGroups.cameralibre}
    <div style="display: flex; gap: 20px; flex-wrap: wrap; margin-bottom: 10px;">
      <label class="check-inline"><input type="checkbox" checked /> {$t('video.camera_free')}</label>
      <label class="check-inline"><input type="checkbox" checked /> {$t('video.camera_orbit')}</label>
    </div>
    <div style="font-size: 0.75rem; color: var(--text-tertiary); line-height: 1.6;">
      💡 {$t('video.help_camera_free')}<br>
      💡 {$t('video.help_camera_orbit')}
    </div>
  {/if}
</div>

<!-- ⭐ AFFICHAGE -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('affichage')}>
    <span style="transform: rotate({openGroups.affichage ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🖥️ {$t('video.group_display')}
  </div>

  {#if openGroups.affichage}
    <!-- Section [VIDEO] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('display')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.display ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.display')}</span>
        <span class="ini-section-raw">[VIDEO]</span>
      </div>
      
      {#if openVideoSections.display}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_aa_quality')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="16" bind:value={videoData.video.aaquality} />
            <input type="number" class="ini-number" bind:value={videoData.video.aaquality} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_aa_samples')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="8" bind:value={videoData.video.aasamples} />
            <input type="number" class="ini-number" bind:value={videoData.video.aasamples} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_anisotropic')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="16" bind:value={videoData.video.anisotropic} />
            <input type="number" class="ini-number" bind:value={videoData.video.anisotropic} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_disable_hdr')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.video.disableLegacyHdr} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_limit_fps')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.video.limitFps} />
          </div>
        </div>
        {#if videoData.video.limitFps}
          <div class="ini-row">
            <div class="ini-label">{$t('video.label_fps_value', { values: { fps: videoData.video.fpsLimit } })}</div>
            <div class="ini-slider-wrap">
              <input type="range" class="ini-range" min="30" max="240" step="1" bind:value={videoData.video.fpsLimit} />
              <input type="number" class="ini-number" min="30" max="240" step="1" bind:value={videoData.video.fpsLimit} />
            </div>
          </div>
        {/if}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_fullscreen')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.video.fullscreen} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">INDEX</div>
          <div class="ini-slider-wrap">
            <input type="text" class="ini-text" bind:value={videoData.video.index} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">REFRESH</div>
          <div class="ini-slider-wrap">
            <input type="text" class="ini-text" bind:value={videoData.video.refresh} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_shadow')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="512" max="4096" step="512" bind:value={videoData.video.shadowMapSize} />
            <input type="number" class="ini-number" bind:value={videoData.video.shadowMapSize} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_vsync')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.video.vsync} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_resolution')}</div>
          <div class="ini-slider-wrap">
            <select class="ini-select" bind:value={videoData.video.width}>
              <option value="3440">3440 × 1440 (native)</option>
              <option value="1920">1920 × 1080</option>
              <option value="2560">2560 × 1440</option>
            </select>
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_window_pos')}</div>
          <div class="ini-slider-wrap">
            <input type="text" class="ini-text" value="px2147483647..." disabled style="opacity: 0.6;" />
          </div>
        </div>
      {/if}
    </div>

    <!-- Section [REFRESH] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('refresh')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.refresh ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.refresh_rate')}</span>
        <span class="ini-section-raw">[REFRESH]</span>
      </div>
      {#if openVideoSections.refresh}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_rate_hz')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="60" max="240" step="1" bind:value={videoData.refresh.value} />
            <input type="number" class="ini-number" bind:value={videoData.refresh.value} />
          </div>
        </div>
      {/if}
    </div>

    <!-- Section [CAMERA] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('camera')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.camera ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.camera')}</span>
        <span class="ini-section-raw">[CAMERA]</span>
      </div>
      {#if openVideoSections.camera}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_mode')}</div>
          <div class="ini-slider-wrap">
            <select class="ini-select" bind:value={videoData.camera.mode}>
              <option value="DEFAULT">{$t('video.camera_mode_default')}</option>
              <option value="ORBIT">{$t('video.camera_mode_orbit')}</option>
            </select>
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- ⭐ EFFETS & QUALITÉ -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('effets')}>
    <span style="transform: rotate({openGroups.effets ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🎨 {$t('video.group_effects')}
  </div>

  {#if openGroups.effets}
    <!-- [POST_PROCESS] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('post_process')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.post_process ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.post_process')}</span>
        <span class="ini-section-raw">[POST_PROCESS]</span>
      </div>
      {#if openVideoSections.post_process}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_dof')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="10" bind:value={videoData.post_process.dof} />
            <input type="number" class="ini-number" bind:value={videoData.post_process.dof} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_fxaa')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.post_process.fxaa} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_glare')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="10" bind:value={videoData.post_process.glare} />
            <input type="number" class="ini-number" bind:value={videoData.post_process.glare} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_heat')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.post_process.heatShimmer} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_quality')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="10" bind:value={videoData.post_process.quality} />
            <input type="number" class="ini-number" bind:value={videoData.post_process.quality} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_rays')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.post_process.raysOfGod} />
          </div>
        </div>
      {/if}
    </div>

    <!-- [SATURATION] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('saturation')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.saturation ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.saturation')}</span>
        <span class="ini-section-raw">[SATURATION]</span>
      </div>
      {#if openVideoSections.saturation}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_saturation_pct')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="200" bind:value={videoData.saturation.level} />
            <input type="number" class="ini-number" bind:value={videoData.saturation.level} />
          </div>
        </div>
      {/if}
    </div>

    <!-- [EFFECTS] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('effects')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.effects ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.effects')}</span>
        <span class="ini-section-raw">[EFFECTS]</span>
      </div>
      {#if openVideoSections.effects}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_fxaa')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.effects.fxaa} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_motion_blur')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.effects.motionBlur} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_smoke_mirror')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.effects.smokeInMirror} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_smoke')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="10" bind:value={videoData.effects.smoke} />
            <input type="number" class="ini-number" bind:value={videoData.effects.smoke} />
          </div>
        </div>
      {/if}
    </div>

    <!-- [MIRROR] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('mirror')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.mirror ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.mirrors')}</span>
        <span class="ini-section-raw">[MIRROR]</span>
      </div>
      {#if openVideoSections.mirror}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_high_quality')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.mirror.hq} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_size')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="512" max="4096" step="512" bind:value={videoData.mirror.size} />
            <input type="number" class="ini-number" bind:value={videoData.mirror.size} />
          </div>
        </div>
      {/if}
    </div>

    <!-- [CUBEMAP] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('cubemap')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.cubemap ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.cubemap')}</span>
        <span class="ini-section-raw">[CUBEMAP]</span>
      </div>
      {#if openVideoSections.cubemap}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_faces_frame')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="1" max="6" bind:value={videoData.cubemap.facesPerFrame} />
            <input type="number" class="ini-number" bind:value={videoData.cubemap.facesPerFrame} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_farplane')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="500" max="5000" step="100" bind:value={videoData.cubemap.farplane} />
            <input type="number" class="ini-number" bind:value={videoData.cubemap.farplane} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_size')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="256" max="2048" step="256" bind:value={videoData.cubemap.size} />
            <input type="number" class="ini-number" bind:value={videoData.cubemap.size} />
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

<!-- ⭐ VISUEL IN-GAME -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('ingame')}>
    <span style="transform: rotate({openGroups.ingame ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🇮🇹 {$t('video.group_ingame')}
  </div>

  {#if openGroups.ingame}
    <!-- [ASSETTOCORSA] -->
    <div class="ini-section">
      <div class="ini-section-header" onclick={() => toggleVideoSection('assetto')}>
        <span class="ini-section-arrow" style="transform: rotate({openVideoSections.assetto ? '90deg' : '0deg'});">▸</span>
        <span class="ini-section-label" style="font-weight: 700;">{$t('video.ac_settings')}</span>
        <span class="ini-section-raw">[ASSETTOCORSA]</span>
      </div>
      {#if openVideoSections.assetto}
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_hide_arms')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.assetto.hideArms} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_hide_steer')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.assetto.hideSteer} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_lock_steer')}</div>
          <div class="ini-slider-wrap" style="justify-content: flex-start;">
            <input type="checkbox" class="ini-checkbox" bind:checked={videoData.assetto.lockSteer} />
          </div>
        </div>
        <div class="ini-row">
          <div class="ini-label">{$t('video.label_world_detail')}</div>
          <div class="ini-slider-wrap">
            <input type="range" class="ini-range" min="0" max="10" bind:value={videoData.assetto.worldDetail} />
            <input type="number" class="ini-number" bind:value={videoData.assetto.worldDetail} />
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>