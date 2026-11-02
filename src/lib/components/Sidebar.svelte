<script lang="ts">
  import { appState } from '$lib/stores/appState.svelte.js';
  import { convertFileSrc, invoke } from '@tauri-apps/api/core';
  import { generateGrid } from '$lib/utils/sessionGenerator.js';
  import { getWeatherIcon, translateWeatherName } from '$lib/utils/weatherUtils.js';
  import { t, locale } from 'svelte-i18n';
  import LanguagePicker from '$lib/components/LanguagePicker.svelte';
  import { alert } from '$lib/stores/dialog.svelte'; // ⭐ On importe l'alerte QARL
  
  // ⭐ Gestion du thème (Sombre, Clair, Bureautique)
  const savedTheme = typeof localStorage !== 'undefined' ? localStorage.getItem('qarl-theme') : null;
  let currentTheme = $state(savedTheme || 'dark');

  // Appliquer le thème au chargement
  if (typeof document !== 'undefined') {
    document.documentElement.setAttribute('data-theme', currentTheme);
  }

  function setTheme(theme: string) {
    currentTheme = theme;
    document.documentElement.setAttribute('data-theme', theme);
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('qarl-theme', theme);
    }
  }

  // ⭐ Gestion de la couleur d'accentuation
  const savedAccent = typeof localStorage !== 'undefined' ? localStorage.getItem('qarl_accent') : null;
  let accentColor = $state(savedAccent || '#319dd3');

  function hexToRgb(hex: string) {
    const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
    return result ? {
      r: parseInt(result[1], 16),
      g: parseInt(result[2], 16),
      b: parseInt(result[3], 16)
    } : { r: 49, g: 157, b: 211 };
  }

  function applyAccentColor(color: string, currentMode: string) {
    if (typeof document === 'undefined') return;
    let { r, g, b } = hexToRgb(color);

    // ⭐ Adaptation selon le thème
    if (currentMode === 'light') {
      // En mode clair, on assombrit la couleur de 25% pour le contraste
      r = Math.floor(r * 0.75);
      g = Math.floor(g * 0.75);
      b = Math.floor(b * 0.75);
    } else {
      // En mode sombre, on éclaircit la couleur de 15% pour le côté néon
      r = Math.min(255, Math.floor(r * 1.15));
      g = Math.min(255, Math.floor(g * 1.15));
      b = Math.min(255, Math.floor(b * 1.15));
    }

    const adjustedHex = `#${r.toString(16).padStart(2, '0')}${g.toString(16).padStart(2, '0')}${b.toString(16).padStart(2, '0')}`;
    
    document.documentElement.style.setProperty('--accent', adjustedHex);
    document.documentElement.style.setProperty('--accent-rgb', `${r}, ${g}, ${b}`);
    
    // Couleur de survol (hover) : on éclaircit encore un peu en mode sombre, on assombrit en mode clair
    const hoverR = currentMode === 'light' ? Math.max(0, r - 20) : Math.min(255, r + 20);
    const hoverG = currentMode === 'light' ? Math.max(0, g - 20) : Math.min(255, g + 20);
    const hoverB = currentMode === 'light' ? Math.max(0, b - 20) : Math.min(255, b + 20);
    
    document.documentElement.style.setProperty('--accent-hover', `rgb(${hoverR}, ${hoverG}, ${hoverB})`);
    document.documentElement.style.setProperty('--accent-muted', `rgba(${r}, ${g}, ${b}, 0.12)`);
    document.documentElement.style.setProperty('--accent-glow', `rgba(${r}, ${g}, ${b}, 0.4)`);
    
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('qarl_accent', color);
    }
  }

  // ⭐ Applique la couleur dès le chargement et à chaque changement de couleur OU de thème
  $effect(() => {
    applyAccentColor(accentColor, currentTheme);
  });

  // ⭐ Génération réactive de la grille
  let aiGrid = $derived.by(() => {
    if (!appState.selectedCar || appState.cars.length === 0) return [];
    return generateGrid(appState.selectedCar, appState.cars, appState.session);
  });

  // ⭐ Affichage météo pour la carte Session
  let weatherDisplay = $derived.by(() => {
    const w = appState.session.weather;
    if (w.mode === 'existing_plan') {
      if (w.existingPlanRandom) return $t('sidebar.plan_random');
      const planName = w.existingPlan || $t('sidebar.plan_none');
      // ⭐ Nettoyage du nom pour retirer le chemin et l'extension
      const cleanName = planName.split('/').pop()?.replace('.json', '').replace(/_/g, ' ') || planName;
      return `${$t('sidebar.plan_prefix')} ${cleanName}`;
    }
    if (w.mode === 'schedule') {
      return $t('sidebar.schedule_display', { values: { count: w.scheduleSlots || 3 } });
    }
    if (w.mode === 'custom_plan') {
      const count = w.customPlanSteps.length;
      const name = w.customPlanName || $t('sidebar.plan_unnamed');
      return $t('sidebar.plan_custom', { values: { name, count } });
    }
    // Mode static (fallback)
    const id = w.staticWeather || '';
    const icon = getWeatherIcon(id);
    const fallback = id.replace(/^\d+_/, '').replace(/_/g, ' ');
    const name = translateWeatherName(id, fallback, $locale);
    return `${icon} ${$t('sidebar.weather_prefix')} ${name}`;
  });

  // ⭐ Helper pour convertir l'index CSP en ID lisible (ex: 15 -> '15_clear')
  function getScheduleWeatherId(index: number) {
    const map: Record<number, string> = {
      15: '15_clear', 16: '16_few_clouds', 17: '17_scattered_clouds', 18: '18_broken_clouds',
      19: '19_overcast_clouds', 21: '21_mist', 20: '20_fog', 3: '3_light_drizzle',
      4: '4_drizzle', 5: '5_heavy_drizzle', 6: '6_light_rain', 7: '7_rain',
      8: '8_heavy_rain', 0: '0_light_thunderstorm', 1: '1_thunderstorm', 2: '2_heavy_thunderstorm',
      12: '12_light_sleet', 13: '13_sleet', 14: '14_heavy_sleet', 9: '9_light_snow',
      10: '10_snow', 11: '11_heavy_snow', 27: '27_tornado', 28: '28_hurricane',
      22: '22_smoke', 23: '23_haze', 24: '24_sand', 25: '25_dust',
      26: '26_squalls', 29: '29_cold', 30: '30_hot', 31: '31_windy', 32: '32_hail'
    };
    return map[index] || '15_clear';
  }

  // ⭐ LANCEMENT DE LA COURSE
  let isLaunching = $state(false);
  
  async function startRace() {
    if (!appState.selectedCar || !appState.selectedTrack) {
      alert($t('sidebar.error_select'));
      return;
    }
    
    // ⭐ Sécurité : On empêche de lancer un plan personnalisé sans nom valide
    if (appState.session.weather.mode === 'custom_plan' && (!appState.session.weather.customPlanName || appState.session.weather.customPlanName.trim().length < 3)) {
      alert($t('sidebar.error_custom_plan_name'));
      return;
    }
    if (!appState.acPath) {
      alert($t('sidebar.error_path'));
      return;
    }

    // ⭐ NOUVEAU : On vérifie aussi le dossier Documents
    if (!appState.documentsPath) {
      alert($t('sidebar.error_docs_path'));
      return;
    }

    isLaunching = true;
    try {
      await invoke('launch_race', {
        acPath: appState.acPath,
        carId: appState.selectedCar.id,
        skinId: appState.selectedCarSkin?.name || "",
        trackId: appState.selectedTrack.id,
        layoutId: appState.selectedTrackLayout?.id || "",
        sessionData: appState.session,
        aiGrid: aiGrid // ⭐ On envoie la grille générée
      });
      // Le jeu se lance en arrière-plan
    } catch (e) {
      alert($t('sidebar.error_launch') + "\n" + e);
    } finally {
      isLaunching = false;
    }
  }
</script>

<div class="sidebar">
  <div class="sidebar-top">
    <div class="app-header" style="flex-direction: column; align-items: stretch; gap: 16px; padding-bottom: 15px; border-bottom: 1px solid var(--border-subtle); margin-bottom: 15px;">
      <!-- Ligne 1 : Logo et Titre -->
      <div style="display: flex; align-items: center; gap: 10px; margin-bottom: 4px;">
        <img class="app-logo" src="/assets/logo.png" alt={$t('sidebar.logo_alt')} />
        <div class="app-titles">
          <div class="app-title" style="font-size: 1.4rem;">QARL</div>
        </div>
      </div>
      
      <!-- Ligne 2 : Langues -->
      <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
        <span style="font-size: 0.75rem; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.5px;">{$t('sidebar.label_language')}</span>
        <div style="display: flex; flex-wrap: wrap; justify-content: flex-end; gap: 4px; max-width: 200px;">
          <LanguagePicker />
        </div>
      </div>

      <!-- Ligne 3 : Thèmes & Couleur -->
      <div style="display: flex; align-items: center; justify-content: space-between; width: 100%;">
        <span style="font-size: 0.75rem; font-weight: 700; color: var(--text-secondary); text-transform: uppercase; letter-spacing: 0.5px;">{$t('sidebar.label_theme')}</span>
        <div class="theme-controls" style="gap: 8px;">
          <!-- ⭐ Sélecteur de couleur -->
          <label class="theme-picker" title={$t('sidebar.accent_color_tooltip')}>
            <input type="color" bind:value={accentColor} />
            <svg class="theme-picker-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path></svg>
          </label>

          <!-- ⭐ Menu déroulant de sélection du thème -->
          <select 
            bind:value={currentTheme} 
            onchange={(e) => setTheme(e.currentTarget.value)}
            class="ini-select"
            style="width: auto; min-width: 110px; height: 34px; padding: 0 10px; cursor: pointer; font-size: 0.8rem; font-weight: 600;"
            title={$t('sidebar.theme_tooltip')}
          >
            <option value="dark">🌙 {$t('sidebar.theme_dark')}</option>
            <option value="light">☀️ {$t('sidebar.theme_light')}</option>
            <option value="office">💼 {$t('sidebar.theme_office')}</option>
          </select>
        </div>
      </div>
    </div>

    <!-- CARTE VOITURE -->
    <div class="recap-card clickable" onclick={() => appState.activeTab = 'tab-cars'}>
      <div class="recap-label">{$t('sidebar.car')}</div>
      {#if appState.selectedCar}
        <div class="recap-title-row">
          <div class="recap-title">{appState.selectedCar.name}</div>
        </div>
        <div class="recap-sub" style="display: flex; align-items: center; gap: 6px;">
          {#if appState.selectedCarSkin}
            <span>{$t('sidebar.skin_prefix')} {appState.selectedCarSkin.name}</span>
          {:else}
            <span>{$t('sidebar.no_skin')}</span>
          {/if}
        </div>
        
        {#if appState.selectedCarSkin}
          <img src={convertFileSrc(appState.selectedCarSkin.image_path)} class="recap-img" style="display: block;" alt={$t('sidebar.preview_alt')} />
        {:else if appState.selectedCar.image_path}
          <img src={convertFileSrc(appState.selectedCar.image_path)} class="recap-img" style="display: block;" alt={$t('sidebar.preview_alt')} />
        {/if}
      {:else}
        <div class="recap-title-row"><div class="recap-title">{$t('sidebar.no_car')}</div></div>
      {/if}
    </div>

    <!-- CARTE CIRCUIT -->
    <div class="recap-card clickable" onclick={() => appState.activeTab = 'tab-tracks'}>
           <div class="recap-label">{$t('sidebar.track')}</div>
      {#if appState.selectedTrack}
        <div class="recap-title-row">
          <span class="recap-title">
            {appState.selectedTrack.name}
            <!-- On force une réaction au changement d'ID -->
            {appState.selectedTrack.id ? '' : ''}
            {#if appState.selectedTrackLayout && appState.selectedTrackLayout.name !== 'DÉFAUT' && appState.selectedTrackLayout.name !== 'Défaut' && appState.selectedTrackLayout.name !== ''}
              <br><span style="font-size: 0.8em; color: var(--text-secondary); font-weight: 400;">({appState.selectedTrackLayout.name})</span>
            {/if}
          </span>
        </div>
        <div class="recap-sub">
          {appState.selectedTrack.city} {$t('countries.' + (appState.selectedTrack.country || '').toLowerCase(), { default: appState.selectedTrack.country })}
        </div>
        
        {#if appState.selectedTrackLayout}
          {@const bgImage = appState.selectedTrackLayout.preview_path || appState.selectedTrack.preview_path}
          {@const outlineImage = appState.selectedTrackLayout.outline_path || appState.selectedTrack.outline_path}
          
          <div class="track-preview" style="display: block; background: #0f1219;">
            {#if bgImage}
              <img src={convertFileSrc(bgImage)} style="opacity: 0.5; object-fit: cover;" alt={$t('sidebar.background_alt')} />
            {/if}
            {#if outlineImage}
              <img src={convertFileSrc(outlineImage)} style="object-fit: contain; z-index: 1;" alt={$t('sidebar.outline_alt')} />
            {/if}
          </div>
        {/if}
      {:else}
        <div class="recap-sub">{$t('sidebar.no_track')}</div>
      {/if}
    </div>

    <!-- CARTE SESSION -->
    <div class="recap-card clickable" onclick={() => { appState.activeTab = 'tab-session'; }}>
      <div class="recap-label">{$t('sidebar.session')}</div>
      <div class="recap-title">
        {appState.session.type === 'RACE' 
          ? $t('sidebar.race_with_laps', { values: { count: appState.session.laps } }) 
          : $t(`session.types.${appState.session.type.toLowerCase()}`)}
      </div>
      <div class="recap-sub">
        {$t('sidebar.opponents', { values: { count: appState.session.opponentsCount } })}
      </div>
    </div>

            <!-- ⭐ CARTE MÉTÉO -->
    <div class="recap-card clickable" onclick={() => { appState.activeTab = 'tab-graphics'; }}>
      <div class="recap-label" style="display: flex; justify-content: space-between; align-items: center;">
        <span>{$t('sidebar.weather')}</span>
        <span style="font-size: 0.65rem; color: var(--text-tertiary); font-weight: 400;">{$t('sidebar.modify')}</span>
      </div>
      
      <div class="recap-sub" style="display: flex; align-items: center; gap: 6px; margin-bottom: 6px;">
        <span>{weatherDisplay}</span>
      </div>

      <!-- ⭐ Plus d'affichage détaillé pour les plans personnalisés (géré dans l'onglet Météo) -->

      <!-- ⭐ Affichage pour les plans existants (chargés depuis l'onglet Météo) -->
      {#if appState.session.weather.mode === 'existing_plan'}
        
        <div style="display: flex; gap: 8px; flex-wrap: wrap; align-items: center; font-size: 0.72rem; font-weight: 600; margin-top: 6px; margin-bottom: 6px;">
          <span style="color: var(--accent); display: flex; align-items: center; gap: 4px;">
            🕒 {$t('sidebar.start_prefix')} {String(appState.session.weather.startHour).padStart(2, '0')}:{String(appState.session.weather.startMinute).padStart(2, '0')}
          </span>
          
          <!-- ⭐ NOUVEAU : Badges Loop et TimeMult -->
          {#if appState.session.weather.existingPlanTimeMult && appState.session.weather.existingPlanTimeMult > 1}
            <span style="color: var(--text-secondary); background: var(--bg-panel); border: 1px solid var(--border-subtle); padding: 2px 6px; border-radius: 4px;">⏱️ x{appState.session.weather.existingPlanTimeMult}</span>
          {/if}
          {#if appState.session.weather.existingPlanLoop}
            <span style="color: var(--text-secondary); background: var(--bg-panel); border: 1px solid var(--border-subtle); padding: 2px 6px; border-radius: 4px;">🔁 {$t('sidebar.loop')}</span>
          {/if}
        </div>

        {#if appState.session.weather.existingPlanSteps && appState.session.weather.existingPlanSteps.length > 0}
          <div style="display: flex; flex-direction: column; gap: 4px; margin-top: 6px;">
            {#each appState.session.weather.existingPlanSteps as step, index}
              <div style="display: flex; align-items: center; justify-content: space-between; font-size: 0.72rem; background: var(--bg-panel); padding: 5px 8px; border-radius: 4px; border: 1px solid var(--border-subtle);">
                <span style="display: flex; align-items: center; gap: 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                  <span style="font-weight: 800; color: var(--accent); width: 12px;">{index + 1}</span>
                  <span style="font-size: 0.9rem;">{getWeatherIcon(step.weather)}</span>
                  <span style="color: var(--text-secondary);">{translateWeatherName(step.weather, step.weather, $locale)}</span>
                </span>
                <span style="color: var(--text-tertiary); font-weight: 600;">{step.duration} min</span>
              </div>
            {/each}
          </div>
        {/if}
      
      <!-- ⭐ NOUVEAU : Affichage pour les plans personnalisés -->
      {:else if appState.session.weather.mode === 'custom_plan'}
        {#if appState.session.weather.customPlanSteps && appState.session.weather.customPlanSteps.length > 0}
          <div style="display: flex; flex-direction: column; gap: 4px; margin-top: 6px;">
            {#each appState.session.weather.customPlanSteps as step, index}
              <div style="display: flex; align-items: center; justify-content: space-between; font-size: 0.72rem; background: var(--bg-panel); padding: 5px 8px; border-radius: 4px; border: 1px solid var(--border-subtle);">
                <span style="display: flex; align-items: center; gap: 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                  <span style="font-weight: 800; color: var(--accent); width: 12px;">{index + 1}</span>
                  <span style="font-size: 0.9rem;">{getWeatherIcon(step.weather)}</span>
                  <span style="color: var(--text-secondary);">{translateWeatherName(step.weather, step.weather, $locale)}</span>
                </span>
                <span style="color: var(--text-tertiary); font-weight: 600;">{step.duration} min</span>
              </div>
            {/each}
          </div>
        {/if}

      <!-- ⭐ NOUVEAU : Affichage pour le mode Schedule -->
      {:else if appState.session.weather.mode === 'schedule'}
        <div style="display: flex; flex-direction: column; gap: 4px; margin-top: 6px;">
          {#each appState.session.weather.scheduleSlotValues as slotIndex, i}
            {@const wId = getScheduleWeatherId(slotIndex)}
            <div style="display: flex; align-items: center; justify-content: space-between; font-size: 0.72rem; background: var(--bg-panel); padding: 5px 8px; border-radius: 4px; border: 1px solid var(--border-subtle);">
              <span style="display: flex; align-items: center; gap: 6px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;">
                <span style="font-weight: 800; color: var(--accent); width: 12px;">{i + 1}</span>
                <span style="font-size: 0.9rem;">{getWeatherIcon(wId)}</span>
                <span style="color: var(--text-secondary);">{translateWeatherName(wId, wId, $locale)}</span>
              </span>
              <span style="color: var(--text-tertiary); font-weight: 600;">{appState.session.weather.scheduleTimePerSlot} min</span>
            </div>
          {/each}
        </div>
      {/if}
    </div>

    <!-- ⭐ CARTE GRILLE DE DÉPART -->
    {#if appState.selectedCar && appState.session.opponentsCount > 0}
      <div class="recap-card clickable" onclick={() => appState.activeTab = 'tab-session'} style="cursor: pointer;">
        <div class="recap-label" style="margin-bottom: 6px;">
          <span>{$t('sidebar.grid')}</span>
          <span style="font-size: 0.65rem; color: var(--accent); background: var(--accent-muted); padding: 2px 6px; border-radius: 4px;">
            {$t('sidebar.cars_count', { values: { count: appState.session.opponentsCount + 1 } })}
          </span>
        </div>
        
        <div style="max-height: 150px; overflow-y: auto; padding-right: 4px; display: flex; flex-direction: column; gap: 4px;">
          <!-- Le joueur en premier -->
          <div style="font-size: 0.75rem; font-weight: 600; color: var(--accent); padding: 4px 6px; background: var(--accent-muted); border-radius: 4px;">
            1. {appState.selectedCar.brand} - {appState.selectedCar.name.replace(appState.selectedCar.brand, '').trim()} <span style="font-weight: 400; opacity: 0.8;">{$t('sidebar.you')}</span>
          </div>
          
          <!-- Les IA -->
          {#each aiGrid as ai, index}
            <div style="font-size: 0.72rem; color: var(--text-secondary); padding: 4px 6px; border-bottom: 1px solid var(--border-subtle);">
              {index + 2}. {ai.brand || $t('common.unknown')} - {ai.model} <br>
              <span style="color: var(--text-tertiary); font-size: 0.65rem;">👤 {ai.driver}</span>
            </div>
          {/each}
        </div>
      </div>
    {/if}

  </div>
  
  <!-- Le fameux bouton -->
  <button class="btn-launch" onclick={startRace} disabled={isLaunching}>
    {isLaunching ? $t('sidebar.launching') : $t('sidebar.launch')}
  </button>
</div>