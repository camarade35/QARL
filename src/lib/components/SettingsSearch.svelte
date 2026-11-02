<script lang="ts">
  import { t } from 'svelte-i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { searchState, clearSearch } from '$lib/stores/searchState.svelte.js';

  let { onNavigate = (tabId: string, groupKey: string, cspFile: string | null) => {} } = $props();

  // Types
  type Entry = {
    file: string;
    section: string;
    key: string;
    value: string;
    tab: string;
    groupKey: string;
    labelKey: string | null;
    labelDefault: string;
    isAlt?: boolean;
    cspFile?: string | null;
  };

  let loading = $state(false);
  let loaded = $state(false);
  let error = $state('');
  let allEntries = $state<Entry[]>([]);

  // ⭐ Mapping Audio
  const AUDIO_MAP: Record<string, Record<string, string>> = {
    LEVELS: {
      BRAKES: 'audio.brakes', DIRT_BOTTOM: 'audio.dirt_bottom', ENGINE: 'audio.engine',
      MASTER: 'audio.master', MUSIC: 'audio.music', OPPONENTS: 'audio.opponents',
      SURFACES: 'audio.surfaces', TYRES: 'audio.tyres', UISOUNDS: 'audio.uisounds', WIND: 'audio.wind'
    },
    LEVELS_EXT: {
      CAR_COMPONENTS: 'audio.car_components', TRACK: 'audio.track',
      WEATHER: 'audio.weather', WIPERS: 'audio.wipers'
    },
    SETTINGS: { DRIVER_NAME: 'audio.audio_device' },
    SKIDS: { ENTRY_POINT: 'audio.entry_point' }
  };

  // ⭐ Mapping Vidéo
  const VIDEO_MAP: Record<string, Record<string, string>> = {
    VIDEO: {
      AAQUALITY: 'video.label_aa_quality', AASAMPLES: 'video.label_aa_samples',
      ANISOTROPIC: 'video.label_anisotropic', DISABLE_LEGACY_HDR: 'video.label_disable_hdr',
      FPS_CAP_MS: 'video.label_fps_cap', FULLSCREEN: 'video.label_fullscreen',
      INDEX: 'video.label_index', REFRESH: 'video.label_refresh',
      SHADOW_MAP_SIZE: 'video.label_shadow', VSYNC: 'video.label_vsync',
      WIDTH: 'video.label_resolution', HEIGHT: 'video.label_resolution'
    },
    POST_PROCESS: {
      DOF: 'video.label_dof', FXAA: 'video.label_fxaa', GLARE: 'video.label_glare',
      HEAT_SHIMMER: 'video.label_heat', QUALITY: 'video.label_quality', RAYS_OF_GOD: 'video.label_rays'
    },
    EFFECTS: {
      FXAA: 'video.label_fxaa', MOTION_BLUR: 'video.label_motion_blur',
      RENDER_SMOKE_IN_MIRROR: 'video.label_smoke_mirror', SMOKE: 'video.label_smoke'
    },
    MIRROR: { HQ: 'video.label_high_quality', SIZE: 'video.label_size' },
    CUBEMAP: {
      FACES_PER_FRAME: 'video.label_faces_frame', FARPLANE: 'video.label_farplane',
      SIZE: 'video.label_size'
    },
    ASSETTOCORSA: {
      HIDE_ARMS: 'video.label_hide_arms', HIDE_STEER: 'video.label_hide_steer',
      LOCK_STEER: 'video.label_lock_steer', WORLD_DETAIL: 'video.label_world_detail'
    },
    CAMERA: { MODE: 'video.label_mode' },
    REFRESH: { VALUE: 'video.label_rate_hz' },
    SATURATION: { LEVEL: 'video.label_saturation_pct' }
  };

  function parseIniIntoEntries(
    content: string, filename: string, tab: string,
    mapping: Record<string, Record<string, string>>
  ): Entry[] {
    const entries: Entry[] = [];
    let currentSection = '';
    for (const line of content.split('\n')) {
      const trimmed = line.trim();
      if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
        currentSection = trimmed.slice(1, -1).toUpperCase();
        continue;
      }
      const parts = trimmed.split('=');
      if (parts.length >= 2 && currentSection) {
        const key = parts[0].trim().toUpperCase();
        const value = parts.slice(1).join('=').trim();
        const labelKey = mapping[currentSection]?.[key];
        if (labelKey) {
          entries.push({
            file: filename, section: currentSection, key, value, tab,
            groupKey: currentSection.toLowerCase(),
            labelKey, labelDefault: key, cspFile: null
          });
        }
      }
    }
    return entries;
  }

  function parseControlsIntoEntries(content: string): Entry[] {
    const entries: Entry[] = [];
    const sections: Record<string, Record<string, string>> = {};
    let currentSection = '';

    for (const line of content.split('\n')) {
      const trimmed = line.trim();
      if (trimmed.startsWith('[') && trimmed.endsWith(']')) {
        currentSection = trimmed.slice(1, -1).toUpperCase();
        continue;
      }
      if (!currentSection || currentSection === 'CONTROLLERS' || currentSection === 'HEADER') continue;
      const parts = trimmed.split('=');
      if (parts.length >= 2) {
        const key = parts[0].trim();
        const value = parts.slice(1).join('=').trim();
        if (!sections[currentSection]) sections[currentSection] = {};
        sections[currentSection][key] = value;
      }
    }

    for (const [section, data] of Object.entries(sections)) {
      let baseSection = section;
      if (baseSection.endsWith('_ALT')) baseSection = baseSection.slice(0, -4);
      const labelKey = `settings.controls.actions.${baseSection}`;

      let primaryValue = '';
      if (data.BUTTON && data.BUTTON !== '-1') primaryValue = `Btn ${data.BUTTON}`;
      else if (data.AXLE && data.AXLE !== '-1') primaryValue = `Axe ${data.AXLE}`;
      else if (data.KEY && data.KEY !== '-1') primaryValue = data.KEY;

      if (primaryValue) {
        entries.push({
          file: 'controls.ini', section, key: section, value: primaryValue,
          tab: 'subtab-controls', groupKey: '',
          labelKey, labelDefault: baseSection, cspFile: null
        });
      }

      if (data.__CM_ALT_BUTTON && data.__CM_ALT_BUTTON !== '-1') {
        entries.push({
          file: 'controls.ini', section, key: section, value: `Btn ${data.__CM_ALT_BUTTON}`,
          tab: 'subtab-controls', groupKey: '',
          labelKey, labelDefault: baseSection, isAlt: true, cspFile: null
        });
      }
    }

    return entries;
  }

  async function loadAllData() {
    if (!appState.documentsPath) {
      error = $t('settings.search.need_config');
      return;
    }
    loading = true;
    error = '';
    const entries: Entry[] = [];

    // Audio
    try {
      const audioContent = await invoke<string>('read_ini_file', {
        documentsPath: appState.documentsPath, filename: 'audio.ini'
      });
      entries.push(...parseIniIntoEntries(audioContent, 'audio.ini', 'subtab-audio', AUDIO_MAP));
    } catch (e) { console.warn('audio load', e); }

    // Video
    try {
      const videoContent = await invoke<string>('read_ini_file', {
        documentsPath: appState.documentsPath, filename: 'video.ini'
      });
      entries.push(...parseIniIntoEntries(videoContent, 'video.ini', 'subtab-video', VIDEO_MAP));
    } catch (e) { console.warn('video load', e); }

    // Controls
    try {
      const controlsContent = await invoke<string>('read_controls_ini', {
        documentsPath: appState.documentsPath, deviceType: 'wheel'
      });
      if (controlsContent) {
        entries.push(...parseControlsIntoEntries(controlsContent));
      }
    } catch (e) { console.warn('controls load', e); }

    // CSP
    if (appState.acPath) {
      try {
        const templates = await invoke<Array<{ filename: string; default_enabled: boolean }>>('list_csp_templates', {
          acPath: appState.acPath
        });
        const cspResults = await Promise.all(templates.map(async (tpl) => {
          try {
            const data = await invoke<any>('read_csp_module_data', {
              documentsPath: appState.documentsPath,
              acPath: appState.acPath,
              filename: tpl.filename
            });
            const result: Entry[] = [];
            for (const sec of data.sections || []) {
              for (const opt of sec.options || []) {
                result.push({
                  file: tpl.filename,
                  section: sec.name,
                  key: opt.key,
                  value: String(opt.value ?? ''),
                  tab: 'subtab-csp',
                  groupKey: sec.name,
                  labelKey: `csp.option_labels.${opt.label}`,
                  labelDefault: opt.label || opt.key,
                  cspFile: tpl.filename
                });
              }
            }
            return result;
          } catch (e) {
            console.warn(`csp ${tpl.filename} load`, e);
            return [] as Entry[];
          }
        }));
        entries.push(...cspResults.flat());
      } catch (e) { console.warn('csp list', e); }
    }

    allEntries = entries;
    loaded = true;
    loading = false;
  }

  // ⭐ Lazy load : au premier caractère tapé
  let hasLoadedOnce = false;
  $effect(() => {
    if (searchState.query.trim() && !hasLoadedOnce && !loading) {
      hasLoadedOnce = true;
      loadAllData();
    }
  });

  // ⭐ Groupes filtrés
  let filteredGroups = $derived.by(() => {
    if (!searchState.query.trim() || !loaded) return [];
    const q = searchState.query.toLowerCase().trim();

    const matches = allEntries.filter((e) => {
      const label = e.labelKey ? $t(e.labelKey, { default: e.labelDefault }) : e.labelDefault;
      const labelStr = String(label).toLowerCase();
      return labelStr.includes(q) || e.key.toLowerCase().includes(q) || e.section.toLowerCase().includes(q);
    });

    const groups: Record<string, { file: string; section: string; tab: string; groupKey: string; cspFile: string | null; entries: Entry[] }> = {};
    for (const m of matches) {
      const id = `${m.file}::${m.section}`;
      if (!groups[id]) {
        groups[id] = {
          file: m.file, section: m.section, tab: m.tab, groupKey: m.groupKey,
          cspFile: m.cspFile || null, entries: []
        };
      }
      groups[id].entries.push(m);
    }
    return Object.values(groups);
  });

  let totalResults = $derived(filteredGroups.reduce((s, g) => s + g.entries.length, 0));

  function jumpTo(group: typeof filteredGroups[number]) {
    const tab = group.tab;
    const grp = group.groupKey;
    const cspFile = group.cspFile;
    clearSearch();
    onNavigate(tab, grp, cspFile);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape' && searchState.query) {
      clearSearch();
    }
  }
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="settings-search-bar">
  <div class="settings-search-input-wrap">
    <input
      type="text"
      placeholder={$t('settings.search.placeholder')}
      bind:value={searchState.query}
    />
    {#if searchState.query}
      <button class="settings-search-clear" onclick={clearSearch} title={$t('common.clear')}>✕</button>
    {/if}
  </div>
</div>

{#if searchState.query.trim()}
  <div class="settings-search-results">
    {#if loading}
      <div class="settings-search-loading">{$t('settings.search.loading')}</div>
    {:else if error}
      <div class="settings-search-empty">{error}</div>
    {:else if filteredGroups.length === 0}
      <div class="settings-search-empty">
        {$t('settings.search.no_results', { values: { query: searchState.query } })}
      </div>
    {:else}
      <div class="settings-search-count">
        {$t('settings.search.count', { values: { count: totalResults } })}
      </div>
      {#each filteredGroups as group}
        <div class="settings-search-group">
          <div class="settings-search-group-header">
            <span class="settings-search-group-file">{group.file}</span>
            <span class="settings-search-group-sep">›</span>
            <span class="settings-search-group-section">[{group.section}]</span>
            <button class="settings-search-group-jump" onclick={() => jumpTo(group)}>
              {$t('settings.search.go')} →
            </button>
          </div>
          <div class="settings-search-group-body">
            {#each group.entries as entry}
              <div class="ini-row" style="grid-template-columns: 240px 1fr; padding: 6px 16px;">
                <div class="ini-label">
                  {#if entry.labelKey}
                    {$t(entry.labelKey, { default: entry.labelDefault })}
                  {:else}
                    {entry.labelDefault}
                  {/if}
                  {#if entry.isAlt} <span style="color: var(--text-tertiary); font-size: 0.72rem;">(Alt)</span>{/if}
                </div>
                <div style="color: var(--text-secondary); font-family: 'Consolas', monospace; font-size: 0.82rem; text-align: right; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                  {entry.value}
                </div>
              </div>
            {/each}
          </div>
        </div>
      {/each}
    {/if}
  </div>
{/if}