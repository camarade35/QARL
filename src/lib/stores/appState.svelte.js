const savedPath = typeof localStorage !== 'undefined' ? localStorage.getItem('acPath') || '' : '';
const savedSteamId = typeof localStorage !== 'undefined' ? localStorage.getItem('qarl_steam_id') || '' : '';
const savedDriverName = typeof localStorage !== 'undefined' ? localStorage.getItem('qarl_driver_name') || 'Driver' : 'Driver';
const savedOnlineName = typeof localStorage !== 'undefined' ? localStorage.getItem('qarl_online_name') || '' : '';
const savedDocumentsPath = typeof localStorage !== 'undefined' ? localStorage.getItem('qarl_documents_path') || '' : '';
const savedFavorites = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('qarl_favorites') || '{"cars":[],"tracks":[]}') : {cars:[],tracks:[]};
const savedUsed = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('qarl_used') || '{"cars":{},"tracks":{}}') : {cars:{},tracks:{}};

// ⭐ Serveurs
const savedCustomServers = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('qarl_custom_servers') || '[]') : [];
const savedRecentServers = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('qarl_recent_servers') || '[]') : [];
const savedFavoriteServers = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('qarl_favorite_servers') || '[]') : [];
const savedServerProfiles = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('qarl_server_profiles') || '{}') : {};
const savedServerPasswords = typeof localStorage !== 'undefined' ? JSON.parse(localStorage.getItem('qarl_server_passwords') || '{}') : {};

// ⭐ Configuration par défaut de la session
const defaultSession = {
  type: 'RACE', // RACE, PRACTICE, HOTLAP
  laps: 5,
  sessionMinutes: 30,
  practiceMinutes: 15,
  qualifyMinutes: 10,
  
  // IA & Grille
  opponentsCount: 10,
  gridType: 'RANDOM',
  startingPositionMode: 'LAST',
  customStartPos: 1,
  jumpStartPenalty: 0,
  aiLevel: 90,
  aiLevelVar: 5,
  aiAggression: 50,
  aiAggressionVar: 10,

  // Pilote & Réalisme
  assists: {
    autoGearbox: false,
    autoClutch: true,
    autoblip: false,
    idealLine: false,
    tc: 1, // 0=Off, 1=Factory, 2=On
    abs: 1,
    stability: 0,
    fuelRate: 1,
    mechanicalDamage: 100,
    tyreWear: 1,
    slipstream: 1,
    tyreBlankets: false
  },

  // ⭐ Météo & Pure Ctrl (migré depuis QACRL)
  weather: {
    // Mode principal
    mode: 'static',              // 'static' | 'schedule' | 'existing_plan' | 'custom_plan'
    
    // Météo statique (ID de preset AC, ex: '3_clear', '7_heavy_clouds')
    staticWeather: '3_clear',

    // ⭐ Schedule (Contrôleur CSP Natif)
    scheduleSlots: 3,
    scheduleTimePerSlot: 5,
    scheduleRandom: false,
    scheduleSlotValues: [15, 18, 19, 7, 20], // [Clear, Broken, Overcast, Rain, Fog]
    
    // Plan existant (Pure / Sol)
    existingPlan: '',            // nom du fichier .plan
    existingPlanRandom: false,   // tirer un plan au hasard à chaque lancement
    existingPlanSteps: [],       // ⭐ Stocke les étapes du plan chargé pour l'affichage
    existingPlanLoop: false,     // ⭐ NOUVEAU
    existingPlanTimeMult: 1,     // ⭐ NOUVEAU
    
    // Plan personnalisé
    customPlanName: 'Mon plan',
    customPlanSteps: [],         // [{ weather: '3_clear', duration: 10 }, ...]
    customPlanLoop: false,
    
    // Contrôle du temps
    startHour: 14,
    startMinute: 0,
    timeMultiplier: 1,
    
    // ⭐ Pure Ctrl — contrôle dynamique de la météo par Pure
    pureCtrl: {
      autostart: false,          // démarre le plan automatiquement (forcé si mode = plan)
      lastUsed: false,           // réutilise le dernier plan joué
      live: false,               // météo live basée sur la vraie météo du circuit
      startWetness: 1,           // 1-10 : humidité de départ de la piste
      startPuddles: 1            // 1-10 : flaques au départ
    }
  }
};

// ⭐ Merge avec la session sauvegardée (robuste aux anciennes sessions sans weather)
// ⭐ NOUVEAU : Chargement des IDs de sélection sauvegardés
const savedSelectionIds = typeof localStorage !== 'undefined'
  ? JSON.parse(localStorage.getItem('qarl_selections') || '{"carId":null,"skinId":null,"trackId":null,"layoutId":null}')
  : { carId: null, skinId: null, trackId: null, layoutId: null };

const savedSessionRaw = typeof localStorage !== 'undefined'
  ? JSON.parse(localStorage.getItem('qarl_session') || 'null')
  : null;

const savedSession = savedSessionRaw
  ? {
      ...defaultSession,
      ...savedSessionRaw,
      assists: { ...defaultSession.assists, ...(savedSessionRaw.assists || {}) },
      weather: {
        ...defaultSession.weather,
        ...(savedSessionRaw.weather || {}),
        pureCtrl: {
          ...defaultSession.weather.pureCtrl,
          ...((savedSessionRaw.weather || {}).pureCtrl || {})
        }
      }
    }
  : null;

export const appState = $state({
  acPath: savedPath,
  steamId: savedSteamId,
  driverName: savedDriverName,
  onlineName: savedOnlineName,
  documentsPath: savedDocumentsPath,
  activeTab: 'tab-cars',
  cars: [], // ⭐ Stocke toutes les voitures pour la génération de la grille
  tracks: [], // ⭐ AJOUT : On initialise le tableau des circuits
  selectedCar: null,
  selectedCarSkin: null,
  selectedTrack: null,
  selectedTrackLayout: null,
  modalOpen: false,
  modalType: null,
  modalData: null,
  favorites: savedFavorites,
  used: savedUsed,
  carFilters: { type: 'all', search: '', tags: [] },
  trackFilters: { type: 'all', search: '', tags: [] },
  
  // ⭐ Serveurs
  customServers: savedCustomServers,
  recentServers: savedRecentServers,
  favoriteServers: savedFavoriteServers,
  serverProfiles: savedServerProfiles,
  serverPasswords: savedServerPasswords,

  // ⭐ Session
  session: savedSession || defaultSession
});

// ⭐ 1. Restauration VOITURE (dès que la liste des voitures est prête)
$effect.root(() => {
  $effect(() => {
    // Si on a des voitures, qu'aucune voiture n'est sélectionnée, et qu'on a un ID sauvegardé
    if (appState.cars.length > 0 && !appState.selectedCar && savedSelectionIds.carId) {
      const car = appState.cars.find(c => c.id === savedSelectionIds.carId);
      if (car) {
        appState.selectedCar = car;
        appState.selectedCarSkin = car.skins.find(s => s.name === savedSelectionIds.skinId) || car.skins[0] || null;
      }
    }
  });
});

// ⭐ 2. Restauration CIRCUIT (dès que la liste des circuits est prête)
$effect.root(() => {
  $effect(() => {
    if (appState.tracks.length > 0 && !appState.selectedTrack && savedSelectionIds.trackId) {
      const track = appState.tracks.find(t => t.id === savedSelectionIds.trackId);
      if (track) {
        appState.selectedTrack = track;
        appState.selectedTrackLayout = track.layouts.find(l => l.id === savedSelectionIds.layoutId) || track.layouts[0] || null;
      }
    }
  });
});

// ⭐ 3. Sauvegarde VOITURE (dès que tu cliques sur une voiture)
$effect.root(() => {
  $effect(() => {
    const carId = appState.selectedCar?.id;
    const skinId = appState.selectedCarSkin?.name;
    
    // On ne sauvegarde QUE si une voiture est sélectionnée (évite d'écraser tes données au démarrage)
    if (carId && appState.cars.length > 0) {
      const current = JSON.parse(localStorage.getItem('qarl_selections') || '{"carId":null,"skinId":null,"trackId":null,"layoutId":null}');
      current.carId = carId;
      current.skinId = skinId || null;
      localStorage.setItem('qarl_selections', JSON.stringify(current));
    }
  });
});

// ⭐ 4. Sauvegarde CIRCUIT (dès que tu cliques sur un circuit)
$effect.root(() => {
  $effect(() => {
    const trackId = appState.selectedTrack?.id;
    const layoutId = appState.selectedTrackLayout?.id;
    
    // On ne sauvegarde QUE si un circuit est sélectionné
    if (trackId && appState.tracks.length > 0) {
      const current = JSON.parse(localStorage.getItem('qarl_selections') || '{"carId":null,"skinId":null,"trackId":null,"layoutId":null}');
      current.trackId = trackId;
      current.layoutId = layoutId || null;
      localStorage.setItem('qarl_selections', JSON.stringify(current));
    }
  });
});

export function setAcPath(newPath) {
  appState.acPath = newPath;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('acPath', newPath);
  }
}

export function setSteamId(id) {
  appState.steamId = id;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_steam_id', id);
  }
}

export function setDriverName(name) {
  appState.driverName = name;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_driver_name', name);
  }
}

export function setOnlineName(name) {
  appState.onlineName = name;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_online_name', name);
  }
}

export function setDocumentsPath(path) {
  appState.documentsPath = path;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_documents_path', path);
  }
}

export function toggleFavorite(kind, id) {
  const list = appState.favorites[kind];
  const idx = list.indexOf(id);
  if (idx >= 0) list.splice(idx, 1);
  else list.push(id);
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_favorites', JSON.stringify(appState.favorites));
  }
}

export function bumpUsedCount(kind, id) {
  appState.used[kind][id] = (appState.used[kind][id] || 0) + 1;
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_used', JSON.stringify(appState.used));
  }
}

// ⭐ Sauvegarde automatique de la session
export function saveSession() {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_session', JSON.stringify(appState.session));
  }
}

// ⭐ Sauvegarde de l'état des serveurs
export function saveServerState() {
  if (typeof localStorage !== 'undefined') {
    localStorage.setItem('qarl_custom_servers', JSON.stringify(appState.customServers));
    localStorage.setItem('qarl_recent_servers', JSON.stringify(appState.recentServers));
    localStorage.setItem('qarl_favorite_servers', JSON.stringify(appState.favoriteServers));
    localStorage.setItem('qarl_server_profiles', JSON.stringify(appState.serverProfiles));
    localStorage.setItem('qarl_server_passwords', JSON.stringify(appState.serverPasswords));
  }
}

// ⭐ Application immédiate de la couleur sauvegardée au démarrage
if (typeof document !== 'undefined') {
  const savedAccent = localStorage.getItem('qarl_accent') || '#319dd3';
  const savedTheme = localStorage.getItem('qarl_theme') || 'dark';
  document.documentElement.setAttribute('data-theme', savedTheme);
  
  // Un mini calcul pour appliquer la couleur sans attendre Svelte
  const r = parseInt(savedAccent.slice(1, 3), 16);
  const g = parseInt(savedAccent.slice(3, 5), 16);
  const b = parseInt(savedAccent.slice(5, 7), 16);
  
  const finalR = savedTheme === 'light' ? Math.floor(r * 0.75) : Math.min(255, Math.floor(r * 1.15));
  const finalG = savedTheme === 'light' ? Math.floor(g * 0.75) : Math.min(255, Math.floor(g * 1.15));
  const finalB = savedTheme === 'light' ? Math.floor(b * 0.75) : Math.min(255, Math.floor(b * 1.15));
  
  const adjustedHex = `#${finalR.toString(16).padStart(2, '0')}${finalG.toString(16).padStart(2, '0')}${finalB.toString(16).padStart(2, '0')}`;
  
  document.documentElement.style.setProperty('--accent', adjustedHex);
  document.documentElement.style.setProperty('--accent-rgb', `${finalR}, ${finalG}, ${finalB}`);
  document.documentElement.style.setProperty('--accent-muted', `rgba(${finalR}, ${finalG}, ${finalB}, 0.12)`);
  document.documentElement.style.setProperty('--accent-glow', `rgba(${finalR}, ${finalG}, ${finalB}, 0.4)`);
}