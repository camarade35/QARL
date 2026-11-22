<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { appState, toggleFavorite, bumpUsedCount } from '$lib/stores/appState.svelte.js';
  import { getFlagUrl, getCountryName } from '$lib/utils/assets.js';
  import { CM_CATEGORIES, getCarCMCategory, matchesCarFilters } from '$lib/utils/carFilters.js';
  import { t } from 'svelte-i18n';
  import { onMount } from 'svelte';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';

  type CarSkin = { name: string; image_path: string };
  type CarInfo = {
    id: string;
    name: string;
    brand: string;
    image_path: string;
    flag_path: string | null;
    brand_logo_path: string | null;
    tags: string[];
    skins: CarSkin[];
    is_kunos: boolean;
    car_class: string;
    bhp: number;
    pwratio: number;
    year: number;
  };

  let cars: CarInfo[] = $derived(appState.cars);
  let searchQuery = $state('');
  let showTags = $state(false);
  
  let selectedBrand = $state('');
  let selectedCountry = $state('');
  let selectedClass = $state('');

  $effect(() => {
    // AppLoadingScreen charge déjà les voitures au démarrage.
    if (appState.acPath && appState.dataLoadedForPath !== appState.acPath) {
      loadCars();
    }
  });

  $effect(() => {
    const path = appState.acPath;
    if (path) {
      invoke('start_content_watcher', { acPath: path }).catch(console.error);
    }
  });

  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let unlistenContentChanged: Promise<UnlistenFn> | null = null;
  let unlistenCarFound: Promise<UnlistenFn> | null = null;

  onMount(() => {
    unlistenContentChanged = listen('content-changed', () => {
      if (debounceTimer !== null) clearTimeout(debounceTimer);
      debounceTimer = setTimeout(() => {
        loadCars();
      }, 2000);
    });

    unlistenCarFound = listen<CarInfo>('car-found', (event) => {
      const newCar = event.payload;
      if (!appState.cars.some(c => c.id === newCar.id)) {
        appState.cars = [...appState.cars, newCar];
      }
    });

    return () => {
      if (debounceTimer !== null) clearTimeout(debounceTimer);
      if (unlistenContentChanged) {
        void unlistenContentChanged.then(unlisten => unlisten()).catch(console.error);
      }
      if (unlistenCarFound) {
        void unlistenCarFound.then(unlisten => unlisten()).catch(console.error);
      }
    };
  });

  let isRefreshing = $state(false);

  async function loadCars(force = false) {
    if (force) isRefreshing = true;
    // The backend emits car-found events while loading the complete list.
    appState.cars = [];
    try {
      const result = await invoke<CarInfo[]>('get_ac_cars', {
        acPath: appState.acPath,
        forceRefresh: force
      });
      
      // Use the returned list only when no streamed event populated the store.
      if (appState.cars.length === 0) {
        appState.cars = result;
      }
      appState.dataLoadedForPath = appState.acPath;
    } catch (e) {
      console.error("Loading error:", e);
    } finally {
      isRefreshing = false;
    }
  }

  let allBrands = $derived.by(() => {
    const brands = new Set<string>();
    cars.forEach(c => {
      // "Mod" is a generic fallback, not a manufacturer.
      if (c.brand && c.brand !== 'Mod') brands.add(c.brand);
    });
    return Array.from(brands).sort();
  });

  let allCountries = $derived.by(() => {
    const countries = new Set<string>();
    cars.forEach(c => {
      const country = getCountryName(c.tags);
      if (country) countries.add(country);
    });
    return Array.from(countries).sort();
  });

  let filteredCars = $derived(
    cars.filter(car => matchesCarFilters(car, {
      type: appState.carFilters.type,
      brand: selectedBrand,
      country: selectedCountry,
      carClass: selectedClass,
      power: appState.carFilters.power,
      tags: appState.carFilters.tags,
      search: searchQuery,
      favoriteIds: appState.favorites.cars,
      usedCars: appState.used.cars
    }))
  );

  // Charge les images uniquement lorsqu'elles approchent de la zone visible.
  function lazySrc(node: HTMLImageElement, src: string) {
    let currentSrc = src;
    let observer: IntersectionObserver | null = null;

    const load = () => {
      if (currentSrc && node.src !== currentSrc) {
        node.src = currentSrc;
      }
    };

    if (typeof IntersectionObserver !== 'undefined') {
      observer = new IntersectionObserver((entries) => {
        if (entries[0].isIntersecting) {
          load();
          observer?.disconnect();
          observer = null;
        }
      }, { rootMargin: '400px' });
      observer.observe(node);
    } else {
      load();
    }

    return {
      update(newSrc: string) {
        currentSrc = newSrc;
        if (!observer) load();
      },
      destroy() {
        if (observer) observer.disconnect();
      }
    };
  }

  function handleCarClick(car: CarInfo) {
    appState.selectedCar = car;
    bumpUsedCount('cars', car.id);
    if (car.skins && car.skins.length > 1) {
      appState.modalType = 'car';
      appState.modalData = car;
      appState.modalOpen = true;
    } else if (car.skins && car.skins.length === 1) {
      appState.selectedCarSkin = car.skins[0];
    } else {
      appState.selectedCarSkin = null;
    }
  }

  function handleCarKeydown(event: KeyboardEvent, car: CarInfo) {
    // Leave keyboard events from the nested favorite button to that button.
    if (event.target !== event.currentTarget || (event.key !== 'Enter' && event.key !== ' ')) return;
    event.preventDefault();
    handleCarClick(car);
  }
</script>

<div class="sticky-header-wrapper">
<div class="section-header">
  <h3 class="section-title">{$t('cars.header', { values: { count: filteredCars.length } })}</h3>
  <div class="filters-group">
    <button class="filter-btn" class:active={appState.carFilters.type === 'all'} onclick={() => appState.carFilters.type = 'all'}>{$t('common.filter_all')}</button>
    <button class="filter-btn" class:active={appState.carFilters.type === 'kunos'} onclick={() => appState.carFilters.type = 'kunos'}>{$t('common.filter_kunos')}</button>
    <button class="filter-btn" class:active={appState.carFilters.type === 'mods'} onclick={() => appState.carFilters.type = 'mods'}>{$t('common.filter_mods')}</button>
    <button class="filter-btn" class:active={appState.carFilters.type === 'favorites'} onclick={() => appState.carFilters.type = 'favorites'}>{$t('common.filter_favorites')}</button>
    <button class="filter-btn" class:active={appState.carFilters.type === 'used'} onclick={() => appState.carFilters.type = 'used'}>{$t('common.filter_used')}</button>
    <button class="filter-btn" class:active={showTags} onclick={() => showTags = !showTags}>{$t('common.filter_tags', { values: { count: appState.carFilters.tags.length } })}</button>
  </div>
</div>

{#if showTags}
  <div style="display: flex; flex-wrap: wrap; gap: 8px; margin-top: 10px; padding: 12px; background: var(--bg-elevated); border-radius: var(--r-sm);">
    {#each Array.from(new Set(cars.flatMap(c => c.tags || []))).sort() as tag}
      <button 
        class="filter-btn" 
        class:active={appState.carFilters.tags.includes(tag)}
        onclick={() => {
          const idx = appState.carFilters.tags.indexOf(tag);
          if (idx >= 0) appState.carFilters.tags.splice(idx, 1);
          else appState.carFilters.tags.push(tag);
        }}
        style="padding: 4px 10px; font-size: 0.75rem;"
      >
        {tag}
      </button>
    {/each}
    {#if appState.carFilters.tags.length > 0}
      <button class="filter-btn" onclick={() => appState.carFilters.tags = []} style="padding: 4px 10px; font-size: 0.75rem; color: #e6a33b;">{$t('common.clear')}</button>
    {/if}
  </div>
{/if}

<div class="toolbar" style="margin-top: 15px; display: flex; align-items: center; justify-content: space-between; gap: 10px; flex-wrap: wrap;">
  <!-- Recherche -->
  <div style="display: flex; gap: 8px; align-items: stretch; flex: 1; min-width: 250px; max-width: 400px;">
    <div class="search-input" style="flex: 1;">
      <input type="text" placeholder={$t('cars.search_placeholder')} bind:value={searchQuery} style="width: 100%; height: 100%;" />
    </div>
    <button 
      class="filter-btn" 
      onclick={() => loadCars(true)} 
      disabled={isRefreshing}
      title="Forcer le rafraîchissement des voitures"
      style="display: flex; align-items: center; justify-content: center; padding: 0 12px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm); color: var(--text-secondary); cursor: pointer; transition: 0.2s;"
    >
      <svg class:spin={isRefreshing} xmlns="http://www.w3.org/2000/svg" width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
        <path d="M21 12a9 9 0 0 0-9-9 9.75 9.75 0 0 0-6.74 2.74L3 8"></path>
        <path d="M3 3v5h5"></path>
        <path d="M3 12a9 9 0 0 0 9 9 9.75 9.75 0 0 0 6.74-2.74L21 16"></path>
        <path d="M16 21v-5h5"></path>
      </svg>
    </button>
  </div>

  <!-- Filtres -->
  <div style="display: flex; gap: 10px; flex-wrap: wrap; align-items: center;">
    <select class="manufacturer-select" bind:value={selectedBrand} style="min-width: 150px; width: auto;">
      <option value="">{$t('cars.all_brands')}</option>
      {#each allBrands as brand}
        <option value={brand}>{brand}</option>
      {/each}
    </select>

    <select class="manufacturer-select" bind:value={selectedCountry} style="min-width: 150px; width: auto;">
      <option value="">{$t('cars.all_countries')}</option>
      {#each allCountries as country}
        <option value={country}>{$t(country, { default: country })}</option>
      {/each}
    </select>

    <select class="manufacturer-select" bind:value={selectedClass} style="min-width: 150px; width: auto;">
      <option value="">{$t('cars.all_classes')}</option>
      {#each CM_CATEGORIES as cat}
        <option value={cat.id}>{$t(`cars.categories.${cat.id}`, { default: cat.id })}</option>
      {/each}
    </select>

    <select class="manufacturer-select" bind:value={appState.carFilters.power} style="min-width: 160px; width: auto;">
      <option value="">{$t('cars.all_powers')}</option>
      <option value="under_200">{$t('cars.power_under_200')}</option>
      <option value="200_400">{$t('cars.power_200_400')}</option>
      <option value="400_600">{$t('cars.power_400_600')}</option>
      <option value="over_600">{$t('cars.power_over_600')}</option>
    </select>
  </div>
</div>
</div>

{#if !appState.acPath}
  <div style="padding: 30px; color: var(--text-secondary); text-align: center; border: 1px dashed var(--border); border-radius: var(--r); background: var(--bg-elevated); margin-top: 20px;">
    <div style="font-size: 2rem; margin-bottom: 10px;">⚠️</div>
    <p style="margin-bottom: 15px; font-weight: 500; color: var(--text-primary);">{$t('cars.no_ac_path_title')}</p>
    <button class="filter-btn" onclick={() => appState.activeTab = 'tab-settings'} style="background: var(--accent); color: white; border-color: var(--accent); padding: 8px 16px;">
      {$t('cars.go_settings')}
    </button>
  </div>
{:else}
<div class="cards-grid" style="margin-top: 20px;">
  {#each filteredCars as car (car.id)}
    {@const brandLogo = car.brand_logo_path ? convertFileSrc(car.brand_logo_path) : null}
    {@const flagUrl = getFlagUrl(car.tags, car.flag_path ? convertFileSrc(car.flag_path) : null)}

    <div 
      class="card" 
      class:selected={appState.selectedCar?.id === car.id}
      onclick={() => handleCarClick(car)}
      onkeydown={(event) => handleCarKeydown(event, car)}
      oncontextmenu={(e) => { 
        e.preventDefault(); 
        invoke('open_folder', { 
          acPath: appState.acPath, 
          documentsPath: appState.documentsPath,
          folderType: 'cars', 
          folderId: car.id 
        }).catch(err => console.error(err)); 
      }}
      role="button"
      tabindex="0"
      title={$t('cars.context_menu_open_folder')}
    >
      {#if brandLogo}
        <img src={brandLogo} class="brand-badge" alt="" onerror={(e) => {
          (e.currentTarget as HTMLImageElement).style.display = 'none';
        }} />
      {/if}

      {#if flagUrl}
        <div class="flag-badge">        <img src={flagUrl} alt="" onerror={(e) => {
          const parent = e.currentTarget.parentElement;
          if (parent) parent.style.display = 'none';
        }} /></div>
      {/if}

      {#if car.image_path}
        <img 
          use:lazySrc={convertFileSrc(car.image_path)} 
          class="car-preview" 
          alt={car.name}
          onload={(e) => e.currentTarget.classList.add('loaded')} 
        />
      {:else}
        <div class="car-preview" style="display: flex; align-items: center; justify-content: center; color: var(--text-tertiary);">{$t('cars.no_image')}</div>
      {/if}
      
      <button class="fav-btn" class:active={appState.favorites.cars.includes(car.id)} onclick={(e) => { e.stopPropagation(); toggleFavorite('cars', car.id); }}>
        {appState.favorites.cars.includes(car.id) ? '★' : '☆'}
      </button>

      <div class="card-title">
        <span class="card-title-text">{car.name}</span>
        {#if appState.used.cars[car.id] > 0}
          <span class="used-badge" title={$t('cars.used_badge_title', { values: { count: appState.used.cars[car.id] } })}>✓</span>
        {/if}
      </div>
    </div>
    {/each}
</div>
{/if}

<style>
  .spin {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    100% { transform: rotate(360deg); }
  }
</style>