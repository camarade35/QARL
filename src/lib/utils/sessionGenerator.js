const FIRST_NAMES = ["Jean", "Pierre", "Marco", "Luca", "Max", "Lewis", "Charles", "Fernando", "Sebastian", "Kimi", "Mick", "Lando", "Carlos", "Sergio", "Valtteri", "Daniel", "Esteban", "Yuki", "Antonio", "Robert", "George", "Nicholas", "Alexander", "Pascal"];
const LAST_NAMES = ["Dupont", "Martin", "Rossi", "Bianchi", "Verstappen", "Hamilton", "Leclerc", "Alonso", "Vettel", "Raikkonen", "Schumacher", "Norris", "Sainz", "Perez", "Bottas", "Ricciardo", "Ocon", "Tsunoda", "Giovinazzi", "Kubica", "Russell", "Latifi", "Albon", "Wehrlein"];

function getRandomName() {
  const fn = FIRST_NAMES[Math.floor(Math.random() * FIRST_NAMES.length)];
  const ln = LAST_NAMES[Math.floor(Math.random() * LAST_NAMES.length)];
  return `${fn} ${ln}`;
}

/**
 * @param {any} playerCar
 * @param {any[]} allCars
 * @param {any} sessionSettings
 * @returns {any[]}
 */
export function generateGrid(playerCar, allCars, sessionSettings) {
  // ⭐ Si ce n'est pas une course, on ne génère pas d'IA
  if (sessionSettings.type !== 'RACE_QUICK' && sessionSettings.type !== 'RACE_WEEKEND') return [];
  
  if (!playerCar || !allCars || allCars.length === 0) return [];

  // ⭐ RESTAURATION : Les variables de base
  const opponentsCount = sessionSettings.opponentsCount || 0;
  const gridType = sessionSettings.gridType || 'RANDOM';

  // ⭐ RESTAURATION : Mode Grille Personnalisée
  if (gridType === 'CUSTOM_GRID') {
    const grid = [];
    if (sessionSettings.customGrid) {
      for (const row of sessionSettings.customGrid) {
        const car = allCars.find(c => c.id === row.carId);
        if (car) {
          for (let i = 0; i < row.quantity; i++) {
            let finalSkin = row.skinId || '';
            
            // ⭐ Si la quantité > 1, on pioche une skin au hasard
            if (row.quantity > 1 && car.skins && car.skins.length > 0) {
              const randomSkin = car.skins[Math.floor(Math.random() * car.skins.length)];
              finalSkin = randomSkin.name;
            }

            grid.push({
              id: car.id,
              skin: finalSkin, 
              driver: row.driverName.trim() || 'Driver',
              brand: car.brand,
              model: car.name
            });
          }
        }
      }
    }
    return grid;
  }

  // ⭐ Gestion de la Restriction (Tout / Kunos / Mods)
  const gridRestriction = sessionSettings.gridRestriction || 'ALL';
  let baseCandidates = [...allCars];
  
  if (gridRestriction === 'KUNOS') {
    baseCandidates = allCars.filter(c => c.is_kunos);
  } else if (gridRestriction === 'MODS') {
    baseCandidates = allCars.filter(c => !c.is_kunos);
  }

  // Sécurité : si le filtre de restriction vide la liste, on remet tout
  if (baseCandidates.length === 0) baseCandidates = [...allCars];

  let candidates = [];
  
  // 1. On filtre les voitures selon la règle de cohérence choisie
  if (gridType === 'SAME_CAR') {
    candidates = [playerCar]; // Ignore la restriction, c'est la voiture du joueur
  } 
  else if (gridType === 'RANDOM') {
    candidates = baseCandidates; // Utilise la restriction
  } 
  else if (gridType === 'SAME_CLASS') {
    candidates = baseCandidates.filter(c => c.car_class === playerCar.car_class);
    if (candidates.length <= 1) candidates = baseCandidates; // Fallback sur la restriction
  } 
  else if (gridType === 'SAME_BRAND') {
    candidates = baseCandidates.filter(c => c.brand === playerCar.brand);
    if (candidates.length <= 1) candidates = baseCandidates;
  }
  else if (gridType.includes('SIMILAR_POWER_')) {
    const tolerance = parseInt(gridType.split('_').pop(), 10) / 100;
    const targetBhp = playerCar.bhp || 0;
    
    if (targetBhp > 0) {
      const min = targetBhp * (1 - tolerance);
      const max = targetBhp * (1 + tolerance);
      candidates = baseCandidates.filter(c => c.bhp >= min && c.bhp <= max);
    } else {
      candidates = baseCandidates;
    }
  }
  else if (gridType.includes('SIMILAR_PWRATIO_')) {
    const tolerance = parseInt(gridType.split('_').pop(), 10) / 100;
    const targetRatio = playerCar.pwratio || 0;
    
    if (targetRatio > 0) {
      const min = targetRatio * (1 - tolerance);
      const max = targetRatio * (1 + tolerance);
      candidates = baseCandidates.filter(c => c.pwratio > 0 && c.pwratio >= min && c.pwratio <= max);
    } else {
      candidates = baseCandidates;
    }
  }
  else if (gridType === 'SAME_ERA_CLASS') {
    const targetYear = playerCar.year || 0;
    if (targetYear > 0) {
      candidates = baseCandidates.filter(c => {
        const sameClass = c.car_class === playerCar.car_class;
        const closeYear = c.year > 0 && Math.abs(c.year - targetYear) <= 10;
        return sameClass && closeYear;
      });
    } else {
      candidates = baseCandidates.filter(c => c.car_class === playerCar.car_class);
    }
    if (candidates.length <= 1) candidates = baseCandidates;
  }

  // 2. Sécurité : si le filtre ne renvoie AUCUNE voiture
  if (candidates.length === 0) candidates = baseCandidates;

  // 3. On génère la grille finale
  const grid = [];
  for (let i = 0; i < opponentsCount; i++) {
    grid.push(candidates[Math.floor(Math.random() * candidates.length)]);
  }

  // 4. On formate pour l'affichage
  return grid.map(car => {
    let model = car.name;
    if (car.brand && car.brand !== 'Mod' && model.toLowerCase().startsWith(car.brand.toLowerCase())) {
      model = model.substring(car.brand.length).trim();
    }

    // ⭐ NOUVEAU : On récupère la première skin par défaut
    let defaultSkin = '';
    if (car.skins && car.skins.length > 0) {
      defaultSkin = car.skins[0].name;
    }

    return {
      brand: car.brand || null,
      model: model,
      driver: getRandomName(),
      id: car.id,
      skin: defaultSkin // ⭐ On ajoute le champ skin pour Rust
    };
  });
}