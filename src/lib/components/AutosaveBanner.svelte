<script lang="ts">
  import { t, locale } from 'svelte-i18n';
  import { searchState, setAutosaveDelay } from '$lib/stores/searchState.svelte.js';

  let {
    hasChanges = false,
    onSave = async () => {},
    label = ''
  } = $props<{ hasChanges: boolean; onSave: () => Promise<void>; label?: string }>();

  let saving = $state(false);
  let justSaved = $state(false);
  let savedAt = $state('');
  let countdown = $state(0);
  let intervalId: ReturnType<typeof setInterval> | null = null;

  export async function doSave() {
    if (saving) return;
    saving = true;
    try {
      await onSave();
      justSaved = true;
      savedAt = new Date().toLocaleTimeString($locale, { hour: '2-digit', minute: '2-digit' });
      setTimeout(() => { justSaved = false; }, 3000);
    } finally {
      saving = false;
    }
  }

  function onDelayChange(e: Event) {
    const v = Number((e.currentTarget as HTMLSelectElement).value) || 0;
    setAutosaveDelay(v);
  }

  $effect(() => {
    const delay = searchState.autosaveDelay;
    if (hasChanges && delay > 0) {
      countdown = delay;
      if (intervalId) clearInterval(intervalId);
      intervalId = setInterval(() => {
        countdown = Math.max(0, countdown - 1);
        if (countdown === 0 && intervalId) clearInterval(intervalId);
      }, 1000);
    } else {
      countdown = 0;
      if (intervalId) clearInterval(intervalId);
    }
    return () => { if (intervalId) clearInterval(intervalId); };
  });
</script>

{#if hasChanges || justSaved}
  <div
    class="settings-search-save-banner"
    class:is-pending={hasChanges && !justSaved}
    class:is-saved={justSaved}
  >
    <span class="settings-search-save-status">
      {#if justSaved}
        {$t('settings.autosave.banner_saved', { values: { time: savedAt } })}
      {:else if hasChanges}
        {$t('settings.autosave.banner_pending')}
        {#if searchState.autosaveDelay > 0 && countdown > 0}
          — {$t('settings.autosave.banner_autosave_in', { values: { seconds: countdown } })}
        {/if}
      {/if}
    </span>

    <div class="settings-search-delay-wrap">
      <label for="qarl-autosave-delay">{$t('settings.autosave.delay_label')}</label>
      <select
        id="qarl-autosave-delay"
        class="settings-search-delay-select"
        value={searchState.autosaveDelay}
        onchange={onDelayChange}
      >
        <option value={0}>{$t('settings.autosave.delay_manual')}</option>
        <option value={2}>{$t('settings.autosave.delay_2s')}</option>
        <option value={5}>{$t('settings.autosave.delay_5s')}</option>
        <option value={10}>{$t('settings.autosave.delay_10s')}</option>
      </select>
    </div>

    <button class="settings-search-save-btn" onclick={doSave} disabled={saving}>
      {saving ? '⏳' : '💾'} {$t('settings.autosave.save_now')}
    </button>
  </div>
{/if}