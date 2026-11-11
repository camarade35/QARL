<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { appState, toggleFavorite, bumpUsedCount } from '$lib/stores/appState.svelte.js';
  import { getFlagUrl, getBrandLogoUrl, getCountryName } from '$lib/utils/assets.js';
  import { t } from 'svelte-i18n';
  import { onMount, onDestroy } from 'svelte';
  import { listen } from '@tauri-apps/api/event';

  let cars = $derived(appState.cars);
  let isLoading = $state(false);
  let searchQuery = $state('');
  let showTags = $state(false);
  
  // Filtres
  let selectedBrand = $state('');
  let selectedCountry = $state('');
  let selectedClass = $state('');

  // ⭐ (Scroll infini supprimé)

  // ⭐ Les catégories exactes de Content Manager avec des mots-clés pour le matching
  const CM_CATEGORIES = [
    { id: 'Classic', keywords: ['classic', 'historic', 'old'] },
    { id: 'Drift', keywords: ['drift'] },
    { id: 'Open-wheel', keywords: ['open-wheel', 'openwheel', 'formula', 'f1'] },
    { id: 'Prototype', keywords: ['prototype', 'lmp', 'hypercar'] },
    { id: 'Race', keywords: ['race', 'gt3', 'gt4', 'gt2', 'touring'] },
    { id: 'Rally', keywords: ['rally'] },
    { id: 'Sportscars', keywords: ['sportscar', 'sports car', 'supercar'] },
    { id: 'Street', keywords: ['street', 'road', 'production'] }
  ];

  $effect(() => {
    // ⭐ On ne recharge que si le chemin AC a changé (le chargement initial est fait par AppLoadingScreen)
    if (appState.acPath && appState.dataLoadedForPath !== appState.acPath) {
      loadCars();
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
        loadCars();
      }, 2000);
    });

    return () => {
      if (unlistenPromise) unlistenPromise.then((unlisten: any) => unlisten());
    };
  });

  async function loadCars() {
    isLoading = true;
    try {
      const result = await invoke<any[]>('get_ac_cars', { acPath: appState.acPath });
      appState.cars = result;
      appState.dataLoadedForPath = appState.acPath;
    } catch (e) {
      console.error("Loading error:", e);
    } finally {
      isLoading = false;
    }
  }

  let allBrands = $derived.by(() => {
    const brands = new Set<string>();
    cars.forEach(c => {
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

    // (Variable allYears supprimée)

  // ⭐ Fonction pour deviner la catégorie CM d'une voiture
  function getCarCMCategory(car: any) {
    const carClass = (car.car_class || '').toLowerCase();
    const carTags = (car.tags || []).map((t: string) => t.toLowerCase());
    const allStrings = [carClass, ...carTags];

    for (const cat of CM_CATEGORIES) {
      for (const kw of cat.keywords) {
        if (allStrings.some(s => s.includes(kw))) {
          return cat.id;
        }
      }
    }
    return null; // Ne rentre dans aucune catégorie CM
  }

  let filteredCars = $derived(
    cars.filter(c => {
      if (appState.carFilters.type === 'kunos' && !c.is_kunos) return false;
      if (appState.carFilters.type === 'mods' && c.is_kunos) return false;
      if (appState.carFilters.type === 'favorites' && !appState.favorites.cars.includes(c.id)) return false;
      if (appState.carFilters.type === 'used' && !appState.used.cars[c.id]) return false;
      
      if (selectedBrand && c.brand !== selectedBrand) return false;
      
      if (selectedCountry) {
        const cCountry = getCountryName(c.tags);
        if (cCountry !== selectedCountry) return false;
      }

      // ⭐ Filtre par catégorie CM
      if (selectedClass) {
        const cmCat = getCarCMCategory(c);
        if (cmCat !== selectedClass) return false;
      }

            // (Filtre Année supprimé)

      // ⭐ Filtre par Puissance
      if (appState.carFilters.power) {
        const p = c.bhp;
        if (appState.carFilters.power === 'under_200' && p >= 200) return false;
        if (appState.carFilters.power === '200_400' && (p < 200 || p > 400)) return false;
        if (appState.carFilters.power === '400_600' && (p < 400 || p > 600)) return false;
        if (appState.carFilters.power === 'over_600' && p <= 600) return false;
      }
      
      if (appState.carFilters.tags.length > 0) {
        if (!c.tags || !appState.carFilters.tags.every((t: string) => c.tags.includes(t))) return false;
      }
      if (searchQuery && !c.name.toLowerCase().includes(searchQuery.toLowerCase())) return false;
      return true;
    })
  );

  function handleCarClick(car) {
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
  <!-- Recherche à gauche -->
  <div class="search-input" style="flex: 1; min-width: 250px; max-width: 400px;">
    <input type="text" placeholder={$t('cars.search_placeholder')} bind:value={searchQuery} style="width: 100%;" />
  </div>

  <!-- Filtres à droite -->
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

        <!-- Filtre Année supprimé -->

    <select class="manufacturer-select" bind:value={appState.carFilters.power} style="min-width: 160px; width: auto;">
      <option value="">{$t('cars.all_powers')}</option>
      <option value="under_200">{$t('cars.power_under_200')}</option>
      <option value="200_400">{$t('cars.power_200_400')}</option>
      <option value="400_600">{$t('cars.power_400_600')}</option>
      <option value="over_600">{$t('cars.power_over_600')}</option>
    </select>
  </div>
</div>
</div> <!-- ⭐ Fermeture du sticky-header-wrapper -->

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
      oncontextmenu={(e) => { 
        e.preventDefault(); 
        invoke('open_folder', { 
          acPath: appState.acPath, 
          documentsPath: appState.documentsPath, // ⭐ Ajout du paramètre manquant
          folderType: 'cars', 
          folderId: car.id 
        }).catch(err => console.error(err)); 
      }}
      role="button"
      tabindex="0"
      title={$t('cars.context_menu_open_folder')}
    >
      {#if brandLogo}
        <img src={brandLogo} class="brand-badge" alt="" onerror={(e) => e.currentTarget.style.display = 'none'} />
      {/if}

      {#if flagUrl}
        <div class="flag-badge"><img src={flagUrl} alt="" onerror={(e) => e.currentTarget.parentElement.style.display = 'none'} /></div>
      {/if}

      {#if car.image_path}
        <img src={convertFileSrc(car.image_path)} class="car-preview" loading="lazy" />
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