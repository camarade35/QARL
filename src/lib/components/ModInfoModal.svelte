<script lang="ts">
  import { t } from 'svelte-i18n';

  let {
    show,
    title,
    credit = '',
    logo,
    images,
    description,
    installed,
    downloadLinks,
    error,
    onClose,
    onDownload
  }: {
    show: boolean;
    title: string;
    credit?: string;
    logo: string;
    images: string[];
    description: string;
    installed: boolean;
    downloadLinks: { label: string; url: string }[];
    error: string;
    onClose: () => void;
    onDownload: (url: string) => void | Promise<void>;
  } = $props();
</script>

{#if show}
  <div class="modal visible" style="display: flex; z-index: 1100;">
    <div class="modal-content app-info-modal">
      <div class="modal-header">
        <h3 class="modal-title">
          <img src={logo} alt={title} />
          <span>{title}</span>
          {#if credit}<span class="app-info-credit">{credit}</span>{/if}
        </h3>
        <button class="modal-close" onclick={onClose} aria-label={$t('common.close')}>&times;</button>
      </div>

      <p class="app-info-description">{description}</p>

      <div class="app-info-gallery">
        {#each images as image}
          <img src={image} alt={$t('mods.app_image_alt', { values: { name: title } })} />
        {/each}
      </div>

      {#if installed}
        <p class="app-info-installed">{$t('mods.app_installed')}</p>
      {:else}
        <div class="app-info-downloads">
          {#each downloadLinks as link}
            <button class="filter-btn" onclick={() => onDownload(link.url)}>
              {link.label}
            </button>
          {/each}
        </div>
      {/if}

      {#if error}
        <p class="app-info-error">{$t('common.error')}: {error}</p>
      {/if}
    </div>
  </div>
{/if}

<style>
  .app-info-modal {
    width: 760px;
    max-width: 94vw;
    max-height: 90vh;
    overflow-y: auto;
  }

  .app-info-modal .modal-title {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .app-info-modal .modal-title img {
    width: 100px;
    max-height: 34px;
    object-fit: contain;
  }

  .app-info-credit {
    color: var(--text-secondary);
    font-size: 0.8rem;
    font-weight: 400;
  }

  .app-info-description {
    margin: 0 0 16px;
    padding: 12px 14px;
    border: 1px solid var(--border);
    border-left: 3px solid var(--accent);
    border-radius: 8px;
    background: var(--bg-elevated);
    color: var(--text-secondary);
    font-size: 0.9rem;
    line-height: 1.55;
    white-space: pre-line;
  }

  .app-info-gallery {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
    gap: 8px;
  }

  .app-info-gallery img {
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
    border: 1px solid var(--border);
    border-radius: 6px;
  }

  .app-info-installed {
    margin: 16px 0 0;
    color: var(--accent);
    font-weight: 600;
  }

  .app-info-downloads {
    display: flex;
    flex-wrap: wrap;
    gap: 8px;
    margin-top: 16px;
  }

  .app-info-error {
    margin-top: 12px;
    color: var(--danger, #f87171);
    overflow-wrap: anywhere;
  }

  @media (max-width: 600px) {
    .app-info-gallery {
      grid-template-columns: 1fr;
    }
  }
</style>
