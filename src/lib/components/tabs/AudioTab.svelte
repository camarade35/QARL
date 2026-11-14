<script lang="ts">
  import { t } from 'svelte-i18n';
  import { alert } from '$lib/stores/dialog.svelte'; // ⭐ AJOUT IMPORT QARL
  import { searchState } from '$lib/stores/searchState.svelte.js';
  import AutosaveBanner from '$lib/components/AutosaveBanner.svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { appState } from '$lib/stores/appState.svelte.js';

  // État pour les sections pliables (repliées par défaut)
  let openGroups = $state<Record<string, boolean>>({
    levels: false, levels_ext: false, settings: false, skids: false
  });

  // Mémoriser l'état des sections
  function toggleGroup(group: string) {
    openGroups[group] = !openGroups[group];
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('qarl_audio_groups', JSON.stringify(openGroups));
    }
  }

  // État des paramètres audio (fidèle à iniSchema.js)
  let audioData = $state({
    LEVELS: {
      BRAKES: 0.8, DIRT_BOTTOM: 1.0, ENGINE: 1.0, MASTER: 0.64, MUSIC: 0.5,
      OPPONENTS: 0.8, SURFACES: 0.9, TYRES: 0.5, UISOUNDS: 0.7, WIND: 0.4
    },
    LEVELS_EXT: {
      CAR_COMPONENTS: 1.0, TRACK: 1.0, WEATHER: 1.0, WIPERS: 0.8
    },
    SETTINGS: {
      DRIVER_NAME: 'Haut-parleurs (Logitech PRO X Gaming Headset)'
    },
    SKIDS: {
      ENTRY_POINT: 100
    }
  });

  let availableDevices = $state<string[]>([]);
  let hasChanges = $state(false);
  let lastSavedJson = '';
  let autosaveTimer: ReturnType<typeof setTimeout> | null = null;

  // Charger la liste des périphériques audio
  async function loadAudioDevices() {
    try {
      const devices = await invoke<string[]>('list_audio_devices');
      availableDevices = devices;
    } catch (e) {
      console.warn("Impossible de lister les périphériques audio:", e);
    }
  }

  // Charger audio.ini
  async function loadAudioIni() {
    if (!appState.documentsPath) return;
    try {
      const content = await invoke<string>('read_ini_file', { 
        documentsPath: appState.documentsPath, 
        filename: 'audio.ini' 
      });
      
      if (content) {
        const lines = content.split('\n');
        let currentSec = '';
        for (const line of lines) {
          const l = line.trim();
          if (l.startsWith('[') && l.endsWith(']')) { currentSec = l.slice(1, -1).toUpperCase(); continue; }
          const parts = l.split('=');
          if (parts.length >= 2) {
            const k = parts[0].trim().toUpperCase();
            const v = parts[1].trim();
            
            if (currentSec === 'LEVELS') {
              if (k in audioData.LEVELS) audioData.LEVELS[k] = parseFloat(v);
            }
            if (currentSec === 'LEVELS_EXT') {
              if (k in audioData.LEVELS_EXT) audioData.LEVELS_EXT[k] = parseFloat(v);
            }
            if (currentSec === 'SETTINGS') {
              if (k === 'DRIVER_NAME') audioData.SETTINGS.DRIVER_NAME = v;
            }
            if (currentSec === 'SKIDS') {
              if (k === 'ENTRY_POINT') audioData.SKIDS.ENTRY_POINT = parseInt(v);
            }
          }
        }
      }
    } catch (e) {
      console.warn("Erreur chargement audio.ini:", e);
    } finally {
      lastSavedJson = JSON.stringify(audioData);
      hasChanges = false;
    }
  }

  // Sauvegarder audio.ini
  async function saveAudioIni(silent = false) {
    try {
      const a = audioData;
      // On reconstruit le fichier section par section
      const content = `[HEADER]
VERSION=2

[LEVELS]
BRAKES=${a.LEVELS.BRAKES.toFixed(2)}
DIRT_BOTTOM=${a.LEVELS.DIRT_BOTTOM.toFixed(2)}
ENGINE=${a.LEVELS.ENGINE.toFixed(2)}
MASTER=${a.LEVELS.MASTER.toFixed(2)}
MUSIC=${a.LEVELS.MUSIC.toFixed(2)}
OPPONENTS=${a.LEVELS.OPPONENTS.toFixed(2)}
SURFACES=${a.LEVELS.SURFACES.toFixed(2)}
TYRES=${a.LEVELS.TYRES.toFixed(2)}
UISOUNDS=${a.LEVELS.UISOUNDS.toFixed(2)}
WIND=${a.LEVELS.WIND.toFixed(2)}

[LEVELS_EXT]
CAR_COMPONENTS=${a.LEVELS_EXT.CAR_COMPONENTS.toFixed(2)}
TRACK=${a.LEVELS_EXT.TRACK.toFixed(2)}
WEATHER=${a.LEVELS_EXT.WEATHER.toFixed(2)}
WIPERS=${a.LEVELS_EXT.WIPERS.toFixed(2)}

[SETTINGS]
DRIVER_NAME=${a.SETTINGS.DRIVER_NAME}

[SKIDS]
ENTRY_POINT=${a.SKIDS.ENTRY_POINT}
`;

      await invoke('write_ini_file', { 
        documentsPath: appState.documentsPath, 
        filename: 'audio.ini', 
        content 
      });
      
      lastSavedJson = JSON.stringify(audioData);
      hasChanges = false;
      if (autosaveTimer) { clearTimeout(autosaveTimer); autosaveTimer = null; }
      if (!silent) alert($t('audio.save_success'));
    } catch (e) {
      if (!silent) alert($t('audio.save_error') + e);
    }
  }

  // ⭐ Auto-save : détecte les modifications
  $effect(() => {
    const current = JSON.stringify(audioData);
    if (!lastSavedJson) return;
    if (current === lastSavedJson) return;
    hasChanges = true;
    if (searchState.autosaveDelay > 0) {
      if (autosaveTimer) clearTimeout(autosaveTimer);
      autosaveTimer = setTimeout(() => {
        if (hasChanges) saveAudioIni(true);
      }, searchState.autosaveDelay * 1000);
    }
  });

  onMount(() => {
    // ⭐ Restauration de l'état des menus
    if (typeof localStorage !== 'undefined') {
      try {
        const saved = localStorage.getItem('qarl_audio_groups');
        if (saved) {
          const parsed = JSON.parse(saved);
          openGroups = { ...openGroups, ...parsed };
        }
      } catch (e) {
        console.warn("Impossible de restaurer l'état des menus audio", e);
      }
    }

    loadAudioIni();
    loadAudioDevices();

    // ⭐ Écouteur pour la recherche globale
    const handler = (e: CustomEvent) => {
      if (e.detail.tab !== 'subtab-audio') return;
      const g = e.detail.group;
      if (g in openGroups) openGroups[g] = true;
    };
    window.addEventListener('qarl-open-section', handler as EventListener);
    return () => window.removeEventListener('qarl-open-section', handler as EventListener);
  });
</script>

<AutosaveBanner hasChanges={hasChanges} onSave={() => saveAudioIni(false)} />

<div style="display: flex; justify-content: flex-end; margin-bottom: 10px;">
  <button class="btn-launch" onclick={() => saveAudioIni(false)} style="padding: 10px 20px; font-size: 0.85rem; display: flex; align-items: center; gap: 8px;">
    💾 {$t('audio.save_btn')}
  </button>
</div>

<!-- ⭐ VOLUMES PRINCIPAUX -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('levels')}>
    <span style="transform: rotate({openGroups.levels ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🔊 {$t('audio.levels_title')} <span class="ini-section-raw" style="margin-left: 8px;">[LEVELS]</span>
  </div>
  
  {#if openGroups.levels}
    <div class="ini-section">
      <div class="ini-row">
        <div class="ini-label">{$t('audio.brakes')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.BRAKES} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.BRAKES} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.dirt_bottom')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.DIRT_BOTTOM} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.DIRT_BOTTOM} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.engine')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.ENGINE} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.ENGINE} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.master')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.MASTER} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.MASTER} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.music')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.MUSIC} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.MUSIC} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.opponents')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.OPPONENTS} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.OPPONENTS} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.surfaces')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.SURFACES} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.SURFACES} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.tyres')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.TYRES} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.TYRES} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.uisounds')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.UISOUNDS} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.UISOUNDS} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.wind')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.WIND} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS.WIND} />
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- ⭐ VOLUMES ÉTENDUS -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('levels_ext')}>
    <span style="transform: rotate({openGroups.levels_ext ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🔊 {$t('audio.levels_ext_title')} <span class="ini-section-raw" style="margin-left: 8px;">[LEVELS_EXT]</span>
  </div>
  
  {#if openGroups.levels_ext}
    <div class="ini-section">
      <div class="ini-row">
        <div class="ini-label">{$t('audio.car_components')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.CAR_COMPONENTS} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.CAR_COMPONENTS} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.track')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.TRACK} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.TRACK} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.weather')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.WEATHER} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.WEATHER} />
        </div>
      </div>
      <div class="ini-row">
        <div class="ini-label">{$t('audio.wipers')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.WIPERS} />
          <input type="number" class="ini-number" min="0" max="1" step="0.01" bind:value={audioData.LEVELS_EXT.WIPERS} />
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- ⭐ PARAMÈTRES (Périphérique) -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('settings')}>
    <span style="transform: rotate({openGroups.settings ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    ⚙️ {$t('audio.settings_title')} <span class="ini-section-raw" style="margin-left: 8px;">[SETTINGS]</span>
  </div>
  
  {#if openGroups.settings}
    <div class="ini-section">
      <div class="ini-row">
        <div class="ini-label">{$t('audio.audio_device')}</div>
        <div class="ini-slider-wrap">
          <select class="ini-select" bind:value={audioData.SETTINGS.DRIVER_NAME}>
            <option value="">🎧 {$t('audio.default_device')}</option>
            {#each availableDevices as device}
              <option value={device}>{device}</option>
            {/each}
            <!-- ⭐ Si le périphérique mémorisé n'est pas branché/introuvable, on l'affiche quand même -->
            {#if audioData.SETTINGS.DRIVER_NAME && !availableDevices.includes(audioData.SETTINGS.DRIVER_NAME)}
              <option value={audioData.SETTINGS.DRIVER_NAME}>{audioData.SETTINGS.DRIVER_NAME} ({$t('audio.not_found')})</option>
            {/if}
          </select>
        </div>
      </div>
    </div>
  {/if}
</div>

<!-- ⭐ DÉRAPAGES -->
<div class="group" style="margin-bottom: 20px; border: 1px solid var(--border); background: var(--bg-panel);">
  <div class="group-title" style="color: var(--accent); display: flex; align-items: center; gap: 8px; cursor: pointer; user-select: none;" onclick={() => toggleGroup('skids')}>
    <span style="transform: rotate({openGroups.skids ? '90deg' : '0deg'}); transition: transform 0.2s; font-size: 1.1rem; line-height: 1;">▸</span>
    🛞 {$t('audio.skids_title')} <span class="ini-section-raw" style="margin-left: 8px;">[SKIDS]</span>
  </div>
  
  {#if openGroups.skids}
    <div class="ini-section">
      <div class="ini-row">
        <div class="ini-label">{$t('audio.entry_point')}</div>
        <div class="ini-slider-wrap">
          <input type="range" class="ini-range" min="0" max="1000" step="10" bind:value={audioData.SKIDS.ENTRY_POINT} />
          <input type="number" class="ini-number" min="0" max="1000" bind:value={audioData.SKIDS.ENTRY_POINT} />
        </div>
      </div>
    </div>
  {/if}
</div>