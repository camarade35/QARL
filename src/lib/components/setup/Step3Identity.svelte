<script lang="ts">
  import { t } from 'svelte-i18n';
  import { invoke } from '@tauri-apps/api/core';
  import { onMount } from 'svelte';
  import { appState, setDriverName, setOnlineName, setSteamId } from '$lib/stores/appState.svelte.js';
  import { setupState } from '$lib/stores/setupState.svelte.js';

  let { onContinue = () => {} } = $props();

  // ⭐ On donne la priorité à appState (qui garde les vraies valeurs) plutôt qu'au setupState
  let initDriverName = appState.driverName;
  if (!initDriverName || initDriverName === 'Driver') {
    initDriverName = setupState.data.driverName;
  }
  if (!initDriverName || initDriverName === 'Driver') {
    initDriverName = ''; // On laisse vide pour forcer la détection Steam si besoin
  }

  let driverName = $state(initDriverName);
  let onlineName = $state(appState.onlineName || setupState.data.onlineName || '');
  let steamId = $state(appState.steamId || setupState.data.steamId || '');
  let nationality = $state(setupState.data.nationality || 'France');
  let detecting = $state(false);

  // Liste minimale de pays avec emoji drapeau
  const COUNTRIES = [
    { code: 'france', name: 'France', flag: '🇫🇷' },
    { code: 'italy', name: 'Italie', flag: '🇮🇹' },
    { code: 'germany', name: 'Allemagne', flag: '🇩🇪' },
    { code: 'spain', name: 'Espagne', flag: '🇪🇸' },
    { code: 'uk', name: 'Royaume-Uni', flag: '🇬🇧' },
    { code: 'england', name: 'Angleterre', flag: '🏴󠁧󠁢󠁥󠁮󠁧󠁿' },
    { code: 'usa', name: 'États-Unis', flag: '🇺🇸' },
    { code: 'japan', name: 'Japon', flag: '🇯🇵' },
    { code: 'netherlands', name: 'Pays-Bas', flag: '🇳🇱' },
    { code: 'belgium', name: 'Belgique', flag: '🇧🇪' },
    { code: 'switzerland', name: 'Suisse', flag: '🇨🇭' },
    { code: 'austria', name: 'Autriche', flag: '🇦🇹' },
    { code: 'portugal', name: 'Portugal', flag: '🇵🇹' },
    { code: 'sweden', name: 'Suède', flag: '🇸🇪' },
    { code: 'canada', name: 'Canada', flag: '🇨🇦' },
    { code: 'brazil', name: 'Brésil', flag: '🇧🇷' },
    { code: 'australia', name: 'Australie', flag: '🇦🇺' },
    { code: 'new_zealand', name: 'Nouvelle-Zélande', flag: '🇳🇿' },
    { code: 'poland', name: 'Pologne', flag: '🇵🇱' },
    { code: 'russia', name: 'Russie', flag: '🇷🇺' },
  ];

  async function detectSteam() {
    detecting = true;
    try {
      // ⭐ Le backend renvoie maintenant un objet avec l'ID et le PersonaName
      const res = await invoke<{ steam_id: string, persona_name: string }>('detect_steam_id');
      if (res) {
        steamId = res.steam_id;
        
        // Si le nom du pilote est vide ou "Driver", on met le pseudo Steam
        if (!driverName || driverName === 'Driver' || driverName.trim() === '') {
          driverName = res.persona_name;
        }
        // Si le nom en ligne est vide, on met aussi le pseudo Steam
        if (!onlineName) {
          onlineName = res.persona_name;
        }
      }
    } catch (e) {
      console.warn('Steam ID detection failed', e);
    } finally {
      detecting = false;
    }
  }

  function handleContinue() {
    // ⭐ Si le nom solo est vide, on prend le nom en ligne ou 'Driver' par défaut
    const finalDriverName = driverName.trim() || onlineName.trim() || 'Driver';
    
    setDriverName(finalDriverName);
    setOnlineName(onlineName.trim());
    setSteamId(steamId.trim());

    setupState.data.driverName = finalDriverName;
    setupState.data.onlineName = onlineName.trim();
    setupState.data.steamId = steamId.trim();
    setupState.data.nationality = nationality;

    onContinue();
  }

  onMount(() => {
    // ⭐ On lance la détection si le SteamID est vide OU si le nom du pilote est vide/Driver
    if (!steamId || !driverName || driverName === 'Driver') {
      detectSteam();
    }
  });
</script>

<div class="setup-field">
  <label>{$t('setup.identity.driver_name')}</label>
  <input type="text" bind:value={driverName} style="width: 100%; padding: 10px 12px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm); color: var(--text-primary); font-family: inherit; font-size: 0.88rem;" />
  <div class="setup-hint">{$t('setup.identity.driver_name_hint')}</div>
</div>

<div class="setup-field">
  <label>{$t('setup.identity.online_name')}</label>
  <input type="text" bind:value={onlineName} placeholder={$t('setup.identity.online_name_placeholder')} style="width: 100%; padding: 10px 12px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm); color: var(--text-primary); font-family: inherit; font-size: 0.88rem;" />
  <div class="setup-hint">{$t('setup.identity.online_name_hint')}</div>
</div>

<div class="setup-field">
  <label>{$t('setup.identity.steam_id')}</label>
  <div class="setup-input-row">
    <input type="text" bind:value={steamId} placeholder={$t('setup.identity.steam_id_placeholder')} />
    <button class="filter-btn" onclick={detectSteam} disabled={detecting}>
      {detecting ? $t('setup.identity.detecting') : $t('setup.identity.detect')}
    </button>
  </div>
</div>

<div class="setup-field">
  <label>{$t('setup.identity.nationality')}</label>
  <select bind:value={nationality} style="padding: 10px 12px; background: var(--bg-elevated); border: 1px solid var(--border); border-radius: var(--r-sm); color: var(--text-primary); font-family: inherit; font-size: 0.88rem;">
    {#each COUNTRIES as c}
      <option value={c.name}>{c.flag} {c.name}</option>
    {/each}
  </select>
</div>

<div style="display: flex; justify-content: flex-end; margin-top: 20px;">
  <button class="filter-btn" onclick={handleContinue} style="padding: 10px 22px; background: var(--accent); color: #fff; border-color: var(--accent); font-weight: 700;">
    {$t('setup.identity.continue')}
  </button>
</div>