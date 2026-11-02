<script lang="ts">
  import { t } from 'svelte-i18n';
  import { setupState, setTutorialDisabled } from '$lib/stores/setupState.svelte.js';

  let { onContinue = () => {} } = $props();

  const SLIDES = [
    { icon: '🏎️', title: 'Voitures',     key: 'setup.tour.slides.cars' },
    { icon: '🏁', title: 'Circuits',     key: 'setup.tour.slides.tracks' },
    { icon: '⚙️', title: 'Session',      key: 'setup.tour.slides.session' },
    { icon: '🌤️', title: 'Météo',        key: 'setup.tour.slides.weather' },
    { icon: '📦', title: 'Gestion Mods', key: 'setup.tour.slides.mods' },
    { icon: '🌐', title: 'En ligne',     key: 'setup.tour.slides.online' },
    { icon: '⚙️', title: 'Paramètres',   key: 'setup.tour.slides.settings' },
  ];

  let currentSlide = $state(0);
  let disableTutorial = $state(setupState.tutorialDisabled);

  function next() {
    if (currentSlide < SLIDES.length - 1) currentSlide++;
  }

  function prev() {
    if (currentSlide > 0) currentSlide--;
  }

  function finish() {
    setTutorialDisabled(disableTutorial);
    onContinue();
  }
</script>

<div class="setup-tour-slide">
  <div class="big-icon">{SLIDES[currentSlide].icon}</div>
  <div class="slide-title">{SLIDES[currentSlide].title}</div>
  <div class="slide-desc">{$t(SLIDES[currentSlide].key)}</div>

  <div class="setup-tour-dots">
    {#each SLIDES as _, idx}
      <div class="setup-tour-dot" class:active={idx === currentSlide}></div>
    {/each}
  </div>
</div>

<div style="display: flex; justify-content: space-between; align-items: center; margin-top: 20px;">
  <label style="display: flex; align-items: center; gap: 8px; font-size: 0.82rem; color: var(--text-secondary); cursor: pointer;">
    <input type="checkbox" bind:checked={disableTutorial} />
    {$t('setup.tour.disable_tutorial')}
  </label>
  <div style="display: flex; gap: 10px;">
    <button class="filter-btn" onclick={prev} disabled={currentSlide === 0}>
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