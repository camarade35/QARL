<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { appState, toggleFavorite, bumpUsedCount } from '$lib/stores/appState.svelte.js';
  import { getFlagUrl, getCountryName } from '$lib/utils/assets.js';
  import { t } from 'svelte-i18n';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let tracks = $derived(appState.tracks);
  let loading = $state(false);
  let searchQuery = $state('');
  let showTags = $state(false);
  let selectedCountry = $state('');

  // ⭐ (Scroll infini supprimé)

  $effect(() => {
    // ⭐ On ne recharge que si le chemin AC a changé (le chargement initial est fait par AppLoadingScreen)
    if (appState.acPath && appState.dataLoadedForPath !== appState.acPath) {
      loadTracks();
    }
    // ⭐ Démarrer la surveillance des dossiers
    if (appState.acPath) {
      invoke('start_content_watcher', { acPath: appState.acPath }).catch(console.error);
    }
  });

  // ⭐ Écouter les changements de contenu
  let debounceTimer: any;
  let unlistenPromise: any;

  onMount(() => {
    unlistenPromise = listen('content-changed', () => {
      // On attend 2 secondes après le dernier événement pour éviter de spammer
      clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        loadTracks();
      }, 2000);
    });

    return () => {
      if (unlistenPromise) unlistenPromise.then((unlisten: any) => unlisten());
    };
  });

  async function loadTracks() {
    loading = true;
    try {
      const result = await invoke<any[]>('get_ac_tracks', { acPath: appState.acPath });
      appState.tracks = result;
      appState.dataLoadedForPath = appState.acPath;
    } catch (e) {
      console.error("Error loading tracks:", e);
    } finally {
      loading = false;
    }
  }

  let allCountries = $derived.by(() => {
    const countries = new Set<string>();
    tracks.forEach(t => {
      const country = getCountryName(t.country);
      if (country) countries.add(country);
    });
    return Array.from(countries).sort();
  });

  let allTags = $derived.by(() => {
    const tags = new Set<string>();
    tracks.forEach(t => t.tags?.forEach((tag: string) => tags.add(tag)));
    return Array.from(tags).sort();
  });

  let filteredTracks = $derived(
    tracks.filter(t => {
      if (appState.trackFilters.type === 'kunos' && !t.is_kunos) return false;
      if (appState.trackFilters.type === 'mods' && t.is_kunos) return false;
      if (appState.trackFilters.type === 'favorites' && !appState.favorites.tracks.includes(t.id)) return false;
      if (appState.trackFilters.type === 'used' && !appState.used.tracks[t.id]) return false;
      
      if (selectedCountry) {
        const tCountry = getCountryName(t.country);
        if (tCountry !== selectedCountry) return false;
      }

      if (appState.trackFilters.tags.length > 0) {
        if (!t.tags || !appState.trackFilters.tags.every((tag: string) => t.tags.includes(tag))) return false;
      }
      if (searchQuery && !t.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      return true;
    })
  );

  function selectTrack(track) {
    appState.selectedTrack = track;
    bumpUsedCount('tracks', track.id);
    if (track.layouts && track.layouts.length > 1) {
      appState.modalType = 'track';
      appState.modalData = track;
      appState.modalOpen = true;
    } else if (track.layouts && track.layouts.length === 1) {
      appState.selectedTrackLayout = track.layouts[0];
    } else {
      appState.selectedTrackLayout = null;
    }
  }
</script>

<div class="sticky-header-wrapper">
<div class="section-header">
  <h2 class="section-title">{$t('tracks.header', { values: { count: filteredTracks.length } })}</h2>
  <div class="filters-group">
    <button class="filter-btn" class:active={appState.trackFilters.type === 'all'} onclick={() => appState.trackFilters.type = 'all'}>{$t('common.filter_all')}</button>
    <button class="filter-btn" class:active={appState.trackFilters.type === 'kunos'} onclick={() => appState.trackFilters.type = 'kunos'}>{$t('common.filter_kunos')}</button>
    <button class="filter-btn" class:active={appState.trackFilters.type === 'mods'} onclick={() => appState.trackFilters.type = 'mods'}>{$t('common.filter_mods')}</button>
    <button class="filter-btn" class:active={appState.trackFilters.type === 'favorites'} onclick={() => appState.trackFilters.type = 'favorites'}>{$t('common.filter_favorites')}</button>
    <button class="filter-btn" class:active={appState.trackFilters.type === 'used'} onclick={() => appState.trackFilters.type = 'used'}>{$t('common.filter_used')}</button>
    <button class="filter-btn" class:active={showTags} onclick={() => showTags = !showTags}>{$t('common.filter_tags', { values: { count: appState.trackFilters.tags.length } })}</button>
  </div>
</div>

{#if showTags}
  <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; padding: 12px; background: var(--bg-elevated); border-radius: var(--r-sm);">
    {#each allTags as tag}
      <button 
        class="filter-btn" 
        class:active={appState.trackFilters.tags.includes(tag)}
        onclick={() => {
          const idx = appState.trackFilters.tags.indexOf(tag);
          if (idx >= 0) appState.trackFilters.tags.splice(idx, 1);
          else appState.trackFilters.tags.push(tag);
        }}
        style="padding: 4px 10px; font-size: 0.75rem;"
      >
        {tag}
      </button>
    {/each}
    {#if appState.trackFilters.tags.length > 0}
      <button class="filter-btn" onclick={() => appState.trackFilters.tags = []} style="padding: 4px 10px; font-size: 0.75rem; color: #e6a33b;">{$t('common.clear')}</button>
    {/if}
  </div>
{/if}

<!-- ⭐ Toolbar alignée : Recherche à gauche, Filtres à droite -->
<div class="toolbar" style="margin-top: 15px; margin-bottom: 20px; display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap;">
  <div class="search-input" style="flex: 1; min-width: 250px; max-width: 400px;">
    <input type="text" placeholder={$t('tracks.search_placeholder')} bind:value={searchQuery} style="width: 100%;" />
  </div>

  <div style="display: flex; gap: 10px; flex-wrap: wrap; align-items: center;">
    <select class="manufacturer-select" bind:value={selectedCountry} style="min-width: 160px; width: auto;">
      <option value="">{$t('tracks.all_countries')}</option>
      {#each allCountries as country}
        <option value={country}>{$t(country, { default: country })}</option>
      {/each}
    </select>
  </div>
</div>
</div> <!-- ⭐ Fermeture du sticky-header-wrapper -->

{#if !appState.acPath}
  <div style="padding: 30px; color: var(--text-secondary); text-align: center; border: 1px dashed var(--border); border-radius: var(--r); background: var(--bg-elevated); margin-top: 15px;">
    <div style="font-size: 2rem; margin-bottom: 10px;">⚠️</div>
    <p style="margin-bottom: 15px; font-weight: 500; color: var(--text-primary);">{$t('tracks.no_ac_path_title')}</p>
    <button class="filter-btn" onclick={() => appState.activeTab = 'tab-settings'} style="background: var(--accent); color: white; border-color: var(--accent); padding: 8px 16px;">
      {$t('tracks.go_settings')}
    </button>
  </div>
{:else if loading}
  <div style="padding: 20px; color: var(--text-secondary);">{$t('tracks.loading')}</div>
{:else}
  <div class="cards-grid">
    {#each filteredTracks as track (track.id)}
      {@const layoutPreview = track.layouts.find(l => l.preview_path)}
      {@const layoutOutline = track.layouts.find(l => l.outline_path)}
      {@const bgImage = track.preview_path || (layoutPreview ? layoutPreview.preview_path : null)}
      {@const outlineImage = track.outline_path || (layoutOutline ? layoutOutline.outline_path : null)}
      
      {@const flagUrl = getFlagUrl([track.id, track.country, ...(track.tags || [])])}

      <div 
        class="card" 
        class:selected={appState.selectedTrack?.id === track.id}
        onclick={() => selectTrack(track)}
        oncontextmenu={(e) => { 
          e.preventDefault(); 
          invoke('open_folder', { 
            acPath: appState.acPath, 
            documentsPath: appState.documentsPath, // ⭐ Ajout du paramètre manquant
            folderType: 'tracks', 
            folderId: track.id 
          }).catch(err => console.error(err)); 
        }}
        role="button"
        tabindex="0"
        title={$t('tracks.context_menu_open_folder')}
      >
        <div class="track-preview-box" style="background: #0f1219; display: flex; align-items: center; justify-content: center;">
          {#if bgImage}
            <img class="track-bg" src={convertFileSrc(bgImage)} alt="" />
          {/if}
          
          {#if outlineImage}
            <img class="track-outline" src={convertFileSrc(outlineImage)} alt="" />
          {:else if !bgImage}
            <!-- Fallback si le circuit n'a AUCUNE image -->
            <div style="position: relative; z-index: 2; color: var(--text-tertiary); font-size: 0.8rem; text-align: center; padding: 10px;">
              🏁<br>{$t('tracks.no_preview')}
            </div>
          {/if}
        </div>

        {#if flagUrl}
          <div class="flag-badge"><img src={flagUrl} alt="" onerror={(e) => e.currentTarget.parentElement.style.display = 'none'} /></div>
        {/if}

        <button class="fav-btn" class:active={appState.favorites.tracks.includes(track.id)} onclick={(e) => { e.stopPropagation(); toggleFavorite('tracks', track.id); }}>
          {appState.favorites.tracks.includes(track.id) ? '★' : '☆'}
        </button>
        
        <div class="card-title">
          <span class="card-title-text">{track.name}</span>
          {#if appState.used.tracks[track.id] > 0}
            <span class="used-badge" title={$t('tracks.used_badge_title', { values: { count: appState.used.tracks[track.id] } })}>✓</span>
          {/if}
        </div>
      </div>
    {/each}
  </div>
  
  {#if filteredTracks.length === 0}
    <div style="padding: 20px; color: var(--text-tertiary);">{$t('tracks.empty')}</div>
  {/if}
{/if}