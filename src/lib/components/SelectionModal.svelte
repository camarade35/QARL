<script>
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { t } from 'svelte-i18n'; 

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
</script>

{#if appState.modalOpen && (appState.modalData || appState.modalType === 'custom_grid_car')}
  <div class="modal visible" id="selection-modal">
    <div class="modal-content">
      <div class="modal-header">
        <h3 class="modal-title">
          {#if appState.modalType === 'track'}
            {$t('selection.track_title')} : {appState.modalData.name}
          {:else if appState.modalType === 'car'}
            {$t('selection.skin_title')} : {appState.modalData.name}
          {:else if appState.modalType === 'custom_grid_car'}
            {$t('session.race_ia.custom_grid_choose_car')}
          {/if}
        </h3>
        <button class="modal-close" onclick={closeModal}>&times;</button>
      </div>
      
      <div class="modal-grid">
        {#if appState.modalType === 'custom_grid_car'}
          {#each appState.cars as car}
            <div 
              class="card" 
              onclick={() => selectCarForCustomGrid(car)}
              role="button"
              tabindex="0"
              style="height: 150px;"
            >
              <div class="track-preview-box" style="height: 100px; background: #0f1219;">
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
          {/each}
        {:else if appState.modalType === 'track'}
          {#each appState.modalData.layouts as layout}
            <div 
              class="card" 
              class:selected={appState.selectedTrackLayout?.id === layout.id}
              onclick={() => selectLayout(layout)}
              role="button"
              tabindex="0"
              style="height: 150px;"
            >
              <div class="track-preview-box" style="height: 100px; background: #0f1219;">
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
        {:else if appState.modalType === 'car'}
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