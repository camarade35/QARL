<script lang="ts">
  import { t } from 'svelte-i18n';
  import { setupState, goToStep, closeSetup, markSetupCompleted, showAfterRedirect, hideForRedirect } from '$lib/stores/setupState.svelte.js';
  import Step0Paths from './setup/Step0Paths.svelte';
  import Step1AcInit from './setup/Step1AcInit.svelte';
  import Step2Devices from './setup/Step2Devices.svelte';
  import Step3Identity from './setup/Step3Identity.svelte';
  import Step4Csp from './setup/Step4Csp.svelte';
  import Step6Recap from './setup/Step6Recap.svelte';

  const STEPS = [
    { id: 'paths',     key: 'setup.steps.paths',     blocking: true  },
    { id: 'ac_init',   key: 'setup.steps.ac_init',   blocking: true  },
    { id: 'devices',   key: 'setup.steps.devices',   blocking: false },
    { id: 'identity',  key: 'setup.steps.identity',  blocking: true  },
    { id: 'csp',       key: 'setup.steps.csp',       blocking: false },
    { id: 'recap',     key: 'setup.steps.recap',     blocking: false },
  ];

  let totalSteps = STEPS.length;
  let currentIdx = $derived(setupState.currentStep);

  function handleContinue() {
    if (currentIdx < totalSteps - 1) {
      goToStep(currentIdx + 1);
    } else {
      markSetupCompleted();
    }
  }

  function handleBack() {
    if (currentIdx > 0) goToStep(currentIdx - 1);
  }

  function handleClose() {
    // Si on ferme manuellement (skip global), on marque comme complété
    markSetupCompleted();
  }

  // Écoute les redirections "Ouvrir Contrôles" depuis Step2Devices
  $effect(() => {
    const handler = () => hideForRedirect();
    window.addEventListener('qarl-setup-hide', handler as EventListener);
    return () => window.removeEventListener('qarl-setup-hide', handler as EventListener);
  });
</script>

{#if setupState.isOpen && setupState.hiddenForRedirect}
  <!-- Bandeau flottant discret -->
  <div class="setup-floating-banner">
    <span class="msg">🏗️ {$t('setup.devices.floating_banner')}</span>
    <button class="primary" onclick={showAfterRedirect}>↗ {$t('setup.devices.floating_reopen')}</button>
    <button onclick={handleClose}>{$t('setup.devices.floating_ignore')}</button>
  </div>
{/if}

{#if setupState.isOpen && !setupState.hiddenForRedirect}
  <div class="setup-overlay">
    <div class="setup-modal">
      <div class="setup-header">
        <div class="setup-title">
          <span>🏗️</span>
          <span>{$t('setup.title')}</span>
        </div>
        <button class="setup-close" onclick={handleClose} title={$t('common.close')}>✕</button>
      </div>

      <div class="setup-steps">
        {#each STEPS as step, idx}
          {#if idx > 0}
            <div class="setup-step-sep"></div>
          {/if}
          <div
            class="setup-step-dot"
            class:active={idx === currentIdx}
            class:done={idx < currentIdx}
          >
            <span class="num">{idx < currentIdx ? '✓' : idx + 1}</span>
            <span>{$t(step.key)}</span>
          </div>
        {/each}
      </div>

      <div class="setup-body">
        {#if currentIdx === 0}
          <Step0Paths onContinue={handleContinue} />
        {:else if currentIdx === 1}
          <Step1AcInit onContinue={handleContinue} />
        {:else if currentIdx === 2}
          <Step2Devices onContinue={handleContinue} />
        {:else if currentIdx === 3}
          <Step3Identity onContinue={handleContinue} />
        {:else if currentIdx === 4}
          <Step4Csp onContinue={handleContinue} />
        {:else if currentIdx === 5}
          <Step6Recap onFinish={markSetupCompleted} />
        {/if}
      </div>

      <div class="setup-footer">
        <button
          class="filter-btn"
          onclick={handleBack}
          disabled={currentIdx === 0}
          style={currentIdx === 0 ? 'visibility: hidden;' : ''}
        >
          ← {$t('common.cancel')}
        </button>
        <div class="spacer"></div>
        {#if currentIdx < totalSteps - 1}
          <button class="filter-btn" onclick={handleClose} style="margin-right: 8px;">
            {$t('setup.welcome.skip')}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}