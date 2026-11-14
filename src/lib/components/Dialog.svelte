<!-- src/lib/components/Dialog.svelte -->
<script lang="ts">
  import { t } from 'svelte-i18n';
  import { dialogState } from '$lib/stores/dialog.svelte.js';

  function handleOk() {
    dialogState.resolve(dialogState.type === 'prompt' ? dialogState.inputValue : true);
    dialogState.isOpen = false;
  }

  function handleCancel() {
    dialogState.resolve(dialogState.type === 'prompt' ? null : false);
    dialogState.isOpen = false;
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter') handleOk();
    if (e.key === 'Escape') handleCancel();
  }
</script>

<svelte:window onkeydown={handleKeydown} />

{#if dialogState.isOpen}
  <div class="qacrl-dialog-overlay visible" onclick={(e) => { if(e.target === e.currentTarget) handleCancel(); }}>
    <div class="qacrl-dialog qacrl-dialog-{dialogState.variant}">
      
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
        <p style="white-space: pre-wrap;">{dialogState.message}</p>
        
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