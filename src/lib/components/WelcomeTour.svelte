<script lang="ts">
  import { t } from 'svelte-i18n';
  import { setupState, setTutorialDisabled } from '$lib/stores/setupState.svelte.js';

  // ⭐ On reçoit isOpen et onFinish depuis App.svelte
  let { isOpen = false, onFinish = () => {} } = $props();

  let isHidden = $state(false);
  let currentSlide = $state(0);
  let disableTutorial = $state(setupState.tutorialDisabled);

  const SLIDES = [
    { icon: '🏎️', title: 'Voitures',     key: 'setup.tour.slides.cars',        tab: 'tab-cars', sub: '' },
    { icon: '🏁', title: 'Circuits',     key: 'setup.tour.slides.tracks',      tab: 'tab-tracks', sub: '' },
    { icon: '⚙️', title: 'Session',      key: 'setup.tour.slides.session',     tab: 'tab-session', sub: '' },
    { icon: '🌤️', title: 'Météo',        key: 'setup.tour.slides.weather',     tab: 'tab-graphics', sub: '' },
    { icon: '📦', title: 'Gestion Mods', key: 'setup.tour.slides.mods',        tab: 'tab-mods', sub: '' },
    { icon: '🌐', title: 'En ligne',     key: 'setup.tour.slides.online',      tab: 'tab-online', sub: '' },
    
    // ⭐ Sous-onglets des Paramètres
    { icon: '🛠️', title: 'Paramètres : Général', key: 'setup.tour.slides.settings_general', tab: 'tab-settings', sub: 'subtab-general' },
    { icon: '🎮', title: 'Paramètres : Contrôles', key: 'setup.tour.slides.settings_controls', tab: 'tab-settings', sub: 'subtab-controls' },
    { icon: '🖥️', title: 'Paramètres : Vidéo', key: 'setup.tour.slides.settings_video', tab: 'tab-settings', sub: 'subtab-video' },
    { icon: '🔊', title: 'Paramètres : Audio', key: 'setup.tour.slides.settings_audio', tab: 'tab-settings', sub: 'subtab-audio' },
    { icon: '🌩️', title: 'Paramètres : CSP', key: 'setup.tour.slides.settings_csp', tab: 'tab-settings', sub: 'subtab-csp' },
    { icon: '❓', title: 'Paramètres : Aide', key: 'setup.tour.slides.settings_help', tab: 'tab-settings', sub: 'subtab-help' },
    
    // ⭐ Personnalisation
    { icon: '🎨', title: 'Personnalisation', key: 'setup.tour.slides.customization', tab: null, sub: '' }
  ];

  function next() {
    if (currentSlide < SLIDES.length - 1) currentSlide++;
  }

  function prev() {
    if (currentSlide > 0) currentSlide--;
  }

  function finish() {
    setTutorialDisabled(disableTutorial);
    isOpen = false;
    onFinish();
  }

  function goToTab(tabId: string | null, subtabId: string = '') {
    if (!tabId) return;
    
    // On met à jour l'onglet principal (pour que App.svelte bascule bien)
    window.dispatchEvent(new CustomEvent('qarl-change-tab', { detail: { tab: tabId } }));
    
    // Si on a un sous-onglet (pour les paramètres), on déclenche l'événement pour SettingsTab.svelte
    if (subtabId) {
      window.dispatchEvent(new CustomEvent('qarl-open-section', {
        detail: { tab: subtabId, group: '', cspFile: null }
      }));
    }
    
    // On cache la visite pour laisser l'utilisateur voir l'onglet
    isHidden = true;
  }

  function showTourAgain() {
    isHidden = false;
  }

  $effect(() => {
    if (!isOpen) isHidden = false; // Reset quand on ferme complètement
  });
</script>

{#if isOpen && isHidden}
  <!-- Bandeau flottant discret -->
  <div class="setup-floating-banner" style="z-index: 9999;">
    <span class="msg">👋 Visite guidée en cours</span>
    <button class="primary" onclick={showTourAgain}>↗ Revenir à la visite</button>
    <button onclick={finish}>Passer</button>
  </div>
{/if}

{#if isOpen && !isHidden}
  <div class="setup-overlay" style="z-index: 9998;">
    <div class="setup-modal" style="max-width: 600px;">
      <div class="setup-header">
        <div class="setup-title">
          <span>👋</span>
          <span>{$t('setup.tour.title')}</span>
        </div>
        <button class="setup-close" onclick={finish} title={$t('common.close')}>✕</button>
      </div>

      <div class="setup-body" style="min-height: 250px; display: flex; flex-direction: column; justify-content: center;">
        <div class="setup-tour-slide">
          <div class="big-icon">{SLIDES[currentSlide].icon}</div>
          <div class="slide-title">{SLIDES[currentSlide].title}</div>
          <div class="slide-desc">{$t(SLIDES[currentSlide].key)}</div>

          <div class="setup-tour-dots" style="margin-top: 15px; margin-bottom: 15px;">
            {#each SLIDES as _, idx}
              <div class="setup-tour-dot" class:active={idx === currentSlide}></div>
            {/each}
          </div>

          {#if SLIDES[currentSlide].tab}
            <button class="filter-btn" onclick={() => goToTab(SLIDES[currentSlide].tab, SLIDES[currentSlide].sub)} style="margin: 0 auto; display: flex; align-items: center; gap: 8px;">
              🔍 Voir l'onglet
            </button>
          {/if}
        </div>
      </div>

      <div class="setup-footer" style="justify-content: space-between; padding-top: 15px; border-top: 1px solid var(--border);">
        <label style="display: flex; align-items: center; gap: 8px; font-size: 0.82rem; color: var(--text-secondary); cursor: pointer;">
          <input type="checkbox" bind:checked={disableTutorial} />
          {$t('setup.tour.disable_tutorial')}
        </label>
        
        <div style="display: flex; gap: 10px;">
          <button class="filter-btn" onclick={prev} disabled={currentSlide === 0} style="visibility: {currentSlide === 0 ? 'hidden' : 'visible'};">
            {$t('setup.tour.prev')}
          </button>
          {#if currentSlide < SLIDES.length - 1}
            <button class="filter-btn" onclick={next} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent);">
              {$t('setup.tour.next')}
            </button>
          {:else}
            <button class="filter-btn" onclick={finish} style="padding: 10px 22px; background: var(--success); color: #fff; border-color: var(--success); font-weight: 700;">
              {$t('setup.tour.finish')}
            </button>
          {/if}
        </div>
      </div>
    </div>
  </div>
{/if}