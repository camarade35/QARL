<!-- src/lib/components/Dialog.svelte -->
<script lang="ts">
  import { t } from 'svelte-i18n';
  import { dialogState } from '$lib/stores/dialog.svelte.js';
  import ReleaseNotesPreview from '$lib/components/ReleaseNotesPreview.svelte';

  function handleOk() {
    dialogState.resolve(dialogState.type === 'prompt' ? dialogState.inputValue : true);
    dialogState.isOpen = false;
  }

  function handleCancel() {
    dialogState.resolve(dialogState.type === 'prompt' ? null : false);
    dialogState.isOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && dialogState.type !== 'update') handleOk();
    if (e.key === 'Escape') handleCancel();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if dialogState.isOpen}
  <div class="qacrl-dialog-overlay visible" onclick={(e) => { if(e.target === e.currentTarget) handleCancel(); }}>
    <div class="qacrl-dialog qacrl-dialog-{dialogState.variant} {dialogState.type === 'update' ? 'qacrl-dialog-update' : ''}">
      
      <div class="qacrl-dialog-header">
        <div class="qacrl-dialog-icon">
          {#if dialogState.variant === 'warning'}⚠️
          {:else if dialogState.variant === 'error'}❌
          {:else if dialogState.variant === 'success'}✅
          {:else}ℹ️{/if}
        </div>
        <div class="qacrl-dialog-title">
          {dialogState.title || (dialogState.type === 'confirm' ? $t('dialog.confirm') : dialogState.type === 'prompt' ? $t('dialog.prompt') : $t('dialog.info'))}
        </div>
      </div>

      <div class="qacrl-dialog-body">
        {#if dialogState.type === 'update'}
          <p class="qacrl-update-summary">{dialogState.message}</p>
          <div class="qacrl-update-notes-label">{dialogState.notesLabel}</div>
          {#if dialogState.notes.trim()}
            <ReleaseNotesPreview notes={dialogState.notes} />
          {:else}
            <p>{dialogState.emptyNotesMessage}</p>
          {/if}
        {:else}
          <p style="white-space: pre-wrap;">{dialogState.message}</p>
        {/if}
        
        {#if dialogState.type === 'prompt'}
          <div class="qacrl-dialog-input" style="margin-top: 12px;">
            <input 
              type="text" 
              class="qacrl-dialog-input-field" 
              bind:value={dialogState.inputValue}
            />
          </div>
        {/if}
      </div>

      <div class="qacrl-dialog-footer">
        {#if dialogState.type !== 'alert'}
          <button class="filter-btn qacrl-dialog-btn-cancel" onclick={handleCancel}>
            {dialogState.cancelLabel || $t('dialog.cancel')}
          </button>
        {/if}
        <button class="filter-btn qacrl-dialog-btn-ok" onclick={handleOk}>
          {dialogState.okLabel || $t('dialog.ok')}
        </button>
      </div>

    </div>
  </div>
{/if}