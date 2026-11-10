<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { listen } from '@tauri-apps/api/event';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { t, locale } from 'svelte-i18n';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import { alert, confirm } from '$lib/stores/dialog.svelte';
  import { openUrl } from '@tauri-apps/plugin-opener';
  // ⭐ NOUVEAU : Import des fonctions météo
  import { getWeatherIcon, translateWeatherName } from '$lib/utils/weatherUtils.js';

  let activeSubTab = $state<'photos' | 'replays'>('photos');
  
  let mediaFiles = $state<{ screens: any[], replays: any[] }>({ screens: [], replays: [] });
  let isLoading = $state(false);
  let lightboxIndex = $state(-1);
  
  // États pour la vue détaillée des replays
  let selectedReplay = $state<any>(null);
  let replayCar = $state<any>(null);
  let replayTrack = $state<any>(null);
  let replayLayout = $state<any>(null);
  let replayWeather = $state<string | null>(null);

  // Pagination
  let visiblePhotos = $state(48);
  let visibleReplays = $state(24);

  // Accordéons (repliés par défaut, mémorisés)
  let openPhotoAccordions = $state<Record<string, boolean>>({});
  let openReplayAccordions = $state<Record<string, boolean>>({});

  // États des paramètres
  let replayIni = $state({ 
    MAX_SIZE_MB: 2000, 
    QUALITY: 4, 
    ENABLED: 1, 
    RACE: 10, 
    QUALIFY: 1, 
    OTHERS: 1, 
    MIN_TIME_SECONDS: 120 
  });

  let photoIni = $state<any>({});
  let screenshotFormat = $state('JPG');

  // ⭐ NOUVEAU : Variables locales pour forcer la réactivité de Svelte 5
  let selectedMode = $state('ORIGINAL');
  let customFormat = $state('');

  // Hints (commentaires INI) pour tooltips
  let photoHints = $state<Record<string, string>>({});
  let replayHints = $state<Record<string, string>>({});

  async function loadData() {
    if (!appState.documentsPath) return;
    isLoading = true;
    try {
      // 1. Médias
      mediaFiles = await invoke('get_media_files', { documentsPath: appState.documentsPath });
      
      // 2. Replay INI
      try {
        const replayContent = await invoke<string>('read_ini_file', { documentsPath: appState.documentsPath, filename: 'replay.ini' });
        parseReplayIni(replayContent);
      } catch (e) { console.warn("replay.ini introuvable"); }

      // Charger les hints du replay.ini
      try {
        replayHints = await invoke('get_ini_hints_from_documents', { documentsPath: appState.documentsPath, filename: 'replay.ini' });
      } catch (e) { console.warn("Hints replay indisponibles", e); }

      // 3. Photos INI
      if (appState.acPath) {
        const photoPath = `${appState.acPath}/extension/config/nice_screenshots.ini`;
        try {
          const photoContent = await invoke<string>('read_specific_ini', { filePath: photoPath });
          parsePhotoIni(photoContent);
        } catch (e) { console.warn("nice_screenshots.ini introuvable"); }

        // Charger les hints du nice_screenshots.ini
        try {
          photoHints = await invoke('get_ini_hints', { filePath: photoPath });
        } catch (e) { console.warn("Hints photo indisponibles", e); }

        // Lecture du format dans assetto_corsa.ini
        const mainIniPath = `${appState.acPath}/system/cfg/assetto_corsa.ini`;
        try {
          const mainContent = await invoke<string>('read_specific_ini', { filePath: mainIniPath });
          const lines = mainContent.split('\n');
          let inScreenshot = false;
          for (const line of lines) {
            const tLine = line.trim();
            if (tLine.startsWith('[') && tLine.endsWith(']')) {
              inScreenshot = tLine.toUpperCase() === '[SCREENSHOT]';
            } else if (inScreenshot && tLine.toUpperCase().startsWith('FORMAT=')) {
              screenshotFormat = tLine.split('=')[1].trim().toUpperCase();
              break;
            }
          }
        } catch (e) { console.warn("assetto_corsa.ini introuvable"); }
      }

    } catch (err) {
      console.error(err);
    } finally {
      isLoading = false;
    }
  }

  function parseReplayIni(content: string) {
    const lines = content.split('\n');
    let currentSec = '';
    for (const line of lines) {
      const tLine = line.trim();
      if (tLine.startsWith('[') && tLine.endsWith(']')) currentSec = tLine.slice(1, -1);
      if (tLine.includes('=') && !tLine.startsWith(';')) {
        const [k, v] = tLine.split('=');
        const val = v.split(';')[0].trim();
        const key = k.trim();
        
        if (currentSec === 'REPLAY' && key === 'MAX_SIZE_MB') replayIni.MAX_SIZE_MB = parseInt(val) || 2000;
        if (currentSec === 'QUALITY' && key === 'LEVEL') replayIni.QUALITY = parseInt(val) || 0;
        if (currentSec === 'AUTOSAVE') {
          if (key === 'ENABLED') replayIni.ENABLED = parseInt(val) || 0;
          if (key === 'RACE') replayIni.RACE = parseInt(val) || 0;
          if (key === 'QUALIFY') replayIni.QUALIFY = parseInt(val) || 0;
          if (key === 'OTHERS') replayIni.OTHERS = parseInt(val) || 0;
          if (key === 'MIN_TIME_SECONDS') replayIni.MIN_TIME_SECONDS = parseInt(val) || 0;
        }
      }
    }
  }

  function parsePhotoIni(content: string) {
    const lines = content.split('\n');
    let currentSec = '';
    let res: any = {};
    for (const line of lines) {
      const tLine = line.trim();
      if (tLine.startsWith('[') && tLine.endsWith(']')) currentSec = tLine.slice(1, -1);
      if (tLine.includes('=') && !tLine.startsWith(';')) {
        const [k, v] = tLine.split('=');
        const val = v.split(';')[0].trim();
        res[`${currentSec}.${k.trim()}`] = val;
      }
    }
    photoIni = res;

    // ⭐ On initialise nos variables locales
    selectedMode = res['NAME_FORMAT.MODE'] || 'ORIGINAL';
    customFormat = res['NAME_FORMAT.FORMAT_CUSTOM'] || '';
  }

  async function saveReplayIni() {
    if (!appState.documentsPath) return;
    
    let content = "";
    try { content = await invoke<string>('read_ini_file', { documentsPath: appState.documentsPath, filename: 'replay.ini' }); } catch(e){}
    
    if (!content) {
      content = `[REPLAY]\nMAX_SIZE_MB=${replayIni.MAX_SIZE_MB}\n[QUALITY]\nLEVEL=${replayIni.QUALITY}\n[AUTOSAVE]\nENABLED=${replayIni.ENABLED}\nRACE=${replayIni.RACE}\nQUALIFY=${replayIni.QUALIFY}\nOTHERS=${replayIni.OTHERS}\nMIN_TIME_SECONDS=${replayIni.MIN_TIME_SECONDS}`;
    } else {
      content = updateIniValue(content, 'REPLAY', 'MAX_SIZE_MB', replayIni.MAX_SIZE_MB.toString());
      content = updateIniValue(content, 'QUALITY', 'LEVEL', replayIni.QUALITY.toString());
      content = updateIniValue(content, 'AUTOSAVE', 'ENABLED', replayIni.ENABLED.toString());
      content = updateIniValue(content, 'AUTOSAVE', 'RACE', replayIni.RACE.toString());
      content = updateIniValue(content, 'AUTOSAVE', 'QUALIFY', replayIni.QUALIFY.toString());
      content = updateIniValue(content, 'AUTOSAVE', 'OTHERS', replayIni.OTHERS.toString());
      content = updateIniValue(content, 'AUTOSAVE', 'MIN_TIME_SECONDS', replayIni.MIN_TIME_SECONDS.toString());
    }

    await invoke('write_ini_file', { documentsPath: appState.documentsPath, filename: 'replay.ini', content });
    alert($t('media.replay.settings.save_success'));
  }

  async function savePhotoIni() {
    if (!appState.acPath) return;
    const path = `${appState.acPath}/extension/config/nice_screenshots.ini`;
    
    let content = "";
    try { content = await invoke<string>('read_specific_ini', { filePath: path }); } catch(e){}

    if (content) {
      for (const [key, val] of Object.entries(photoIni)) {
        const [sec, k] = key.split('.');
        content = updateIniValue(content, sec, k, String(val));
      }
    }
    await invoke('write_specific_ini', { filePath: path, content });

    // Sauvegarde du format dans assetto_corsa.ini
    if (appState.acPath) {
      const mainIniPath = `${appState.acPath}/system/cfg/assetto_corsa.ini`;
      try {
        let mainContent = await invoke<string>('read_specific_ini', { filePath: mainIniPath });
        let lines = mainContent.split('\n');
        let inScreenshot = false;
        let foundFormat = false;

        for (let i = 0; i < lines.length; i++) {
          const tLine = lines[i].trim();
          if (tLine.startsWith('[') && tLine.endsWith(']')) {
            inScreenshot = tLine.toUpperCase() === '[SCREENSHOT]';
          } else if (inScreenshot && tLine.toUpperCase().startsWith('FORMAT=')) {
            lines[i] = `FORMAT=${screenshotFormat}`;
            foundFormat = true;
            break;
          }
        }

        if (!foundFormat) {
          if (!mainContent.includes('[SCREENSHOT]')) {
            lines.push('', '[SCREENSHOT]', `FORMAT=${screenshotFormat}`);
          } else {
            const secIdx = lines.findIndex(l => l.trim().toUpperCase() === '[SCREENSHOT]');
            if (secIdx !== -1) {
              lines.splice(secIdx + 1, 0, `FORMAT=${screenshotFormat}`);
            }
          }
        }

        await invoke('write_specific_ini', { filePath: mainIniPath, content: lines.join('\n') });
      } catch (e) {
        console.error("Erreur de mise à jour de assetto_corsa.ini", e);
      }
    }

    alert($t('media.photo.settings.save_success'));
  }

  function updateIniValue(content: string, section: string, key: string, newValue: string): string {
    const lines = content.split('\n');
    let inSec = false;
    let changed = false;
    let insertIndex = -1;
    
    for (let i = 0; i < lines.length; i++) {
      const tLine = lines[i].trim();
      if (tLine.startsWith('[') && tLine.endsWith(']')) {
        if (inSec && insertIndex === -1) insertIndex = i;
        inSec = (tLine.slice(1, -1) === section);
        if (inSec) insertIndex = i + 1;
      }
      if (inSec && tLine.startsWith(key + '=')) {
        const parts = lines[i].split(';');
        lines[i] = `${key}=${newValue}${parts.length > 1 ? ' ;' + parts.slice(1).join(';') : ''}`;
        changed = true;
        break;
      }
    }
    
    if (!changed) {
      if (insertIndex !== -1) {
        lines.splice(insertIndex, 0, `${key}=${newValue}`);
      } else {
        if (!content.includes(`[${section}]`)) lines.push(`[${section}]`);
        lines.push(`${key}=${newValue}`);
      }
    }
    return lines.join('\n');
  }

  async function deleteMedia(file: any) {
    const msg = $t('media.replay.delete_confirm', { values: { name: file.name } });
    if (await confirm(msg, $t('dialog.confirm'), "warning")) {
      try {
        await invoke('delete_media_file', { filePath: file.path });
        loadData();
      } catch (e) { alert("Erreur : " + e); }
    }
  }

  async function selectReplay(file: any) {
    selectedReplay = file;
    replayCar = null;
    replayTrack = null;
    replayLayout = null;
    replayWeather = null;

    try {
      const extractedStrings = await invoke<string[]>('get_replay_info', { filePath: file.path });
      
      const fileNameLower = file.name.toLowerCase();

      replayCar = appState.cars.find(c => {
        const idLower = c.id.toLowerCase();
        return extractedStrings.some(s => s.toLowerCase() === idLower) || fileNameLower.includes(idLower);
      });

      replayTrack = appState.tracks.find(t => {
        const idLower = t.id.toLowerCase();
        return extractedStrings.some(s => s.toLowerCase() === idLower) || fileNameLower.includes(idLower);
      });

      replayLayout = null;
      if (replayTrack && replayTrack.layouts) {
        replayLayout = replayTrack.layouts.find((l: any) => {
          const lId = l.id.toLowerCase();
          return fileNameLower.includes(lId) || extractedStrings.some(s => s.toLowerCase() === lId);
        });
      }

      replayWeather = null;
      const weatherKeywords = extractedStrings.find(s => {
        const sl = s.toLowerCase();
        return sl.includes('clear') || sl.includes('rain') || sl.includes('cloud') || sl.includes('sol_') || sl.includes('pure_') || sl.includes('weather') || sl.includes('fog');
      });
      
      if (weatherKeywords) {
        replayWeather = weatherKeywords.replace(/[^a-zA-Z0-9_\s-]/g, '').trim();
      }

    } catch (e) {
      console.warn("Impossible d'analyser le replay", e);
    }
  }

  async function playReplay(file: any) {
    if (!appState.acPath) return;
    try {
      await invoke('launch_replay', { acPath: appState.acPath, replayFilename: file.name });
    } catch (e) { alert("Erreur de lancement : " + e); }
  }

  function openFolder(file: any) {
    const dir = file.path.substring(0, file.path.lastIndexOf('/'));
    invoke('open_folder', { acPath: appState.acPath, documentsPath: appState.documentsPath, folderType: 'absolute', folderId: dir });
  }

  $effect(() => {
    if (appState.documentsPath) loadData();
  });

  import { onMount } from 'svelte';
  onMount(() => {
    let unlistenThumbs: any;
    let unlistenClosed: any;

    // On recharge quand les miniatures sont prêtes
    listen('thumbs-ready', () => {
      loadData();
    }).then(u => unlistenThumbs = u);

    // On recharge quand AC se ferme (pour avoir les nouveaux ralentis)
    listen('ac-closed', () => {
      loadData();
    }).then(u => unlistenClosed = u);

    // ⭐ Restauration de l'état des accordéons (MÊME LOGIQUE QUE CSP)
    if (typeof localStorage !== 'undefined') {
      try {
        const savedPhoto = localStorage.getItem('qarl_media_photo_categories');
        if (savedPhoto) {
          openPhotoAccordions = JSON.parse(savedPhoto);
        }

        const savedReplay = localStorage.getItem('qarl_media_replay_categories');
        if (savedReplay) {
          openReplayAccordions = JSON.parse(savedReplay);
        }
      } catch (e) {}
    }

    return () => { 
      if (unlistenThumbs) unlistenThumbs(); 
      if (unlistenClosed) unlistenClosed();
    };
  });

  function formatSize(bytes: number) {
    if (bytes === 0) return '0 B';
    const k = 1024, sizes = ['B', 'KB', 'MB', 'GB'];
    const i = Math.floor(Math.log(bytes) / Math.log(k));
    return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i];
  }

  function togglePhotoAccordion(key: string) {
    openPhotoAccordions[key] = !openPhotoAccordions[key];
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('qarl_media_photo_categories', JSON.stringify(openPhotoAccordions));
    }
  }
  
  function toggleReplayAccordion(key: string) {
    openReplayAccordions[key] = !openReplayAccordions[key];
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('qarl_media_replay_categories', JSON.stringify(openReplayAccordions));
    }
  }

  // ⭐ Traduit un hint INI en utilisant la clé i18n (fallback sur l'anglais brut si absente)
    function tHint(key: string, fallback: string | undefined): string | undefined {
    if (!fallback) return undefined;
    const i18nKey = `media.hints.${key}`;
    const translated = $t(i18nKey);
    return translated === i18nKey ? fallback : translated;
  }

  // ⭐ NOUVEAU : Liste des photos actuellement affichées (pour la navigation)
  let displayedPhotos = $derived(mediaFiles.screens.slice(0, visiblePhotos));

  // ⭐ NOUVEAU : Fonctions de navigation pour la Lightbox
  function openLightbox(index: number) {
    lightboxIndex = index;
  }

  function closeLightbox() {
    lightboxIndex = -1;
  }

  function nextPhoto(e?: Event) {
    if (e) e.stopPropagation();
    if (lightboxIndex < displayedPhotos.length - 1) {
      lightboxIndex++;
    } else {
      lightboxIndex = 0; // Boucler à la fin
    }
  }

  function prevPhoto(e?: Event) {
    if (e) e.stopPropagation();
    if (lightboxIndex > 0) {
      lightboxIndex--;
    } else {
      lightboxIndex = displayedPhotos.length - 1; // Boucler au début
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (lightboxIndex === -1) return;
    if (e.key === 'Escape') closeLightbox();
    if (e.key === 'ArrowRight') nextPhoto();
    if (e.key === 'ArrowLeft') prevPhoto();
  }
</script>

<svelte:window on:keydown={handleKeydown} />

<!-- Snippet réutilisable : Label avec tooltip natif au survol -->
{#snippet optLabel(text: string, hint: string | undefined)}
  <label class="csp-opt-label" class:has-hint={!!hint} title={hint || undefined}>
    {text}
  </label>
{/snippet}

<div class="media-container">
  <!-- Sous-onglets -->
  <div class="subtabs" style="margin-bottom: 20px;">
    <button class="subtab-btn" class:active={activeSubTab === 'photos'} onclick={() => activeSubTab = 'photos'}>{$t('media.tabs.photos')}</button>
    <button class="subtab-btn" class:active={activeSubTab === 'replays'} onclick={() => activeSubTab = 'replays'}>{$t('media.tabs.replays')}</button>
  </div>

  {#if activeSubTab === 'photos'}
    <!-- PARAMÈTRES PHOTOS (Style CSP) -->
    <div class="csp-toolbar" style="margin-bottom: 20px;">
      <div class="csp-title">{$t('media.photo.settings.general')}</div>
      <button class="filter-btn" onclick={savePhotoIni} style="background: var(--success); color: white; border-color: var(--success);">💾 {$t('media.photo.settings.save_btn')}</button>
    </div>

    <div class="csp-container" style="margin-bottom: 30px;">
      
      <!-- Général -->
      <div class="csp-category" class:collapsed={!openPhotoAccordions.general}>
        <div class="csp-category-header" onclick={() => togglePhotoAccordion('general')}>
          <span class="csp-category-arrow">▸</span>
          <span class="csp-category-label">{$t('media.photo.settings.general')}</span>
        </div>
        {#if openPhotoAccordions.general}
          <div class="csp-category-content">
            <div class="csp-opt-row">
              {@render optLabel($t('media.photo.settings.enabled'), tHint('ns_basic_enabled', photoHints['BASIC.ENABLED']))}
              <div class="csp-opt-widget">
                <input type="checkbox" class="csp-module-checkbox" checked={photoIni['BASIC.ENABLED'] === '1'} onchange={(e) => photoIni['BASIC.ENABLED'] = e.currentTarget.checked ? '1' : '0'} />
              </div>
            </div>
            <div class="csp-opt-row">
              {@render optLabel($t('media.photo.settings.steam'), tHint('ns_basic_steam', photoHints['BASIC.STEAM_INTEGRATION']))}
              <div class="csp-opt-widget">
                <input type="checkbox" class="csp-module-checkbox" checked={photoIni['BASIC.STEAM_INTEGRATION'] === '1'} onchange={(e) => photoIni['BASIC.STEAM_INTEGRATION'] = e.currentTarget.checked ? '1' : '0'} />
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Format des captures & Noms -->
      <div class="csp-category" class:collapsed={!openPhotoAccordions.compression}>
        <div class="csp-category-header" onclick={() => togglePhotoAccordion('compression')}>
          <span class="csp-category-arrow">▸</span>
          <span class="csp-category-label">{$t('media.photo.settings.format_title')}</span>
        </div>
        {#if openPhotoAccordions.compression}
          <div class="csp-category-content">
            
            <div class="csp-opt-row">
              <label class="csp-opt-label">{$t('media.photo.settings.format')}</label>
              <div class="csp-opt-widget">
                <select class="ini-select" bind:value={screenshotFormat}>
                  <option value="JPG">{$t('media.photo.settings.format_jpg')}</option>
                  <option value="PNG">{$t('media.photo.settings.format_png')}</option>
                  <option value="BMP">{$t('media.photo.settings.format_bmp')}</option>
                </select>
              </div>
            </div>

            <div class="csp-opt-row">
              {@render optLabel($t('media.photo.settings.name_format'), tHint('ns_name_format', photoHints['NAME_FORMAT.MODE']))}
              <div class="csp-opt-widget">
                <!-- ⭐ On bind sur selectedMode et on sync avec photoIni -->
                <select class="ini-select" bind:value={selectedMode} onchange={(e) => photoIni['NAME_FORMAT.MODE'] = e.currentTarget.value}>
                  <option value="ORIGINAL">{$t('media.photo.settings.name_original')}</option>
                  <option value="ORDERED">{$t('media.photo.settings.name_ordered')}</option>
                  <option value="READABLE">{$t('media.photo.settings.name_readable')}</option>
                  <option value="SEPARATED">{$t('media.photo.settings.name_separated')}</option>
                  <option value="CUSTOM">{$t('media.photo.settings.name_custom')}</option>
                </select>
              </div>
            </div>

            <!-- ⭐ On utilise selectedMode pour la condition -->
            {#if selectedMode === 'CUSTOM'}
              <div class="csp-opt-row">
                {@render optLabel($t('media.photo.settings.custom_format'), photoHints['NAME_FORMAT.FORMAT_CUSTOM'])}
                <div class="csp-opt-widget">
                  <input 
                    type="text" 
                    class="ini-select" 
                    style="width: 100%; padding: 4px 8px; border: 1px solid var(--border-subtle); background: var(--bg-panel); color: var(--text-primary); border-radius: 4px;"
                    bind:value={customFormat} 
                    oninput={(e) => photoIni['NAME_FORMAT.FORMAT_CUSTOM'] = e.currentTarget.value}
                    placeholder={'{Year}{Month}{Day}-{TrackID}'} 
                  />
                </div>
              </div>
              
              <!-- ⭐ Liste de tags avec le lien vers la doc -->
              <div class="csp-opt-row" style="padding-top: 0; align-items: flex-start;">
                <div style="flex: 1; font-size: 0.75rem; color: var(--text-tertiary); margin-top: 5px; display: flex; flex-direction: column; gap: 8px;">
                  <span>{$t('media.photo.settings.custom_format_hint')}</span>
                  <button 
                    type="button" 
                    class="tag-btn" 
                    style="background: none; border: none; text-decoration: underline; padding: 0; justify-content: flex-start;"
                    onclick={() => openUrl('https://github.com/ac-custom-shaders-patch/acc-extension-config/wiki/General-%E2%80%93-Screenshots-name-format')}
                  >
                    📖 Voir la documentation officielle CSP
                  </button>
                </div>
                <div style="flex: 1.5; display: flex; flex-wrap: wrap; gap: 6px; justify-content: flex-end;">
                  {#each ['{CarID}', '{Car}', '{TrackID}', '{Track}', '{LayoutID}', '{PPFilter}', '{Year}', '{Month}', '{Day}', '{Hour}', '{Min}', '{Sec}'] as tag}
                    <button 
                      type="button" 
                      class="tag-btn" 
                      onclick={() => {
                        customFormat = (customFormat || '') + tag;
                        photoIni['NAME_FORMAT.FORMAT_CUSTOM'] = customFormat;
                      }}
                    >
                      {tag}
                    </button>
                  {/each}
                </div>
              </div>
            {/if}

          </div>
        {/if}
      </div>

      <!-- Accumulation AA -->
      <div class="csp-category" class:collapsed={!openPhotoAccordions.accumulation}>
        <div class="csp-category-header" onclick={() => togglePhotoAccordion('accumulation')}>
          <span class="csp-category-arrow">▸</span>
          <span class="csp-category-label">{$t('media.photo.settings.aa_title')}</span>
        </div>
        {#if openPhotoAccordions.accumulation}
          <div class="csp-category-content">
            <div class="csp-opt-row">
              {@render optLabel($t('media.photo.settings.aa_enabled'), tHint('ns_aa_enabled', photoHints['ACCUMULATION_AA.ENABLED']))}
              <div class="csp-opt-widget">
                <input type="checkbox" class="csp-module-checkbox" checked={photoIni['ACCUMULATION_AA.ENABLED'] === '1'} onchange={(e) => photoIni['ACCUMULATION_AA.ENABLED'] = e.currentTarget.checked ? '1' : '0'} />
              </div>
            </div>
            <div class="csp-opt-row">
              {@render optLabel($t('media.photo.settings.aa_iterations'), tHint('aa_iterations', photoHints['ACCUMULATION_AA.ITERATIONS']))}
              <div class="csp-opt-widget ini-slider-wrap">
                <input type="range" class="ini-range" min="4" max="80" step="1" bind:value={photoIni['ACCUMULATION_AA.ITERATIONS']} />
                <input type="number" class="ini-number" min="4" max="80" bind:value={photoIni['ACCUMULATION_AA.ITERATIONS']} />
              </div>
            </div>
            <div class="csp-opt-row">
              {@render optLabel($t('media.photo.settings.aa_res_mult'), tHint('aa_res_mult', photoHints['ACCUMULATION_AA.RESOLUTION_MULTIPLIER']))}
              <div class="csp-opt-widget ini-slider-wrap">
                <input type="range" class="ini-range" min="1" max="4" step="1" bind:value={photoIni['ACCUMULATION_AA.RESOLUTION_MULTIPLIER']} />
                <input type="number" class="ini-number" min="1" max="4" bind:value={photoIni['ACCUMULATION_AA.RESOLUTION_MULTIPLIER']} />
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- GALERIE PHOTOS -->
    <div class="group">
      <div class="group-title">{$t('media.photo.gallery_title')} ({mediaFiles.screens.length})</div>
      
      {#if mediaFiles.screens.length === 0}
        <div style="text-align: center; color: var(--text-tertiary); padding: 20px;">{$t('media.photo.no_photos')}</div>
      {:else}
        <div class="photo-grid">
          {#each displayedPhotos as photo, index}
            <div class="photo-card" onclick={() => openLightbox(index)} oncontextmenu={(e) => { e.preventDefault(); openFolder(photo); }}>
              <img src={convertFileSrc(photo.thumbnail_path)} alt={photo.name} loading="lazy" />
              <div class="photo-name" title={photo.name}>{photo.name}</div>
            </div>
          {/each}
        </div>
        
        {#if mediaFiles.screens.length > visiblePhotos}
          <button class="filter-btn" style="margin-top: 15px; width: 100%; justify-content: center;" onclick={() => visiblePhotos += 48}>
            {$t('media.photo.load_more', { values: { count: mediaFiles.screens.length - visiblePhotos } })}
          </button>
        {/if}
      {/if}
    </div>
  {:else}
    <!-- PARAMÈTRES REPLAYS (Style CSP + options CM) -->
    <div class="csp-toolbar" style="margin-bottom: 20px;">
      <div class="csp-title">{$t('media.replay.settings.title')}</div>
      <button class="filter-btn" onclick={saveReplayIni} style="background: var(--success); color: white; border-color: var(--success);">💾 {$t('media.replay.settings.save_btn')}</button>
    </div>

    <div class="csp-container" style="margin-bottom: 30px;">
      <!-- Section Général -->
      <div class="csp-category" class:collapsed={!openReplayAccordions.general}>
        <div class="csp-category-header" onclick={() => toggleReplayAccordion('general')}>
          <span class="csp-category-arrow">▸</span>
          <span class="csp-category-label">{$t('media.replay.settings.general')}</span>
        </div>
        {#if openReplayAccordions.general}
          <div class="csp-category-content">
            <div class="csp-opt-row">
              {@render optLabel($t('media.replay.settings.quality'), tHint('replay_quality', replayHints['QUALITY.LEVEL']))}
              <div class="csp-opt-widget">
                <select class="ini-select" bind:value={replayIni.QUALITY}>
                  <option value={0}>{$t('media.replay.settings.quality_0')}</option>
                  <option value={1}>{$t('media.replay.settings.quality_1')}</option>
                  <option value={2}>{$t('media.replay.settings.quality_2')}</option>
                  <option value={3}>{$t('media.replay.settings.quality_3')}</option>
                  <option value={4}>{$t('media.replay.settings.quality_4')}</option>
                </select>
              </div>
            </div>
            <div class="csp-opt-row">
              {@render optLabel($t('media.replay.settings.max_size'), tHint('replay_max_size', replayHints['REPLAY.MAX_SIZE_MB']))}
              <div class="csp-opt-widget ini-slider-wrap">
                <input type="range" class="ini-range" min="10" max="5000" step="10" bind:value={replayIni.MAX_SIZE_MB} />
                <input type="number" class="ini-number" min="10" max="5000" bind:value={replayIni.MAX_SIZE_MB} />
                <span style="font-size: 0.75rem; color: var(--text-secondary);">MB</span>
              </div>
            </div>
          </div>
        {/if}
      </div>

      <!-- Section Sauvegarde Auto -->
      <div class="csp-category" class:collapsed={!openReplayAccordions.autosave}>
        <div class="csp-category-header" onclick={() => toggleReplayAccordion('autosave')}>
          <span class="csp-category-arrow">▸</span>
          <span class="csp-category-label">{$t('media.replay.settings.autosave_title')}</span>
        </div>
        {#if openReplayAccordions.autosave}
          <div class="csp-category-content">
            <div class="csp-opt-row">
              {@render optLabel($t('media.replay.settings.autosave_enabled'), tHint('autosave_enabled', replayHints['AUTOSAVE.ENABLED']))}
              <div class="csp-opt-widget">
                <input type="checkbox" class="csp-module-checkbox" checked={replayIni.ENABLED === 1} onchange={(e) => replayIni.ENABLED = e.currentTarget.checked ? 1 : 0} />
              </div>
            </div>
            
            <div class="csp-opt-row" style:opacity={replayIni.ENABLED === 0 ? 0.5 : 1}>
              {@render optLabel($t('media.replay.settings.min_time'), tHint('autosave_min_time', replayHints['AUTOSAVE.MIN_TIME_SECONDS']))}
              <div class="csp-opt-widget ini-slider-wrap">
                <input type="range" class="ini-range" min="0" max="300" step="10" bind:value={replayIni.MIN_TIME_SECONDS} disabled={replayIni.ENABLED === 0} />
                <input type="number" class="ini-number" min="0" max="300" bind:value={replayIni.MIN_TIME_SECONDS} disabled={replayIni.ENABLED === 0} />
              </div>
            </div>

            <div class="csp-opt-row" style:opacity={replayIni.ENABLED === 0 ? 0.5 : 1}>
              {@render optLabel($t('media.replay.settings.race_limit'), tHint('autosave_race', replayHints['AUTOSAVE.RACE']))}
              <div class="csp-opt-widget ini-slider-wrap">
                <input type="range" class="ini-range" min="-1" max="50" step="1" bind:value={replayIni.RACE} disabled={replayIni.ENABLED === 0} />
                <input type="number" class="ini-number" min="-1" max="50" bind:value={replayIni.RACE} disabled={replayIni.ENABLED === 0} />
              </div>
            </div>

            <div class="csp-opt-row" style:opacity={replayIni.ENABLED === 0 ? 0.5 : 1}>
              {@render optLabel($t('media.replay.settings.qualify_limit'), tHint('autosave_qualify', replayHints['AUTOSAVE.QUALIFY']))}
              <div class="csp-opt-widget ini-slider-wrap">
                <input type="range" class="ini-range" min="-1" max="50" step="1" bind:value={replayIni.QUALIFY} disabled={replayIni.ENABLED === 0} />
                <input type="number" class="ini-number" min="-1" max="50" bind:value={replayIni.QUALIFY} disabled={replayIni.ENABLED === 0} />
              </div>
            </div>

            <div class="csp-opt-row" style:opacity={replayIni.ENABLED === 0 ? 0.5 : 1}>
              {@render optLabel($t('media.replay.settings.others_limit'), tHint('autosave_others', replayHints['AUTOSAVE.OTHERS']))}
              <div class="csp-opt-widget ini-slider-wrap">
                <input type="range" class="ini-range" min="-1" max="50" step="1" bind:value={replayIni.OTHERS} disabled={replayIni.ENABLED === 0} />
                <input type="number" class="ini-number" min="-1" max="50" bind:value={replayIni.OTHERS} disabled={replayIni.ENABLED === 0} />
              </div>
            </div>
          </div>
        {/if}
      </div>
    </div>

    <!-- LISTE REPLAYS (Split View) -->
    <div class="group" style="padding: 0; background: transparent; border: none; gap: 10px;">
      
      {#if mediaFiles.replays.length === 0}
        <div style="text-align: center; color: var(--text-tertiary); padding: 20px; background: var(--bg-panel); border-radius: var(--r-lg); border: 1px solid var(--border-subtle);">{$t('media.replay.no_replays')}</div>
      {:else}
        <div class="replay-split-layout">
          
          <!-- Colonne de gauche : La liste -->
          <div class="replay-list-col">
            {#each mediaFiles.replays.slice(0, visibleReplays) as replay}
              <div class="replay-item" class:selected={selectedReplay?.path === replay.path} onclick={() => selectReplay(replay)}>
                <div style="flex: 1; min-width: 0;">
                  <div style="font-weight: 600; font-size: 0.9rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;">{replay.name}</div>
                  <div style="font-size: 0.75rem; color: var(--text-secondary);">{formatSize(replay.size)} • {new Date(replay.date * 1000).toLocaleString()}</div>
                </div>
                <button class="filter-btn" onclick={(e) => { e.stopPropagation(); playReplay(replay); }} style="border-color: var(--accent); color: var(--accent);">▶</button>
              </div>
            {/each}

            {#if mediaFiles.replays.length > visibleReplays}
              <button class="filter-btn" style="margin-top: 15px; width: 100%; justify-content: center;" onclick={() => visibleReplays += 24}>
                {$t('media.replay.load_more', { values: { count: mediaFiles.replays.length - visibleReplays } })}
              </button>
            {/if}
          </div>

          <!-- Colonne de droite : Les détails -->
          <div class="replay-detail-col">
            {#if selectedReplay}
              <div style="display: flex; flex-direction: column; gap: 20px;">
                
                <!-- En-tête détails -->
                <div style="border-bottom: 1px solid var(--border-subtle); padding-bottom: 15px;">
                  <h3 style="font-size: 1.1rem; margin-bottom: 5px;">{selectedReplay.name}</h3>
                  <div style="font-size: 0.8rem; color: var(--text-secondary);">{$t('media.replay.detail.size')} {formatSize(selectedReplay.size)}</div>
                  <div style="font-size: 0.8rem; color: var(--text-secondary);">{$t('media.replay.detail.date')} {new Date(selectedReplay.date * 1000).toLocaleString()}</div>
                </div>

                <!-- Aperçu Voiture -->
                <div>
                  <div style="font-size: 0.75rem; font-weight: 700; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px;">{$t('media.replay.detail.car')}</div>
                  {#if replayCar}
                    <div style="background: var(--bg-elevated); border-radius: 8px; overflow: hidden; border: 1px solid var(--border);">
                      {#if replayCar.image_path}
                        <img src={convertFileSrc(replayCar.image_path)} style="width: 100%; height: 140px; object-fit: cover;" alt="Car" />
                      {:else}
                        <div style="width: 100%; height: 140px; display: flex; align-items: center; justify-content: center; font-size: 3rem; background: var(--bg-panel);">🏎️</div>
                      {/if}
                      <div style="padding: 10px; font-size: 0.85rem; font-weight: 600;">{replayCar.name}</div>
                    </div>
                  {:else}
                    <div class="detail-placeholder">{$t('media.replay.detail.car_unknown')}</div>
                  {/if}
                </div>

                <!-- Météo (si trouvée) -->
                {#if replayWeather}
                  <div>
                    <div style="font-size: 0.75rem; font-weight: 700; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px;">{$t('media.replay.detail.weather')}</div>
                    <div style="background: var(--bg-elevated); border: 1px solid var(--border); border-radius: 6px; padding: 10px; font-size: 0.85rem; font-weight: 600; display: flex; align-items: center; gap: 8px;">
                      <!-- ⭐ Utilisation des fonctions de traduction -->
                      <span style="font-size: 1.2rem;">{getWeatherIcon(replayWeather)}</span>
                      <span style="text-transform: capitalize;">{translateWeatherName(replayWeather, replayWeather, $locale)}</span>
                    </div>
                  </div>
                {/if}

                <!-- Aperçu Circuit -->
                <div>
                  <div style="font-size: 0.75rem; font-weight: 700; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px;">{$t('media.replay.detail.track')}</div>
                  {#if replayTrack}
                    {@const bgImage = replayLayout?.preview_path || replayTrack.preview_path || replayTrack.layouts?.[0]?.preview_path}
                    {@const outlineImage = replayLayout?.outline_path || replayTrack.outline_path || replayTrack.layouts?.[0]?.outline_path}
                    <div style="background: var(--bg-elevated); border-radius: 8px; overflow: hidden; border: 1px solid var(--border);">
                      <div style="position: relative; height: 140px; background: #0f1219;">
                        {#if bgImage}
                          <img src={convertFileSrc(bgImage)} style="width: 100%; height: 100%; object-fit: cover; opacity: 0.5;" alt="Track" />
                        {/if}
                        {#if outlineImage}
                          <img src={convertFileSrc(outlineImage)} style="position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain; padding: 10px;" alt="Outline" />
                        {:else if !bgImage}
                          <div style="display: flex; align-items: center; justify-content: center; height: 100%; color: var(--text-tertiary); font-size: 2rem;">🏁</div>
                        {/if}
                      </div>
                      <div style="padding: 10px; font-size: 0.85rem; font-weight: 600;">
                        {replayTrack.name}
                        {#if replayLayout && replayLayout.name && replayLayout.name.toLowerCase() !== 'défaut'}
                          <span style="color: var(--text-tertiary); font-weight: 400; font-size: 0.75rem; margin-left: 6px;">({replayLayout.name})</span>
                        {/if}
                      </div>
                    </div>
                  {:else}
                    <div class="detail-placeholder">{$t('media.replay.detail.track_unknown')}</div>
                  {/if}
                </div>

                <!-- Actions -->
                <div style="display: flex; gap: 10px; margin-top: auto;">
                  <button class="filter-btn" onclick={() => playReplay(selectedReplay)} style="flex: 1; justify-content: center; border-color: var(--accent); color: var(--accent);">{$t('media.replay.detail.play_btn')}</button>
                  <button class="filter-btn" onclick={() => deleteMedia(selectedReplay)} style="color: #e57373; border-color: rgba(229, 115, 115, 0.3);">🗑️</button>
                </div>

              </div>
            {:else}
              <div style="display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: var(--text-tertiary); text-align: center; gap: 10px;">
                <div style="font-size: 3rem; opacity: 0.5;">🎬</div>
                <div style="font-size: 0.9rem;">{$t('media.replay.detail.select_hint')}</div>
              </div>
            {/if}
          </div>
        </div>
      {/if}
    </div>
  {/if}
</div>

{#if lightboxIndex >= 0 && displayedPhotos[lightboxIndex]}
  <div class="lightbox" onclick={closeLightbox}>
    <!-- Bouton Précédent -->
    <button class="lightbox-nav-btn prev" onclick={prevPhoto} title="Précédent (←)">‹</button>
    
    <div class="lightbox-content" onclick={(e) => e.stopPropagation()}>
      <img src={convertFileSrc(displayedPhotos[lightboxIndex].path)} alt="Preview" />
      <div class="lightbox-caption">
        {displayedPhotos[lightboxIndex].name} 
        <span style="opacity: 0.5; margin-left: 10px;">({lightboxIndex + 1} / {displayedPhotos.length})</span>
      </div>
    </div>

    <!-- Bouton Suivant -->
    <button class="lightbox-nav-btn next" onclick={nextPhoto} title="Suivant (→)">›</button>
  </div>
{/if}

<style>
  .media-container { display: flex; flex-direction: column; height: 100%; overflow-y: auto; padding-right: 10px; }
  
  /* Grille photos */
  .photo-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 15px; margin-top: 10px; }
  .photo-card { background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: 6px; overflow: hidden; cursor: pointer; transition: transform 0.2s, border-color 0.2s; position: relative; }
  .photo-card:hover { transform: scale(1.02); border-color: var(--accent); z-index: 10; }
  .photo-card img { width: 100%; height: 130px; object-fit: cover; display: block; }
  .photo-name { padding: 8px; font-size: 0.72rem; color: var(--text-secondary); white-space: nowrap; overflow: hidden; text-overflow: ellipsis; background: var(--bg-elevated); }

  /* ⭐ NOUVEAUX STYLES POUR LA LIGHTBOX */
  .lightbox {
    position: fixed; top: 0; left: 0; width: 100vw; height: 100vh;
    background: rgba(0,0,0,0.92);
    display: flex; align-items: center; justify-content: center;
    z-index: 9999; backdrop-filter: blur(8px);
    animation: fadeIn 0.2s ease-out;
  }
  
  @keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }

  .lightbox-content {
    display: flex; flex-direction: column; align-items: center;
    max-width: 90vw; max-height: 90vh;
  }

  .lightbox-content img {
    max-width: 90vw; max-height: 80vh; object-fit: contain;
    border-radius: 8px; box-shadow: 0 10px 40px rgba(0,0,0,0.8);
  }

  .lightbox-caption {
    color: #fff; font-size: 0.9rem; margin-top: 15px;
    text-shadow: 0 2px 4px rgba(0,0,0,0.8);
  }

  .lightbox-nav-btn {
    position: absolute; top: 50%; transform: translateY(-50%);
    background: rgba(255,255,255,0.1); border: 1px solid rgba(255,255,255,0.2);
    color: #fff; font-size: 3rem; width: 60px; height: 100px;
    display: flex; align-items: center; justify-content: center;
    cursor: pointer; border-radius: 8px; transition: all 0.2s;
    backdrop-filter: blur(4px);
  }

  .lightbox-nav-btn:hover { background: rgba(255,255,255,0.2); border-color: #fff; }
  .lightbox-nav-btn.prev { left: 20px; padding-right: 5px; }
  .lightbox-nav-btn.next { right: 20px; padding-left: 5px; }

  /* Liste replays Split View */
  .replay-split-layout {
    display: flex;
    gap: 20px;
    height: 100%;
    min-height: 500px;
  }

  .replay-list-col {
    flex: 1;
    min-width: 300px;
    overflow-y: auto;
    padding-right: 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .replay-detail-col {
    flex: 0 0 400px;
    max-width: 400px;
    background: var(--bg-panel);
    border: 1px solid var(--border-subtle);
    border-radius: var(--r-lg);
    padding: 20px;
    display: flex;
    flex-direction: column;
    height: fit-content;
    position: sticky;
    top: 20px;
    overflow: hidden;
  }

  .replay-detail-col h3 {
    word-break: break-all;
    font-size: 1.05rem;
    line-height: 1.4;
  }

  .replay-item { 
    display: flex; align-items: center; gap: 12px; padding: 12px; 
    background: var(--bg-elevated); border: 1px solid var(--border-subtle); 
    border-radius: 6px; cursor: pointer; transition: all 0.2s;
  }
  .replay-item:hover { border-color: var(--border-strong); background: var(--bg-hover); }
  .replay-item.selected { border-color: var(--accent); background: var(--accent-muted); box-shadow: 0 0 0 1px var(--accent); }
  
  .detail-placeholder {
    background: var(--bg-elevated);
    border: 1px dashed var(--border-subtle);
    border-radius: 8px;
    padding: 20px;
    text-align: center;
    color: var(--text-tertiary);
    font-size: 0.8rem;
    font-style: italic;
  }

  /* ⭐ NOUVEAU : Style du label avec tooltip (curseur d'aide + soulignement pointillé) */
  .csp-opt-label.has-hint {
    cursor: help;
    border-bottom: 1px dotted var(--text-tertiary);
    padding-bottom: 1px;
    transition: border-color 0.15s;
  }
  .csp-opt-label.has-hint:hover {
    border-bottom-color: var(--accent);
  }

  /* ⭐ NOUVEAU : Style des boutons de tags */
  .tag-btn {
    background: var(--bg-elevated);
    border: 1px solid var(--border-subtle);
    color: var(--text-secondary);
    font-size: 0.7rem;
    padding: 2px 6px;
    border-radius: 4px;
    cursor: pointer;
    font-family: monospace;
    transition: all 0.15s;
  }
  .tag-btn:hover {
    background: var(--accent-muted);
    border-color: var(--accent);
    color: var(--text-primary);
  }
</style>