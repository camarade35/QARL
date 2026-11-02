<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { onMount } from 'svelte';
  import { listen } from '@tauri-apps/api/event';
  import { alert, confirm } from '$lib/stores/dialog.svelte'; // ⭐ On importe confirm aussi !
  import { t } from 'svelte-i18n';
  import { searchState } from '$lib/stores/searchState.svelte.js';
  import AutosaveBanner from '$lib/components/AutosaveBanner.svelte';
  // ⭐ Import des icônes Lucide
    import { 
    Lightbulb, Zap, Megaphone, CloudRain, CloudDrizzle, Sun, 
    BatteryCharging, Wind, Timer, Power, Camera, 
    ArrowLeft, ArrowRight, ArrowUp, ArrowDown, ScanEye,
    ChevronUp, ChevronDown, ArrowLeftCircle, ArrowRightCircle, TriangleAlert,
    Plus, Minus, UserCheck, Settings, FastForward, RotateCcw, ShieldAlert, ShieldCheck, 
    Disc, ChevronsUp, ChevronsDown, PauseCircle, RefreshCcw, Flag, PlayCircle, 
    Wrench, Square, LayoutGrid, EyeOff, Eye, Users, Route, SkipForward, SkipBack, 
    ChevronRight, ChevronLeft, User
  } from 'lucide-svelte';

  // ⭐ Récupération de la prop "active" depuis SettingsTab
  let { active = false } = $props();

  // États généraux
  let deviceType = $state<'wheel' | 'x360' | 'keyboard'>('wheel');
  let currentMode = $state<'simple' | 'advanced' | 'expert'>('simple');
  let iniContent = $state('');
  let isLoading = $state(false);
  let isSaving = $state(false);
  let hasChanges = $state(false);
  let lastSavedIniContent = '';
  let autosaveTimer: ReturnType<typeof setTimeout> | null = null;

  // Profils
  let profiles = $state<{name: string, path: string, profile_type: string}[]>([]);
  let selectedProfilePath = $state('');
  let saveAsName = $state('');
  let showSaveAs = $state(false);

  // Wizard de capture
  let isCapturing = $state(false);
  let capturingSection = $state('');
  let capturingActionName = $state('');
  let isAssigning = false; // Verrou anti-double assignation
  
  // ⭐ État pour basculer entre boîte en H et mode séquentiel
  let useHPattern = $state(true);

  // Cache des fichiers JSON de traduction des axes
  let controllerMappings = $state<Record<string, any>>({});
  let loadingMappings = new Set<string>();

  // Parseur INI
  function parseIni(text: string) {
    const result: Record<string, Record<string, string>> = {};
    let currentSection = '';

    text.split('\n').forEach(line => {
      const rawLine = line;
      line = line.trim();
      if (!line || line.startsWith(';')) return;

      if (line.startsWith('[') && line.endsWith(']')) {
        currentSection = line.slice(1, -1);
        result[currentSection] = { _raw: {} };
      } else if (currentSection && line.includes('=')) {
        const [key, ...valueParts] = line.split('=');
        const k = key.trim();
        const v = valueParts.join('=').trim();
        result[currentSection][k] = v;
        result[currentSection]._raw[k] = rawLine;
      }
    });
    return result;
  }

  function stringifyIni(obj: Record<string, any>) {
    let out = '';
    for (const section in obj) {
      out += `[${section}]\n`;
      for (const key in obj[section]) {
        if (key === '_raw') continue;
        if (obj[section]._raw && obj[section]._raw[key]) {
          let raw = obj[section]._raw[key];
          const parts = raw.split('=');
          out += `${parts[0]}=${obj[section][key]}\n`;
        } else {
          out += `${key}=${obj[section][key]}\n`;
        }
      }
      out += '\n';
    }
    return out;
  }

  let parsedIni = $derived(parseIni(iniContent));

  // Charger les profils
  async function loadProfiles() {
    if (!appState.documentsPath) return;
    try {
      profiles = await invoke('get_controller_profiles', { documentsPath: appState.documentsPath });
    } catch (e) {
      console.error("Erreur chargement profils:", e);
    }
  }

  // Charger le fichier INI
  async function loadIni(profilePath: string = '') {
    if (!appState.documentsPath) return;
    isLoading = true;
    hasChanges = false;
    try {
      let content = '';
      if (profilePath) {
        content = await invoke<string>('read_specific_ini', { filePath: profilePath });
        selectedProfilePath = profilePath;
      } else {
        content = await invoke<string>('read_controls_ini', { 
          documentsPath: appState.documentsPath, 
          deviceType: deviceType 
        });
        selectedProfilePath = '';
      }
      iniContent = content;
      lastSavedIniContent = content;
      hasChanges = false;
    } catch (e) {
      console.error("Erreur chargement INI:", e);
    } finally {
      isLoading = false;
    }
  }

  // Sauvegarder
  async function saveIni(silent = false) {
    if (!appState.documentsPath) return;
    isSaving = true;
    
    const isCreatingProfile = !!saveAsName;
    try {
      await invoke('write_controls_ini', {
        documentsPath: appState.documentsPath,
        deviceType: deviceType,
        content: iniContent,
        saveAsName: isCreatingProfile ? saveAsName : null,
        applyToGame: !isCreatingProfile
      });
      
      hasChanges = false;
      lastSavedIniContent = iniContent;
      if (autosaveTimer) { clearTimeout(autosaveTimer); autosaveTimer = null; }
      
      if (isCreatingProfile) {
        await loadProfiles();
        saveAsName = '';
        showSaveAs = false;
      }
      if (!silent) alert(isCreatingProfile ? $t('settings.controls.msg_save_profile') : $t('settings.controls.msg_save_ini'));
    } catch (e) {
      if (!silent) alert($t('settings.controls.msg_save_error') + e);
    } finally {
      isSaving = false;
    }
  }

  // ⭐ Auto-save
  $effect(() => {
    const current = iniContent;
    if (!lastSavedIniContent) { lastSavedIniContent = current; return; }
    if (current === lastSavedIniContent) return;
    // On ne déclenche PAS l'autosave si un profil est sélectionné ou si on crée un profil
    if (selectedProfilePath || showSaveAs) return;
    hasChanges = true;
    if (searchState.autosaveDelay > 0) {
      if (autosaveTimer) clearTimeout(autosaveTimer);
      autosaveTimer = setTimeout(() => {
        if (hasChanges) saveIni(true);
      }, searchState.autosaveDelay * 1000);
    }
  });

  // Appliquer un profil existant
  async function applyProfile() {
    if (!selectedProfilePath) {
      alert($t('settings.controls.err_select_profile'));
      return;
    }
    try {
      await invoke('write_controls_ini', {
        documentsPath: appState.documentsPath,
        deviceType: deviceType,
        content: iniContent,
        saveAsName: null,
        applyToGame: true
      });
      alert($t('settings.controls.success_profile_applied'));
      loadIni();
    } catch (e) {
      alert($t('settings.controls.err_apply_profile') + e);
    }
  }

  function updateIniValue(section: string, key: string, value: string) {
    if (!parsedIni[section]) parsedIni[section] = { _raw: {} };
    parsedIni[section][key] = value;
    iniContent = stringifyIni(parsedIni);
    hasChanges = true;
  }

  // Gestion de la capture
  function openCapture(sectionId: string, actionName: string) {
    capturingSection = sectionId;
    capturingActionName = actionName;
    isCapturing = true;
  }

  function closeCapture() {
    isCapturing = false;
    capturingSection = '';
    capturingActionName = '';
    isAssigning = false; // On libère le verrou
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (!isCapturing || !capturingSection) return;
    e.preventDefault();
    e.stopPropagation();
    
    const hexCode = '0x' + e.keyCode.toString(16).toUpperCase();
    
    if (!parsedIni[capturingSection]) parsedIni[capturingSection] = { _raw: {} };
    parsedIni[capturingSection]['KEY'] = hexCode;
    parsedIni[capturingSection]['BUTTON'] = '-1';
    
    iniContent = stringifyIni(parsedIni);
    hasChanges = true;
    closeCapture();
  }

  function isAssigned(section: string) {
    if (!parsedIni[section]) return false;
    const btn = parsedIni[section]['BUTTON'];
    const key = parsedIni[section]['KEY'];
    const axl = parsedIni[section]['AXLE'];
    return (btn && btn !== '-1') || (key && key !== '-1') || (axl && axl !== '-1');
  }

  // ⭐ Cache des périphériques DirectInput (PGUID)
  let dinputDevices = $state<any[]>([]);
  
  // ⭐ Nouveau : Liste parsée de [CONTROLLERS]
  let controllersSection = $state<any[]>([]);

  async function loadDInputDevices() {
    try {
      dinputDevices = await invoke('get_directinput_devices');
    } catch (e) {
      console.warn("Impossible de charger les périphériques DirectInput:", e);
    }
  }

  async function loadControllersSection() {
    if (!appState.documentsPath) return;
    try {
      controllersSection = await invoke('parse_controllers_section', { documentsPath: appState.documentsPath });
    } catch (e) {
      console.warn("Impossible de lire [CONTROLLERS]:", e);
    }
  }

    // ⭐ Extrait le PGUID depuis un Gamepad de la WebView
  function getPguidFromGamepad(gp: Gamepad): string | null {
    // L'ID ressemble à : "FANATEC Wheel (Vendor: 0eb7 Product: 0020)"
    const vendorMatch = gp.id.match(/Vendor:\s*([0-9a-fA-F]+)/i);
    const productMatch = gp.id.match(/Product:\s*([0-9a-fA-F]+)/i);
    if (!vendorMatch || !productMatch) return null;
    
    const vid = vendorMatch[1].toUpperCase().padStart(4, '0');
    const pid = productMatch[1].toUpperCase().padStart(4, '0');
    
    // Format standard des PGUID dans controls.ini : ProductID + VendorID + suffixe
    return `${pid}${vid}-0000-0000-0000-504944564944`;
  }

  // ⭐ LECTURE DU FICHIER INI
  function getAcJoyIndex(webviewIndex: number, gp: Gamepad | null): number {
    if (!gp || !parsedIni['CONTROLLERS']) return -1;

    const pguid = getPguidFromGamepad(gp);
    if (!pguid) return -1;

    // ⭐ On utilise l'ordre DirectInput (dinputDevices) qui est fiable
    const gamepads = navigator.getGamepads();
    let occurrence = 0;
    for (let i = 0; i < webviewIndex; i++) {
      if (gamepads[i] && getPguidFromGamepad(gamepads[i]!) === pguid) {
        occurrence++;
      }
    }

    const matchingDevices = dinputDevices.filter(d => d.pguid.toUpperCase() === pguid.toUpperCase());
    if (occurrence >= matchingDevices.length) return -1;

    // ⭐ FIX FANATEC GHOST : Si Windows détecte un doublon (périphérique fantôme),
    // la WebView a tendance à mettre le fantôme en premier. On inverse l'index de sélection
    // pour que le "vrai" périphérique récupère le bon numéro de JOY attendu par le controls.ini.
    let targetDevice = matchingDevices[occurrence];
    if (matchingDevices.length > 1) {
        targetDevice = matchingDevices[matchingDevices.length - 1 - occurrence];
    }

    // Le dinput_index correspond à l'ordre dans Windows (et donc dans le fichier CM)
    const targetDinputIndex = targetDevice.dinput_index;

    // On cherche dans le fichier INI si un CON correspond à cet index
    const controllers = parsedIni['CONTROLLERS'];
    for (const key in controllers) {
      if (key.startsWith('__IGUID')) {
        const conIdx = parseInt(key.replace('__IGUID', ''));
        if (!isNaN(conIdx) && conIdx === targetDinputIndex) {
          return conIdx;
        }
      }
    }

    return -1;
  }

  // ⭐ RÉSOLUTION INFaillible : On demande à Rust qui a bougé
  async function resolveAcJoyFromGamepad(webviewIndex: number, gp: Gamepad | null): Promise<number> {
    if (!gp) return -1;
    
    try {
      const currentStates = await invoke<any[]>('get_directinput_states');
      let bestMatchIndex = -1;
      let maxDelta = 0;

      for (let i = 0; i < currentStates.length; i++) {
        const state = currentStates[i];
        let delta = 0;

        for (let a = 0; a < state.axes.length; a++) {
          const normVal = state.axes[a] / 32768.0;
          if (Math.abs(normVal) > 0.15) {
             delta += Math.abs(normVal);
          }
        }

        if (delta > maxDelta) {
          maxDelta = delta;
          bestMatchIndex = i;
        }
      }

      if (bestMatchIndex !== -1) {
        return bestMatchIndex;
      }

      const pguid = getPguidFromGamepad(gp);
      if (pguid && dinputDevices.length > 0) {
        const matchingDevices = dinputDevices.filter(d => d.pguid.toUpperCase() === pguid.toUpperCase());
        if (matchingDevices.length > 0) {
          return matchingDevices[0].dinput_index; 
        }
      }
      
      return getAcJoyIndex(webviewIndex, gp);

    } catch (e) {
      console.warn("Erreur lecture DirectInput states:", e);
      return getAcJoyIndex(webviewIndex, gp);
    }
  }

  // ⭐ LOGIQUE DE TRADUCTION DES AXES (JSON)
  async function loadMappingForGamepad(gp: Gamepad) {
    if (controllerMappings[gp.id] || loadingMappings.has(gp.id)) return;
    loadingMappings.add(gp.id);
    
    const vendorMatch = gp.id.match(/Vendor:\s*([0-9a-fA-F]+)/i);
    const productMatch = gp.id.match(/Product:\s*([0-9a-fA-F]+)/i);
    
    if (vendorMatch && productMatch) {
      try {
        const mapping = await invoke('get_controller_mapping', {
          vendorId: vendorMatch[1],
          productId: productMatch[1]
        });
        if (mapping) {
          controllerMappings[gp.id] = mapping;
        }
      } catch (e) {}
    }
    loadingMappings.delete(gp.id);
  }

  function getAcAxis(webviewAxis: number, gp: Gamepad): number {
    const mapping = controllerMappings[gp.id];
    if (mapping && mapping.axis) {
      const key = webviewAxis.toString();
      if (mapping.axis[key] !== undefined) {
        const acAxis = parseInt(mapping.axis[key]);
        if (!isNaN(acAxis)) return acAxis;
      }
    }
    return webviewAxis;
  }

  function getWebviewAxis(acAxis: number, gp: Gamepad): number {
    const mapping = controllerMappings[gp.id];
    if (mapping && mapping.axis) {
      for (const [webviewKey, acValue] of Object.entries(mapping.axis)) {
        if (parseInt(acValue as string) === acAxis) {
          return parseInt(webviewKey);
        }
      }
    }
    return acAxis;
  }

  // ⭐ BOUCLE DE CAPTURE
  let gamepadInterval: any;
  let captureStartTime = 0;
  let initialGamepadState: any[] = [];
  let liveGamepadData = $state<{ gp: Gamepad, originalIndex: number }[]>([]);
  let refreshInterval: any;

  // ⭐ Force la WebView à rescanner les périphériques
  function refreshGamepads() {
    const currentGamepads = navigator.getGamepads();
    liveGamepadData = Array.from(currentGamepads)
      .map((gp, idx) => ({ gp, originalIndex: idx }))
      .filter(item => item.gp !== null) as { gp: Gamepad, originalIndex: number }[];
  }

  function startGamepadPolling() {
    if (gamepadInterval) return;
    captureStartTime = Date.now(); 
    
    const gamepads = navigator.getGamepads();
    initialGamepadState = [];
    for (let i = 0; i < gamepads.length; i++) {
      const gp = gamepads[i];
      if (!gp) { initialGamepadState.push(null); continue; }
      initialGamepadState.push({
        buttons: gp.buttons.map(b => b.pressed),
        axes: Array.from(gp.axes)
      });
    }

    gamepadInterval = setInterval(() => {
      if (!isCapturing) return;
      const currentGamepads = navigator.getGamepads();
      
      liveGamepadData = Array.from(currentGamepads)
        .map((gp, idx) => ({ gp, originalIndex: idx }))
        .filter(item => item.gp !== null) as { gp: Gamepad, originalIndex: number }[];

      if (Date.now() - captureStartTime < 300) return;

      for (let i = 0; i < currentGamepads.length; i++) {
        const gp = currentGamepads[i];
        if (!gp) continue;
        const initState = initialGamepadState[i];
        if (!initState) continue; 

        for (let b = 0; b < gp.buttons.length; b++) {
          if (gp.buttons[b].pressed && !initState.buttons[b] && !isAssigning) {
            isAssigning = true;
            assignGamepadInput(capturingSection, 'BUTTON', b, i, gp);
            return;
          }
        }
        if (!capturingSection.endsWith('_ALT')) {
          for (let a = 0; a < gp.axes.length; a++) {
            if (Math.abs(gp.axes[a] - initState.axes[a]) > 0.2 && !isAssigning) {
              isAssigning = true;
              const initVal = initState.axes[a];
              const isInverted = (initVal > 0.5); 
              assignGamepadInput(capturingSection, 'AXLE', a, i, gp, isInverted);
              return;
            }
          }
        }
      }
    }, 50);
  }

    async function assignGamepadInput(section: string, type: 'BUTTON'|'AXLE', webviewIndex: number, joyIndex: number, gp: Gamepad, isInverted: boolean = false) {
    
    // ⭐ On utilise la version asynchrone qui enregistre le device s'il est absent
    const realJoyIndex = await resolveAcJoyFromGamepad(joyIndex, gp);
    
    // ⭐ GARDE-FOU ULTIME : Si même l'enregistrement automatique échoue (périphérique exotique DirectInput)
    if (realJoyIndex === -1) {
      alert($t('settings.controls.err_map_device'));
      closeCapture();
      return;
    }

    if (!parsedIni[section]) parsedIni[section] = { _raw: {} };
    
    let finalIndex = webviewIndex;
    if (type === 'AXLE') finalIndex = getAcAxis(webviewIndex, gp);

    // ⭐ GESTION SPECIALE SHIFTER (Boîte en H) - CORRIGÉ
    if (section.startsWith('GEAR_') && type === 'BUTTON') {
      // 1. On initialise la section SHIFTER si elle n'existe pas
      if (!parsedIni['SHIFTER']) parsedIni['SHIFTER'] = { _raw: {} };
      
      // 2. On configure le SHIFTER avec le bon JOY
      parsedIni['SHIFTER']['ACTIVE'] = '1';
      parsedIni['SHIFTER']['JOY'] = realJoyIndex.toString();
      
      // 3. On synchronise le numéro de bouton pour cette vitesse dans SHIFTER
      parsedIni['SHIFTER'][section] = finalIndex.toString();
      
      // 4. ⭐ NOUVEAU : On assigne AUSSI normalement dans GEAR_X (comme le fait Content Manager)
      // C'est ce qui manquait pour que ça marche !
      parsedIni[section][type] = finalIndex.toString();
      parsedIni[section]['JOY'] = realJoyIndex.toString();
      parsedIni[section]['KEY'] = '-1';
    }

    if (section.endsWith('_ALT')) {
      const baseSection = section.replace('_ALT', '');
      if (!parsedIni[baseSection]) parsedIni[baseSection] = { _raw: {} };
      
      if (type === 'BUTTON') {
        parsedIni[baseSection]['__CM_ALT_BUTTON'] = finalIndex.toString();
        parsedIni[baseSection]['__CM_ALT_JOY'] = realJoyIndex.toString();
      } else {
        return; 
      }
    } else {
      parsedIni[section][type] = finalIndex.toString();
      parsedIni[section]['JOY'] = realJoyIndex.toString();
      
      if (type === 'BUTTON') {
        parsedIni[section]['KEY'] = '-1';
      } else { 
        // ⭐ NOUVEAU : Auto-calibration pour les axes directionnels (Joystick Fanatec)
        if (section.startsWith('GLANCE')) {
          // On lit la valeur exacte de l'axe à l'instant où tu bouges le joystick
          const currentVal = gp.axes[webviewIndex];
          
          // On crée une petite fenêtre de déclenchement autour de cette valeur (+/- 0.3)
          let calcMin = currentVal - 0.3;
          let calcMax = currentVal + 0.3;

          // On s'assure de ne pas dépasser les limites du jeu (-1.00 à 1.00)
          calcMin = Math.max(-1.00, calcMin);
          calcMax = Math.min(1.00, calcMax);

          parsedIni[section]['MIN'] = calcMin.toFixed(2);
          parsedIni[section]['MAX'] = calcMax.toFixed(2);
        } else {
          // Comportement normal pour les pédales, le volant et le frein à main
          if (isInverted) {
            parsedIni[section]['MIN'] = "1"; 
            parsedIni[section]['MAX'] = "-1";
          } else {
            parsedIni[section]['MIN'] = "-1"; 
            parsedIni[section]['MAX'] = "1";
          }
        }
      }
    }
    
    iniContent = stringifyIni(parsedIni);
    hasChanges = true;
    closeCapture();
  }

  // ⭐ NOUVEAU : Désassigner une touche
  function unbindAction(section: string) {
    // On sépare la section de base et le suffixe _ALT
    let baseSection = section;
    let isAlt = false;
    
    if (section.endsWith('_ALT')) {
      baseSection = section.replace('_ALT', '');
      isAlt = true;
    }

    if (!parsedIni[baseSection]) return;

    if (isAlt) {
      // On reset UNIQUEMENT la partie Alt
      parsedIni[baseSection]['__CM_ALT_BUTTON'] = '-1';
      parsedIni[baseSection]['__CM_ALT_JOY'] = '-1';
    } else {
      // ⭐ Si c'est une vitesse, on la retire aussi du SHIFTER
      if (baseSection.startsWith('GEAR_') && parsedIni['SHIFTER']) {
        delete parsedIni['SHIFTER'][baseSection];
        // Si plus aucune vitesse n'est assignée, on désactive le shifter
        const hasGears = ['GEAR_1','GEAR_2','GEAR_3','GEAR_4','GEAR_5','GEAR_6','GEAR_7','GEAR_R'].some(g => parsedIni['SHIFTER'][g] !== undefined && parsedIni['SHIFTER'][g] !== '-1');
        if (!hasGears) {
          parsedIni['SHIFTER']['ACTIVE'] = '0';
        }
      }
      
      // On reset l'assignation principale
      parsedIni[baseSection]['BUTTON'] = '-1';
      parsedIni[baseSection]['AXLE'] = '-1';
      parsedIni[baseSection]['JOY'] = '-1';
      parsedIni[baseSection]['KEY'] = '-1';
      
      // Par sécurité, on reset aussi le Alt quand on unbind la pédale principale
      parsedIni[baseSection]['__CM_ALT_BUTTON'] = '-1';
      parsedIni[baseSection]['__CM_ALT_JOY'] = '-1';
    }

    iniContent = stringifyIni(parsedIni);
    hasChanges = true;
  }

  function toggleInvert(section: string) {
    if (!parsedIni[section]) return;
    const isInverted = parsedIni[section]['MIN'] === '1' || parsedIni[section]['MIN'] === '1.00' || (parseFloat(parsedIni[section]['MIN'] || '0') > parseFloat(parsedIni[section]['MAX'] || '1'));
    if (isInverted) {
      parsedIni[section]['MIN'] = '-1';
      parsedIni[section]['MAX'] = '1';
    } else {
      parsedIni[section]['MIN'] = '1';
      parsedIni[section]['MAX'] = '-1';
    }
    iniContent = stringifyIni(parsedIni);
    hasChanges = true;
  }

  function stopGamepadPolling() {
    if (gamepadInterval) { clearInterval(gamepadInterval); gamepadInterval = null; }
  }

  $effect(() => {
    if (isCapturing) startGamepadPolling();
    else stopGamepadPolling();
  });

  // ⭐ TEMPS RÉEL POUR L'AFFICHAGE
  let demoValues = $state({ brake: 0, throttle: 0, clutch: 0, handbrake: 0, steer: 50 });
  let animationFrameId: number;
  let gamepadStatus = $state('Recherche de périphérique... Appuie sur une pédale pour activer.');
  let liveAxisValues = $state<Record<string, number>>({});
  let liveButtonStates = $state<Record<string, boolean>>({});

  function updateLiveInputs() {
    const currentGamepads = navigator.getGamepads();
    let connectedDevice = null;

    liveGamepadData = Array.from(currentGamepads)
      .map((gp, idx) => ({ gp, originalIndex: idx }))
      .filter(item => item.gp !== null) as { gp: Gamepad, originalIndex: number }[];

    for (let i = 0; i < currentGamepads.length; i++) {
      const gp = currentGamepads[i];
      if (!gp) continue;
      
      if (!connectedDevice) connectedDevice = gp;
      const acJoyIdx = getAcJoyIndex(i, gp);

      Object.keys(actionDefinitions).forEach(actionId => {
        const section = parsedIni[actionId];
        if (section && section['AXLE'] !== undefined && section['AXLE'] !== '-1' && section['JOY'] !== undefined) {
          if (section['JOY'] === acJoyIdx.toString()) {
            const webviewAxis = getWebviewAxis(parseInt(section['AXLE']), gp);
            if (gp.axes[webviewAxis] !== undefined) {
              liveAxisValues[actionId] = gp.axes[webviewAxis];
            }
          }
        }
      });

                Object.keys(parsedIni).forEach(sectionId => {
            const section = parsedIni[sectionId];
            if (section && section['BUTTON'] !== undefined && section['BUTTON'] !== '-1') {
              if (section['JOY'] !== undefined && section['JOY'] === acJoyIdx.toString()) {
                const btnIdx = parseInt(section['BUTTON']);
                if (gp.buttons[btnIdx] !== undefined) {
                  liveButtonStates[sectionId] = gp.buttons[btnIdx].pressed;
                }
              }
            }
            if (section && section['__CM_ALT_BUTTON'] !== undefined && section['__CM_ALT_BUTTON'] !== '-1') {
              const altJoy = section['__CM_ALT_JOY'] !== undefined ? section['__CM_ALT_JOY'] : section['JOY'];
              if (altJoy !== undefined && altJoy === acJoyIdx.toString()) {
                const btnIdx = parseInt(section['__CM_ALT_BUTTON']);
                if (gp.buttons[btnIdx] !== undefined) {
                  liveButtonStates[sectionId + '_ALT'] = gp.buttons[btnIdx].pressed;
                }
              }
            }
          });

          // ⭐ NOUVEAU : Gestion visuelle de la Boîte en H via la section [SHIFTER] d'Assetto Corsa
          const shifter = parsedIni['SHIFTER'];
          if (shifter && shifter['ACTIVE'] === '1' && shifter['JOY'] === acJoyIdx.toString()) {
            const gears = ['GEAR_1', 'GEAR_2', 'GEAR_3', 'GEAR_4', 'GEAR_5', 'GEAR_6', 'GEAR_7', 'GEAR_R'];
            gears.forEach(gear => {
              if (shifter[gear] !== undefined && shifter[gear] !== '-1') {
                const btnIdx = parseInt(shifter[gear]);
                if (gp.buttons[btnIdx] !== undefined) {
                  liveButtonStates[gear] = gp.buttons[btnIdx].pressed;
                }
              }
            });
          }

      if (currentMode === 'simple' && !isCapturing) {
        const mapAxis = (val: number, invert: boolean) => {
          if (val === undefined) return 0;
          let mapped = invert ? ((-val + 1) / 2) * 100 : ((val + 1) / 2) * 100;
          return Math.max(0, Math.min(100, Math.round(mapped)));
        };

        const readAction = (actionId: string) => {
          const section = parsedIni[actionId];
          if (!section || section['AXLE'] === undefined || section['AXLE'] === '-1') return null;
          
          if (section['JOY'] !== undefined && section['JOY'] === acJoyIdx.toString()) {
            const webviewAxis = getWebviewAxis(parseInt(section['AXLE']), gp);
            if (gp.axes[webviewAxis] !== undefined) {
              const min = parseFloat(section['MIN'] || '0');
              const max = parseFloat(section['MAX'] || '1');
              const isInverted = min > max; 
              return mapAxis(gp.axes[webviewAxis], isInverted);
            }
          }
          return null;
        };

        const steerVal = readAction('STEER'); if (steerVal !== null) demoValues.steer = steerVal;
        const clutchVal = readAction('CLUTCH'); if (clutchVal !== null) demoValues.clutch = clutchVal;
        const throttleVal = readAction('THROTTLE'); if (throttleVal !== null) demoValues.throttle = throttleVal;
        const brakeVal = readAction('BRAKES'); if (brakeVal !== null) demoValues.brake = brakeVal;
        const handbrakeVal = readAction('HANDBRAKE'); if (handbrakeVal !== null) demoValues.handbrake = handbrakeVal;
      }
    }
    
    if (connectedDevice) {
      gamepadStatus = `✅ Connecté : ${connectedDevice.id.substring(0, 30)}...`;
    } else {
      gamepadStatus = '⚠️ Aucun périphérique détecté. Appuie sur une pédale ou tourne le volant.';
    }

    // ⭐ On ne continue la boucle que si le composant est actif
    if (active) {
      animationFrameId = requestAnimationFrame(updateLiveInputs);
    }
  }

  onMount(() => {
    loadMenuState(); // ⭐ On restaure l'état des menus
    loadProfiles();
    loadIni();
    loadDInputDevices();
    loadControllersSection(); // ⭐ Nouveau : On charge la section [CONTROLLERS]

    // ⭐ LOT 3 : Écoute du Watchdog
    let unlistenControllers: any;
    listen('controllers-changed', async () => {
      // ⭐ On utilise confirm au lieu d'alert pour avoir 2 boutons
      const shouldRestart = await confirm(
        $t('settings.controls.device_changed_msg'),
        $t('settings.controls.device_changed_title'),
        "warning",
        $t('settings.controls.btn_restart'),
        $t('settings.controls.btn_ignore')
      );
      
      if (shouldRestart) {
        window.location.reload();
      }
    }).then(unlisten => { unlistenControllers = unlisten; });
    
    // ⭐ On ne lance la boucle au démarrage que si on est sur le bon onglet
    if (active) {
      animationFrameId = requestAnimationFrame(updateLiveInputs);
    }
    
    window.addEventListener('keydown', handleKeyDown);

    // ⭐ Rafraîchissement automatique toutes les 2 secondes
    refreshInterval = setInterval(() => {
      if (!isCapturing) refreshGamepads();
    }, 2000);
    
    window.addEventListener('gamepadconnected', (e) => {
      gamepadStatus = `✅ Connecté : ${e.gamepad.id.substring(0, 30)}...`;
      loadMappingForGamepad(e.gamepad);
    });

    const initialGamepads = navigator.getGamepads();
    for (let i = 0; i < initialGamepads.length; i++) {
      if (initialGamepads[i]) loadMappingForGamepad(initialGamepads[i]!);
    }
    window.addEventListener('gamepaddisconnected', () => {
      gamepadStatus = '⚠️ Périphérique déconnecté.';
    });
    
    return () => {
      window.removeEventListener('keydown', handleKeyDown);
      if (animationFrameId) cancelAnimationFrame(animationFrameId);
      if (refreshInterval) clearInterval(refreshInterval);
      if (unlistenControllers) unlistenControllers(); // ⭐ Nettoyage du Watchdog
      stopGamepadPolling();
    };
  });

  $effect(() => {
    if (!selectedProfilePath) { deviceType; loadIni(); }
  });

      const actionDefinitions = {
    // Axes
    'STEER': { label: 'settings.controls.actions.STEER', cat: 'settings.controls.categories.axes', type: 'axis' },
    'THROTTLE': { label: 'settings.controls.actions.THROTTLE', cat: 'settings.controls.categories.axes', type: 'axis' },
    'BRAKES': { label: 'settings.controls.actions.BRAKES', cat: 'settings.controls.categories.axes', type: 'axis' },
    'CLUTCH': { label: 'settings.controls.actions.CLUTCH', cat: 'settings.controls.categories.axes', type: 'axis' },
    'HANDBRAKE': { label: 'settings.controls.actions.HANDBRAKE', cat: 'settings.controls.categories.axes', type: 'mixed' },

    // Moteur & Conduite
    'STARTER': { label: 'settings.controls.actions.STARTER', cat: 'settings.controls.categories.engine', type: 'button' },
    'KERS': { label: 'settings.controls.actions.KERS', cat: 'settings.controls.categories.engine', type: 'button' },
    'DRS': { label: 'settings.controls.actions.DRS', cat: 'settings.controls.categories.engine', type: 'button' },
    '__EXT_PIT_LIMITER': { label: 'settings.controls.actions.__EXT_PIT_LIMITER', cat: 'settings.controls.categories.engine', type: 'button' },
    'ACTIVATE_AI': { label: 'settings.controls.actions.ACTIVATE_AI', cat: 'settings.controls.categories.engine', type: 'button' },
    'AUTO_SHIFTER': { label: 'settings.controls.actions.AUTO_SHIFTER', cat: 'settings.controls.categories.engine', type: 'button' },
    'SLOWMO': { label: 'settings.controls.actions.SLOWMO', cat: 'settings.controls.categories.engine', type: 'button' },
    'REV': { label: 'settings.controls.actions.REV', cat: 'settings.controls.categories.engine', type: 'button' },

    // Aides & Réglages
    'ABS': { label: 'settings.controls.actions.ABS', cat: 'settings.controls.categories.assists', type: 'button' },
    'TRACTION_CONTROL': { label: 'settings.controls.actions.TRACTION_CONTROL', cat: 'settings.controls.categories.assists', type: 'button' },
    '__CM_ENGINE_BRAKE': { label: 'settings.controls.actions.__CM_ENGINE_BRAKE', cat: 'settings.controls.categories.assists', type: 'button' },
    'BALANCEUP': { label: 'settings.controls.actions.BALANCEUP', cat: 'settings.controls.categories.assists', type: 'button' },
    'BALANCEDN': { label: 'settings.controls.actions.BALANCEDN', cat: 'settings.controls.categories.assists', type: 'button' },

    // Session & Course
    '__CM_PAUSE': { label: 'settings.controls.actions.__CM_PAUSE', cat: 'settings.controls.categories.session', type: 'button' },
    'RESET_RACE': { label: 'settings.controls.actions.RESET_RACE', cat: 'settings.controls.categories.session', type: 'button' },
    '__CM_TO_PITS': { label: 'settings.controls.actions.__CM_TO_PITS', cat: 'settings.controls.categories.session', type: 'button' },
    '__CM_START_SESSION': { label: 'settings.controls.actions.__CM_START_SESSION', cat: 'settings.controls.categories.session', type: 'button' },
    '__CM_SETUP_CAR': { label: 'settings.controls.actions.__CM_SETUP_CAR', cat: 'settings.controls.categories.session', type: 'button' },
    '__CM_START_STOP_SESSION': { label: 'settings.controls.actions.__CM_START_STOP_SESSION', cat: 'settings.controls.categories.session', type: 'button' },

    // Éclairage & Signalisation
    'ACTION_HEADLIGHTS': { label: 'settings.controls.actions.ACTION_HEADLIGHTS', cat: 'settings.controls.categories.lights', type: 'button' },
    'ACTION_HEADLIGHTS_HIGH_LOW': { label: 'settings.controls.actions.ACTION_HEADLIGHTS_HIGH_LOW', cat: 'settings.controls.categories.lights', type: 'button' },
    'ACTION_HEADLIGHTS_FLASH': { label: 'settings.controls.actions.ACTION_HEADLIGHTS_FLASH', cat: 'settings.controls.categories.lights', type: 'button' },
    '__EXT_TURNLIGHTS_LEFT': { label: 'settings.controls.actions.__EXT_TURNSIGNAL_LEFT', cat: 'settings.controls.categories.lights', type: 'button' },
    '__EXT_TURNLIGHTS_RIGHT': { label: 'settings.controls.actions.__EXT_TURNSIGNAL_RIGHT', cat: 'settings.controls.categories.lights', type: 'button' },
    '__EXT_HAZARDS': { label: 'settings.controls.actions.__EXT_HAZARDS', cat: 'settings.controls.categories.lights', type: 'button' },
    'ACTION_HORN': { label: 'settings.controls.actions.ACTION_HORN', cat: 'settings.controls.categories.lights', type: 'button' },

    // Vue & Interface
    'ACTION_CHANGE_CAMERA': { label: 'settings.controls.actions.ACTION_CHANGE_CAMERA', cat: 'settings.controls.categories.view', type: 'button' },
    'GLANCELEFT': { label: 'settings.controls.actions.GLANCELEFT', cat: 'settings.controls.categories.view', type: 'mixed' },
    'GLANCERIGHT': { label: 'settings.controls.actions.GLANCERIGHT', cat: 'settings.controls.categories.view', type: 'mixed' },
    'GLANCEBACK': { label: 'settings.controls.actions.GLANCEBACK', cat: 'settings.controls.categories.view', type: 'mixed' },
    '__EXT_VIRTUAL_MIRROR': { label: 'settings.controls.actions.__EXT_VIRTUAL_MIRROR', cat: 'settings.controls.categories.view', type: 'button' },
    'HIDE_APPS': { label: 'settings.controls.actions.HIDE_APPS', cat: 'settings.controls.categories.view', type: 'button' },
    'HIDE_DAMAGE': { label: 'settings.controls.actions.HIDE_DAMAGE', cat: 'settings.controls.categories.view', type: 'button' },
    'SHOW_DAMAGE': { label: 'settings.controls.actions.SHOW_DAMAGE', cat: 'settings.controls.categories.view', type: 'button' },
    'DRIVER_NAMES': { label: 'settings.controls.actions.DRIVER_NAMES', cat: 'settings.controls.categories.view', type: 'button' },
    'IDEAL_LINE': { label: 'settings.controls.actions.IDEAL_LINE', cat: 'settings.controls.categories.view', type: 'button' },
    '__EXT_WIPERS_MORE': { label: 'settings.controls.actions.__EXT_WIPERS_MORE', cat: 'settings.controls.categories.view', type: 'button' },
    '__EXT_WIPERS_LESS': { label: 'settings.controls.actions.__EXT_WIPERS_LESS', cat: 'settings.controls.categories.view', type: 'button' },
    '__EXT_WIPERS_OFF': { label: 'settings.controls.actions.__EXT_WIPERS_OFF', cat: 'settings.controls.categories.view', type: 'button' },

    // Replay & Avancé
    'START_REPLAY': { label: 'settings.controls.actions.START_REPLAY', cat: 'settings.controls.categories.replay', type: 'button' },
    'PAUSE_REPLAY': { label: 'settings.controls.actions.PAUSE_REPLAY', cat: 'settings.controls.categories.replay', type: 'button' },
    'NEXT_LAP': { label: 'settings.controls.actions.NEXT_LAP', cat: 'settings.controls.categories.replay', type: 'button' },
    'PREVIOUS_LAP': { label: 'settings.controls.actions.PREVIOUS_LAP', cat: 'settings.controls.categories.replay', type: 'button' },
    'NEXT_CAR': { label: 'settings.controls.actions.NEXT_CAR', cat: 'settings.controls.categories.replay', type: 'button' },
    'PLAYER_CAR': { label: 'settings.controls.actions.PLAYER_CAR', cat: 'settings.controls.categories.replay', type: 'button' },
    'PREVIOUS_CAR': { label: 'settings.controls.actions.PREVIOUS_CAR', cat: 'settings.controls.categories.replay', type: 'button' }
  };

  // ⭐ GESTION DE LA PERSISTANCE DES MENUS (localStorage)
  function loadMenuState() {
    try {
      const saved = localStorage.getItem('qarl_controls_menu_state');
      if (saved) {
        const parsed = JSON.parse(saved);
        // On restaure l'état des catégories (ouvertes ou fermées)
        openCategories = parsed.categories || {};
        // On restaure l'état des deux menus du haut
        openDinputDevices = parsed.dinput ?? false;
        openWebviewDevices = parsed.webview ?? false;
      }
    } catch (e) {
      console.warn("Impossible de charger l'état des menus", e);
    }
  }

  function saveMenuState() {
    try {
      localStorage.setItem('qarl_controls_menu_state', JSON.stringify({
        categories: openCategories,
        dinput: openDinputDevices,
        webview: openWebviewDevices
      }));
    } catch (e) {
      console.warn("Impossible de sauvegarder l'état des menus", e);
    }
  }

  // ⭐ État pour gérer l'ouverture/fermeture des catégories
  // (Fermé par défaut au premier lancement)
  let openCategories = $state<Record<string, boolean>>({});
  let openDinputDevices = $state(false);
  let openWebviewDevices = $state(false);

  const categories = [...new Set(Object.values(actionDefinitions).map(a => a.cat))];
  
  const groupedActions = categories.map(cat => {
    return {
      name: cat,
      items: Object.entries(actionDefinitions)
        .filter(([id, def]) => def.cat === cat)
        .map(([id, def]) => ({ id, ...def }))
    };
  });

  // ⭐ Écouteur de visibilité : Stop la boucle si on quitte l'onglet, relance si on revient
  $effect(() => {
    if (active) {
      if (!animationFrameId) {
        animationFrameId = requestAnimationFrame(updateLiveInputs);
      }
    } else {
      if (animationFrameId) {
        cancelAnimationFrame(animationFrameId);
        animationFrameId = undefined as any;
      }
    }
  });
</script>

{#if !selectedProfilePath && !showSaveAs}
  <AutosaveBanner
    hasChanges={hasChanges}
    onSave={() => saveIni(false)}
  />
{/if}

<div class="qacrl-ctrl-root">
  <div class="qacrl-ctrl-section-bare">
    <div class="qacrl-ctrl-method-picker">
      <button class="qacrl-ctrl-method-btn" class:active={deviceType === 'wheel' && !selectedProfilePath} onclick={() => { deviceType = 'wheel'; selectedProfilePath = ''; }}>
        <span class="qacrl-ctrl-method-icon">🏎️</span><span class="qacrl-ctrl-method-label">{$t('settings.controls.device_wheel')}</span>
      </button>
      <button class="qacrl-ctrl-method-btn" class:active={deviceType === 'x360' && !selectedProfilePath} onclick={() => { deviceType = 'x360'; selectedProfilePath = ''; }}>
        <span class="qacrl-ctrl-method-icon">🎮</span><span class="qacrl-ctrl-method-label">{$t('settings.controls.device_gamepad')}</span>
      </button>
      <button class="qacrl-ctrl-method-btn" class:active={deviceType === 'keyboard' && !selectedProfilePath} onclick={() => { deviceType = 'keyboard'; selectedProfilePath = ''; }}>
        <span class="qacrl-ctrl-method-icon">⌨️</span><span class="qacrl-ctrl-method-label">{$t('settings.controls.device_keyboard')}</span>
      </button>
    </div>
  </div>

  <div class="ini-toolbar" style="flex-wrap: wrap; gap: 10px;">
    <div class="ini-mode-toggle">
      <button class="ini-mode-btn" class:active={currentMode === 'simple'} onclick={() => currentMode = 'simple'}>{$t('settings.controls.mode_simple')}</button>
      <button class="ini-mode-btn" class:active={currentMode === 'advanced'} onclick={() => currentMode = 'advanced'}>{$t('settings.controls.mode_advanced')}</button>
      <button class="ini-mode-btn" class:active={currentMode === 'expert'} onclick={() => currentMode = 'expert'}>{$t('settings.controls.mode_expert')}</button>
    </div>

    <select class="ini-select" style="min-width: 200px; flex: 1;" bind:value={selectedProfilePath} onchange={() => loadIni(selectedProfilePath)}>
      <option value="">{$t('settings.controls.active_file', { values: { type: deviceType === 'wheel' ? 'controls.ini' : deviceType === 'x360' ? 'x360.ini' : 'keyboard.ini' } })}</option>
      <optgroup label={$t('settings.controls.profiles_saved')}>
        {#each profiles.filter(p => p.profile_type === 'savedsetup') as p}
          <option value={p.path}>{p.name}</option>
        {/each}
      </optgroup>
      <optgroup label={$t('settings.controls.profiles_controllers')}>
        {#each profiles.filter(p => p.profile_type === 'controller') as p}
          <option value={p.path}>{p.name}</option>
        {/each}
      </optgroup>
    </select>
    
    <div class="ini-toolbar-actions">
      <button class="btn-secondary" onclick={() => loadIni(selectedProfilePath)} disabled={isLoading}>{$t('settings.controls.btn_reload')}</button>
      
      {#if selectedProfilePath}
        <button class="btn-secondary" style="background: var(--success); color: white; border-color: var(--success);" onclick={applyProfile}>
          {$t('settings.controls.btn_apply')}
        </button>
      {:else}
        {#if showSaveAs}
          <input type="text" class="ini-text" style="width: 150px;" placeholder={$t('settings.controls.save_as_placeholder')} bind:value={saveAsName} />
          <button class="btn-secondary ini-btn-save" onclick={saveIni} disabled={!saveAsName}>{$t('settings.controls.btn_save_new')}</button>
          <button class="btn-secondary" onclick={() => showSaveAs = false}>{$t('settings.controls.btn_cancel')}</button>
        {:else}
          <button class="btn-secondary" onclick={() => showSaveAs = true} title={$t('settings.controls.title_save_as')}>{$t('settings.controls.btn_copy')}</button>
          <button class="btn-secondary ini-btn-save" class:dirty={hasChanges} onclick={saveIni} disabled={isSaving || !hasChanges}>
            {$t('settings.controls.btn_save')}
          </button>
        {/if}
      {/if}
    </div>
  </div>

  {#if !appState.documentsPath}
    <div class="server-join-warning error" style="margin-top: 20px;">{$t('settings.controls.no_docs_path')}</div>
  {:else}
        {#if currentMode === 'simple'}
      <div class="group" style="margin-top: 15px; background: var(--bg-panel);">
        <div style="display: flex; justify-content: space-between; align-items: baseline;">
          <div class="group-title">{$t('settings.controls.cockpit_overview')}</div>
          <span style="font-size: 0.7rem; color: var(--text-tertiary);">{$t('settings.controls.hint_click')}</span>
        </div>

        <div style="display: flex; flex-direction: column; gap: 20px; margin-top: 10px;">
          
          <!-- ⭐ SNIPPET POUR LES BOUTONS -->
                    {#snippet cockpitBtn(section, IconComponent, label)}
            {@const assigned = isAssigned(section)}
            {@const axisVal = liveAxisValues[section]}
            {@const min = parseFloat(parsedIni[section]?.['MIN'] || '-1')}
            {@const max = parseFloat(parsedIni[section]?.['MAX'] || '1')}
            {@const isAxisActive = axisVal !== undefined && parsedIni[section]?.['AXLE'] !== '-1' && Math.abs(axisVal) > 0.1 && axisVal >= Math.min(min, max) && axisVal <= Math.max(min, max)}
            {@const isPressed = liveButtonStates[section] === true || isAxisActive}
            
            {@const val = parsedIni[section]?.['BUTTON'] !== '-1' && parsedIni[section]?.['BUTTON'] 
              ? $t('settings.controls.btn_prefix') + ' ' + parsedIni[section]?.['BUTTON'] 
              : (parsedIni[section]?.['AXLE'] !== '-1' && parsedIni[section]?.['AXLE'] 
                ? $t('settings.controls.axis_prefix') + ' ' + parsedIni[section]?.['AXLE'] 
                : (parsedIni[section]?.['KEY'] !== '-1' && parsedIni[section]?.['KEY'] ? $t('settings.controls.key_prefix') : ''))}
            <button 
              class="filter-btn" 
              onclick={() => openCapture(section, label)} 
              oncontextmenu={(e) => { e.preventDefault(); unbindAction(section); }}
              style="
                width: 100%; 
                max-width: 95px; 
                height: 95px; 
                border-radius: 8px; 
                display: flex; 
                flex-direction: column; 
                align-items: center; 
                justify-content: center; 
                cursor: pointer; 
                transition: all 0.15s ease; 
                /* ⭐ Bordure plus contrastée */
                border: 1px solid {isPressed ? 'var(--accent)' : (assigned ? 'var(--border-strong)' : 'var(--border)')}; 
                /* ⭐ Fond plus clair pour détacher le bouton de la carte */
                background: {isPressed ? 'var(--accent-muted)' : 'var(--bg-elevated)'}; 
                /* ⭐ Ombre portée pour l'effet de relief */
                box-shadow: {isPressed ? '0 0 15px var(--accent-glow), inset 0 0 10px var(--accent-muted)' : '0 4px 6px rgba(0,0,0,0.3), inset 0 1px 0 rgba(255,255,255,0.05)'}; 
                transform: {isPressed ? 'scale(0.95)' : 'scale(1)'}; 
                padding: 8px 4px; 
                margin: 0 auto; 
              "
            >
              <div style="color: {isPressed ? 'var(--accent)' : (assigned ? 'var(--text-primary)' : 'var(--text-tertiary)')}; filter: {isPressed ? 'drop-shadow(0 0 6px var(--accent-glow))' : 'none'}; transition: all 0.15s;">
                <IconComponent size={22} strokeWidth={1.8} />
              </div>
              
              <span style="
                font-size: 0.65rem; 
                font-weight: 600; 
                text-align: center; 
                line-height: 1.1; 
                color: {isPressed ? 'var(--accent)' : (assigned ? 'var(--text-primary)' : 'var(--text-secondary)')};
                width: 100%;
                word-wrap: break-word;
                margin-top: 6px;
                display: -webkit-box;
                -webkit-line-clamp: 2;
                -webkit-box-orient: vertical;
                overflow: hidden;
              ">{$t(label)}</span>
              
              <span style="
                font-size: 0.55rem; 
                color: {isPressed ? 'var(--accent)' : 'var(--text-tertiary)'}; 
                font-weight: 700; 
                letter-spacing: 0.5px;
                margin-top: auto;
              ">
                {#if assigned}{val}{/if}
              </span>
            </button>
          {/snippet}

          {#snippet compactBtn(section, isPlus)}
            {@const isPressed = liveButtonStates[section] === true}
            {@const val = parsedIni[section]?.['BUTTON'] !== '-1' && parsedIni[section]?.['BUTTON'] ? `Btn ${parsedIni[section]?.['BUTTON']}` : (parsedIni[section]?.['AXLE'] !== '-1' ? `Axe ${parsedIni[section]?.['AXLE']}` : '')}
            <button 
              onclick={() => openCapture(section, section)}
              oncontextmenu={(e) => { e.preventDefault(); unbindAction(section); }}
              style="
                display: flex; flex-direction: column; align-items: center; justify-content: center;
                width: 38px; height: 38px; border-radius: 6px;
                border: 1px solid {isPressed ? 'var(--accent)' : 'var(--border)'};
                background: {isPressed ? 'var(--accent-muted)' : 'var(--bg-panel)'};
                color: {isPressed ? 'var(--accent)' : 'var(--text-primary)'};
                cursor: pointer; transition: all 0.15s;
                box-shadow: {isPressed ? '0 0 10px var(--accent-glow)' : '0 2px 4px rgba(0,0,0,0.2)'};
              "
            >
              {#if isPlus}
                <Plus size={16} strokeWidth={2.5} />
              {:else}
                <Minus size={16} strokeWidth={2.5} />
              {/if}
              <span style="font-size: 0.4rem; color: {isPressed ? 'var(--accent)' : 'var(--text-tertiary)'}; margin-top: 2px; font-weight: 700; line-height: 1;">{val}</span>
            </button>
          {/snippet}

          <!-- ⭐ SNIPPET POUR LES PÉDALES (Style Image 3D) -->
          {#snippet pedalCard(section, imagePath, label, color, isAlt = false)}
            {@const demoKey = section === 'BRAKES' ? 'brake' : section.toLowerCase()}
            {@const altPressed = liveButtonStates['HANDBRAKE_ALT'] === true}
            {@const currentVal = isAlt ? 0 : (demoValues[demoKey] || 0)}
            {@const assignedAxis = parsedIni[section]?.['AXLE']}
            
            <div style="display: flex; align-items: stretch; gap: 15px; background: var(--bg-panel); border: 1px solid var(--border-subtle); border-radius: var(--r-sm); padding: 12px; width: 100%; transition: border-color 0.2s;">
              
              <!-- Image de la pédale (à gauche) -->
              <div 
                style="display: flex; align-items: center; justify-content: center; width: 60px; cursor: pointer; position: relative;" 
                onclick={() => openCapture(isAlt ? section + '_ALT' : section, label)}
                oncontextmenu={(e) => { e.preventDefault(); unbindAction(isAlt ? section + '_ALT' : section); }}
              >
                <img 
                  src={imagePath} 
                  alt={label} 
                  style="
                    width: 100%; height: 85px; object-fit: contain;
                    /* ⭐ Effet visuel : s'enfonce légèrement quand on appuie */
                    transform: translateY({currentVal > 5 ? '3px' : '0px'}) rotate({currentVal > 5 ? '1deg' : '0deg'});
                    transition: transform 0.1s ease, filter 0.1s ease;
                    /* ⭐ Halo lumineux de la couleur de la pédale quand elle est pressée */
                    filter: {currentVal > 5 ? `drop-shadow(0 0 15px ${color})` : `drop-shadow(0 4px 8px rgba(0,0,0,0.5))`};
                  " 
                />
              </div>
              
              <!-- Détails de la pédale (à droite) -->
              <div style="display: flex; flex-direction: column; flex: 1; justify-content: center; gap: 6px;">
                
                <div style="display: flex; justify-content: space-between; align-items: flex-end;">
                  <div style="cursor: pointer;" onclick={() => openCapture(isAlt ? section + '_ALT' : section, label)}>
                    <div style="font-size: 0.85rem; font-weight: 700; color: var(--text-primary);">{$t(label)}</div>
                    <div style="font-size: 0.7rem; color: var(--text-tertiary);">
                      {assignedAxis && assignedAxis !== '-1' ? `${$t('settings.controls.axis_prefix')} ${assignedAxis}` : $t('settings.controls.unassigned')}
                    </div>
                  </div>
                  <div style="font-size: 0.85rem; font-weight: 800; color: {color};">
                    {currentVal}%
                  </div>
                </div>
                
                <!-- Barre de progression horizontale -->
                <div style="width: 100%; height: 8px; background: var(--bg-elevated); border-radius: 4px; overflow: hidden; cursor: pointer;" onclick={() => openCapture(isAlt ? section + '_ALT' : section, label)}>
                  <div style="height: 100%; width: {currentVal}%; background-color: {color}; transition: width 0.1s linear;"></div>
                </div>
                
                <!-- Boutons du bas (Inverser + Alt) -->
                <div style="display: flex; justify-content: space-between; align-items: center; margin-top: 8px; gap: 10px;">
                  
                  <!-- Toggle Inverser (Style interrupteur) -->
                  <label style="display: flex; align-items: center; gap: 8px; font-size: 0.7rem; cursor: pointer; color: var(--text-secondary);">
                    <div style="position: relative; width: 28px; height: 16px; background: {parsedIni[section]?.['MIN'] === '1' ? 'var(--accent)' : 'var(--border-strong)'}; border-radius: 8px; transition: background 0.2s;">
                      <div style="position: absolute; top: 2px; left: {parsedIni[section]?.['MIN'] === '1' ? '14px' : '2px'}; width: 12px; height: 12px; background: #fff; border-radius: 50%; transition: left 0.2s;"></div>
                    </div>
                    <input type="checkbox" checked={parsedIni[section]?.['MIN'] === '1'} onchange={() => toggleInvert(section)} style="display: none;" /> 
                    {$t('settings.controls.invert')}
                  </label>

                  {#if !isAlt}
                    {@const altMapped = parsedIni[section]?.['__CM_ALT_BUTTON'] !== undefined && parsedIni[section]?.['__CM_ALT_BUTTON'] !== '-1'}
                    <button 
                      class="ini-binding-btn" 
                      class:active={liveButtonStates[section + '_ALT'] === true}
                      onclick={() => openCapture(section + '_ALT', label + ' (Alt)')}
                      oncontextmenu={(e) => { e.preventDefault(); unbindAction(section + '_ALT'); }}
                      style="display: flex; align-items: center; justify-content: center; gap: 6px; font-size: 0.7rem; padding: 4px 10px; height: 26px; border-radius: 4px; margin-top: 0; width: auto; flex: 1; border-color: {altMapped ? color : 'var(--border)'}; color: {altMapped ? color : 'var(--text-secondary)'};"
                    >
                      <Plus size={14} />
                      {#if altMapped}
                        {$t('settings.controls.btn_prefix')} {parsedIni[section]['__CM_ALT_BUTTON']}
                      {:else}
                        {$t('settings.controls.alt')}
                      {/if}
                    </button>
                  {/if}
                  
                </div>
              </div>

            </div>
          {/snippet}
          
          <!-- ⭐ LIGNE DU HAUT : Matériel (Pédales, Volant, Boîte) -->
          <div style="display: grid; grid-template-columns: 1.3fr 1.6fr 1.2fr; gap: 20px; width: 100%; align-items: stretch;">
            
            <!-- Bloc Pédales -->
            <div style="display: flex; flex-direction: column; gap: 10px; padding: 15px; background: var(--bg-elevated); border-radius: var(--r); justify-content: center; height: 100%;">
              {@render pedalCard('THROTTLE', '/assets/pedals/throttle.png', 'settings.controls.actions.THROTTLE', '#6fc98a')}
              {@render pedalCard('BRAKES', '/assets/pedals/brake.png', 'settings.controls.actions.BRAKES', '#e57373')}
              {@render pedalCard('CLUTCH', '/assets/pedals/clutch.png', 'settings.controls.actions.CLUTCH', '#c58aff')}
              {@render pedalCard('HANDBRAKE', '/assets/pedals/handbrake.png', 'settings.controls.actions.HANDBRAKE', '#e6a33b')}
            </div>

            <!-- Bloc Volant & Vitesses -->
            <div style="display: flex; flex-direction: column; gap: 15px; align-items: center; height: 100%; justify-content: center; background: var(--bg-elevated); padding: 15px; border-radius: var(--r); position: relative;">
                            <!-- Volant Rotatif -->
              <div style="display: flex; flex-direction: column; align-items: center; width: 100%; position: relative;">
                
                <!-- Zone cliquable pour assigner/mapper -->
                <div 
                  style="cursor: pointer; width: 100%; display: flex; flex-direction: column; align-items: center; position: relative; padding: 10px 0;" 
                  onclick={() => openCapture('STEER', $t('settings.controls.actions.STEER'))}
                  oncontextmenu={(e) => { e.preventDefault(); unbindAction('STEER'); }}
                >
                  <!-- Image du volant qui tourne (Agrandie à 340px) -->
                  <div style="position: relative; width: 340px; height: 340px; display: flex; align-items: center; justify-content: center;">
                    <img 
                      src="/assets/volant.png" 
                      alt="Volant" 
                      style="
                        width: 100%; height: 100%; object-fit: contain;
                        transform: rotate({(demoValues.steer - 50) * 10.8}deg);
                        transition: transform 0.05s linear;
                        filter: drop-shadow(0 10px 20px rgba(0,0,0,0.5));
                      " 
                    />
                  </div>
                  
                  <!-- Valeur en degrés -->
                  <div style="text-align: center; font-size: 1.1rem; font-weight: 800; color: var(--accent); margin-top: 5px;">
                    {Math.round((demoValues.steer - 50) * 10.8)}°
                  </div>
                  

                  <!-- ⭐ NOUVELLE BARRE DE MOUVEMENT HORIZONTALE -->
                  <div class="calib-wheel-bar" style="width: 100%; max-width: 280px; margin-top: 12px;">
                    <div class="calib-wheel-center"></div>
                    <div class="calib-wheel-indicator" style="left: {demoValues.steer}%; transition: left 0.1s linear;"></div>
                  </div>
                </div>
                
                <!-- Case à cocher (déplacée sous la barre) -->
                <label style="display: flex; align-items: center; gap: 4px; font-size: 0.75rem; cursor: pointer; margin-top: 15px; color: var(--text-secondary);">
                  <input type="checkbox" checked={parsedIni['STEER']?.['MIN'] === '1'} onchange={() => toggleInvert('STEER')} /> {$t('settings.controls.invert_steering')}
                </label>

                <!-- ⭐ Valeur LOCK du volant (Curseur + Saisie) -->
                <div style="display: flex; align-items: center; gap: 12px; margin-top: 15px; width: 100%; max-width: 320px; background: var(--bg-elevated); padding: 8px 14px; border-radius: var(--r-sm); border: 1px solid var(--border-subtle);">
                  <span style="font-size: 0.75rem; color: var(--text-secondary); font-weight: 600; white-space: nowrap;">Lock :</span>
                  <input 
                    type="range" 
                    min="90" 
                    max="2520" 
                    step="10"
                    class="ini-range" 
                    style="flex: 1;"
                    value={parsedIni['STEER']?.['LOCK'] || '900'}
                    oninput={(e) => {
                      updateIniValue('STEER', 'LOCK', e.currentTarget.value);
                    }}
                  />
                  <div style="display: flex; align-items: center; gap: 2px;">
                    <input 
                      type="number" 
                      min="90" 
                      max="2520" 
                      step="10"
                      class="ini-number" 
                      style="width: 65px; height: 26px; font-size: 0.8rem; text-align: center; padding: 0 4px;"
                      value={parsedIni['STEER']?.['LOCK'] || '900'}
                      onchange={(e) => {
                        let val = parseInt(e.currentTarget.value);
                        if (isNaN(val)) val = 900;
                        val = Math.max(90, Math.min(2520, val));
                        e.currentTarget.value = val.toString();
                        updateIniValue('STEER', 'LOCK', val.toString());
                      }}
                    />
                    <span style="font-size: 0.75rem; color: var(--text-secondary); font-weight: 600;">°</span>
                  </div>
                </div>
              </div>

            </div>

                              <!-- Bloc Boîte en H -->
            <div style="display: flex; flex-direction: column; align-items: center; gap: 10px; background: var(--bg-elevated); padding: 15px; border-radius: var(--r); height: 100%; justify-content: space-between;">
              
              <!-- En-tête avec le bouton toggle -->
              <div style="display: flex; flex-direction: column; gap: 10px; width: 100%; margin-bottom: 5px;">
                <div style="display: flex; justify-content: space-between; align-items: center;">
                  <div style="font-weight: 800; color: var(--accent); font-size: 0.85rem; text-transform: uppercase; letter-spacing: 1px;">{$t('settings.controls.gearbox_title')}</div>
                </div>
                
                <!-- ⭐ NOUVEAU BOUTON TOGGLE BOÎTE EN H -->
                <label style="display: flex; align-items: center; gap: 8px; font-size: 0.75rem; cursor: pointer; color: var(--text-secondary); background: var(--bg-panel); padding: 8px 12px; border-radius: 6px; border: 1px solid var(--border-subtle); width: 100%;">
                  <div style="position: relative; width: 28px; height: 16px; background: {useHPattern ? 'var(--accent)' : 'var(--border-strong)'}; border-radius: 8px; transition: background 0.2s;">
                    <div style="position: absolute; top: 2px; left: {useHPattern ? '14px' : '2px'}; width: 12px; height: 12px; background: #fff; border-radius: 50%; transition: left 0.2s;"></div>
                  </div>
                  <input type="checkbox" bind:checked={useHPattern} style="display: none;" /> 
                  {$t('settings.controls.h_patern_enable')}
                </label>
              </div>
              
              <!-- Conteneur de la boîte en H -->
              <div style="display: flex; flex-direction: column; align-items: center; width: 100%; margin-top: 10px;">
                  
                  <div style="position: relative; width: 380px; height: 380px; background: transparent;">
                    
                    <!-- Image de fond (la base) -->
                    <img src="/assets/base_levier.png" alt="Base Boîte en H" style="position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain;" />

                    <!-- ⭐ Lumières de la boîte colorées dynamiquement selon le thème -->
                    <div style="
                      position: absolute; inset: 0; width: 100%; height: 100%;
                      background-color: var(--accent);
                      -webkit-mask-image: url('/assets/base_levier_lights.png');
                      mask-image: url('/assets/base_levier_lights.png');
                      -webkit-mask-size: contain;
                      mask-size: contain;
                      -webkit-mask-position: center;
                      mask-position: center;
                      -webkit-mask-repeat: no-repeat;
                      mask-repeat: no-repeat;
                      filter: drop-shadow(0 0 8px var(--accent));
                      pointer-events: none;
                      z-index: 1;
                    "></div>

                    <!-- ⭐ Le Pommeau (Base + Lumières dynamiques) -->
                    <div 
                      style="
                        position: absolute; width: 68px; height: 68px;
                        transform: translate(-50%, -50%);
                        transition: top 0.15s cubic-bezier(0.4, 0, 0.2, 1), left 0.15s cubic-bezier(0.4, 0, 0.2, 1);
                        filter: drop-shadow(0 5px 5px rgba(0,0,0,0.6));
                        z-index: 30; /* ⭐ Passe AU-DESSUS des gear-hitbox (qui sont à 20) */
                        
                        left: {
                          liveButtonStates['GEAR_1'] ? '32.6%' :
                          liveButtonStates['GEAR_2'] ? '32.6%' :
                          liveButtonStates['GEAR_3'] ? '44%' :
                          liveButtonStates['GEAR_4'] ? '44%' :
                          liveButtonStates['GEAR_5'] ? '55%' :
                          liveButtonStates['GEAR_6'] ? '55%' :
                          liveButtonStates['GEAR_7'] ? '66.6%' :
                          liveButtonStates['GEAR_R'] ? '66.6%' : '50%'
                        };
                        
                        top: {
                          liveButtonStates['GEAR_1'] ? '24%' :
                          liveButtonStates['GEAR_3'] ? '24%' :
                          liveButtonStates['GEAR_5'] ? '24%' :
                          liveButtonStates['GEAR_7'] ? '24%' :
                          liveButtonStates['GEAR_2'] ? '66.8%' :
                          liveButtonStates['GEAR_4'] ? '66.8%' :
                          liveButtonStates['GEAR_6'] ? '66.8%' :
                          liveButtonStates['GEAR_R'] ? '66.8%' : '50%'
                        };
                      "
                    >
                      <!-- Image de base -->
                      <img 
                        src="/assets/pommeau_base.png" 
                        alt="Pommeau" 
                        style="position: absolute; inset: 0; width: 100%; height: 100%; object-fit: contain;" 
                      />
                      <!-- Lumières colorées dynamiquement -->
                      <div style="
                        position: absolute; inset: 0; width: 100%; height: 100%;
                        background-color: var(--accent);
                        -webkit-mask-image: url('/assets/pommeau_lights.png');
                        mask-image: url('/assets/pommeau_lights.png');
                        -webkit-mask-size: contain;
                        mask-size: contain;
                        -webkit-mask-position: center;
                        mask-position: center;
                        -webkit-mask-repeat: no-repeat;
                        mask-repeat: no-repeat;
                        filter: drop-shadow(0 0 6px var(--accent));
                        pointer-events: none;
                      "></div>
                    </div>

                                        <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_1']} style="top: 24%; left: 32.6%;" onclick={() => openCapture('GEAR_1', $t('settings.controls.gear_1'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_1'); }} title={$t('settings.controls.gear_1')}></div>
                    <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_2']} style="top: 66.8%; left: 32.6%;" onclick={() => openCapture('GEAR_2', $t('settings.controls.gear_2'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_2'); }} title={$t('settings.controls.gear_2')}></div>
                    
                    <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_3']} style="top: 24%; left: 44%;" onclick={() => openCapture('GEAR_3', $t('settings.controls.gear_3'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_3'); }} title={$t('settings.controls.gear_3')}></div>
                    <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_4']} style="top: 66.8%; left: 44%;" onclick={() => openCapture('GEAR_4', $t('settings.controls.gear_4'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_4'); }} title={$t('settings.controls.gear_4')}></div>
                    
                    <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_5']} style="top: 24%; left: 55%;" onclick={() => openCapture('GEAR_5', $t('settings.controls.gear_5'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_5'); }} title={$t('settings.controls.gear_5')}></div>
                    <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_6']} style="top: 66.8%; left: 55%;" onclick={() => openCapture('GEAR_6', $t('settings.controls.gear_6'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_6'); }} title={$t('settings.controls.gear_6')}></div>
                    
                    <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_7']} style="top: 24%; left: 66.6%;" onclick={() => openCapture('GEAR_7', $t('settings.controls.gear_7'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_7'); }} title={$t('settings.controls.gear_7')}></div>
                    <div class="gear-hitbox" class:pressed={liveButtonStates['GEAR_R']} style="top: 66.8%; left: 66.6%;" onclick={() => openCapture('GEAR_R', $t('settings.controls.gear_r'))} oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEAR_R'); }} title={$t('settings.controls.gear_r')}></div>

                  </div>
                  
                  <div style="font-size: 0.7rem; color: var(--text-tertiary); margin-top: 15px; text-align: center; background: var(--bg-panel); padding: 4px 12px; border-radius: 20px; border: 1px solid var(--border);">
                    {$t('settings.controls.h_patern_hint')}
                  </div>

                </div>

                <!-- ⭐ PALLETS (Toujours visibles) -->
                <div style="display: flex; justify-content: space-between; width: 100%; max-width: 300px; margin-top: 20px;">
                  <!-- Palette Gauche (Descendre) -->
                  <button 
                    onclick={() => openCapture('GEARDN', $t('settings.controls.gear_down'))}
                    oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEARDN'); }}
                    class="paddle-btn paddle-left"
                    class:pressed={liveButtonStates['GEARDN']}
                  >
                    <span style="font-size: 0.6rem; font-weight: 800; color: var(--text-tertiary); letter-spacing: 1px; margin-bottom: 2px;">{$t('settings.controls.down')}</span>
                    <span style="font-size: 1rem; font-weight: 900; display: flex; align-items: center; gap: 6px;">
                      <ChevronDown size={18} /> 
                      {parsedIni['GEARDN']?.['BUTTON'] !== '-1' ? `${$t('settings.controls.btn_prefix')} ${parsedIni['GEARDN']?.['BUTTON']}` : '---'}
                    </span>
                  </button>

                  <!-- Palette Droite (Monter) -->
                  <button 
                    onclick={() => openCapture('GEARUP', $t('settings.controls.gear_up'))}
                    oncontextmenu={(e) => { e.preventDefault(); unbindAction('GEARUP'); }}
                    class="paddle-btn paddle-right"
                    class:pressed={liveButtonStates['GEARUP']}
                  >
                    <span style="font-size: 0.6rem; font-weight: 800; color: var(--text-tertiary); letter-spacing: 1px; margin-bottom: 2px;">{$t('settings.controls.up')}</span>
                    <span style="font-size: 1rem; font-weight: 900; display: flex; align-items: center; gap: 6px;">
                      {parsedIni['GEARUP']?.['BUTTON'] !== '-1' ? `${$t('settings.controls.btn_prefix')} ${parsedIni['GEARUP']?.['BUTTON']}` : '---'}
                      <ChevronUp size={18} /> 
                    </span>
                  </button>
                </div>
              </div>
          </div>
                    <!-- ⭐ LIGNE DU BAS : 6 Cartes Harmonieuses avec Pastille de Couleur -->
          <div style="display: grid; grid-template-columns: repeat(auto-fit, minmax(320px, 1fr)); gap: 16px; width: 100%; align-items: stretch;">
            
            {#snippet cardHeader(titleKey, dotColor)}
              <div style="position: relative; display: flex; align-items: center; justify-content: center; border-bottom: 1px solid var(--border); padding-bottom: 8px; margin-bottom: 4px; min-height: 20px;">
                <!-- Le point plus grand et plus lumineux -->
                <div style="position: absolute; left: 4px; width: 16px; height: 16px; border-radius: 50%; background: {dotColor}; box-shadow: 0 0 16px {dotColor}, 0 0 4px {dotColor};"></div>
                
                <div style="font-size: 0.8rem; color: var(--text-primary); font-weight: 800; text-transform: uppercase; letter-spacing: 1.5px; text-shadow: 0 2px 4px rgba(0,0,0,0.5);">
                  {$t(titleKey)}
                </div>
              </div>
            {/snippet}

            <!-- Carte 1 : Moteur & Conduite -->
            <div style="background: var(--bg-panel); padding: 16px; border-radius: var(--r); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 12px; border-top: 3px solid var(--accent); box-shadow: var(--shadow);">
              {@render cardHeader('settings.controls.categories.engine', '#6fc98a')}
              <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px;">
                {@render cockpitBtn('STARTER', Power, 'settings.controls.actions.STARTER')}
                {@render cockpitBtn('KERS', BatteryCharging, 'settings.controls.actions.KERS')}
                {@render cockpitBtn('DRS', Wind, 'settings.controls.actions.DRS')}
                {@render cockpitBtn('__EXT_PIT_LIMITER', Timer, 'settings.controls.actions.__EXT_PIT_LIMITER')}
                {@render cockpitBtn('ACTIVATE_AI', UserCheck, 'settings.controls.actions.ACTIVATE_AI')}
                {@render cockpitBtn('AUTO_SHIFTER', Settings, 'settings.controls.actions.AUTO_SHIFTER')}
                {@render cockpitBtn('SLOWMO', FastForward, 'settings.controls.actions.SLOWMO')}
                {@render cockpitBtn('REV', RotateCcw, 'settings.controls.actions.REV')}
              </div>
            </div>

            <!-- Carte 2 : Aides & Réglages -->
            <div style="background: var(--bg-panel); padding: 16px; border-radius: var(--r); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 12px; border-top: 3px solid var(--accent); box-shadow: var(--shadow);">
              {@render cardHeader('settings.controls.categories.assists', '#4c9aff')}
              <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px;">
                {@render cockpitBtn('ABS', ShieldAlert, 'settings.controls.actions.ABS')}
                {@render cockpitBtn('TRACTION_CONTROL', ShieldCheck, 'settings.controls.actions.TRACTION_CONTROL')}
                {@render cockpitBtn('__CM_ENGINE_BRAKE', Disc, 'settings.controls.actions.__CM_ENGINE_BRAKE')}
                {@render cockpitBtn('BALANCEUP', ChevronsUp, 'settings.controls.actions.BALANCEUP')}
                {@render cockpitBtn('BALANCEDN', ChevronsDown, 'settings.controls.actions.BALANCEDN')}
              </div>
              <!-- ABS / TC / FFB intégrés directement ici -->
              <div style="margin-top: auto; background: var(--bg-elevated); padding: 12px; border-radius: var(--r-sm); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 8px;">
                <div style="display: flex; align-items: center; justify-content: center; gap: 15px;">
                  {@render compactBtn('ABSDN', false)}
                  <span style="font-size: 0.75rem; font-weight: 800; color: var(--text-secondary); width: 40px; text-align: center;">ABS</span>
                  {@render compactBtn('ABSUP', true)}
                </div>
                <div style="display: flex; align-items: center; justify-content: center; gap: 15px;">
                  {@render compactBtn('TCDN', false)}
                  <span style="font-size: 0.75rem; font-weight: 800; color: var(--text-secondary); width: 40px; text-align: center;">TC</span>
                  {@render compactBtn('TCUP', true)}
                </div>
                <div style="display: flex; align-items: center; justify-content: center; gap: 15px;">
                  {@render compactBtn('__EXT_FFB_DECREASE', false)}
                  <span style="font-size: 0.75rem; font-weight: 800; color: var(--text-secondary); width: 40px; text-align: center;">FFB</span>
                  {@render compactBtn('__EXT_FFB_INCREASE', true)}
                </div>
              </div>
            </div>

            <!-- Carte 3 : Session & Course -->
            <div style="background: var(--bg-panel); padding: 16px; border-radius: var(--r); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 12px; border-top: 3px solid var(--accent); box-shadow: var(--shadow);">
              {@render cardHeader('settings.controls.categories.session', '#c58aff')}
              <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px;">
                {@render cockpitBtn('__CM_PAUSE', PauseCircle, 'settings.controls.actions.__CM_PAUSE')}
                {@render cockpitBtn('RESET_RACE', RefreshCcw, 'settings.controls.actions.RESET_RACE')}
                {@render cockpitBtn('__CM_TO_PITS', Flag, 'settings.controls.actions.__CM_TO_PITS')}
                {@render cockpitBtn('__CM_START_SESSION', PlayCircle, 'settings.controls.actions.__CM_START_SESSION')}
                {@render cockpitBtn('__CM_SETUP_CAR', Wrench, 'settings.controls.actions.__CM_SETUP_CAR')}
                {@render cockpitBtn('__CM_START_STOP_SESSION', Square, 'settings.controls.actions.__CM_START_STOP_SESSION')}
              </div>
            </div>

            <!-- Carte 4 : Éclairage & Signalisation -->
            <div style="background: var(--bg-panel); padding: 16px; border-radius: var(--r); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 12px; border-top: 3px solid var(--accent); box-shadow: var(--shadow);">
              {@render cardHeader('settings.controls.categories.lights', '#e6a33b')}
              <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px;">
                {@render cockpitBtn('ACTION_HEADLIGHTS', Lightbulb, 'settings.controls.actions.ACTION_HEADLIGHTS')}
                {@render cockpitBtn('ACTION_HEADLIGHTS_HIGH_LOW', Sun, 'settings.controls.actions.ACTION_HEADLIGHTS_HIGH_LOW')}
                {@render cockpitBtn('ACTION_HEADLIGHTS_FLASH', Zap, 'settings.controls.actions.ACTION_HEADLIGHTS_FLASH')}
                {@render cockpitBtn('__EXT_TURNLIGHTS_LEFT', ArrowLeftCircle, 'settings.controls.actions.__EXT_TURNSIGNAL_LEFT')}
                {@render cockpitBtn('__EXT_TURNLIGHTS_RIGHT', ArrowRightCircle, 'settings.controls.actions.__EXT_TURNSIGNAL_RIGHT')}
                {@render cockpitBtn('__EXT_HAZARDS', TriangleAlert, 'settings.controls.actions.__EXT_HAZARDS')}
                {@render cockpitBtn('ACTION_HORN', Megaphone, 'settings.controls.actions.ACTION_HORN')}
              </div>
            </div>

            <!-- Carte 5 : Vue & Interface -->
            <div style="background: var(--bg-panel); padding: 16px; border-radius: var(--r); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 12px; border-top: 3px solid var(--accent); box-shadow: var(--shadow);">
              {@render cardHeader('settings.controls.categories.view', '#e57373')}
              <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px;">
                {@render cockpitBtn('ACTION_CHANGE_CAMERA', Camera, 'settings.controls.actions.ACTION_CHANGE_CAMERA')}
                {@render cockpitBtn('GLANCELEFT', ArrowLeft, 'settings.controls.actions.GLANCELEFT')}
                {@render cockpitBtn('GLANCERIGHT', ArrowRight, 'settings.controls.actions.GLANCERIGHT')}
                {@render cockpitBtn('GLANCEBACK', ArrowDown, 'settings.controls.actions.GLANCEBACK')}
                {@render cockpitBtn('__EXT_VIRTUAL_MIRROR', ScanEye, 'settings.controls.actions.__EXT_VIRTUAL_MIRROR')}
                {@render cockpitBtn('HIDE_APPS', LayoutGrid, 'settings.controls.actions.HIDE_APPS')}
                {@render cockpitBtn('HIDE_DAMAGE', EyeOff, 'settings.controls.actions.HIDE_DAMAGE')}
                {@render cockpitBtn('SHOW_DAMAGE', Eye, 'settings.controls.actions.SHOW_DAMAGE')}
                {@render cockpitBtn('DRIVER_NAMES', Users, 'settings.controls.actions.DRIVER_NAMES')}
                {@render cockpitBtn('IDEAL_LINE', Route, 'settings.controls.actions.IDEAL_LINE')}
              </div>
              <div style="margin-top: auto; display: flex; justify-content: center; gap: 10px;">
                {@render cockpitBtn('__EXT_WIPERS_MORE', CloudRain, 'settings.controls.actions.__EXT_WIPERS_MORE')}
                {@render cockpitBtn('__EXT_WIPERS_LESS', CloudDrizzle, 'settings.controls.actions.__EXT_WIPERS_LESS')}
                {@render cockpitBtn('__EXT_WIPERS_OFF', Sun, 'settings.controls.actions.__EXT_WIPERS_OFF')}
              </div>
            </div>

            <!-- Carte 6 : Replay & Avancé -->
            <div style="background: var(--bg-panel); padding: 16px; border-radius: var(--r); border: 1px solid var(--border); display: flex; flex-direction: column; gap: 12px; border-top: 3px solid var(--accent); box-shadow: var(--shadow);">
              {@render cardHeader('settings.controls.categories.replay', '#98a0b5')}
              <div style="display: flex; flex-wrap: wrap; justify-content: center; gap: 10px;">
                {@render cockpitBtn('START_REPLAY', PlayCircle, 'settings.controls.actions.START_REPLAY')}
                {@render cockpitBtn('PAUSE_REPLAY', PauseCircle, 'settings.controls.actions.PAUSE_REPLAY')}
                {@render cockpitBtn('NEXT_LAP', SkipForward, 'settings.controls.actions.NEXT_LAP')}
                {@render cockpitBtn('PREVIOUS_LAP', SkipBack, 'settings.controls.actions.PREVIOUS_LAP')}
                {@render cockpitBtn('NEXT_CAR', ChevronRight, 'settings.controls.actions.NEXT_CAR')}
                {@render cockpitBtn('PLAYER_CAR', User, 'settings.controls.actions.PLAYER_CAR')}
                {@render cockpitBtn('PREVIOUS_CAR', ChevronLeft, 'settings.controls.actions.PREVIOUS_CAR')}
              </div>
            </div>

          </div>
          <!-- ⭐ FIN DE LA LIGNE DU BAS -->

        </div>
      </div>

    {:else if currentMode === 'advanced'}
      <div class="qacrl-ctrl-section" style="margin-top: 15px; border: 1px solid #e6a33b; border-radius: var(--r-sm); overflow: hidden;">
        <div 
          class="qacrl-ctrl-section-header" 
          style="padding: 12px 16px; background: var(--bg-elevated); border-bottom: 1px solid {openDinputDevices ? '#e6a33b' : 'transparent'}; cursor: pointer; display: flex; align-items: center; gap: 10px; transition: background 0.2s;"
          onclick={() => { openDinputDevices = !openDinputDevices; saveMenuState(); }}
        >
          <span class="qacrl-ctrl-section-arrow" style="transform: rotate({openDinputDevices ? '90deg' : '0deg'}); transition: transform 0.2s; color: #e6a33b;">▸</span>
          <span class="qacrl-ctrl-section-title" style="font-size: 0.85rem; color: #e6a33b;">{$t('settings.controls.advanced_dinput_title')}</span>
        </div>
        
        {#if openDinputDevices}
          <div style="padding: 12px 16px; display: flex; flex-direction: column; gap: 10px;">
            <p style="font-size: 0.75rem; color: var(--text-secondary); margin-bottom: 10px;">
              {$t('settings.controls.advanced_dinput_desc')}
            </p>
            {#each dinputDevices as dev}
              <div style="font-size: 0.8rem; background: var(--bg-panel); padding: 10px; border-radius: 6px; border: 1px solid var(--border-subtle);">
                <strong style="color: var(--accent);">{dev.name}</strong><br>
                <span style="color: var(--text-tertiary);">PGUID: <strong>{dev.pguid}</strong></span><br>
                <span style="color: var(--text-tertiary); font-size: 0.7rem;">IGUID: {dev.iguid}</span>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="qacrl-ctrl-section" style="margin-top: 15px; border: 1px solid #e6a33b; border-radius: var(--r-sm); overflow: hidden;">
        <div 
          class="qacrl-ctrl-section-header" 
          style="padding: 12px 16px; background: var(--bg-elevated); border-bottom: 1px solid {openWebviewDevices ? '#e6a33b' : 'transparent'}; cursor: pointer; display: flex; align-items: center; gap: 10px; transition: background 0.2s;"
          onclick={() => { openWebviewDevices = !openWebviewDevices; saveMenuState(); }}
        >
          <span class="qacrl-ctrl-section-arrow" style="transform: rotate({openWebviewDevices ? '90deg' : '0deg'}); transition: transform 0.2s; color: #e6a33b;">▸</span>
          <span class="qacrl-ctrl-section-title" style="font-size: 0.85rem; color: #e6a33b;">{$t('settings.controls.advanced_webview_title')}</span>
        </div>
        
        {#if openWebviewDevices}
          <div style="padding: 12px 16px; display: flex; flex-direction: column; gap: 10px;">
            <p style="font-size: 0.75rem; color: var(--text-secondary); margin-bottom: 10px;">
              {$t('settings.controls.advanced_webview_desc')}
            </p>
            {#each liveGamepadData as item}
              {@const gp = item.gp}
              {@const acIdx = getAcJoyIndex(item.originalIndex, gp)}
              <div style="font-size: 0.8rem; background: var(--bg-panel); padding: 10px; border-radius: 6px; border: 1px solid var(--border-subtle);">
                <strong style="color: var(--accent);">{gp.id}</strong><br>
                <span style="color: var(--text-tertiary);">
                  {#if acIdx === -1}
                    {$t('settings.controls.webview_mapping', { values: { webview: item.originalIndex, ac: '' } })}
                    <strong style="color: #e6a33b;">{$t('settings.controls.webview_unmapped')}</strong>
                  {:else}
                    {$t('settings.controls.webview_mapping', { values: { webview: item.originalIndex, ac: acIdx } })}
                  {/if}
                </span>
                <div style="display: flex; flex-wrap: wrap; gap: 5px; margin-top: 8px;">
                  {#each gp.axes as ax, aIdx}
                    <span style="background: var(--bg-elevated); padding: 2px 6px; border-radius: 4px; font-family: monospace; border: 1px solid var(--border); color: {Math.abs(ax) > 0.1 && acIdx !== -1 ? 'var(--accent)' : 'inherit'};">
                      Axe {aIdx}: {ax.toFixed(2)}
                    </span>
                  {/each}
                  {#each gp.buttons as btn, bIdx}
                    {#if btn.pressed}
                      <span style="background: #e57373; color: white; padding: 2px 6px; border-radius: 4px; font-family: monospace;">
                        BTN {bIdx}
                      </span>
                    {/if}
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>

      <div class="group" style="margin-top: 15px;">
        <div class="group-title">{$t('settings.controls.advanced_config_title')}</div>
        {#if iniContent === ''}
          <p style="color: #e6a33b; text-align: center; padding: 20px;">{$t('settings.controls.file_empty')}</p>
        {:else}
          <div style="display: flex; flex-direction: column; gap: 12px; margin-top: 15px;">
            {#each groupedActions as group}
              {@const isOpen = openCategories[group.name] === true} <!-- Fermé par défaut -->
              <div class="qacrl-ctrl-section" style="border: 1px solid var(--border); border-radius: var(--r-sm); overflow: hidden;">
                
                <!-- Header cliquable -->
                <div 
                  class="qacrl-ctrl-section-header" 
                  style="padding: 12px 16px; background: var(--bg-elevated); border-bottom: 1px solid {isOpen ? 'var(--border)' : 'transparent'}; cursor: pointer; display: flex; align-items: center; gap: 10px; transition: background 0.2s;"
                  onclick={() => { openCategories[group.name] = !isOpen; saveMenuState(); }}
                >
                  <span class="qacrl-ctrl-section-arrow" style="transform: rotate({isOpen ? '90deg' : '0deg'}); transition: transform 0.2s;">▸</span>
                  <span class="qacrl-ctrl-section-title" style="font-size: 0.85rem;">{$t(group.name)}</span>
                  <span style="font-size: 0.7rem; color: var(--text-tertiary); margin-left: auto; background: var(--bg-panel); padding: 2px 8px; border-radius: 10px;">{group.items.length}</span>
                </div>
                
                <!-- Corps -->
                {#if isOpen}
                  <div class="qacrl-ctrl-section-body" style="padding: 12px 16px; display: flex; flex-direction: column; gap: 10px;">
                    {#each group.items as action}
                      <div style="display: grid; grid-template-columns: 200px 1fr 1fr; gap: 15px; align-items: start; padding: 10px; background: var(--bg-panel); border-radius: 6px; border: 1px solid var(--border-subtle);">
                          
                          <div style="font-weight: 600; color: var(--text-primary); font-size: 0.85rem; margin-top: 8px;">{$t(action.label)}</div>
                          
                          <div style="display: flex; flex-direction: column; gap: 8px;">
                            {#if action.type === 'mixed' || action.type === 'axis'}
                              <select class="ini-select" style="width: 100%;" value={parsedIni[action.id]?.['AXLE'] || '-1'} onchange={(e) => updateIniValue(action.id, 'AXLE', e.currentTarget.value)}>
                                {#if action.type === 'mixed'}
                                  <option value="-1">{$t('settings.controls.select_no_axis')}</option>
                                {/if}
                                <option value="0">Axe X (0)</option>
                                <option value="1">Axe Y (1)</option>
                                <option value="2">Axe Z (2)</option>
                                <option value="3">Axe RX (3)</option>
                                <option value="4">Axe RY (4)</option>
                                <option value="5">Axe RZ (5)</option>
                                <option value="6">Axe Slider 1 (6)</option>
                                <option value="7">Axe Slider 2 (7)</option>
                                <option value="8">Axe Slider 3 (8)</option>
                                <option value="9">Axe Slider 4 (9)</option>
                              </select>
                              {#if parsedIni[action.id]?.['AXLE'] !== '-1' && parsedIni[action.id]?.['AXLE'] !== undefined}
                                <div style="display: flex; gap: 10px; align-items: center; background: var(--bg-elevated); padding: 4px 8px; border-radius: 4px;">
                                  <span style="font-family: monospace; font-size: 0.75rem; color: var(--accent);">{liveAxisValues[action.id]?.toFixed(2) || '0.00'}</span>
                                  <label style="font-size: 0.7rem; display: flex; align-items: center; gap: 4px; cursor: pointer; color: var(--text-secondary);">
                                    <input type="checkbox" checked={parsedIni[action.id]?.['MIN'] === '1'} onchange={() => toggleInvert(action.id)} /> {$t('settings.controls.invert')}
                                  </label>
                                </div>
                              {/if}
                            {/if}
                            
                            <button class="ini-binding-btn" class:active={liveButtonStates[action.id]} style="width: 100%; display: flex; justify-content: space-between;" onclick={() => openCapture(action.id, $t(action.label))}>
                              <span>
                                {#if parsedIni[action.id]?.['BUTTON'] !== '-1' && parsedIni[action.id]?.['BUTTON']}
                                  {$t('settings.controls.btn_prefix')} {parsedIni[action.id]?.['BUTTON']}
                                {:else if parsedIni[action.id]?.['KEY'] !== '-1' && parsedIni[action.id]?.['KEY']}
                                  {$t('settings.controls.key_prefix')} {parsedIni[action.id]?.['KEY']}
                                {:else}
                                  {$t('settings.controls.btn_unassigned')}
                                {/if}
                              </span>
                            </button>
                          </div>

                          <div style="display: flex; flex-direction: column; gap: 8px;">
                            <button class="ini-binding-btn" class:active={liveButtonStates[action.id + '_ALT']} style="width: 100%; display: flex; justify-content: space-between; border-style: dashed;" onclick={() => openCapture(action.id + '_ALT', $t(action.label) + ' (' + $t('settings.controls.alt') + ')')}>
                              <span>
                                {#if parsedIni[action.id]?.['__CM_ALT_BUTTON'] !== '-1' && parsedIni[action.id]?.['__CM_ALT_BUTTON']}
                                  {$t('settings.controls.alt_btn_prefix')} {parsedIni[action.id]?.['__CM_ALT_BUTTON']}
                                {:else if parsedIni[action.id]?.['JOY_MODIFICATOR'] !== undefined && parsedIni[action.id]?.['JOY_MODIFICATOR'] !== '-1'}
                                  {$t('settings.controls.alt_btn_prefix')} {parsedIni[action.id]?.['JOY_MODIFICATOR']}
                                {:else}
                                  {$t('settings.controls.btn_alt_unassigned')}
                                {/if}
                              </span>
                            </button>
                          </div>

                        </div>
                    {/each}
                  </div>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>

    {:else if currentMode === 'expert'}
      <div class="group" style="margin-top: 15px;">
        <div class="group-title">{$t('settings.controls.expert_edit')}</div>
        <textarea class="ini-expert" bind:value={iniContent} oninput={() => hasChanges = true} spellcheck="false"></textarea>
      </div>
    {/if}
  {/if}

  {#if isCapturing}
    <div class="qacrl-dialog-overlay visible">
      <div class="qacrl-dialog">
        <div class="qacrl-dialog-header">
          <span class="qacrl-dialog-icon">🎮</span>
          <span class="qacrl-dialog-title">{$t('settings.controls.capture_title', { values: { name: capturingActionName } })}</span>
        </div>
        <div class="qacrl-dialog-body" style="text-align: center; padding: 20px;">
          <p style="font-size: 1.1rem; font-weight: 600; color: var(--accent);">{$t('settings.controls.capture_hint')}</p>
          <div style="text-align: left; background: var(--bg-panel); padding: 12px; border-radius: 8px; border: 1px solid var(--border); max-height: 180px; overflow-y: auto; margin-top: 15px;">
             {#each liveGamepadData as item}
               {@const gp = item.gp}
               <div style="font-size: 0.75rem; margin-bottom: 10px;">
                 <strong style="color: var(--accent);">{gp.id.substring(0, 20)}...</strong>
                 <div style="display: flex; gap: 4px; flex-wrap: wrap; margin-top: 4px;">
                   {#each gp.buttons as btn, bIdx}
                     {#if btn.pressed}<span style="background: #e57373; color: white; padding: 2px 6px; border-radius: 4px;">BTN {bIdx}</span>{/if}
                   {/each}
                   {#each gp.axes as ax, aIdx}
                     {#if Math.abs(ax) > 0.1}<span style="background: var(--bg-elevated); padding: 2px 6px; border-radius: 4px;">AXE {aIdx} : {ax.toFixed(2)}</span>{/if}
                   {/each}
                 </div>
               </div>
             {/each}
          </div>
        </div>
        <div class="qacrl-dialog-footer">
          <button class="btn-secondary" onclick={closeCapture}>{$t('common.cancel')}</button>
        </div>
      </div>
    </div>
  {/if}
</div>