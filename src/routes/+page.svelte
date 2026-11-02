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

  // ⭐ État pour afficher la visite au démarrage
  let showTour = $state(false);

  onMount(() => {
    // ⭐ On écoute le changement d'onglet demandé par la visite
    const handleTabChange = (e: any) => {
      if (e.detail && e.detail.tab) {
        appState.activeTab = e.detail.tab;
      }
    };
    window.addEventListener('qarl-change-tab', handleTabChange);

    // ⭐ On affiche la visite si elle n'a jamais été désactivée
    if (!setupState.tutorialDisabled) {
      // Petit délai pour ne pas clignoter au tout premier chargement
      setTimeout(() => { showTour = true; }, 500);
    }

    return () => window.removeEventListener('qarl-change-tab', handleTabChange);
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