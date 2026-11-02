<script lang="ts">
  import { appState, saveSession } from '$lib/stores/appState.svelte.js';
  import { onMount } from 'svelte';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { getWeatherIcon, translateWeatherName } from '$lib/utils/weatherUtils.js';
  import { generateGrid } from '$lib/utils/sessionGenerator.js';
  import { t, locale } from 'svelte-i18n';
  import { alert, confirm } from '$lib/stores/dialog.svelte';

  // ⭐ Gestion de l'onglet actif
  let activeSubTab = $state('subtab-session');

  // ⭐ HISTORIQUE
  let history = $state<any[]>([]);

  async function loadHistory() {
    try {
      history = await invoke<any[]>('read_race_history');
    } catch (e) {
      console.error("Error loading history:", e);
    }
  }

  // Recharge l'historique à chaque fois qu'on clique sur l'onglet
  $effect(() => {
    if (activeSubTab === 'subtab-history') {
      loadHistory();
    }
  });

  onMount(() => {
    loadHistory();
  });

  async function clearHistory() {
    if (await confirm($t('session.history.confirm_clear'), $t('dialog.confirm'), "warning")) {
      await invoke('clear_race_history');
      history = [];
    }
  }

  async function deleteEntry(id: string, e: Event) {
    e.stopPropagation();
    await invoke('delete_history_entry', { entryId: id });
    history = history.filter(h => h.id !== id);
  }

  function formatDate(timestampStr: string) {
    const ts = parseInt(timestampStr) * 1000;
    const d = new Date(ts);
    return d.toLocaleString($locale, { day: '2-digit', month: '2-digit', year: 'numeric', hour: '2-digit', minute: '2-digit' });
  }

  // Récupère l'image de la voiture depuis l'état global
  function getCarImage(carId: string, skinId: string) {
    const car = appState.cars.find(c => c.id === carId);
    if (!car) return '';
    const skin = car.skins.find(s => s.name === skinId);
    if (skin) return convertFileSrc(skin.image_path);
    if (car.image_path) return convertFileSrc(car.image_path);
    return '';
  }

  function getCarName(carId: string) {
    const car = appState.cars.find(c => c.id === carId);
    return car ? car.name : carId;
  }

  function getTrackName(trackId: string, layoutId: string) {
    const track = appState.tracks.find(t => t.id === trackId);
    if (!track) return trackId;
    let name = track.name;
    if (layoutId && layoutId !== trackId && layoutId.toLowerCase() !== 'défaut' && layoutId.toLowerCase() !== 'default') {
      const layout = track.layouts.find(l => l.id === layoutId);
      if (layout) name += ` (${layout.name})`;
    }
    return name;
  }

  function loadConfigFromHistory(h: any) {
    // Restaure la voiture
    const car = appState.cars.find(c => c.id === h.car_id);
    if (car) {
      appState.selectedCar = car;
      appState.selectedCarSkin = car.skins.find((s: any) => s.name === h.skin_id) || car.skins[0] || null;
    }
    // Restaure le circuit
    const track = appState.tracks.find(t => t.id === h.track_id);
    if (track) {
      appState.selectedTrack = track;
      appState.selectedTrackLayout = track.layouts.find((l: any) => l.id === h.layout_id) || track.layouts[0] || null;
    }
    // Restaure la session
    appState.session = h.session_data;
  }

  async function launchFromHistory(h: any) {
    loadConfigFromHistory(h);
    
    // Génère la grille IA
    const aiGrid = generateGrid(appState.selectedCar, appState.cars, appState.session);

    try {
      await invoke('launch_race', {
        acPath: appState.acPath,
        carId: appState.selectedCar.id,
        skinId: appState.selectedCarSkin?.name || "",
        trackId: appState.selectedTrack.id,
        layoutId: appState.selectedTrackLayout?.id || "",
        sessionData: appState.session,
        aiGrid: aiGrid
      });
    } catch (e) {
      alert($t('session.history.launch_error') + e);
    }
  }

  // ⭐ État local pour les presets (sauvegardé en local pour l'instant)
  let presetName = $state('');
  let presets = $state<any[]>([]);
  
  $effect(() => {
    JSON.stringify(appState.session); 
    saveSession();
  });

  function switchSubTab(tabId: string) {
    activeSubTab = tabId;
  }

  function savePreset() {
    if (!presetName.trim()) return;
    presets = [...presets, { id: Date.now(), name: presetName, data: JSON.parse(JSON.stringify(appState.session)) }];
    presetName = '';
    alert($t('session.presets.save_success'));
  }
</script>

<div class="subtabs">
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-session'} onclick={() => switchSubTab('subtab-session')}>{$t('session.subtabs.session')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-race-ia'} onclick={() => switchSubTab('subtab-race-ia')}>{$t('session.subtabs.race_ia')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-pilot-realism'} onclick={() => switchSubTab('subtab-pilot-realism')}>{$t('session.subtabs.pilot')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-presets'} onclick={() => switchSubTab('subtab-presets')}>{$t('session.subtabs.presets')}</button>
  <button class="subtab-btn" class:active={activeSubTab === 'subtab-history'} onclick={() => switchSubTab('subtab-history')}>{$t('session.subtabs.history')}</button>
</div>

<!-- ⭐ SOUS-ONGLET 1 : SESSION -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-session'}>
  <div class="group">
    <div class="group-title">{$t('session.session.group_title')}</div>
    <div class="row">
      <div class="field">
        <label>{$t('session.session.type_label')}</label>
        <select bind:value={appState.session.type}>
          <option value="PRACTICE">{$t('session.types.practice')}</option>
          <option value="RACE">{$t('session.types.race')}</option>
          <option value="HOTLAP">{$t('session.types.hotlap')}</option>
        </select>
      </div>
    </div>

    {#if appState.session.type !== 'RACE'}
      <div class="row">
        <div class="field">
          <label>{$t('session.session.duration_label')}</label>
          <input type="number" bind:value={appState.session.sessionMinutes} min="1" max="240" />
        </div>
      </div>
    {/if}

    {#if appState.session.type === 'RACE'}
      <div class="row">
        <div class="field">
          <label>{$t('session.session.practice_label')}</label>
          <input type="number" bind:value={appState.session.practiceMinutes} min="0" max="240" />
        </div>
        <div class="field">
          <label>{$t('session.session.qualify_label')}</label>
          <input type="number" bind:value={appState.session.qualifyMinutes} min="0" max="120" />
        </div>
        <div class="field">
          <label>{$t('session.session.laps_label')}</label>
          <input type="number" bind:value={appState.session.laps} min="1" max="99" />
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- ⭐ SOUS-ONGLET 2 : COURSE & IA -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-race-ia'}>
  <div class="group">
    <div class="group-title">{$t('session.race_ia.group_opponents')}</div>
    <div class="row">
      <div class="field">
        <label>{$t('session.race_ia.opponents_count')}</label>
        <input type="number" bind:value={appState.session.opponentsCount} min="0" max="35" />
      </div>
      <div class="field">
        <label>{$t('session.race_ia.grid_type')}</label>
        <select bind:value={appState.session.gridType}>
          <option value="SAME_CAR">{$t('session.race_ia.grid_same_car')}</option>
          <option value="RANDOM">{$t('session.race_ia.grid_random')}</option>
          
          <optgroup label={$t('session.race_ia.grid_coherence_group').toUpperCase()}>
            <option value="SAME_CLASS">{$t('session.race_ia.grid_same_class')}</option>
            <option value="SAME_BRAND">{$t('session.race_ia.grid_same_brand')}</option>
            <option value="SAME_ERA_CLASS">{$t('session.race_ia.grid_same_era_class')}</option>
            <option value="SIMILAR_POWER_3">{$t('session.race_ia.grid_similar_power_3')}</option>
            <option value="SIMILAR_POWER_5">{$t('session.race_ia.grid_similar_power_5')}</option>
            <option value="SIMILAR_PWRATIO_3">{$t('session.race_ia.grid_similar_pwratio_3')}</option>
            <option value="SIMILAR_PWRATIO_5">{$t('session.race_ia.grid_similar_pwratio_5')}</option>
          </optgroup>
          
          <optgroup label={$t('session.race_ia.grid_restrictions_group')}>
            <option value="KUNOS_ONLY">{$t('session.race_ia.grid_kunos_only')}</option>
            <option value="MODS_ONLY">{$t('session.race_ia.grid_mods_only')}</option>
          </optgroup>
        </select>
      </div>
    </div>
    
    <div class="row">
      <div class="field">
        <label>{$t('session.race_ia.start_position')}</label>
        <select bind:value={appState.session.startingPositionMode}>
          <option value="LAST">{$t('session.race_ia.pos_last')}</option>
          <option value="RANDOM">{$t('session.race_ia.pos_random')}</option>
          <option value="FIRST">{$t('session.race_ia.pos_first')}</option>
          <option value="SECOND">{$t('session.race_ia.pos_second')}</option>
          <option value="THIRD">{$t('session.race_ia.pos_third')}</option>
          <option value="CUSTOM">{$t('session.race_ia.pos_custom')}</option>
        </select>
      </div>
      {#if appState.session.startingPositionMode === 'CUSTOM'}
        <div class="field">
          <label>{$t('session.race_ia.pos_exact')}</label>
          <input type="number" bind:value={appState.session.customStartPos} min="1" max="36" />
        </div>
      {/if}
      <div class="field">
        <label>{$t('session.race_ia.jump_start')}</label>
        <select bind:value={appState.session.jumpStartPenalty}>
          <option value={0}>{$t('session.race_ia.jump_disabled')}</option>
          <option value={1}>{$t('session.race_ia.jump_drivethrough')}</option>
          <option value={2}>{$t('session.race_ia.jump_stopgo')}</option>
        </select>
      </div>
    </div>
  </div>

  <div class="group">
    <div class="group-title">{$t('session.race_ia.group_ai')}</div>
    <div class="row">
      <div class="field">
        <label>{$t('session.race_ia.ai_level', { values: { level: appState.session.aiLevel, var: appState.session.aiLevelVar } })}</label>
        <input type="range" bind:value={appState.session.aiLevel} min="50" max="100" />
        <input type="range" bind:value={appState.session.aiLevelVar} min="0" max="20" />
      </div>
      <div class="field">
        <label>{$t('session.race_ia.ai_aggression', { values: { level: appState.session.aiAggression, var: appState.session.aiAggressionVar } })}</label>
        <input type="range" bind:value={appState.session.aiAggression} min="0" max="100" />
        <input type="range" bind:value={appState.session.aiAggressionVar} min="0" max="50" />
      </div>
    </div>
  </div>
</div>

<!-- ⭐ SOUS-ONGLET 3 : PILOTE & RÉALISME -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-pilot-realism'}>
  <div class="group">
    <div class="group-title">{$t('session.pilot.group_assists')}</div>
    <div class="row">
      <div class="field"><label class="check"><input type="checkbox" bind:checked={appState.session.assists.autoGearbox} /> {$t('session.pilot.auto_gearbox')}</label></div>
      <div class="field"><label class="check"><input type="checkbox" bind:checked={appState.session.assists.autoClutch} /> {$t('session.pilot.auto_clutch')}</label></div>
    </div>
    <div class="row">
      <div class="field"><label class="check"><input type="checkbox" bind:checked={appState.session.assists.autoblip} /> {$t('session.pilot.autoblip')}</label></div>
      <div class="field"><label class="check"><input type="checkbox" bind:checked={appState.session.assists.idealLine} /> {$t('session.pilot.ideal_line')}</label></div>
    </div>
    <div class="row">
      <div class="field">
        <label>{$t('session.pilot.tc')}</label>
        <select bind:value={appState.session.assists.tc}>
          <option value={0}>{$t('session.pilot.assist_off')}</option>
          <option value={1}>{$t('session.pilot.assist_factory')}</option>
          <option value={2}>{$t('session.pilot.assist_on')}</option>
        </select>
      </div>
      <div class="field">
        <label>{$t('session.pilot.abs')}</label>
        <select bind:value={appState.session.assists.abs}>
          <option value={0}>{$t('session.pilot.assist_off')}</option>
          <option value={1}>{$t('session.pilot.assist_factory')}</option>
          <option value={2}>{$t('session.pilot.assist_on')}</option>
        </select>
      </div>
    </div>
  </div>

  <div class="group">
    <div class="group-title">{$t('session.pilot.group_realism')}</div>
    <div class="row">
      <div class="field">
        <label>{$t('session.pilot.fuel', { values: { value: appState.session.assists.fuelRate } })}</label>
        <input type="range" bind:value={appState.session.assists.fuelRate} min="0" max="5" step="1" />
      </div>
      <div class="field">
        <label>{$t('session.pilot.damage', { values: { value: appState.session.assists.mechanicalDamage } })}</label>
        <input type="range" bind:value={appState.session.assists.mechanicalDamage} min="0" max="100" step="10" />
      </div>
    </div>
    <div class="row">
      <div class="field">
        <label>{$t('session.pilot.tyre_wear', { values: { value: appState.session.assists.tyreWear } })}</label>
        <input type="range" bind:value={appState.session.assists.tyreWear} min="0" max="5" step="1" />
      </div>
      <div class="field">
        <label>{$t('session.pilot.slipstream', { values: { value: appState.session.assists.slipstream } })}</label>
        <input type="range" bind:value={appState.session.assists.slipstream} min="0" max="5" step="1" />
      </div>
    </div>
  </div>
</div>

<!-- ⭐ SOUS-ONGLET 4 : PRESETS -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-presets'}>
  <div class="group">
    <div class="group-title">{$t('session.presets.group_save')}</div>
    <div class="row">
      <div class="field">
        <label>{$t('session.presets.name_label')}</label>
        <input type="text" bind:value={presetName} placeholder={$t('session.presets.name_placeholder')} />
      </div>
      <div class="field" style="flex: 0; min-width: auto; justify-content: flex-end;">
        <button class="filter-btn" onclick={savePreset} style="padding: 10px 22px; background-color: var(--accent); color: white; border-color: var(--accent); white-space: nowrap;">
          {$t('session.presets.save_btn')}
        </button>
      </div>
    </div>
  </div>

  <div class="group">
    <div class="group-title">{$t('session.presets.group_saved')}</div>
    {#if presets.length === 0}
      <div style="text-align: center; padding: 30px; color: var(--text-tertiary); font-style: italic; font-size: 0.85rem;">
        {$t('session.presets.empty')}
      </div>
    {:else}
      {#each presets as p}
        <div class="preset-row" style="display: flex; justify-content: space-between; align-items: center; padding: 12px; border-bottom: 1px solid var(--border);">
          <div>
            <div style="font-weight: 600;">{p.name}</div>
            <div style="font-size: 0.75rem; color: var(--text-tertiary);">{$t('session.history.race_laps', { values: { count: p.data.laps } })}</div>
          </div>
          <button class="filter-btn" onclick={() => appState.session = p.data}>{$t('session.presets.load')}</button>
        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- ⭐ SOUS-ONGLET 5 : HISTORIQUE -->
<div class="subtab-content" class:active={activeSubTab === 'subtab-history'}>
  <div class="group">
    <div class="group-title" style="display: flex; justify-content: space-between; align-items: center;">
      <span>{$t('session.history.group_title')}</span>
      {#if history.length > 0}
        <button class="filter-btn" onclick={clearHistory} style="font-size: 0.75rem; padding: 4px 10px;">{$t('session.history.clear_all')}</button>
      {/if}
    </div>

    {#if history.length === 0}
      <div style="text-align: center; padding: 30px; color: var(--text-tertiary); font-style: italic; font-size: 0.85rem;">
        {$t('session.history.empty')}
      </div>
    {:else}
      <div style="display: flex; flex-direction: column; gap: 10px; margin-top: 15px;">
        {#each history as h}
          {@const carImg = getCarImage(h.car_id, h.skin_id)}
          {@const carName = getCarName(h.car_id)}
          {@const trackName = getTrackName(h.track_id, h.layout_id)}
          
          <div style="display: flex; align-items: center; justify-content: space-between; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 6px; padding: 12px;">
            
            <!-- Infos -->
            <div style="display: flex; align-items: center; gap: 15px;">
              {#if carImg}
                <img src={carImg} style="width: 80px; height: 45px; object-fit: cover; border-radius: 4px; border: 1px solid var(--border-subtle);" alt={$t('sidebar.car')} />
              {:else}
                <div style="width: 80px; height: 45px; background: #000; border-radius: 4px;"></div>
              {/if}
              
              <div style="display: flex; flex-direction: column; gap: 4px;">
                <div style="font-weight: 600; font-size: 0.9rem; color: var(--text-primary);">
                  {carName} <span style="color: var(--text-tertiary); font-weight: normal;">•</span> {trackName}
                </div>
                <div style="font-size: 0.75rem; color: var(--text-secondary); display: flex; align-items: center; gap: 10px;">
                  <span>📅 {formatDate(h.date)}</span>
                  <span>{h.session_data.type === 'RACE' ? $t('session.history.race_laps', { values: { count: h.session_data.laps } }) : $t(`session.types.${h.session_data.type.toLowerCase()}`)}</span>
                  <span>{$t('session.history.ai_count', { values: { count: h.session_data.opponentsCount } })}</span>
                  <span>🕒 {String(h.session_data.weather.startHour).padStart(2, '0')}:{String(h.session_data.weather.startMinute).padStart(2, '0')}</span>
                  <span>{getWeatherIcon(h.weather_summary)} {translateWeatherName(h.weather_summary, h.weather_summary, $locale)}</span>
                </div>
              </div>
            </div>

            <!-- Boutons -->
            <div style="display: flex; gap: 8px;">
              <button class="filter-btn" onclick={() => launchFromHistory(h)} style="padding: 6px 14px; background-color: var(--accent); color: white; border-color: var(--accent); display: flex; align-items: center; gap: 6px;">
                {$t('session.history.launch')}
              </button>
              <button class="filter-btn" onclick={() => loadConfigFromHistory(h)} style="padding: 6px 14px;">
                {$t('session.history.load')}
              </button>
              <button class="filter-btn" onclick={(e) => deleteEntry(h.id, e)} style="padding: 6px 10px; color: var(--danger); border-color: var(--border);">
                🗑️
              </button>
            </div>

          </div>
        {/each}
      </div>
    {/if}
  </div>
</div>