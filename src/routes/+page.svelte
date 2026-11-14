<script lang="ts">
  import { t } from 'svelte-i18n';
  import Sidebar from '$lib/components/Sidebar.svelte';
  import CarsTab from '$lib/components/tabs/CarsTab.svelte';
  import TracksTab from '$lib/components/tabs/TracksTab.svelte';
  import SessionTab from '$lib/components/tabs/SessionTab.svelte';
  import WeatherTab from '$lib/components/tabs/WeatherTab.svelte';
  import ModsTab from '$lib/components/tabs/ModsTab.svelte';
  import SettingsTab from '$lib/components/tabs/SettingsTab.svelte';
  import OnlineTab from '$lib/components/tabs/OnlineTab.svelte';
  import MediaTab from '$lib/components/tabs/MediaTab.svelte';
  import SelectionModal from '$lib/components/SelectionModal.svelte';
  import WelcomeTour from '$lib/components/WelcomeTour.svelte';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { searchState } from '$lib/stores/searchState.svelte.js';
  import { setupState } from '$lib/stores/setupState.svelte.js';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event'; // ⭐ Import pour écouter le Deep Link

  // ⭐ État pour afficher la visite au démarrage
  let showTour = $state(false);
  let hasShownTourThisSession = false; // ⭐ Évite que le tour se relance si on rouvre l'assistant

  onMount(() => {
    // ⭐ On écoute le changement d'onglet demandé par la visite
    const handleTabChange = (e: any) => {
      if (e.detail && e.detail.tab) {
        appState.activeTab = e.detail.tab;
      }
    };
    window.addEventListener('qarl-change-tab', handleTabChange);

    // ⭐ Écoute du Deep Link provenant de Rust
    let unlisten: (() => void) | undefined;
    listen<string>('qarl-deep-link', (event) => {
      const url = event.payload;
      console.log('🔗 Deep Link reçu dans Svelte:', url);
      
      // On bascule sur l'onglet Online
      appState.activeTab = 'tab-online';
      
      // On émet un événement personnalisé que OnlineTab.svelte pourra écouter
      // (avec un petit délai pour laisser le temps à l'onglet de s'afficher)
      setTimeout(() => {
        window.dispatchEvent(new CustomEvent('qarl-process-deep-link', { detail: { url } }));
      }, 300);
    }).then((fn) => { unlisten = fn; });

    return () => {
      window.removeEventListener('qarl-change-tab', handleTabChange);
      if (unlisten) unlisten();
    };
  });

  // ⭐ On affiche la visite UNIQUEMENT quand l'assistant est fermé
  $effect(() => {
    // Si l'assistant est ouvert, on ne fait rien (on attend qu'il se ferme)
    if (setupState.isOpen) return;

    // Si on a déjà montré le tour, ou si l'utilisateur l'a désactivé, on ne fait rien
    if (hasShownTourThisSession || setupState.tutorialDisabled) return;

    // Petit délai pour laisser l'interface se charger
    const timer = setTimeout(() => {
      showTour = true;
      hasShownTourThisSession = true;
    }, 500);

    return () => clearTimeout(timer);
  });
  
  const tabs = [
    { id: 'tab-cars',     key: 'tabs.cars'     },
    { id: 'tab-tracks',   key: 'tabs.tracks'   },
    { id: 'tab-session',  key: 'tabs.session'  },
    { id: 'tab-graphics', key: 'tabs.weather'  },
    { id: 'tab-mods',     key: 'tabs.mods'     },
    { id: 'tab-online',   key: 'tabs.online'   },
    { id: 'tab-media',    key: 'tabs.media'    },
    { id: 'tab-settings', key: 'tabs.settings' }
  ];
</script>

<!-- ⭐ Blocage global du menu contextuel natif de Windows/WebView2 -->
<svelte:window oncontextmenu={(e) => e.preventDefault()} />

<Sidebar />

<div class="main">
  <!-- Les onglets horizontaux -->
  <div class="tabs">
    {#each tabs as tab}
      <button 
        class="tab-btn" 
        class:active={appState.activeTab === tab.id}
        onclick={() => appState.activeTab = tab.id}
      >
        {$t(tab.key)}
      </button>
    {/each}
  </div>

  <!-- Contenu des onglets -->
  <div class="tab-content" class:active={appState.activeTab === 'tab-cars'}>
    <CarsTab />
  </div>

  <div class="tab-content" class:active={appState.activeTab === 'tab-tracks'}>
    <TracksTab />
  </div>

  <div class="tab-content" class:active={appState.activeTab === 'tab-session'}>
    <SessionTab />
  </div>

  <div class="tab-content" class:active={appState.activeTab === 'tab-online'}>
    <OnlineTab />
  </div>

  <div class="tab-content" class:active={appState.activeTab === 'tab-graphics'}>
    <WeatherTab />
  </div>

  <div class="tab-content" class:active={appState.activeTab === 'tab-mods'}>
    <ModsTab />
  </div>
  
  <div class="tab-content" class:active={appState.activeTab === 'tab-media'}>
    <MediaTab />
  </div>

  <div class="tab-content" id="tab-settings" class:active={appState.activeTab === 'tab-settings'} class:searching={searchState.query.trim().length > 0}>
    <SettingsTab />
  </div>
</div>

<SelectionModal />

<!-- ⭐ Visite guidée indépendante -->
<WelcomeTour bind:isOpen={showTour} onFinish={() => showTour = false} />