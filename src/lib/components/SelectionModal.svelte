<script>
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { t } from 'svelte-i18n'; 

  function closeModal() {
    appState.modalOpen = false;
    appState.modalType = null;
    appState.modalData = null;
  }

  function selectLayout(layout) {
    appState.selectedTrackLayout = layout;
    closeModal();
  }

  function selectSkin(skin) {
    appState.selectedCarSkin = skin;
    closeModal();
  }
</script>

{#if appState.modalOpen && appState.modalData}
  <div class="modal visible" id="selection-modal">
    <div class="modal-content">
      <div class="modal-header">
        <h3 class="modal-title">
          {appState.modalType === 'track' ? $t('selection.track_title') : $t('selection.skin_title')}
          : {appState.modalData.name}
        </h3>
        <button class="modal-close" onclick={closeModal}>&times;</button>
      </div>
      
      <div class="modal-grid">
        {#if appState.modalType === 'track'}
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
                  <img class="track-bg" src={convertFileSrc(layout.preview_path)} alt="" />
                {/if}
                {#if layout.outline_path}
                  <img class="track-outline" src={convertFileSrc(layout.outline_path)} alt="" />
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