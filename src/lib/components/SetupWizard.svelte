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
      finishSetup();
    }
  }

  // ⭐ NOUVELLE FONCTION : Appelée par Step6Recap ET par handleContinue
  async function finishSetup() {
    await alert(
      $t('setup.reboot_required_msg', { default: "L'installation est terminée !\n\nVeuillez redémarrer QARL pour que toutes les météos et les filtres PP soient correctement chargés." }),
      $t('setup.reboot_required_title', { default: "Redémarrage nécessaire" }),
      "info"
    );
    
    markSetupCompleted();
    
    // ⭐ On utilise l'API Tauri pour forcer un vrai rechargement de la WebView (équivalent F5)
    try {
      await getCurrentWindow().reload();
    } catch (e) {
      // Fallback au cas où l'API Tauri échoue
      window.location.reload();
    }
  }

  function handleBack() {
    if (currentIdx > 0) goToStep(currentIdx - 1);
  }

  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { confirm, alert } from '$lib/stores/dialog.svelte';

  // ⭐ État pour savoir si la config AC est incomplète
  let hasError = $state(false);

  async function checkAcSetup() {
    if (!appState.documentsPath) {
      hasError = true;
      return;
    }
    try {
      const res = await invoke<any>('check_ac_initialized', { documentsPath: appState.documentsPath });
      // Si les contrôles ne sont pas configurés, on active l'erreur
      hasError = !res.hasControls || !res.hasControllers;
    } catch (e) {
      hasError = true;
    }
  }

  // ⭐ Vérification automatique au chargement et à chaque changement de chemins
  $effect(() => {
    if (appState.acPath && appState.documentsPath) {
      checkAcSetup();
    } else {
      hasError = true;
    }
  });

  async function handleClose() {
    // ⭐ On demande confirmation si on est sur une étape bloquante OU si hasError est vrai
    const isBlocking = STEPS[currentIdx].blocking;
    
    if (isBlocking || hasError) {
      const msg = $t('setup.warning_close_blocked');
      if (await confirm(msg, $t('common.warning'), "warning")) {
        getCurrentWindow().close();
      }
    } else {
      getCurrentWindow().close();
    }
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
        <!-- ⭐ Avertissement conditionnel (caché sur l'étape 1 pour éviter le doublon) -->
        {#if hasError && currentIdx !== 1}
          <div style="background: rgba(229, 115, 115, 0.1); border: 1px solid rgba(229, 115, 115, 0.3); padding: 12px 16px; border-radius: 6px; margin-bottom: 20px; color: #e57373; font-size: 0.85rem; font-weight: 600; display: flex; align-items: center; gap: 8px; line-height: 1.4;">
            {$t('setup.warning_ac_required')}
          </div>
        {/if}

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
          <Step6Recap onFinish={finishSetup} />
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
        
        <!-- ⭐ Le bouton Skip réapparaît SEULEMENT si tout va bien ET que l'étape n'est pas bloquante -->
        {#if currentIdx < totalSteps - 1 && !STEPS[currentIdx].blocking && !hasError}
          <button class="filter-btn" onclick={handleClose} style="margin-right: 8px;">
            {$t('setup.welcome.skip')}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* ⭐ FORCE LA TAILLE POUR ÉVITER LES SAUTS */
  .setup-modal {
    min-width: 800px;
    min-height: 700px; /* Assez grand pour contenir l'étape CSP sans sauter */
    max-height: 90vh;  /* Ne dépasse pas l'écran */
    display: flex;
    flex-direction: column;
  }

  .setup-body {
    flex: 1; /* Le corps prend tout l'espace disponible */
    overflow-y: auto; /* Scroll interne si le contenu est trop grand (ex: installation en cours) */
  }
</style>