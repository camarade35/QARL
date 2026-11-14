<script lang="ts">
  import { t } from 'svelte-i18n';
  import { setupState } from '$lib/stores/setupState.svelte.js';
  import { appState } from '$lib/stores/appState.svelte.js';

  let { onFinish = () => {} } = $props();

  function truncate(s: string, len: number = 40) {
    if (!s) return '—';
    return s.length > len ? '…' + s.slice(-len) : s;
  }
</script>

<div style="display: flex; flex-direction: column; gap: 12px; padding: 8px 0;">
  <div style="display: flex; justify-content: space-between; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
    <span style="font-weight: 600; color: var(--text-secondary);">{$t('setup.recap.ac_path')}</span>
    <span style="font-weight: 600; text-align: right; word-break: break-all;">{truncate(setupState.data.acPath)}</span>
  </div>
  <div style="display: flex; justify-content: space-between; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
    <span style="font-weight: 600; color: var(--text-secondary);">{$t('setup.recap.documents')}</span>
    <span style="font-weight: 600; text-align: right; word-break: break-all;">{truncate(setupState.data.documentsPath)}</span>
  </div>
  <div style="display: flex; justify-content: space-between; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
    <span style="font-weight: 600; color: var(--text-secondary);">{$t('setup.recap.driver_name')}</span>
    <span style="font-weight: 600;">{setupState.data.driverName || '—'}</span>
  </div>
  <div style="display: flex; justify-content: space-between; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
    <span style="font-weight: 600; color: var(--text-secondary);">{$t('setup.recap.steam_id')}</span>
    <span style="font-weight: 600;">{setupState.data.steamId || $t('setup.recap.missing')}</span>
  </div>
  <div style="display: flex; justify-content: space-between; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
    <span style="font-weight: 600; color: var(--text-secondary);">{$t('setup.recap.nationality')}</span>
    <span style="font-weight: 600;">{setupState.data.nationality || '—'}</span>
  </div>
  <div style="display: flex; justify-content: space-between; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
    <span style="font-weight: 600; color: var(--text-secondary);">{$t('setup.recap.content')}</span>
    <span style="font-weight: 600;">
      {$t('setup.recap.content_value', { values: { cars: setupState.data.carsCount || 0, tracks: setupState.data.tracksCount || 0 } })}
    </span>
  </div>
  <div style="display: flex; justify-content: space-between; padding: 10px 14px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm);">
    <span style="font-weight: 600; color: var(--text-secondary);">{$t('setup.recap.csp_weather')}</span>
    <span style="font-weight: 600;">
      {#if setupState.data.hasCsp && (setupState.data.hasSol || setupState.data.hasPure)}
        ✅ CSP + {setupState.data.hasPure ? 'Pure' : 'Sol'}
      {:else if setupState.data.hasCsp}
        ⚠️ {$t('setup.recap.csp_only')}
      {:else}
        ❌ {$t('setup.recap.missing')}
      {/if}
    </span>
  </div>
</div>

<div style="display: flex; justify-content: center; margin-top: 24px;">
  <button class="filter-btn" onclick={onFinish} style="padding: 12px 26px; background: var(--success); color: #fff; border-color: var(--success); font-weight: 700; font-size: 0.95rem;">
    {$t('setup.recap.start')}
  </button>
</div>