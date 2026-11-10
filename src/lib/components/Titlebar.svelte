<script>
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { appState } from '$lib/stores/appState.svelte.js';
  const appWindow = getCurrentWindow();
</script>

<div class="titlebar" data-tauri-drag-region>
  <div class="titlebar-brand">
    <img src="/assets/logo.png" alt="QARL" class="titlebar-logo" />
    <span class="titlebar-title">QARL - Quick Assetto Race Launcher</span>
  </div>
  <div class="titlebar-controls">
    <button 
      class="titlebar-btn" 
      id="tb-help" 
      title="Aide" 
      onclick={() => {
        appState.activeTab = 'tab-settings';
        window.dispatchEvent(new CustomEvent('open-help-tab'));
      }}
    >❓</button>
    <button class="titlebar-btn" onclick={(e) => { e.stopPropagation(); appWindow.minimize(); }}   title="Réduire">
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M0 5h10" stroke="currentColor" stroke-width="1.5"/></svg>
    </button>
    <button class="titlebar-btn" onclick={(e) => { e.stopPropagation(); appWindow.toggleMaximize(); }} title="Agrandir">
      <svg width="10" height="10" viewBox="0 0 10 10"><rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1.5"/></svg>
    </button>
    <button class="titlebar-btn titlebar-btn-close" onclick={(e) => { e.stopPropagation(); appWindow.close(); }} title="Fermer">
      <svg width="10" height="10" viewBox="0 0 10 10"><path d="M1 1l8 8M9 1L1 9" stroke="currentColor" stroke-width="1.5"/></svg>
    </button>
  </div>
</div>