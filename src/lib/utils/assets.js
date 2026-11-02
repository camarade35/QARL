// Dictionnaire : { 'nom_normalise': { name: 'Nom Lisible', file: 'Fichier.png' } }
const COUNTRY_MAP = {
  // Europe
  'fr': { name: 'countries.france', file: 'france23.png' },
  'france': { name: 'countries.france', file: 'france23.png' },
  'it': { name: 'countries.italy', file: 'italy23.png' },
  'italy': { name: 'countries.italy', file: 'italy23.png' },
  'italie': { name: 'countries.italy', file: 'italy23.png' },
  'de': { name: 'countries.germany', file: 'germany23.png' },
  'germany': { name: 'countries.germany', file: 'germany23.png' },
  'allemagne': { name: 'countries.germany', file: 'germany23.png' },
  'es': { name: 'countries.spain', file: 'spain23.png' },
  'spain': { name: 'countries.spain', file: 'spain23.png' },
  'espagne': { name: 'countries.spain', file: 'spain23.png' },
  'espana': { name: 'countries.spain', file: 'spain23.png' },
  'uk': { name: 'countries.uk', file: 'uk23.png' },
  'gb': { name: 'countries.uk', file: 'uk23.png' },
  'unitedkingdom': { name: 'countries.uk', file: 'uk23.png' },
  'greatbritain': { name: 'countries.uk', file: 'uk23.png' },
  'scotland': { name: 'countries.uk', file: 'uk23.png' },
  'ecosse': { name: 'countries.uk', file: 'uk23.png' },
  'england': { name: 'countries.england', file: 'england23.png' },
  'angleterre': { name: 'countries.england', file: 'england23.png' },
  'nl': { name: 'countries.netherlands', file: 'netherlands23.png' },
  'netherlands': { name: 'countries.netherlands', file: 'netherlands23.png' },
  'holland': { name: 'countries.netherlands', file: 'netherlands23.png' },
  'be': { name: 'countries.belgium', file: 'belgium23.png' },
  'belgium': { name: 'countries.belgium', file: 'belgium23.png' },
  'belgique': { name: 'countries.belgium', file: 'belgium23.png' },
  'ch': { name: 'countries.switzerland', file: 'switzerland23.png' },
  'switzerland': { name: 'countries.switzerland', file: 'switzerland23.png' },
  'suisse': { name: 'countries.switzerland', file: 'switzerland23.png' },
  'at': { name: 'countries.austria', file: 'austria23.png' },
  'austria': { name: 'countries.austria', file: 'austria23.png' },
  'autriche': { name: 'countries.austria', file: 'austria23.png' },
  'pt': { name: 'countries.portugal', file: 'portugal23.png' },
  'portugal': { name: 'countries.portugal', file: 'portugal23.png' },
  'se': { name: 'countries.sweden', file: 'sweden23.png' },
  'sweden': { name: 'countries.sweden', file: 'sweden23.png' },
  'suede': { name: 'countries.sweden', file: 'sweden23.png' },
  'pl': { name: 'countries.poland', file: 'poland23.png' },
  'pl': { name: 'countries.poland', file: 'poland23.png' },
  'ru': { name: 'countries.russia', file: 'russia23.png' },
  'russia': { name: 'countries.russia', file: 'russia23.png' },
  'russie': { name: 'countries.russia', file: 'russia23.png' },
  'ua': { name: 'countries.ukraine', file: 'ukraine23.png' },
  'fi': { name: 'countries.finland', file: 'finland23.png' },
  'no': { name: 'countries.norway', file: 'norway23.png' },
  'dk': { name: 'countries.denmark', file: 'denmark23.png' },
  'ie': { name: 'countries.ireland', file: 'ireland23.png' },
  'cz': { name: 'countries.czech_republic', file: 'czech_republic23.png' },
  'hu': { name: 'countries.hungary', file: 'hungary23.png' },
  'ro': { name: 'countries.romania', file: 'romania23.png' },
  'gr': { name: 'countries.greece', file: 'greece23.png' },
  'is': { name: 'countries.iceland', file: 'iceland23.png' },

  // Amériques
  'us': { name: 'countries.usa', file: 'usa23.png' },
  'usa': { name: 'countries.usa', file: 'usa23.png' },
  'unitedstates': { name: 'countries.usa', file: 'usa23.png' },
  'unitedstatesofamerica': { name: 'countries.usa', file: 'usa23.png' },
  'america': { name: 'countries.usa', file: 'usa23.png' },
  'ca': { name: 'countries.canada', file: 'canada23.png' },
  'canada': { name: 'countries.canada', file: 'canada23.png' },
  'br': { name: 'countries.brazil', file: 'brasil23.png' },
  'brazil': { name: 'countries.brazil', file: 'brasil23.png' },
  'bresil': { name: 'countries.brazil', file: 'brasil23.png' },
  'mx': { name: 'countries.mexico', file: 'mexico23.png' },
  'mexico': { name: 'countries.mexico', file: 'mexico23.png' },
  'mexique': { name: 'countries.mexico', file: 'mexico23.png' },
  'ar': { name: 'countries.argentina', file: 'argentina23.png' },
  'cl': { name: 'countries.chile', file: 'chile23.png' },
  'co': { name: 'countries.colombia', file: 'colombia23.png' },
  'uy': { name: 'countries.uruguay', file: 'uruguay23.png' },

  // Asie / Océanie / Afrique
  'jp': { name: 'countries.japan', file: 'japan23.png' },
  'japan': { name: 'countries.japan', file: 'japan23.png' },
  'japon': { name: 'countries.japan', file: 'japan23.png' },
  'cn': { name: 'countries.china', file: 'china23.png' },
  'china': { name: 'countries.china', file: 'china23.png' },
  'chine': { name: 'countries.china', file: 'china23.png' },
  'kr': { name: 'countries.south_korea', file: 'southkorea23.png' },
  'tw': { name: 'countries.taiwan', file: 'taiwan23.png' },
  'hk': { name: 'countries.hong_kong', file: 'hongkong23.png' },
  'sg': { name: 'countries.singapore', file: 'singapore23.png' },
  'my': { name: 'countries.malaysia', file: 'malaysia23.png' },
  'th': { name: 'countries.thailand', file: 'thailand23.png' },
  'vn': { name: 'countries.vietnam', file: 'vietnam23.png' },
  'id': { name: 'countries.indonesia', file: 'indonesia23.png' },
  'ph': { name: 'countries.philippines', file: 'philippines23.png' },
  'in': { name: 'countries.india', file: 'india23.png' },
  'pk': { name: 'countries.pakistan', file: 'pakistan23.png' },
  'bd': { name: 'countries.bangladesh', file: 'bangladesh23.png' },
  'ae': { name: 'countries.uae', file: 'uae23.png' },
  'sa': { name: 'countries.saudi_arabia', file: 'saudiarabia23.png' },
  'tr': { name: 'countries.turkey', file: 'turkey23.png' },
  'il': { name: 'countries.israel', file: 'israel23.png' },
  'za': { name: 'countries.south_africa', file: 'southafrica23.png' },
  'southafrica': { name: 'countries.south_africa', file: 'southafrica23.png' },
  'afriquedusud': { name: 'countries.south_africa', file: 'southafrica23.png' },
  'eg': { name: 'countries.egypt', file: 'egypt23.png' },
  'ke': { name: 'countries.kenya', file: 'kenya23.png' },
  'ng': { name: 'countries.nigeria', file: 'nigeria23.png' },
  'au': { name: 'countries.australia', file: 'australia23.png' },
  'australia': { name: 'countries.australia', file: 'australia23.png' },
  'australie': { name: 'countries.australia', file: 'australia23.png' },
  'nz': { name: 'countries.new_zealand', file: 'newzealand23.png' },
  'newzealand': { name: 'countries.new_zealand', file: 'newzealand23.png' },
  'nouvellezelande': { name: 'countries.new_zealand', file: 'newzealand23.png' }
};

// Nettoie une chaîne : minuscules, sans espaces, sans points, sans accents
function normalizeStr(str) {
  if (!str) return '';
  return str.toLowerCase()
            .normalize('NFD').replace(/[\u0300-\u036f]/g, '') // Enlève les accents
            .replace(/[^a-z0-9]/g, ''); // Enlève tout ce qui n'est pas lettre/chiffre
}

/**
 * Retourne le nom normalisé du pays à partir d'une chaîne ou d'un tableau de tags.
 */
export function getCountryName(input) {
  if (!input) return null;

  if (Array.isArray(input)) {
    for (const item of input) {
      const norm = normalizeStr(item);
      if (COUNTRY_MAP[norm]) return COUNTRY_MAP[norm].name;
    }
  } else if (typeof input === 'string') {
    const norm = normalizeStr(input);
    if (COUNTRY_MAP[norm]) return COUNTRY_MAP[norm].name;
  }
  return null;
}

/**
 * Retourne l'URL du drapeau local.
 */
export function getFlagUrl(input, fallbackPath = null) {
  if (!input) return fallbackPath ? fallbackPath : null;

  let norm = '';
  if (Array.isArray(input)) {
    for (const item of input) {
      const n = normalizeStr(item);
      if (COUNTRY_MAP[n]) { norm = n; break; }
    }
  } else if (typeof input === 'string') {
    norm = normalizeStr(input);
  }

  if (norm && COUNTRY_MAP[norm]) {
    return `/assets/flags/${COUNTRY_MAP[norm].file}`;
  }
  
  return fallbackPath ? fallbackPath : null;
}

/**
 * Tente de trouver un logo local pour la marque.
 */
export function getBrandLogoUrl(brandName, fallbackPath = null) {
  if (!brandName || brandName === 'Mod') {
    return fallbackPath ? fallbackPath : null;
  }
  return `/assets/manufacturers/${brandName.trim()}.png`;
}