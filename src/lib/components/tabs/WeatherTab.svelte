<script lang="ts">
  import { onMount } from 'svelte';
  import { invoke } from '@tauri-apps/api/core';
  import { appState, saveSession } from '$lib/stores/appState.svelte.js';
  import { getWeatherIcon, translateWeatherName } from '$lib/utils/weatherUtils.js';
  import { t, locale } from 'svelte-i18n';
  import { alert, confirm } from '$lib/stores/dialog.svelte'; // ⭐ Import des pop-ups QARL

  let availableWeathers = $state<{ id: string, name: string }[]>([]);

  // ⭐ Liste statique exacte des météos CSP Schedule (d'après settings.ini)
  const CSP_SCHEDULE_WEATHERS = [
    { index: 15, id: '15_clear' }, { index: 16, id: '16_few_clouds' },
    { index: 17, id: '17_scattered_clouds' }, { index: 18, id: '18_broken_clouds' },
    { index: 19, id: '19_overcast_clouds' }, { index: 21, id: '21_mist' },
    { index: 20, id: '20_fog' }, { index: 3, id: '3_light_drizzle' },
    { index: 4, id: '4_drizzle' }, { index: 5, id: '5_heavy_drizzle' },
    { index: 6, id: '6_light_rain' }, { index: 7, id: '7_rain' },
    { index: 8, id: '8_heavy_rain' }, { index: 0, id: '0_light_thunderstorm' },
    { index: 1, id: '1_thunderstorm' }, { index: 2, id: '2_heavy_thunderstorm' },
    { index: 12, id: '12_light_sleet' }, { index: 13, id: '13_sleet' },
    { index: 14, id: '14_heavy_sleet' }, { index: 9, id: '9_light_snow' },
    { index: 10, id: '10_snow' }, { index: 11, id: '11_heavy_snow' },
    { index: 27, id: '27_tornado' }, { index: 28, id: '28_hurricane' },
    { index: 22, id: '22_smoke' }, { index: 23, id: '23_haze' },
    { index: 24, id: '24_sand' }, { index: 25, id: '25_dust' },
    { index: 26, id: '26_squalls' }, { index: 29, id: '29_cold' },
    { index: 30, id: '30_hot' }, { index: 31, id: '31_windy' },
    { index: 32, id: '32_hail' }
  ];
  let availablePlans = $state<{ id: string, name: string, start_time?: string, steps?: any[], loop_plan?: boolean, time_mult?: number }[]>([]);

  // ⭐ Fonction pour traduire l'index Pure (15, 16...) en icône
  function getIconFromIndex(index: number) {
    const map: Record<number, string> = {
      1: '☀️', 2: '☀️', 3: '🌤️', 4: '⛅', 5: '💨', 6: '☁️', 7: '☁️', 8: '🌫️', 9: '🌫️', 10: '🌫️',
      14: '🌧️', 15: '🌧️', 17: '🌧️', 18: '🌧️', 19: '🌧️', 0: '⛈️', 1: '⛈️', 2: '⛈️'
    };
    return map[index] || '☁️';
  }

  // ⭐ Traduction intelligente : on cherche la vraie météo dans la liste chargée
  function getWeatherNameFromIndex(index: number) {
    // On cherche d'abord une météo Pure ou Sol, sinon une Kunos
    const match = availableWeathers.find(w => w.id.startsWith(`pure_${index}_`) || w.id.startsWith(`sol_${index}_`)) 
               || availableWeathers.find(w => w.id.startsWith(`${index}_`));
               
    if (match) {
      return translateWeatherName(match.id, match.name);
    }
    
    // Fallback si jamais on ne trouve pas
    const names: Record<number, string> = {
      1: 'Peu nuageux', 2: 'Dégagé', 3: 'Nuages épars', 4: 'Nuageux', 5: 'Venteux',
      6: 'Nuages fragmentés', 7: 'Couvert', 8: 'Brumeux', 9: 'Brume', 10: 'Brouillard',
      14: 'Bruine légère', 15: 'Bruine', 17: 'Pluie légère', 18: 'Pluie', 19: 'Forte pluie', 0: 'Orage'
    };
    return names[index] || $t('common.unknown');
  }
  
  // ⭐ Trouve le vrai ID complet (ex: "sol_25" -> "sol_25_heavy_rain")
  function resolveWeatherId(rawId: string) {
    if (!rawId) return 'clear';
    // Si l'ID est déjà complet, on le garde
    if (availableWeathers.some(w => w.id === rawId)) return rawId;
    
    // Sinon, on cherche dans la liste des météos installées
    const match = availableWeathers.find(w => w.id.startsWith(rawId + "_"));
    return match ? match.id : rawId;
  }

  async function refreshPlans() {
    if (!appState.acPath) return;
    try {
      availablePlans = await invoke<{ id: string, name: string, start_time?: string }[]>('get_pure_plans', { acPath: appState.acPath });
    } catch (e) {
      console.error("Error refreshing plans", e);
    }
  }

  async function deletePlan(planId: string) {
    // ⭐ Ajout du await
    if (!await confirm($t('weather.existing.confirm_delete'), $t('common.delete'), 'warning')) return;
    try {
      await invoke('delete_pure_plan', { acPath: appState.acPath, planId });
      refreshPlans();
    } catch (e) {
      alert($t('weather.errors.delete_error', { values: { error: e } }));
    }
  }

  // ⭐ Charger un plan existant et extraire ses étapes pour la Sidebar
  async function loadExistingPlan(planId: string) {
    appState.session.weather.existingPlan = planId;
    try {
      const planData = await invoke<any>('load_pure_plan', { acPath: appState.acPath, planId });
      if (planData && planData.data && planData.data.container) {
        
        // ⭐ NOUVEAU : On sauvegarde les paramètres de contrôle pour la Sidebar
        if (planData.data.control) {
          appState.session.weather.existingPlanLoop = planData.data.control.loop || false;
          appState.session.weather.existingPlanTimeMult = planData.data.control.timemulti || 1;
        }

        // On extrait les étapes pour la Sidebar
        appState.session.weather.existingPlanSteps = planData.data.container.map((c: any) => {
          let rawId = c.data.weather.__qarl_weather_id;
          
          // Si c'est un vieux plan sans notre ID magique, on cherche par l'index (ex: 25)
          if (!rawId && c.data.weather.index !== undefined) {
            const index = c.data.weather.index;
            const match = availableWeathers.find(w => 
              w.id.startsWith(`sol_${index}_`) || w.id.startsWith(`pure_${index}_`) || 
              w.id.startsWith(`sfx_${index}_`) || w.id.startsWith(`${index}_`)
            );
            if (match) rawId = match.id;
          }

          // Fallback ultime pour éviter d'avoir un vide
          if (!rawId) rawId = availableWeathers[0]?.id || '3_clear';

          return {
            weather: rawId, 
            duration: Math.round(c.data.duration / 60)
          };
        });
        saveSession();
      }
    } catch (e) {
      console.warn("Unable to read plan details:", e);
    }
  }

  // ⭐ Éditer un plan existant (le transformer en plan personnalisé)
  async function editExistingPlan(planId: string) {
    try {
      const planData = await invoke<any>('load_pure_plan', { acPath: appState.acPath, planId });
      if (planData && planData.data && planData.data.container) {
        const steps = planData.data.container.map((c: any) => {
          let rawId = c.data.weather.__qarl_weather_id;
          
          // Si c'est un vieux plan, on cherche par l'index
          if (!rawId && c.data.weather.index !== undefined) {
            const index = c.data.weather.index;
            const match = availableWeathers.find(w => 
              w.id.startsWith(`sol_${index}_`) || w.id.startsWith(`pure_${index}_`) || 
              w.id.startsWith(`sfx_${index}_`) || w.id.startsWith(`${index}_`)
            );
            if (match) rawId = match.id;
          }

          if (!rawId) rawId = availableWeathers[0]?.id || '3_clear';

          return {
            weather: rawId, 
            duration: Math.round(c.data.duration / 60)
          };
        });

        // ⭐ NETTOYAGE DU NOM : On enlève Timed/, Custom_, .json et les underscores
        let cleanPlanName = planData.planName.split('/').pop().replace('.json', '');
        if (cleanPlanName.startsWith('QARL_')) cleanPlanName = cleanPlanName.substring(5);
        cleanPlanName = cleanPlanName.replace(/_/g, ' ');

        // On bascule sur l'onglet "Créer un plan"
        pureSubTab = 'custom';
        appState.session.weather.customPlanName = cleanPlanName + ' (Copie)';
        appState.session.weather.customPlanSteps = steps;
        
        alert($t('weather.errors.edit_loaded'));
      }
    } catch (e) {
      alert($t('weather.errors.edit_error', { values: { error: e } }));
    }
  }
  let isSavingPlan = $state(false);
  let pureSubTab = $state<'existing' | 'custom'>('existing'); // ⭐ Gère l'affichage interne
  
  // ⭐ NOUVEAU : Détection de ce qui est installé
  let capabilities = $state({ hasCsp: false, hasPure: false, hasSol: false });

  // Auto-save à chaque changement (même pattern que SessionTab)
  $effect(() => {
    JSON.stringify(appState.session);
    saveSession();
  });

  // ⭐ On extrait la logique dans une fonction réutilisable
  async function loadWeatherData() {
    if (!appState.acPath) return;
    
    try {
      // 1. Vérifier les capacités (CSP / Pure / Sol)
      capabilities = await invoke('check_ac_capabilities', { acPath: appState.acPath });

      // 2. Charger les météos
      availableWeathers = await invoke<{ id: string, name: string }[]>('get_ac_weathers', { acPath: appState.acPath });
      
      // 3. Charger les plans Pure (si existants)
      availablePlans = await invoke<{ id: string, name: string }[]>('get_pure_plans', { acPath: appState.acPath });

      // 4. Charger la config pureCtrl sauvegardée
      const pureCtrlSettings = await invoke<{
        autostart: boolean, last_used: boolean, live: boolean, start_wetness: number, start_puddles: number
      }>('read_pure_ctrl', { documentsPath: appState.documentsPath });

      if (pureCtrlSettings) {
        appState.session.weather.pureCtrl = {
          autostart: pureCtrlSettings.autostart,
          lastUsed: pureCtrlSettings.last_used,
          live: pureCtrlSettings.live,
          startWetness: pureCtrlSettings.start_wetness,
          startPuddles: pureCtrlSettings.start_puddles
        };
      }
    } catch (e) {
      console.error("Error loading weather/plans:", e);
    }
  }

  // ⭐ NOUVEAU : On recharge les données dès que le chemin AC est disponible
  // (utile après le SetupWizard au premier lancement)
  $effect(() => {
    if (appState.acPath) {
      loadWeatherData();
    }
  });

  // On garde l'appel au montage pour les lancements suivants (où le chemin est déjà là)
  onMount(() => {
    if (appState.acPath) {
      loadWeatherData();
    }
  });

  // ⭐ Sauvegarde de pureCtrl dans le disque AC
  async function savePureCtrlToDisk() {
    if (!appState.acPath) return;
    try {
      await invoke('write_pure_ctrl', {
        documentsPath: appState.documentsPath,
        settings: {
          autostart: appState.session.weather.pureCtrl.autostart,
          last_used: appState.session.weather.pureCtrl.lastUsed,
          live: appState.session.weather.pureCtrl.live,
          start_wetness: appState.session.weather.pureCtrl.startWetness,
          start_puddles: appState.session.weather.pureCtrl.startPuddles
        }
      });
    } catch (e) {
      console.error("Error saving pureCtrl:", e);
    }
  }

  // ⭐ Toggles pureCtrl mutuellement exclusifs
  function togglePureCtrl(field: 'autostart' | 'lastUsed' | 'live') {
    const ctrl = appState.session.weather.pureCtrl;
    
    let wasOn = false;
    if (field === 'autostart') wasOn = ctrl.autostart;
    else if (field === 'lastUsed') wasOn = ctrl.lastUsed;
    else if (field === 'live') wasOn = ctrl.live;

    // Tout éteindre
    ctrl.autostart = false;
    ctrl.lastUsed  = false;
    ctrl.live      = false;

    // Rallumer si on était en train d'activer
    if (!wasOn) {
      if (field === 'autostart') ctrl.autostart = true;
      else if (field === 'lastUsed') ctrl.lastUsed = true;
      else if (field === 'live') ctrl.live = true;
    }
    
    savePureCtrlToDisk();
  }

  // ⭐ Forcer pureCtrl.autostart quand on passe en mode plan (comportement QACRL)
  function applyModeToPureCtrl() {
    const mode = appState.session.weather.mode;
    const ctrl = appState.session.weather.pureCtrl;
    if (mode === 'pure_plan') {
      ctrl.autostart = true;
      ctrl.lastUsed  = false;
      ctrl.live      = false;
    } else {
      ctrl.autostart = false;
      ctrl.lastUsed  = false;
      ctrl.live      = false;
    }
    savePureCtrlToDisk();
  }

  // ⭐ Sauvegarde du plan personnalisé via Rust
  async function saveCustomPlan() {
    if (!appState.acPath) {
      alert($t('weather.errors.need_path'));
      return;
    }
    if (!appState.session.weather.customPlanName.trim() || appState.session.weather.customPlanName.trim().length < 3) {
      alert($t('weather.errors.name_too_short'));
      return;
    }
    if (appState.session.weather.customPlanSteps.length === 0) {
      alert($t('weather.errors.need_steps'));
      return;
    }

    isSavingPlan = true;
    try {
      const hour = String(appState.session.weather.startHour).padStart(2, '0');
      const minute = String(appState.session.weather.startMinute).padStart(2, '0');

      const result = await invoke<string>('save_custom_pure_plan', {
        acPath: appState.acPath,
        planName: appState.session.weather.customPlanName,
        steps: appState.session.weather.customPlanSteps.map(s => ({
          weather: s.weather,
          duration: parseInt(String(s.duration)) || 10
        })),
        sessionStartTime: `${hour}:${minute}`,
        timeMult: appState.session.weather.timeMultiplier,
        loopPlan: appState.session.weather.customPlanLoop
      });

      // On rafraîchit la liste des plans pour que le nouveau apparaisse
      availablePlans = await invoke<{ id: string, name: string }[]>('get_pure_plans', { acPath: appState.acPath });

      // ⭐ On bascule sur l'onglet "Mes plans" et on l'active
      pureSubTab = 'existing';
      applyModeToPureCtrl();
      
      // On charge les détails pour la Sidebar
      await loadExistingPlan(result);

      alert($t('weather.errors.save_success', { values: { path: result } }));
    } catch (e) {
      alert($t('weather.errors.save_error', { values: { error: e } }));
    } finally {
      isSavingPlan = false;
    }
  }

  // ⭐ Plan personnalisé : ajouter / retirer une étape
  function addStep() {
    const defaultW = availableWeathers.length > 0 ? availableWeathers[0].id : '3_clear';
    appState.session.weather.customPlanSteps.push({
      weather: defaultW,
      duration: 10
    });
  }

  function removeStep(index: number) {
    appState.session.weather.customPlanSteps.splice(index, 1);
  }

  // Durée totale du plan perso
  let totalDuration = $derived(
    appState.session.weather.customPlanSteps.reduce(
      (acc, s) => acc + (parseInt(String(s.duration)) || 0),
      0
    )
  );

  // ⭐ Tris séparés pour les listes déroulantes
  // ⭐ On définit l'ordre logique des météos Kunos
  const KUNOS_WEATHER_ORDER = [
    'clear', 'mid_clear', 'light_clouds', 'mid_clouds', 'heavy_clouds', 'light_fog', 'heavy_fog'
  ];

  // ⭐ Fonction pour extraire l'heure de début d'un plan JSON
  function getPlanStartTime(planId: string) {
    // On cherche le plan dans la liste pour avoir ses infos
    // (Cette fonction est simplifiée, l'idéal serait de lire le fichier JSON mais pour l'UI on peut se contenter de l'heure)
    return ''; 
  }

  let kunosWeathers = $derived(
    (availableWeathers || [])
      .filter(w => /^\d/.test(w.id))
      .sort((a, b) => {
        const nameA = a.id.replace(/^\d+_/, '').toLowerCase();
        const nameB = b.id.replace(/^\d+_/, '').toLowerCase();
        const idxA = KUNOS_WEATHER_ORDER.indexOf(nameA);
        const idxB = KUNOS_WEATHER_ORDER.indexOf(nameB);
        if (idxA !== -1 && idxB !== -1) return idxA - idxB;
        return a.id.localeCompare(b.id, undefined, { numeric: true });
      })
  );
  
  // ⭐ Sécurité : on vérifie que availableWeathers existe bien avant de filtrer
  let solWeathers = $derived(
    (availableWeathers || [])
      .filter(w => /^(sol|pure|sfx)/i.test(w.id))
      .sort((a, b) => a.id.localeCompare(b.id, undefined, { numeric: true }))
  );
  
  let otherWeathers = $derived(
    (availableWeathers || [])
      .filter(w => !/^\d/.test(w.id) && !/^(sol|pure|sfx)/i.test(w.id))
      .sort((a, b) => a.id.localeCompare(b.id))
  );

  const HOURS   = Array.from({ length: 24 }, (_, i) => i);
  const MINUTES = Array.from({ length: 60 }, (_, i) => i);

  // ⭐ PRESETS SCHEDULE
  let schedulePresets = $state<string[]>([]);
  let newSchedulePresetName = $state('');
  let selectedSchedulePreset = $state('');

  async function loadSchedulePresets() {
    try {
      schedulePresets = await invoke<string[]>('list_schedule_presets', { documentsPath: appState.documentsPath });
    } catch (e) {
      console.warn("Error loading schedule presets", e);
    }
  }

  async function saveCurrentSchedule() {
    if (!newSchedulePresetName.trim()) return;
    try {
      await invoke('save_schedule_preset', {
        documentsPath: appState.documentsPath,
        name: newSchedulePresetName,
        data: {
          slots: appState.session.weather.scheduleSlots,
          time_per_slot: appState.session.weather.scheduleTimePerSlot,
          random: appState.session.weather.scheduleRandom,
          slot_values: appState.session.weather.scheduleSlotValues
        }
      });
      newSchedulePresetName = '';
      await loadSchedulePresets();
      alert($t('weather.schedule_presets.success_save'));
    } catch (e) {
      alert($t('common.error') + " : " + e);
    }
  }

  async function loadSelectedSchedule() {
    if (!selectedSchedulePreset) return;
    try {
      const data = await invoke<any>('load_schedule_preset', { documentsPath: appState.documentsPath, name: selectedSchedulePreset });
      appState.session.weather.scheduleSlots = data.slots;
      appState.session.weather.scheduleTimePerSlot = data.time_per_slot;
      appState.session.weather.scheduleRandom = data.random;
      appState.session.weather.scheduleSlotValues = data.slot_values;
      alert($t('weather.schedule_presets.success_load'));
    } catch (e) {
      alert($t('common.error') + " : " + e);
    }
  }

  async function deleteSelectedSchedule() {
    // ⭐ Ajout du await
    if (!selectedSchedulePreset || !await confirm($t('weather.schedule_presets.confirm_delete'), $t('common.delete'), 'warning')) return;
    try {
      await invoke('delete_schedule_preset', { documentsPath: appState.documentsPath, name: selectedSchedulePreset });
      selectedSchedulePreset = '';
      await loadSchedulePresets();
    } catch (e) {
      alert($t('common.error') + " : " + e);
    }
  }

  $effect(() => {
    if (appState.session.weather.mode === 'schedule' && schedulePresets.length === 0) {
      loadSchedulePresets();
    }
  });

  // ⭐ Sécurité : on s'assure que le tableau des slots a toujours la bonne taille
  $effect(() => {
    const target = appState.session.weather.scheduleSlots || 3;
    const current = appState.session.weather.scheduleSlotValues;
    if (current.length < target) {
      for (let i = current.length; i < target; i++) {
        current.push(15); // 15 = Clear par défaut
      }
    } else if (current.length > target) {
      current.length = target;
    }
  });
</script>

<!-- ⭐ Heure et progression du temps -->
<div class="group">
  <div class="group-title">{$t('weather.group_time')}</div>
  <div class="row">
    <div class="field">
      <label>{$t('weather.start_hour')}</label>
      <div class="time-picker">
        <select bind:value={appState.session.weather.startHour}>
          {#each HOURS as h}
            <option value={h}>{String(h).padStart(2, '0')}</option>
          {/each}
        </select>
        <span class="sep">:</span>
        <select bind:value={appState.session.weather.startMinute}>
          {#each MINUTES as m}
            <option value={m}>{String(m).padStart(2, '0')}</option>
          {/each}
        </select>
      </div>
    </div>
    <div class="field">
      <label>{$t('weather.time_mult')}</label>
      <input type="number" bind:value={appState.session.weather.timeMultiplier} min="1" max="24" />
    </div>
  </div>
</div>

<!-- ⭐ Conditions météo -->
<div class="group">
  <div class="group-title" style="display: flex; justify-content: space-between; align-items: center;">
    <span>{$t('weather.group_conditions')}</span>
    <button class="filter-btn" onclick={() => { loadWeatherData(); refreshPlans(); }} style="padding: 2px 8px; font-size: 0.7rem; height: auto;" title="Rafraîchir les météos et les plans">
      🔄 Rafraîchir
    </button>
  </div>
  <div class="row">
    <div class="field">
      <label>{$t('weather.mode_label')}</label>
      <select bind:value={appState.session.weather.mode} onchange={applyModeToPureCtrl}>
        <option value="static">{$t('weather.mode_static')}</option>
        <option value="pure_plan">Plans Pure</option>
      </select>
    </div>
  </div>

  {#if appState.session.weather.mode === 'static'}
    <div class="row">
      <div class="field">
        <label>{$t('weather.weather_label')}</label>
        <select bind:value={appState.session.weather.staticWeather}>
          {#if availableWeathers.length === 0}
            <option value="3_clear">{$t('weather.loading_weathers')}</option>
          {:else}
            <optgroup label={$t('weather.group_kunos')}>
              {#each kunosWeathers as weather}
                <option value={weather.id}>{getWeatherIcon(weather.id)} {translateWeatherName(weather.id, weather.name, $locale)}</option>
              {/each}
            </optgroup>
            {#if solWeathers.length > 0}
              <optgroup label={$t('weather.group_sol_pure')}>
                {#each solWeathers as weather}
                  <option value={weather.id}>{getWeatherIcon(weather.id)} {translateWeatherName(weather.id, weather.name, $locale)}</option>
                {/each}
              </optgroup>
            {/if}
            {#if otherWeathers.length > 0}
              <optgroup label={$t('weather.group_other')}>
                {#each otherWeathers as weather}
                  <option value={weather.id}>{getWeatherIcon(weather.id)} {translateWeatherName(weather.id, weather.name, $locale)}</option>
                {/each}
              </optgroup>
            {/if}
          {/if}
        </select>
      </div>
    </div>
  {/if}

  {#if false} <!-- ⭐ SCHEDULE DÉSACTIVÉ -->
    <div class="row">
      <div class="field">
        <label>{$t('weather.schedule.slots_count')}</label>
        <input type="number" bind:value={appState.session.weather.scheduleSlots} min="1" max="5" />
      </div>
      <div class="field">
        <label>{$t('weather.schedule.time_per_slot')}</label>
        <input type="number" bind:value={appState.session.weather.scheduleTimePerSlot} min="1" max="30" />
      </div>
      <div class="field">
        <label class="check" style="margin-top: 28px;">
          <input type="checkbox" bind:checked={appState.session.weather.scheduleRandom} />
          {$t('weather.schedule.random')}
        </label>
      </div>
    </div>

    {#each Array(appState.session.weather.scheduleSlots || 3) as _, i}
      <div class="row" style="align-items: center; gap: 10px; margin-top: 8px;">
        <label style="min-width: 60px;">{$t('weather.schedule.slot_label', { values: { n: i + 1 } })}</label>
        <select bind:value={appState.session.weather.scheduleSlotValues[i]} style="flex: 1;">
          {#each CSP_SCHEDULE_WEATHERS as weather}
            <option value={weather.index}>
              {getWeatherIcon(weather.id)} {translateWeatherName(weather.id, weather.id, $locale)}
            </option>
          {/each}
        </select>
      </div>
    {/each}

    <!-- Gestionnaire de presets de Schedule -->
    <div style="margin-top: 20px; padding: 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 6px;">
      <div style="font-size: 0.75rem; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; margin-bottom: 12px;">
        {$t('weather.schedule_presets.title')}
      </div>
      
      <div style="display: flex; gap: 10px; align-items: center; flex-wrap: wrap;">
        <input type="text" bind:value={newSchedulePresetName} placeholder={$t('weather.schedule_presets.placeholder')} style="flex: 1; min-width: 150px;" />
        <button class="filter-btn" onclick={saveCurrentSchedule} disabled={!newSchedulePresetName.trim()} style="padding: 8px 16px; background: var(--accent); color: white; border-color: var(--accent);">
          {$t('weather.schedule_presets.save_btn')}
        </button>
        <div style="width: 1px; height: 20px; background: var(--border); margin: 0 4px;"></div>
        <select bind:value={selectedSchedulePreset} style="flex: 1; min-width: 150px; padding: 8px 12px;">
          <option value="">{$t('weather.schedule_presets.empty')}</option>
          {#each schedulePresets as p}
            <option value={p}>{p.replace(/_/g, ' ')}</option>
          {/each}
        </select>
        <button class="filter-btn" onclick={loadSelectedSchedule} disabled={!selectedSchedulePreset} style="padding: 8px 16px;">
          {$t('weather.schedule_presets.load_btn')}
        </button>
        <button class="filter-btn" onclick={deleteSelectedSchedule} disabled={!selectedSchedulePreset} style="padding: 8px 12px; color: var(--danger); border-color: var(--border);">
          🗑️
        </button>
      </div>
    </div>
  {/if}
</div>

<!-- ⭐ LE NOUVEAU GROUPE UNIFIÉ POUR PLANS PURE -->
{#if appState.session.weather.mode === 'pure_plan'}
<div class="group">
  <div class="group-title">⚙️ Plans Pure & Paramètres</div>

  <!-- 1. Paramètres PureCtrl -->
  <div class="row" style="background: var(--bg-elevated); padding: 14px; border-radius: 8px; border: 1px solid var(--border-subtle);">
    <div class="row">
      <div class="field">
        <label class="check">
          <input type="checkbox" checked={appState.session.weather.pureCtrl.autostart} onclick={() => togglePureCtrl('autostart')} />
          {$t('weather.pure_ctrl.autostart')}
        </label>
      </div>
      <div class="field">
        <label class="check">
          <input type="checkbox" checked={appState.session.weather.pureCtrl.lastUsed} onclick={() => togglePureCtrl('lastUsed')} />
          {$t('weather.pure_ctrl.last_used')}
        </label>
      </div>
      <div class="field">
        <label class="check">
          <input type="checkbox" checked={appState.session.weather.pureCtrl.live} onclick={() => togglePureCtrl('live')} />
          {$t('weather.pure_ctrl.live')}
        </label>
      </div>
    </div>
    <div class="row" style="margin-top: 14px;">
      <div class="field">
        <label>{$t('weather.pure_ctrl.start_wetness')}</label>
        <input type="number" bind:value={appState.session.weather.pureCtrl.startWetness} min="1" max="10" />
      </div>
      <div class="field">
        <label>{$t('weather.pure_ctrl.start_puddles')}</label>
        <input type="number" bind:value={appState.session.weather.pureCtrl.startPuddles} min="1" max="10" />
      </div>
    </div>
  </div>

  <!-- 2. Séparateur / Onglets internes -->
  <div style="display: flex; gap: 10px; margin-top: 10px; margin-bottom: 10px; border-bottom: 1px solid var(--border); padding-bottom: 10px;">
    <button class="filter-btn" class:active={pureSubTab === 'existing'} onclick={() => pureSubTab = 'existing'}>
      📁 Mes plans existants
    </button>
    <button class="filter-btn" class:active={pureSubTab === 'custom'} onclick={() => pureSubTab = 'custom'}>
      ➕ Créer un plan
    </button>
  </div>

  <!-- 3. Affichage conditionnel -->
  {#if pureSubTab === 'existing'}
    <div class="row" style="align-items: center; justify-content: space-between; margin-bottom: 10px;">
      <div class="field">
        <label class="check" style="margin-top: 0;">
          <input type="checkbox" bind:checked={appState.session.weather.existingPlanRandom} />
          {$t('weather.existing.random_pick')}
        </label>
      </div>
      <button class="filter-btn" onclick={refreshPlans} style="padding: 6px 14px; display: flex; align-items: center; gap: 6px;">
        {$t('weather.existing.refresh')}
      </button>
    </div>

    {#if availablePlans.length === 0}
      <div style="text-align: center; padding: 20px; color: var(--text-tertiary); font-style: italic; font-size: 0.85rem;">
        {$t('weather.existing.empty')}
      </div>
    {:else}
      <div style="display: flex; flex-direction: column; gap: 8px; max-height: 250px; overflow-y: auto; padding-right: 4px;">
        {#each availablePlans as plan}
          {@const isSelected = appState.session.weather.existingPlan === plan.id}
          
          <div style="display: flex; align-items: center; justify-content: space-between; background: {isSelected ? 'var(--accent-muted)' : 'var(--bg-elevated)'}; border: 1px solid {isSelected ? 'var(--accent)' : 'var(--border)'}; border-radius: 6px; padding: 10px 14px;">
            
            <div style="display: flex; flex-direction: column; gap: 6px; flex: 1;">
              <div style="font-weight: 600; font-size: 0.9rem; color: var(--text-primary); display: flex; align-items: center; gap: 10px;">
                {plan.name}
                {#if plan.start_time}
                  <span style="font-size: 0.75rem; font-weight: normal; color: var(--accent); background: var(--accent-muted); padding: 2px 6px; border-radius: 4px;">{$t('weather.existing.start_prefix', { values: { time: plan.start_time } })}</span>
                {/if}
                {#if plan.time_mult && plan.time_mult > 1}
                  <span style="font-size: 0.75rem; font-weight: normal; color: var(--text-secondary); background: var(--bg-panel); border: 1px solid var(--border-subtle); padding: 2px 6px; border-radius: 4px;">⏱️ x{plan.time_mult}</span>
                {/if}
                {#if plan.loop_plan}
                  <span style="font-size: 0.75rem; font-weight: normal; color: var(--text-secondary); background: var(--bg-panel); border: 1px solid var(--border-subtle); padding: 2px 6px; border-radius: 4px;">🔁 {$t('weather.custom.loop')}</span>
                {/if}
              </div>
              
              {#if plan.steps && plan.steps.length > 0}
                <div style="display: flex; gap: 4px; flex-wrap: wrap;">
                  {#each plan.steps as step}
                    {@const index = step.weather_index}
                    {@const match = availableWeathers.find(w => w.id === step.weather_id) 
                                || availableWeathers.find(w => w.id.startsWith(step.weather_id + "_"))
                                || (index !== undefined ? availableWeathers.find(w => 
                                     w.id.startsWith(`sol_${index}_`) || w.id.startsWith(`pure_${index}_`) || 
                                     w.id.startsWith(`sfx_${index}_`) || w.id.startsWith(`${index}_`)
                                   ) : null)}
                    {@const fallback = availableWeathers.length > 0 ? availableWeathers[0] : null}
                    {@const realId = match ? match.id : (fallback ? fallback.id : step.weather_id)}
                    {@const realName = match ? match.name : (fallback ? fallback.name : getWeatherNameFromIndex(step.weather_index))}
                    
                    <span style="display: inline-flex; align-items: center; gap: 4px; font-size: 0.72rem; background: var(--bg-panel); padding: 3px 6px; border-radius: 4px; border: 1px solid var(--border-subtle);">
                      {getWeatherIcon(realId)} {translateWeatherName(realId, realName, $locale)} <span style="color: var(--text-tertiary);">{step.duration} min</span>
                    </span>
                  {/each}
                </div>
              {/if}
            </div>

            <div style="display: flex; gap: 8px;">
              <button class="filter-btn" onclick={() => loadExistingPlan(plan.id)} style="padding: 6px 14px; {isSelected ? 'background-color: var(--accent); color: white; border-color: var(--accent);' : ''}">
                {isSelected ? $t('weather.existing.loaded') : $t('weather.existing.load_btn')}
              </button>
              <button class="filter-btn" onclick={() => editExistingPlan(plan.id)} style="padding: 6px 10px;" title={$t('weather.existing.edit_btn_title')}>
                ✏️
              </button>
              <button class="filter-btn" onclick={() => deletePlan(plan.id)} style="padding: 6px 10px; color: var(--danger); border-color: var(--border);" title={$t('weather.existing.delete_btn_title')}>
                🗑️
              </button>
            </div>
          </div>
        {/each}
      </div>
    {/if}

  {:else if pureSubTab === 'custom'}
    <div class="row">
      <div class="field">
        <label>{$t('weather.custom.plan_name')}</label>
        <input type="text" bind:value={appState.session.weather.customPlanName} placeholder={$t('weather.custom.plan_name_placeholder')} />
      </div>
      <div class="field">
        <label class="check" style="margin-top: 28px;">
          <input type="checkbox" bind:checked={appState.session.weather.customPlanLoop} />
          {$t('weather.custom.loop')}
        </label>
      </div>
    </div>

    {#if appState.session.weather.customPlanSteps.length === 0}
      <div class="weather-empty">
        {$t('weather.custom.empty')}
      </div>
    {:else}
      {#each appState.session.weather.customPlanSteps as step, index}
        <div class="weather-step">
          <div class="weather-step-index">{index + 1}</div>
          <select bind:value={step.weather}>
            {#if availableWeathers.length === 0}
              <option value="3_clear">{$t('weather.loading_weathers')}</option>
            {:else}
              <optgroup label={$t('weather.group_kunos')}>
                {#each kunosWeathers as weather}
                  <option value={weather.id}>{getWeatherIcon(weather.id)} {translateWeatherName(weather.id, weather.name, $locale)}</option>
                {/each}
              </optgroup>
              {#if solWeathers.length > 0}
                <optgroup label={$t('weather.group_sol_pure')}>
                  {#each solWeathers as weather}
                    <option value={weather.id}>{getWeatherIcon(weather.id)} {translateWeatherName(weather.id, weather.name, $locale)}</option>
                  {/each}
                </optgroup>
              {/if}
              {#if otherWeathers.length > 0}
                <optgroup label={$t('weather.group_other')}>
                  {#each otherWeathers as weather}
                    <option value={weather.id}>{getWeatherIcon(weather.id)} {translateWeatherName(weather.id, weather.name, $locale)}</option>
                  {/each}
                </optgroup>
              {/if}
            {/if}
          </select>
          <input type="number" bind:value={step.duration} min="1" max="600" placeholder={$t('weather.custom.duration_placeholder')} title={$t('weather.custom.duration_title')} />
          <button class="step-remove" onclick={() => removeStep(index)} title={$t('weather.custom.remove_title')}>×</button>
        </div>
      {/each}
    {/if}

    <div style="display: flex; gap: 10px; align-items: center; flex-wrap: wrap; margin-top: 10px;">
      <button class="filter-btn" onclick={addStep} style="padding: 10px 20px; background-color: var(--accent); color: white; border-color: var(--accent);">
        {$t('weather.custom.add_weather')}
      </button>
      <button class="filter-btn" onclick={() => appState.session.weather.customPlanSteps = []} style="padding: 10px 20px;">
        {$t('weather.custom.clear_list')}
      </button>
      <button class="filter-btn" onclick={saveCustomPlan} disabled={isSavingPlan} style="padding: 10px 20px; background-color: var(--accent); color: white; border-color: var(--accent);">
        {isSavingPlan ? $t('weather.custom.saving_btn') : $t('weather.custom.save_btn')}
      </button>
      <span style="font-size: 0.82rem; color: var(--text-tertiary);">
        {#if appState.session.weather.customPlanSteps.length > 0}
          {$t('weather.custom.total_duration', { values: { minutes: totalDuration, count: appState.session.weather.customPlanSteps.length } })}
        {/if}
      </span>
    </div>

    <div style="font-size: 0.72rem; color: var(--text-tertiary); line-height: 1.5; margin-top: 8px;">
      💡 {$t('weather.custom.hint')}
    </div>
  {/if}
</div>
{/if}<!-- Fermeture du groupe "Plans Pure" -->

<!-- Fin du fichier -->