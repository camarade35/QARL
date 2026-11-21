<script>
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { getCountryName } from '$lib/utils/assets.js';
  import { CM_CATEGORIES, matchesCarFilters } from '$lib/utils/carFilters.js';
  import { t } from 'svelte-i18n'; 

  /** @typedef {import('$lib/utils/carFilters.js').CarFilterable & { image_path?: string, skins?: { name: string }[] }} GridCar */

  /** @type {GridCar[]} */
  let gridCars = $derived(appState.cars);
  let gridCarSearch = $state('');
  let gridCarType = $state('all');
  let gridCarBrand = $state('');
  let gridCarCountry = $state('');
  let gridCarClass = $state('');
  let gridCarPower = $state('');
  let gridCarTags = /** @type {string[]} */ ($state([]));
  let showGridCarTags = $state(false);

  let gridCarBrands = $derived.by(() => {
    const brands = new Set(gridCars
      .map(car => car.brand)
      .filter(brand => brand && brand !== 'Mod'));
    return Array.from(brands).sort();
  });
  let gridCarCountries = $derived.by(() => {
    const countries = /** @type {Set<string>} */ (new Set());
    for (const car of gridCars) {
      const country = getCountryName(car.tags);
      if (country) countries.add(country);
    }
    return Array.from(countries).sort();
  });
  let gridCarAvailableTags = $derived(
    Array.from(new Set(gridCars.flatMap(car => car.tags || []))).sort()
  );
  let filteredGridCars = $derived(
    gridCars.filter(car => matchesCarFilters(car, {
      type: gridCarType,
      brand: gridCarBrand,
      country: gridCarCountry,
      carClass: gridCarClass,
      power: gridCarPower,
      tags: gridCarTags,
      search: gridCarSearch,
      favoriteIds: appState.favorites.cars,
      usedCars: appState.used.cars
    }))
  );

  function closeModal() {
    appState.modalOpen = false;
    appState.modalType = null;
    appState.modalData = null;
    appState.modalContext = null; // ⭐ On nettoie le contexte
  }

  function selectLayout(layout) {
    appState.selectedTrackLayout = layout;
    closeModal();
  }

  function selectSkin(skin) {
    appState.selectedCarSkin = skin;
    closeModal();
  }

  // ⭐ NOUVEAU : Sélection d'une voiture pour la grille personnalisée
  /** @param {GridCar} car */
  function selectCarForCustomGrid(car) {
    if (appState.modalContext && appState.modalContext.rowIndex !== undefined) {
      const row = appState.session.customGrid[appState.modalContext.rowIndex];
      if (row) {
        row.carId = car.id;
        // On sélectionne automatiquement la première skin de la voiture
        row.skinId = car.skins && car.skins.length > 0 ? car.skins[0].name : '';
        
        // ⭐ Sécurité Svelte 5 : on force la réassignation du tableau pour que l'UI se mette à jour
        appState.session.customGrid = [...appState.session.customGrid];
      }
    }
    closeModal();
  }

  /** @param {string} tag */
  function toggleGridCarTag(tag) {
    gridCarTags = gridCarTags.includes(tag)
      ? gridCarTags.filter(selectedTag => selectedTag !== tag)
      : [...gridCarTags, tag];
  }
</script>

{#if appState.modalOpen && (appState.modalData || appState.modalType === 'custom_grid_car')}
  <div class="modal visible" id="selection-modal">
    <div class="modal-content selection-modal-content">
      <div class="modal-header">
        <h3 class="modal-title">
          {#if appState.modalType === 'track' && appState.modalData}
            {$t('selection.track_title')} : {appState.modalData.name}
          {:else if appState.modalType === 'car' && appState.modalData}
            {$t('selection.skin_title')} : {appState.modalData.name}
          {:else if appState.modalType === 'custom_grid_car'}
            {$t('session.race_ia.custom_grid_choose_car')}
          {/if}
        </h3>
        <button class="modal-close" onclick={closeModal}>&times;</button>
      </div>

      {#if appState.modalType === 'custom_grid_car'}
        <div class="custom-grid-car-picker">
          <div class="custom-grid-car-type-filters">
            <button type="button" class="filter-btn" class:active={gridCarType === 'all'} onclick={() => gridCarType = 'all'}>{$t('common.filter_all')}</button>
            <button type="button" class="filter-btn" class:active={gridCarType === 'kunos'} onclick={() => gridCarType = 'kunos'}>{$t('common.filter_kunos')}</button>
            <button type="button" class="filter-btn" class:active={gridCarType === 'mods'} onclick={() => gridCarType = 'mods'}>{$t('common.filter_mods')}</button>
            <button type="button" class="filter-btn" class:active={gridCarType === 'favorites'} onclick={() => gridCarType = 'favorites'}>{$t('common.filter_favorites')}</button>
            <button type="button" class="filter-btn" class:active={gridCarType === 'used'} onclick={() => gridCarType = 'used'}>{$t('common.filter_used')}</button>
            <button type="button" class="filter-btn" class:active={showGridCarTags} onclick={() => showGridCarTags = !showGridCarTags}>{$t('common.filter_tags', { values: { count: gridCarTags.length } })}</button>
          </div>

          <div class="custom-grid-car-fields">
            <input class="custom-grid-car-search" type="search" placeholder={$t('cars.search_placeholder')} bind:value={gridCarSearch} />
            <select class="manufacturer-select" bind:value={gridCarBrand}>
              <option value="">{$t('cars.all_brands')}</option>
              {#each gridCarBrands as brand}
                <option value={brand}>{brand}</option>
              {/each}
            </select>
            <select class="manufacturer-select" bind:value={gridCarCountry}>
              <option value="">{$t('cars.all_countries')}</option>
              {#each gridCarCountries as country}
                <option value={country}>{$t(country, { default: country })}</option>
              {/each}
            </select>
            <select class="manufacturer-select" bind:value={gridCarClass}>
              <option value="">{$t('cars.all_classes')}</option>
              {#each CM_CATEGORIES as category}
                <option value={category.id}>{$t(`cars.categories.${category.id}`, { default: category.id })}</option>
              {/each}
            </select>
            <select class="manufacturer-select" bind:value={gridCarPower}>
              <option value="">{$t('cars.all_powers')}</option>
              <option value="under_200">{$t('cars.power_under_200')}</option>
              <option value="200_400">{$t('cars.power_200_400')}</option>
              <option value="400_600">{$t('cars.power_400_600')}</option>
              <option value="over_600">{$t('cars.power_over_600')}</option>
            </select>
          </div>

          {#if showGridCarTags}
            <div class="custom-grid-car-tags">
              {#each gridCarAvailableTags as tag}
                <button type="button" class="filter-btn" class:active={gridCarTags.includes(tag)} onclick={() => toggleGridCarTag(tag)}>{tag}</button>
              {/each}
              {#if gridCarTags.length > 0}
                <button type="button" class="filter-btn" onclick={() => gridCarTags = []}>{$t('common.clear')}</button>
              {/if}
            </div>
          {/if}
        </div>
      {/if}

      <div
        class="modal-grid selection-modal-grid"
        class:custom-grid-car-list={appState.modalType === 'custom_grid_car'}
      >
        {#if appState.modalType === 'custom_grid_car'}
          {#each filteredGridCars as car}
            <div 
              class="card custom-grid-car-card"
              onclick={() => selectCarForCustomGrid(car)}
              onkeydown={(event) => event.key === 'Enter' && selectCarForCustomGrid(car)}
              role="button"
              tabindex="0"
            >
              <div class="track-preview-box custom-grid-car-preview">
                {#if car.image_path}
                  <img class="track-bg" src={convertFileSrc(car.image_path)} alt="" onload={(e) => e.currentTarget.classList.add('loaded')} />
                {:else}
                  <div style="display: flex; align-items: center; justify-content: center; height: 100%; font-size: 2rem;">🚗</div>
                {/if}
              </div>
              <div class="card-title" style="padding: 6px 10px;">
                <span class="card-title-text" style="white-space: normal; line-height: 1.2; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; max-width: 100%;">
                  {car.name}
                </span>
              </div>
            </div>
          {:else}
            <div class="custom-grid-car-empty">{$t('cars.empty')}</div>
          {/each}
        {:else if appState.modalType === 'track' && appState.modalData}
          {#each appState.modalData.layouts as layout}
            <div 
              class="card selection-layout-card"
              class:selected={appState.selectedTrackLayout?.id === layout.id}
              onclick={() => selectLayout(layout)}
              role="button"
              tabindex="0"
            >
              <div class="track-preview-box selection-layout-preview">
                {#if layout.preview_path}
                  <img class="track-bg" src={convertFileSrc(layout.preview_path)} alt="" onload={(e) => e.currentTarget.classList.add('loaded')} />
                {/if}
                {#if layout.outline_path}
                  <img class="track-outline" src={convertFileSrc(layout.outline_path)} alt="" onload={(e) => e.currentTarget.classList.add('loaded')} />
                {/if}
              </div>
              <!-- ⭐ Texte sur 2 lignes max, centré -->
              <div class="card-title" style="padding: 6px 10px;">
                <span class="card-title-text" style="white-space: normal; line-height: 1.2; display: -webkit-box; -webkit-line-clamp: 2; -webkit-box-orient: vertical; overflow: hidden; max-width: 100%;">
                  {layout.name}
                </span>
              </div>
            </div>
          {/each}
        {:else if appState.modalType === 'car' && appState.modalData}
          {#each appState.modalData.skins as skin}
            <div 
              class="skin-select-row" 
              class:selected={appState.selectedCarSkin?.name === skin.name}
              onclick={() => selectSkin(skin)}
              role="button"
              tabindex="0"
            >
              {#if skin.image_path}
                <img class="skin-select-preview" src={convertFileSrc(skin.image_path)} alt="" />
              {:else}
                <div class="skin-select-preview placeholder">🚗</div>
              {/if}
              <div class="skin-select-name">{skin.name}</div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}