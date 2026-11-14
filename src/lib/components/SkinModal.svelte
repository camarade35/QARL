<script lang="ts">
  import { appState } from '$lib/stores/appState.svelte.js';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { t } from 'svelte-i18n';

  function selectSkin(skin) {
    appState.selectedSkin = skin;
    appState.showSkinModal = false;
  }
</script>

{#if appState.showSkinModal && appState.selectedCar}
  <div class="modal visible" style="display: flex;">
    <div class="modal-content" style="width: 800px;">
      <div class="modal-header">
        <h3 class="modal-title">{$t('selection.skin_title')} - {appState.selectedCar.name}</h3>
        <button class="modal-close" onclick={() => appState.showSkinModal = false}>&times;</button>
      </div>
      
      <div class="modal-grid" style="max-height: 60vh;">
        {#each appState.selectedCar.skins as skin}
          <div 
            class="skin-select-row" 
            class:selected={appState.selectedSkin?.name === skin.name}
            onclick={() => selectSkin(skin)}
          >
            <img src={convertFileSrc(skin.image_path)} class="skin-select-preview" alt={skin.name} />
            <div class="skin-select-name">{skin.name}</div>
          </div>
        {/each}
      </div>
    </div>
  </div>
{/if}