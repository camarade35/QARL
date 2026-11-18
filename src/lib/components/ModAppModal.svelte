<script lang="ts">
  import { t } from 'svelte-i18n';

  let {
    show,
    installed,
    enabled,
    volumePercent,
    error,
    onClose,
    onDownload,
    onEnabledChange,
    onVolumeChange
  } = $props<{
    show: boolean;
    installed: boolean;
    enabled: boolean;
    volumePercent: number;
    error: string;
    onClose: () => void;
    onDownload: () => void;
    onEnabledChange: (enabled: boolean) => void;
    onVolumeChange: (volume: number) => void;
  }>();
</script>

{#if show}
  <div class="modal visible" style="display: flex; z-index: 1100;">
    <div class="modal-content crashfx-modal">
      <div class="modal-header">
        <h3 class="modal-title">
          <img src="/assets/CrashFX/logo.png" alt="CrashFX" />
          by JWL
        </h3>
        <button class="modal-close" onclick={onClose} aria-label={$t('common.close')}>&times;</button>
      </div>

      <p class="crashfx-description">{$t('mods.crashfx_description')}</p>

      <div class="crashfx-gallery">
        <img src="/assets/CrashFX/CrashFX_1.png" alt={$t('mods.crashfx_image_alt')} />
        <img src="/assets/CrashFX/CrashFX_2.png" alt={$t('mods.crashfx_image_alt')} />
        <img src="/assets/CrashFX/CrashFX_3.png" alt={$t('mods.crashfx_image_alt')} />
      </div>

      {#if installed}
        <div class="crashfx-control">
          <label>
            <span>{$t('mods.crashfx_enabled')}</span>
            <input
              type="checkbox"
              checked={enabled}
              onchange={(event) => onEnabledChange(event.currentTarget.checked)}
            />
          </label>

          <label>
            <span>{$t('mods.crashfx_volume', { values: { volume: volumePercent } })}</span>
            <input
              type="range"
              min="0"
              max="200"
              step="1"
              value={volumePercent}
              onchange={(event) => onVolumeChange(Number(event.currentTarget.value))}
            />
          </label>
        </div>
      {:else}
        <button class="filter-btn" onclick={onDownload}>
          {$t('mods.crashfx_download')}
        </button>
      {/if}

      {#if error}
        <p class="crashfx-error">{$t('common.error')}: {error}</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .crashfx-modal {
    width: 760px;
    max-width: 94vw;
    max-height: 90vh;
    overflow-y: auto;
  }

  .crashfx-modal .modal-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .crashfx-modal .modal-title img {
    width: 100px;
    max-height: 34px;
    object-fit: contain;
  }

  .crashfx-description {
    margin: 0 0 16px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent);
    border-radius: 8px;
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-size: 0.9rem;
    line-height: 1.55;
  }

  .crashfx-gallery {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 8px;
  }

  .crashfx-gallery img {
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
    border: 1px solid var(--border);
    border-radius: 6px;
  }

  .crashfx-control {
    display: grid;
    gap: 14px;
    margin-top: 18px;
  }

  .crashfx-control label {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
  }

  .crashfx-control input[type='range'] {
    width: 55%;
  }

  .crashfx-error {
    margin-top: 12px;
    color: var(--danger, #f87171);
    overflow-wrap: anywhere;
  }

  @media (max-width: 600px) {
    .crashfx-gallery {
      grid-template-columns: 1fr;
    }
  }
</style>