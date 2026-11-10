<script lang="ts">
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { appState, saveServerState } from '$lib/stores/appState.svelte.js';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { t } from 'svelte-i18n';
  import { getFlagUrl } from '$lib/utils/assets.js'; // ⭐ L'import manquant !
   import { alert } from '$lib/stores/dialog.svelte'; // ⭐ AJOUT IMPORT QARL

  let servers = $state<any[]>([]);
  let manualFilter = $state({ favoritesOnly: false });
  let recentFilter = $state({ favoritesOnly: false });
  let isLoading = $state(false);
  let errorMsg = $state('');
  let pingStatus = $state<Record<string, any>>({});
  let serverInfos = $state<Record<string, any>>({}); // ⭐ Stocke les détails de chaque serveur

  // ⭐ Nouveaux états pour la pagination et les filtres publics
  let currentPage = $state(1);
  let hasNextPage = $state(true);
  let isLoadingMore = $state(false);
  let searchQuery = $state('');
  let filterCountry = $state('');
  let filterTrack = $state('');
  let filterMinPlayers = $state(0);
  let filterOnlyPlayable = $state(false); // ⭐ Nouveau filtre : n'afficher que les serveurs jouables

  // ⭐ Liste filtrée (Svelte 5 utilise $derived pour ça)
  let filteredServers = $derived(
    servers.filter(server => {
      // ⭐ 1. Masquer les serveurs hors ligne (ping échoué)
      const key = server.ip + ':' + server.port;
      const ping = pingStatus[key];
      if (ping && ping.status === 'fail') {
        return false;
      }

      // 2. Filtre texte (nom ou IP)
      if (searchQuery) {
        const q = searchQuery.toLowerCase();
        if (!server.name.toLowerCase().includes(q) && !`${server.ip}:${server.port}`.includes(q)) {
          return false;
        }
      }
      
      // 3. Filtre pays
      if (filterCountry && server.country !== filterCountry) return false;
      
      // 4. Filtre circuit
      if (filterTrack && cleanTrackName(server.track) !== filterTrack) return false;
      
      // 5. Filtre joueurs min
      if (filterMinPlayers > 0 && server.players < filterMinPlayers) return false;

      // 6. ⭐ Filtre "Jouable" (on a le circuit + les voitures)
      if (filterOnlyPlayable) {
        const info = serverInfos[key];
        if (info) {
          const trackData = getTrackDisplay(info.track);
          const carsData = getCarsDisplay(info.cars);
          if (!trackData.exists || !carsData.allExist) {
            return false;
          }
        }
      }

      return true;
    })
  );

  // ⭐ Listes uniques pour les menus déroulants de filtres
  let availableCountries = $derived(
    Array.from(new Set(servers.map(s => s.country).filter(Boolean))).sort()
  );
  let availableTracks = $derived(
    Array.from(new Set(servers.map(s => cleanTrackName(s.track)).filter(Boolean))).sort()
  );

  // États pour la partie haute
  let pastedLink = $state('');
  let manualIp = $state('');
  let manualName = $state('');

  // États pour la modale
  let selectedServer = $state<any>(null);
  let serverDetails = $state<any>(null);
  let loadingDetails = $state(false);
  
  let commonCars = $state<any[]>([]);
  let missingCars = $state<string[]>([]);
  let missingTracks = $state<string[]>([]);
  let serverFeatures = $state<string[]>([]);
  let serverDescription = $state('');
  let passwordRequired = $state(false);
  let serverPassword = $state('');
  
  let selectedCar = $state<any>(null);
  let availableSkins = $state<any[]>([]);
  let selectedSkin = $state<any>(null);
  let isJoining = $state(false);

  // ⭐ États pour la modale d'édition de serveur
  let editingServer = $state<any>(null);
  let editName = $state('');
  let editIp = $state('');

  // ⭐ État pour l'association du protocole
  let isAssociated = $state(false);

  // ⭐ Vérifier l'état au chargement
  $effect(() => {
    async function checkStatus() {
      try {
        isAssociated = await invoke<boolean>('get_protocol_status');
      } catch (e) {
        console.error("Erreur de vérification du protocole:", e);
      }
    }
    checkStatus();
  });

  async function toggleAssociation() {
    try {
      if (isAssociated) {
        await invoke('dissociate_protocol');
        isAssociated = false;
      } else {
        await invoke('associate_protocol');
        isAssociated = true;
        alert($t('online.protocol_assoc_success'));
      }
    } catch (e) {
      alert(String(e));
    }
  }

  // ⭐ Fonction utilitaire pour nettoyer les chemins bizarres (ex: "csp/.../D:/../ks_nordschleife")
  function cleanTrackName(raw: string) {
    if (!raw) return $t('common.unknown');
    let s = raw;
    // Virer le préfixe csp/...
    s = s.replace(/^csp\/[^/]*\/\.\.\//i, '');
    // Virer les éventuels chemins Windows (on garde le dernier élément après un slash)
    if (s.includes('/')) {
      const parts = s.split('/');
      s = parts[parts.length - 1];
    }
    return s;
  }

  // ⭐ Analyse du circuit (croisement avec le local)
  function getTrackDisplay(serverTrackId: string) {
    if (!serverTrackId) return { name: $t('common.unknown'), exists: false };

    // ⭐ On utilise la fonction de nettoyage globale
    let cleanTrack = cleanTrackName(serverTrackId);
    let trackId = cleanTrack;
    let layoutId = '';
    
    // Séparation track / layout (ex: ks_nordschleife-touristenfahrten)
    const dashIdx = cleanTrack.indexOf('-');
    if (dashIdx > 0) {
      trackId = cleanTrack.substring(0, dashIdx);
      layoutId = cleanTrack.substring(dashIdx + 1);
    }

    const localTrack = appState.tracks.find(t => t.id.toLowerCase() === trackId.toLowerCase());
    
    // Si le circuit n'existe pas
    if (!localTrack) {
      return { name: cleanTrack, exists: false };
    }

    // Si le circuit existe mais pas le layout spécifique
    if (layoutId && localTrack.layouts && localTrack.layouts.length > 0) {
      // On normalise la recherche (minuscules, on enlève les "layout_" éventuels)
      const searchLayout = layoutId.toLowerCase().replace('layout_', '');
      const hasLayout = localTrack.layouts.some((l: any) => 
        l.id.toLowerCase() === layoutId.toLowerCase() || 
        l.id.toLowerCase() === `layout_${searchLayout}` ||
        l.id.toLowerCase() === searchLayout
      );
      if (!hasLayout) return { name: cleanTrack, exists: false };
    }

    // Tout est bon, on renvoie le vrai nom du circuit
    return { name: localTrack.name, exists: true };
  }

  // ⭐ Analyse des voitures (croisement avec le local)
  function getCarsDisplay(serverCars: string[]) {
    if (!serverCars || !Array.isArray(serverCars) || serverCars.length === 0) {
      return { allExist: true, missingCount: 0, total: 0 }; // On considère OK si pas de liste
    }
    
    let missing = 0;
    for (const carId of serverCars) {
      const localCar = appState.cars.find(c => c.id.toLowerCase() === carId.toLowerCase());
      if (!localCar) missing++;
    }
    
    return { 
      allExist: missing === 0, 
      missingCount: missing, 
      total: serverCars.length 
    };
  }

  function getCountryName(code: string) {
    if (!code) return $t('common.unknown');
    return $t('countries.' + code.toLowerCase(), { default: code.toUpperCase() });
  }

    // ⭐ On utilise la fonction globale de assets.js
  function getFlagImage(countryCode: string) {
    return getFlagUrl(countryCode);
  }

  // ⭐ Ajouter un serveur récent aux serveurs enregistrés
  function addRecentToSaved(srv: any) {
    const exists = appState.customServers.some((s: any) => s.ip === srv.ip && s.port === srv.port);
    if (exists) {
      alert($t('online.error_already_saved'));
      return;
    }
    appState.customServers.push({
      id: 'srv_' + Date.now(),
      ip: srv.ip,
      port: srv.port,
      httpPort: srv.httpPort || (srv.port + 1),
      name: srv.name || `${srv.ip}:${srv.port}`
    });
    saveServerState();
  }

  // ⭐ Modifier un serveur enregistré
  function openEditModal(srv: any) {
    editingServer = srv;
    editName = srv.name;
    editIp = `${srv.ip}:${srv.port}`;
  }

  function saveEditedServer() {
    if (!editingServer) return;
    const m = editIp.match(/^([\w.\-]+):(\d{1,5})$/);
    if (!m) {
      alert($t('online.error_format'));
      return;
    }
    editingServer.name = editName || `${m[1]}:${m[2]}`;
    editingServer.ip = m[1];
    editingServer.port = parseInt(m[2], 10);
    editingServer.httpPort = editingServer.port + 1;
    
    saveServerState();
    editingServer = null;
  }

  // Parser BBCode (version avancée reprise de QACRL)
  function bbcodeToHtml(input: string) {
    if (!input || typeof input !== 'string') return '';

    let s = input
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;');

    // Code et citation
    s = s.replace(/\[code\]([\s\S]*?)\[\/code\]/gi, '<pre class="bb-code">$1</pre>');
    s = s.replace(/\[quote\]([\s\S]*?)\[\/quote\]/gi, '<blockquote class="bb-quote">$1</blockquote>');

    // Images
    s = s.replace(/\[img="?([^"\]]+)"?\]([\s\S]*?)\[\/img\]/gi, (m, url, alt) => {
      const cleanUrl = String(url).trim().replace(/^"|"$/g, '');
      const safeAlt = String(alt).trim() || 'image';
      return `<img class="bb-img" src="${cleanUrl}" alt="${safeAlt}" onerror="this.style.display='none'">`;
    });
    s = s.replace(/\[img\]([\s\S]*?)\[\/img\]/gi, (m, url) => {
      const cleanUrl = String(url).trim().replace(/^"|"$/g, '');
      return `<img class="bb-img" src="${cleanUrl}" alt="" onerror="this.style.display='none'">`;
    });

    // Liens
    s = s.replace(/\[url="?([^"\]]+)"?\]([\s\S]*?)\[\/url\]/gi, (m, url, text) => {
      const cleanUrl = String(url).trim().replace(/^"|"$/g, '');
      return `<a class="bb-link" href="${cleanUrl}" target="_blank">${text}</a>`;
    });
    s = s.replace(/\[url\]([\s\S]*?)\[\/url\]/gi, (m, url) => {
      const cleanUrl = String(url).trim().replace(/^"|"$/g, '');
      return `<a class="bb-link" href="${cleanUrl}" target="_blank">${cleanUrl}</a>`;
    });

    // Couleurs, tailles, alignement
    s = s.replace(/\[color=([^\]]+)\]([\s\S]*?)\[\/color\]/gi, (m, color, text) => `<span style="color:${color}">${text}</span>`);
    s = s.replace(/\[size=(\d+)\]([\s\S]*?)\[\/size\]/gi, (m, size, text) => {
      const n = Math.max(8, Math.min(48, parseInt(size, 10) || 14));
      return `<span style="font-size:${n}px;line-height:1.3">${text}</span>`;
    });
    s = s.replace(/\[center\]([\s\S]*?)\[\/center\]/gi, '<div style="text-align:center">$1</div>');
    s = s.replace(/\[left\]([\s\S]*?)\[\/left\]/gi, '<div style="text-align:left">$1</div>');
    s = s.replace(/\[right\]([\s\S]*?)\[\/right\]/gi, '<div style="text-align:right">$1</div>');

    // Style de texte
    s = s.replace(/\[b\]([\s\S]*?)\[\/b\]/gi, '<strong>$1</strong>');
    s = s.replace(/\[i\]([\s\S]*?)\[\/i\]/gi, '<em>$1</em>');
    s = s.replace(/\[u\]([\s\S]*?)\[\/u\]/gi, '<u>$1</u>');
    s = s.replace(/\[s\]([\s\S]*?)\[\/s\]/gi, '<s>$1</s>');

    // Nettoyage des balises orphelines
    s = s.replace(/\[(size|color|url|img|b|i|u|s|center|left|right|quote|code)=?[^\]]*\]/gi, '');
    s = s.replace(/\[\/(size|color|url|img|b|i|u|s|center|left|right|quote|code)\]/gi, '');

    // Sauts de ligne
    s = s.replace(/\r\n/g, '\n').replace(/\n/g, '<br>');

    return s;
  }

  // ⭐ Détecte si le texte est déjà du HTML (pour le Welcome Message de CM)
  function formatServerDescription(text: string | null | undefined) {
    if (!text) return '';
    try {
      let formatted = text;

      // Si c'est du HTML pur, on remplace les variables CM si elles sont présentes
      if (/<[a-z][\s\S]*>/i.test(formatted)) {
        // Remplacement des variables CM courantes
        if (serverDetails) {
          formatted = formatted.replace(/\{\{\s*\.ServerName\s*\}\}/gi, serverDetails.name || '');
          formatted = formatted.replace(/\{\{\s*\.TrackName\s*\}\}/gi, serverDetails.track || '');
          // Ajout d'une cible _blank pour les liens
          formatted = formatted.replace(/<a\s+(?!.*?target=)/gi, '<a target="_blank" ');
        }
        return formatted;
      }
      
      // Sinon, on applique le parser BBCode classique
      return bbcodeToHtml(formatted);
    } catch (e) {
      console.error("Erreur parsing description:", e);
      return text;
    }
  }

  // Parseur de lien (acstuff, acmanager, IP:PORT)
  function parseServerLink(raw: string) {
    let s = raw.trim().replace(/^["'<>\s]+|["'<>\s]+$/g, '');
    let ip = '', httpPort = 0, password = '';
    
    const ipMatch = s.match(/[?&]ip=([^&\s]+)/i);
    // ⭐ On enlève le ^ (caret) et les espaces qui pourraient traîner à cause de CMD
    if (ipMatch) ip = decodeURIComponent(ipMatch[1]).replace(/\^/g, '').trim();
    
    const httpMatch = s.match(/[?&]httpPort=([^&\s]+)/i);
    if (httpMatch) httpPort = parseInt(httpMatch[1], 10) || 0;

    const pwdMatch = s.match(/[?&](?:password|pass|pwd)=([^&\s]*)/i);
    if (pwdMatch) password = decodeURIComponent(pwdMatch[1] || '');

    if (!ip) {
      const ipport = s.match(/([\d\.a-zA-Z\-]+):(\d+)$/);
      if (ipport) {
        ip = ipport[1];
        httpPort = parseInt(ipport[2], 10);
      }
    }

    if (!ip) return null;
    return { ip, httpPort, password };
  }

  async function chargerServeurs(isLoadMore = false) {
    if (isLoadMore) {
      isLoadingMore = true;
    } else {
      isLoading = true;
      servers = [];
      currentPage = 1;
      hasNextPage = true;
      errorMsg = '';
    }

    try {
      const res = await invoke<{ servers: any[], has_next: boolean }>('get_online_servers', { 
        page: isLoadMore ? currentPage + 1 : 1 
      });
      
      if (isLoadMore) {
        servers = [...servers, ...res.servers];
        currentPage++;
      } else {
        servers = res.servers;
        currentPage = 1;
      }
      
      hasNextPage = res.has_next;
      pingAllServers();
    } catch (e) {
      errorMsg = String(e);
      if (!isLoadMore) servers = [];
    } finally {
      isLoading = false;
      isLoadingMore = false;
    }
  }

  async function pingAllServers() {
    // ⭐ On filtre pour ne garder que les serveurs qui n'ont pas encore été analysés
    const serversToPing = servers.filter(server => !serverInfos[server.ip + ':' + server.port]);

    // ⭐ On lance TOUTES les requêtes en même temps au lieu de les faire une par une
    await Promise.all(serversToPing.map(async (server) => {
      const key = server.ip + ':' + server.port;
      try {
        const start = Date.now();
        const info = await invoke<any>('get_server_details', { ip: server.ip, httpPort: server.port });
        const ms = Date.now() - start;
        
        pingStatus[key] = { ms, status: 'ok' };
        serverInfos[key] = info;
      } catch (e) {
        pingStatus[key] = { ms: null, status: 'fail' };
      }
    }));
  }

  function handlePasteLink() {
    const parsed = parseServerLink(pastedLink);
    if (!parsed) {
      alert($t('online.error_format_link'));
      return;
    }
    openJoinModal({ 
      id: 'link_' + Date.now(), 
      ip: parsed.ip, 
      port: parsed.httpPort, // On suppose que c'est le port HTTP ou de jeu
      name: `${parsed.ip}:${parsed.httpPort}`,
      source: 'link'
    });
    pastedLink = '';
  }

  function handleAddManual() {
    if (!manualIp) return;
    const m = manualIp.match(/^([\w.\-]+):(\d{1,5})$/);
    if (!m) {
      alert($t('online.error_format'));
      return;
    }
    const port = parseInt(m[2], 10);
    
    // ⭐ Enregistrement dans la liste
    const newServer = {
      id: 'srv_' + Date.now(),
      ip: m[1],
      port: port,
      httpPort: port + 1,
      name: manualName || `${m[1]}:${port}`
    };
    
    appState.customServers.push(newServer);
    saveServerState();
    
    manualIp = '';
    manualName = '';
  }

  async function openJoinModal(server: any) {
    selectedServer = server;
    serverDetails = null;
    commonCars = [];
    missingCars = [];
    missingTracks = [];
    serverFeatures = [];
    serverDescription = '';
    passwordRequired = false;
    selectedCar = null;
    availableSkins = [];
    selectedSkin = null;
    loadingDetails = true;

    // ⭐ Pré-remplir le mot de passe sauvegardé
    const pwdKey = `${server.ip}:${server.port}`;
    serverPassword = (appState.serverPasswords || {})[pwdKey] || '';

    try {
      const info = await invoke<any>('get_server_details', { ip: server.ip, httpPort: server.port });
      serverDetails = info;

      // ⭐ On récupère la description
      serverDescription = info.description || info.desc || '';

      // 1. Détection circuit (en séparant le circuit et le layout)
      const rawTrack = info.track || '';
      
      // ⭐ NETTOYAGE ROBUSTE : On isole le nom du dossier final (ex: "D:/../ks_nordschleife" -> "ks_nordschleife")
      let cleanTrack = rawTrack.replace(/^csp\/[^/]*\/\.\.\//i, '').replace(/^csp\//i, '');
      
      // On prend le dernier élément du chemin après un slash (si c'est un chemin Windows)
      const pathParts = cleanTrack.split('/');
      cleanTrack = pathParts[pathParts.length - 1];
      
      // Si le chemin contenait le layout (ex: ks_nordschleife-touristenfahrten)
      let trackId = cleanTrack;
      let layoutId = '';

      // On sépare sur le premier tiret (ex: "ks_nordschleife-touristenfahrten")
      const dashIdx = cleanTrack.indexOf('-');
      if (dashIdx > 0) {
        trackId = cleanTrack.substring(0, dashIdx);
        layoutId = cleanTrack.substring(dashIdx + 1);
      }

      if (trackId) {
        const localTrack = appState.tracks.find(t => t.id.toLowerCase() === trackId.toLowerCase());
        if (!localTrack) {
          missingTracks.push(trackId);
        } else if (layoutId && localTrack.layouts && localTrack.layouts.length > 0) {
          // Si on a un layout, on vérifie qu'il existe dans ce circuit
          const hasLayout = localTrack.layouts.some((l: any) => l.id.toLowerCase() === layoutId.toLowerCase());
          if (!hasLayout) {
            missingTracks.push(`${trackId}-${layoutId}`);
          }
        }
      }

      // 2. Croisement voitures
      if (info.cars && Array.isArray(info.cars)) {
        info.cars.forEach((carId: string) => {
          const localCar = appState.cars.find(c => c.id.toLowerCase() === carId.toLowerCase());
          if (localCar) commonCars.push(localCar);
          else missingCars.push(carId);
        });
        commonCars.sort((a, b) => a.name.localeCompare(b.name));
      }

      // 3. Description & features (AssettoServer ou Kunos)
      serverDescription = info.description || info.desc || '';
      passwordRequired = !!(info.pass || info.passwordRequired);
      if (Array.isArray(info.features)) serverFeatures = info.features;

    } catch (e) {
      console.error(e);
    } finally {
      loadingDetails = false;
    }
  }

  function selectCar(car: any) {
    selectedCar = car;
    availableSkins = car.skins || [];
    selectedSkin = availableSkins.length > 0 ? availableSkins[0] : null;
  }

  // ⭐ Helpers pour les serveurs récents / favoris
  function formatTimeAgo(timestamp: number) {
    if (!timestamp) return '—';
    const diff = Date.now() - timestamp;
    const min = Math.floor(diff / 60000);
    if (min < 1) return $t('online.time_ago_now');
    if (min < 60) return $t('online.time_ago_min', { values: { min } });
    const h = Math.floor(min / 60);
    if (h < 24) return $t('online.time_ago_hour', { values: { h } });
    const d = Math.floor(h / 24);
    return $t('online.time_ago_day', { values: { d } });
  }

  function isFavoriteServer(srv: any) {
    if (!srv) return false;
    const key = `${srv.ip}:${srv.port}`;
    return appState.favoriteServers.includes(key);
  }

  function toggleFavoriteServer(srv: any) {
    if (!srv) return;
    const key = `${srv.ip}:${srv.port}`;
    const idx = appState.favoriteServers.indexOf(key);
    if (idx >= 0) appState.favoriteServers.splice(idx, 1);
    else appState.favoriteServers.push(key);
    saveServerState();
  }

  function pushRecentServer(server: any, info: any) {
    if (!server || !server.ip || !server.port) return;
    const key = `${server.ip}:${server.port}`;
    // On retire l'ancienne entrée si elle existe
    appState.recentServers = appState.recentServers.filter((s: any) => `${s.ip}:${s.port}` !== key);
    
    appState.recentServers.unshift({
      ip: server.ip,
      port: server.port,
      httpPort: info?.httpPort || server.httpPort || (server.port + 1),
      name: info?.name || server.name || `${server.ip}:${server.port}`,
      track: info?.track || '',
      countryCode: server.country || '',
      playersCurrent: info?.clients || server.players || 0,
      playersMax: info?.maxclients || server.max_players || 0,
      date: Date.now()
    });
    
    if (appState.recentServers.length > 15) {
      appState.recentServers.length = 15;
    }
    saveServerState();
  }

  async function rejoindre() {
    if (!selectedCar || !selectedServer) return;
    isJoining = true;

    try {
      // ⭐ On extrait les features du serveur (si l'API /api/details les a fournies)
      let features: string[] = [];
      if (serverDetails?.features && Array.isArray(serverDetails.features)) {
        features = serverDetails.features;
      }

            // ⭐ Sécurité : si le SteamID est vide, on bloque pour éviter le Handshake Failed
      if (!appState.steamId) {
        alert($t('online_servers.error_steamid'));
        isJoining = false;
        return;
      }

            // ⭐ On extrait l'ID du circuit et du layout pour le race.ini
      const rawTrack = serverDetails?.track || selectedServer.track || '';
      let cleanTrack = rawTrack.replace(/^csp\/[^/]*\/\.\.\//i, '').replace(/^csp\//i, '');
      
      const pathParts = cleanTrack.split('/');
      cleanTrack = pathParts[pathParts.length - 1];
      
      let trackId = cleanTrack;
      let layoutId = '';
      const dashIdx = cleanTrack.indexOf('-');
      if (dashIdx > 0) {
        trackId = cleanTrack.substring(0, dashIdx);
        layoutId = cleanTrack.substring(dashIdx + 1);
      }

      await invoke('join_server', {
        acPath: appState.acPath,
        documentsPath: appState.documentsPath,
        ip: selectedServer.ip,
        // ⭐ On prend le port de jeu (UDP) qui est 'port' dans /INFO ou 'gamePort' dans /api/details
        port: serverDetails?.gamePort || serverDetails?.port || selectedServer.port,
        // ⭐ Le port HTTP pour le launcher
        httpPort: serverDetails?.httpPort || serverDetails?.cport || selectedServer.port,
        carId: selectedCar.id,
        skinId: selectedSkin ? selectedSkin.name : "",
        onlineName: appState.onlineName || appState.driverName || "Player",
        steamId: appState.steamId || "", 
        serverName: selectedServer.name,
        track: trackId,        // ⭐ Ex: "ks_nordschleife"
        trackConfig: layoutId, // ⭐ Ex: "touristenfahrten"
        password: serverPassword, 
        features: features,       
      });
      
      // ⭐ Ajouter aux serveurs récents
      pushRecentServer(selectedServer, serverDetails);

      // ⭐ Mémoriser le mot de passe pour la prochaine fois
      const pwdKey = `${selectedServer.ip}:${selectedServer.port}`;
      if (serverPassword) appState.serverPasswords[pwdKey] = serverPassword;
      else delete appState.serverPasswords[pwdKey];
      saveServerState();
      
      selectedServer = null;
    } catch (e) {
      alert($t('online_servers.error_join') + e);
    } finally {
      isJoining = false;
    }
  }

  $effect(() => {
    chargerServeurs(false);
  });

  // ⭐ Écoute du Deep Link pour ouvrir automatiquement la modale
  $effect(() => {
    const handleDeepLink = (e: any) => {
      const url = e.detail?.url;
      if (!url) return;

      console.log('🔗 OnlineTab traite le lien:', url);

      // On utilise la fonction parseServerLink existante
      const parsed = parseServerLink(url);
      if (parsed) {
        // On ouvre la modale de connexion avec les infos extraites
        openJoinModal({ 
          id: 'deeplink_' + Date.now(), 
          ip: parsed.ip, 
          port: parsed.httpPort || parsed.port, // On s'assure d'avoir un port
          name: parsed.ip,
          source: 'deeplink'
        });
      } else {
        alert($t('online.error_format_link'));
      }
    };

    window.addEventListener('qarl-process-deep-link', handleDeepLink);
    return () => window.removeEventListener('qarl-process-deep-link', handleDeepLink);
  });
</script>

<!-- PARTIE HAUTE : Coller un lien & Ajouter manuellement -->
<div class="group">
  <div class="group-title" style="display: flex; justify-content: space-between; align-items: center;">
    <span>{$t('online.paste_title')}</span>
    <button class="btn-secondary" onclick={toggleAssociation} style="font-size: 0.75rem; padding: 4px 10px; height: auto;">
      {isAssociated ? '🔓 DISSOCIER' : '🔗 ASSOCIER'}
    </button>
  </div>
  <div class="row">
    <div class="field" style="flex: 3;">
      <input type="text" bind:value={pastedLink} placeholder={$t('online.paste_placeholder')} />
    </div>
    <div class="field" style="flex: 0 0 auto; justify-content: flex-end;">
      <button class="btn-secondary" onclick={handlePasteLink} style="padding: 10px 20px;">{$t('online.join_btn')}</button>
    </div>
  </div>
</div>

<div class="group">
  <div class="group-title">{$t('online.add_manual_title')}</div>
  <div class="row">
    <div class="field">
      <input type="text" bind:value={manualIp} placeholder={$t('online.manual_ip_placeholder')} />
    </div>
    <div class="field">
      <input type="text" bind:value={manualName} placeholder={$t('online.manual_name_placeholder')} />
    </div>
    <div class="field" style="flex: 0 0 auto; justify-content: flex-end;">
      <button class="btn-secondary" onclick={handleAddManual} style="padding: 10px 20px;">{$t('online.add_btn')}</button>
    </div>
  </div>
</div>

<!-- ⭐ SERVEURS RÉCENTS -->
<div class="group" style="margin-top: 20px;">
  <div class="group-title" style="display: flex; justify-content: space-between; align-items: center;">
    <span>{$t('online.recent_title')}</span>
    <button class="filter-btn" onclick={() => recentFilter.favoritesOnly = !recentFilter.favoritesOnly} style="font-size: 0.75rem; padding: 4px 10px;" class:active={recentFilter.favoritesOnly}>
      {$t('online.fav_only')}
    </button>
  </div>
  
  <div class="servers-list">
    {#if appState.recentServers.length === 0}
      <div class="servers-empty">{$t('online.recent_empty')}</div>
    {:else}
      {#each appState.recentServers.filter((s: any) => !recentFilter.favoritesOnly || isFavoriteServer(s)) as srv}
        <div class="server-row">
          <div class="server-flag" title={srv.countryCode || $t('common.unknown')}>
            {#if getFlagImage(srv.countryCode)}
              <img src={getFlagImage(srv.countryCode)} alt={srv.countryCode} style="width: 100%; height: 100%; object-fit: cover; border-radius: 2px;" />
            {:else}
              {srv.countryCode ? srv.countryCode.toUpperCase() : '🌐'}
            {/if}
          </div>
          <div class="server-info">
            <div class="server-name">{srv.name}</div>
            <div class="server-meta">
              <code>{srv.ip}:{srv.port}</code>
              <span>🕒 {formatTimeAgo(srv.date)}</span>
              {#if srv.track}<span>{srv.track}</span>{/if}
            </div>
          </div>
          <div class="server-players empty">—</div>
          <button class="server-fav-btn" class:active={isFavoriteServer(srv)} onclick={() => toggleFavoriteServer(srv)} title={$t('common.filter_favorites')}>
            {isFavoriteServer(srv) ? '★' : '☆'}
          </button>
          <button class="server-action" style="background: var(--bg-hover); color: var(--text-primary); border: 1px solid var(--border);" onclick={() => addRecentToSaved(srv)} title={$t('online.recent_save_tooltip')}>💾</button>
          <button class="server-action" onclick={() => openJoinModal(srv)}>{$t('online.join_btn').replace('▶ ', '')}</button>
          <button class="server-action delete" onclick={() => { appState.recentServers = appState.recentServers.filter((s: any) => s !== srv); saveServerState(); }}>🗑️</button>
        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- ⭐ MES SERVEURS ENREGISTRÉS -->
<div class="group">
  <div class="group-title" style="display: flex; justify-content: space-between; align-items: center;">
    <span>{$t('online.saved_title')}</span>
    <button class="filter-btn" onclick={() => manualFilter.favoritesOnly = !manualFilter.favoritesOnly} style="font-size: 0.75rem; padding: 4px 10px;" class:active={manualFilter.favoritesOnly}>
      {$t('online.fav_only')}
    </button>
  </div>

  <div class="servers-list">
    {#if appState.customServers.length === 0}
      <div class="servers-empty">{$t('online.saved_empty')}</div>
    {:else}
      {#each appState.customServers.filter((s: any) => !manualFilter.favoritesOnly || isFavoriteServer(s)) as srv}
        <div class="server-row">
          <div class="server-flag">🔧</div>
          <div class="server-info">
            <div class="server-name">{srv.name}</div>
            <div class="server-meta">
              <code>{srv.ip}:{srv.port}</code>
              <span>{$t('online.saved_manual')}</span>
            </div>
          </div>
          <div class="server-players empty">—</div>
          <button class="server-fav-btn" class:active={isFavoriteServer(srv)} onclick={() => toggleFavoriteServer(srv)} title={$t('common.filter_favorites')}>
            {isFavoriteServer(srv) ? '★' : '☆'}
          </button>
          <button class="server-action" style="background: var(--bg-hover); color: var(--text-primary); border: 1px solid var(--border);" onclick={() => openEditModal(srv)} title="Modifier">⚙️</button>
          <button class="server-action" onclick={() => openJoinModal(srv)}>{$t('online.join_btn').replace('▶ ', '')}</button>
          <button class="server-action delete" onclick={() => { 
            appState.customServers = appState.customServers.filter((s: any) => s !== srv); 
            saveServerState(); 
          }}>🗑️</button>
        </div>
      {/each}
    {/if}
  </div>
</div>

<!-- LISTE DES SERVEURS PUBLICS -->
<!-- ⭐ Conteneur Sticky pour le titre, les boutons et les filtres -->
<div style="position: sticky; top: -24px; z-index: 100; background: var(--bg-root); margin: -24px -24px 0 -24px; padding: 24px 24px 15px 24px; border-bottom: 1px solid var(--border-subtle); box-shadow: 0 4px 10px rgba(0,0,0,0.3);">
  
  <div class="section-header" style="margin-bottom: 15px;">
    <h2 class="section-title">{$t('online_servers.public_title', { values: { filtered: filteredServers.length, total: servers.length } })}</h2>
    <div style="display: flex; gap: 10px;">
      {#if hasNextPage}
        <button class="btn-secondary" onclick={() => chargerServeurs(true)} disabled={isLoadingMore || isLoading} style="padding: 0 15px;">
          {isLoadingMore ? '⏳...' : $t('online_servers.load_more')}
        </button>
      {/if}
      <button class="btn-secondary" onclick={() => chargerServeurs(false)} disabled={isLoading}>
        {isLoading ? $t('common.loading') : $t('online_servers.refresh')}
      </button>
    </div>
  </div>

  <!-- ⭐ BARRE DE FILTRES -->
  <div class="group" style="padding: 12px; display: flex; flex-direction: row; flex-wrap: wrap; gap: 12px; align-items: flex-end; margin-bottom: 0;">
    <div class="field" style="flex: 2; min-width: 200px;">
      <input type="text" bind:value={searchQuery} placeholder={$t('online_servers.search_placeholder')} />
    </div>
    <div class="field" style="flex: 1; min-width: 140px;">
      <select bind:value={filterCountry}>
        <option value="">{$t('online_servers.all_countries')}</option>
        {#each availableCountries as c}
          <option value={c}>{getCountryName(c)}</option>
        {/each}
      </select>
    </div>
    <div class="field" style="flex: 2; min-width: 180px;">
      <select bind:value={filterTrack}>
        <option value="">{$t('online_servers.all_tracks')}</option>
        {#each availableTracks as t}
          <option value={t}>{t}</option>
        {/each}
      </select>
    </div>
    <div class="field" style="flex: 0 0 100px;">
      <input type="number" bind:value={filterMinPlayers} min="0" placeholder={$t('online_servers.min_players')} title={$t('online_servers.min_players')} />
    </div>
    <div class="field" style="flex: 0 0 auto; min-width: 150px; flex-direction: row; align-items: center; gap: 8px;">
      <input type="checkbox" id="filter-playable" bind:checked={filterOnlyPlayable} style="width: 16px; height: 16px; margin: 0; cursor: pointer;" />
      <label for="filter-playable" style="font-size: 0.8rem; font-weight: 600; color: var(--text-secondary); cursor: pointer; white-space: nowrap;">
        {$t('online_servers.only_playable')}
      </label>
    </div>
  </div>

</div> <!-- Fin du conteneur sticky -->

{#if errorMsg}
  <div class="group" style="border-color: #a83a3a;"><p style="color: #e57373;">❌ {errorMsg}</p></div>
{:else if isLoading && servers.length === 0}
  <div class="servers-loading">{$t('online_servers.loading')}</div>
{:else if filteredServers.length === 0}
  <div class="servers-empty">{$t('online_servers.no_results')}</div>
{:else}
  <div class="servers-list">
    {#each filteredServers as server}
      {@const key = server.ip + ':' + server.port}
      {@const ping = pingStatus[key]}
      {@const isFull = server.players >= server.max_players}
      
      <div class="server-row">
        <div class="server-flag" title={server.country || $t('common.unknown')}>
          {#if getFlagImage(server.country)}
            <img src={getFlagImage(server.country)} alt={server.country} style="width: 100%; height: 100%; object-fit: cover; border-radius: 2px;" />
          {:else}
            {server.country ? server.country.toUpperCase() : '🌐'}
          {/if}
        </div>
        <div class="server-info">
          <div class="server-name">{server.name}</div>
          <div class="server-meta">
            <code>{server.ip}:{server.port}</code>
            
            <!-- ⭐ Affichage intelligent du Circuit et des Voitures -->
            {#if serverInfos[key]}
              {@const trackData = getTrackDisplay(serverInfos[key].track)}
              {@const carsData = getCarsDisplay(serverInfos[key].cars)}
              
              <span style="color: {trackData.exists ? '#6fc98a' : '#e57373'}; font-weight: 600; display: flex; align-items: center; gap: 4px;" title={trackData.exists ? $t('online_servers.track_installed') : $t('online_servers.track_missing')}>
                {trackData.exists ? '✅' : '❌'} {cleanTrackName(server.track)}
              </span>

              <span style="color: {carsData.allExist ? '#6fc98a' : '#e57373'}; font-weight: 600; display: flex; align-items: center; gap: 4px;" title={carsData.allExist ? $t('online_servers.cars_installed') : $t('online_servers.cars_missing', { values: { count: carsData.missingCount } })}>
                {#if carsData.total > 0}
                  {carsData.allExist ? '✅' : '❌'} 
                  {carsData.allExist ? $t('online_servers.cars_ok') : $t('online_servers.mods_missing', { values: { count: carsData.missingCount } })}
                {/if}
              </span>
            {:else}
              <span style="color: var(--text-tertiary); display: flex; align-items: center; gap: 4px;">
                {$t('online_servers.analyzing')}
              </span>
            {/if}
          </div>
        </div>
        <div class="server-players" class:full={isFull} class:empty={server.players === 0}>
          {server.players} / {server.max_players}
        </div>
        {#if ping}
          {#if ping.status === 'ok'}<div class="server-ping good">{ping.ms} ms</div>
          {:else}<div class="server-ping bad">✕</div>{/if}
        {:else}
          <div class="server-ping">⏳</div>
        {/if}
        <button class="server-action" onclick={() => openJoinModal(server)}>{$t('online_servers.join')}</button>
      </div>
    {/each}
  </div>
{/if}

<!-- MODALE DE CONNEXION (Style QACRL) -->
{#if selectedServer}
  <div class="modal" style="display: flex; z-index: 1000;">
    <div class="modal-content server-join-modal" style="width: 700px; max-width: 95vw; max-height: 90vh; overflow-y: auto;">
      
      <div class="modal-header">
        <h3 class="modal-title">{$t('online_servers.modal_title')}</h3>
        <button class="modal-close" onclick={() => selectedServer = null}>&times;</button>
      </div>

      {#if loadingDetails}
        <div style="padding: 40px; text-align: center;">
          <div class="qacrl-loading-spinner" style="margin: 0 auto 10px;"></div>
          {$t('online_servers.loading_details')}
        </div>
      {:else if serverDetails}
        {@const trackInfo = getTrackDisplay(serverDetails.track)}
        
        <!-- INFOS SERVEUR -->
        <div class="server-join-info">
          <div class="item">
            <span class="label">{$t('online_servers.info_server')}</span>
            <span class="value">{serverDetails.name || selectedServer.name}</span>
          </div>
          <div class="item">
            <span class="label">{$t('online_servers.info_track')}</span>
            <span class="value">{trackInfo.name} {serverDetails.track_config ? `(${serverDetails.track_config})` : ''}</span>
          </div>
          <div class="item">
            <span class="label">{$t('online_servers.info_players')}</span>
            <span class="value">
              {typeof serverDetails.clients === 'number' ? serverDetails.clients : (typeof serverDetails.players === 'number' ? serverDetails.players : 0)} 
              / 
              {typeof serverDetails.maxclients === 'number' ? serverDetails.maxclients : (typeof serverDetails.max_players === 'number' ? serverDetails.max_players : 0)}
            </span>
          </div>
          <div class="item">
            <span class="label">{$t('online_servers.info_password')}</span>
            <span class="value">{passwordRequired ? `🔒 ${$t('common.yes')}` : $t('common.no')}</span>
          </div>
        </div>

        <!-- CSP FEATURES -->
        {#if serverFeatures.length > 0}
          <div style="margin-top: 15px; padding: 10px 14px; background: var(--bg-elevated); border-radius: 6px;">
            <div style="font-size: 0.65rem; font-weight: 800; color: var(--text-tertiary); text-transform: uppercase; margin-bottom: 8px;">{$t('online_servers.features')}</div>
            <div style="display: flex; flex-wrap: wrap; gap: 6px;">
              {#each serverFeatures as feat}
                <span class="csp-feature" style="font-size: 0.7rem; padding: 3px 8px; background: var(--accent-muted); border: 1px solid rgba(var(--accent-rgb), 0.25); border-radius: 12px;">
                  {feat}
                </span>
              {/each}
            </div>
          </div>
        {/if}

        <!-- DESCRIPTION -->
        {#if serverDescription}
          <div class="server-description-wrap" style="margin-top: 15px;">
            <div class="server-description-label">{$t('online_servers.description')}</div>
            <div class="server-description" style="color: white;">
              {@html formatServerDescription(serverDescription)}
            </div>
          </div>
        {/if}

        <!-- WARNINGS (Mods manquants) -->
        <div style="margin-top: 15px;">
          {#if missingTracks.length > 0}
            <div class="server-join-warning error">
              {@html $t('online_servers.missing_track', { values: { track: missingTracks[0] } })}
            </div>
          {/if}

          {#if commonCars.length === 0}
            <div class="server-join-warning error">
              {$t('online_servers.no_common_cars')}
            </div>
          {/if}

          {#if missingTracks.length > 0 || missingCars.length > 0}
            <div class="missing-mods-block">
              {#if missingTracks.length > 0}
                <div class="missing-mods-header">
                  <span>{$t('online_servers.missing_tracks_header', { values: { count: missingTracks.length } })}</span>
                </div>
                <div class="missing-mods-list">
                  {#each missingTracks as track}
                    <span class="missing-mod-id">
                      <span class="missing-mod-name">{track}</span>
                      <button class="missing-mod-search" style="font-size: 14px; padding: 2px 6px;" onclick={() => openUrl(`https://duckduckgo.com/?q=${encodeURIComponent(track + ' Assetto Corsa track mod')}`)} title={$t('online_servers.search_mod')}>🔍</button>
                    </span>
                  {/each}
                </div>
              {/if}

              {#if missingCars.length > 0}
                <div class="missing-mods-header" style="margin-top: 10px;">
                  <span>{$t('online_servers.missing_cars_header', { values: { count: missingCars.length } })}</span>
                </div>
                <div class="missing-mods-list">
                  {#each missingCars as carId}
                    <span class="missing-mod-id">
                      <span class="missing-mod-name">{carId}</span>
                      <button class="missing-mod-search" onclick={() => openUrl(`https://duckduckgo.com/?q=${encodeURIComponent(carId + ' Assetto Corsa mod')}`)} title={$t('online_servers.search_mod')}>🔍</button>
                    </span>
                  {/each}
                </div>
              {/if}
            </div>
          {/if}
        </div>

        <!-- CHOIX VOITURE -->
        <div class="field" style="margin-top: 15px;">
          <label>{$t('online_servers.choose_car')}</label>
          <div class="car-select-list">
            {#if commonCars.length === 0}
              <div style="padding: 15px; color: var(--text-tertiary);">{$t('online_servers.no_cars')}</div>
            {:else}
              {#each commonCars as car}
                <div class="car-select-row" class:selected={selectedCar?.id === car.id} onclick={() => selectCar(car)}>
                  <div class="car-select-radio"></div>
                  <div class="car-select-main">
                    <div class="car-select-name">{car.name}</div>
                    <div class="car-select-id">{car.id}</div>
                  </div>
                  <div class="car-select-count free">(1/1)</div>
                </div>
              {/each}
            {/if}
          </div>
        </div>

        <!-- CHOIX SKIN -->
        {#if selectedCar && availableSkins.length > 0}
          <div class="field" style="margin-top: 15px;">
            <label>{$t('online_servers.choose_skin')}</label>
            <div class="skin-select-list">
              {#each availableSkins as skin}
                <div class="skin-select-row" class:selected={selectedSkin?.name === skin.name} onclick={() => selectedSkin = skin}>
                  {#if skin.image_path}
                    <img class="skin-select-preview" src={convertFileSrc(skin.image_path)} alt="preview" />
                  {:else}
                    <div class="skin-select-preview placeholder">🚗</div>
                  {/if}
                  <div class="skin-select-name">{skin.name}</div>
                </div>
              {/each}
            </div>
          </div>
        {/if}

        <!-- MOT DE PASSE -->
        {#if passwordRequired}
          <div class="field" style="margin-top: 15px;">
            <label>{$t('online_servers.password_label')}</label>
            <input type="text" bind:value={serverPassword} placeholder={$t('online_servers.password_placeholder')} />
          </div>
        {/if}

      {:else}
        <div style="padding: 20px; color: #e57373; text-align: center;">
          {$t('online_servers.error_details')}
        </div>
      {/if}

      <!-- BOUTONS -->
      <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 20px; border-top: 1px solid var(--border-subtle); padding-top: 15px;">
        <button class="btn-secondary" onclick={() => selectedServer = null}>{$t('common.cancel')}</button>
        <button 
          class="btn-launch" 
          style="padding: 10px 20px; font-size: 0.85rem;"
          disabled={!selectedCar || missingTracks.length > 0 || isJoining || commonCars.length === 0}
          onclick={rejoindre}
        >
          {isJoining ? $t('online_servers.joining') : $t('online.join_btn')}
        </button>
      </div>

    </div>
  </div>
{/if}

<!-- MODALE D'ÉDITION DE SERVEUR -->
{#if editingServer}
  <div class="modal" style="display: flex; z-index: 1000;">
    <div class="modal-content" style="width: 500px;">
      <div class="modal-header">
        <h3 class="modal-title">{$t('online.edit_title')}</h3>
        <button class="modal-close" onclick={() => editingServer = null}>&times;</button>
      </div>
      
      <div class="field">
        <label>{$t('online.edit_name')}</label>
        <input type="text" bind:value={editName} placeholder={$t('online.edit_name_placeholder')} />
      </div>
      
      <div class="field">
        <label>{$t('online.edit_ip')}</label>
        <input type="text" bind:value={editIp} placeholder={$t('online.edit_ip_placeholder')} />
      </div>

      <div style="font-size: 0.72rem; color: var(--text-tertiary); line-height: 1.5; margin-top: -5px;">
        {@html $t('online.edit_hint')}
      </div>

      <div style="display: flex; justify-content: flex-end; gap: 10px; margin-top: 10px;">
        <button class="btn-secondary" onclick={() => editingServer = null}>{$t('common.cancel')}</button>
        <button class="btn-launch" style="padding: 10px 20px; font-size: 0.85rem;" onclick={saveEditedServer}>💾 {$t('common.save')}</button>
      </div>
    </div>
  </div>
{/if}