const FIRST_NAMES = ["Jean", "Pierre", "Marco", "Luca", "Max", "Lewis", "Charles", "Fernando", "Sebastian", "Kimi", "Mick", "Lando", "Carlos", "Sergio", "Valtteri", "Daniel", "Esteban", "Yuki", "Antonio", "Robert", "George", "Nicholas", "Alexander", "Pascal"];
const LAST_NAMES = ["Dupont", "Martin", "Rossi", "Bianchi", "Verstappen", "Hamilton", "Leclerc", "Alonso", "Vettel", "Raikkonen", "Schumacher", "Norris", "Sainz", "Perez", "Bottas", "Ricciardo", "Ocon", "Tsunoda", "Giovinazzi", "Kubica", "Russell", "Latifi", "Albon", "Wehrlein"];

function getRandomName() {
  const fn = FIRST_NAMES[Math.floor(Math.random() * FIRST_NAMES.length)];
  const ln = LAST_NAMES[Math.floor(Math.random() * LAST_NAMES.length)];
  return `${fn} ${ln}`;
}

export function generateGrid(playerCar, allCars, sessionSettings) {
  if (!playerCar || !allCars || allCars.length === 0) return [];

  const { opponentsCount, gridType } = sessionSettings;
  let candidates = [];
  
  // 1. On filtre les voitures selon la règle choisie
  if (gridType === 'SAME_CAR') {
    candidates = [playerCar];
  } 
  else if (gridType === 'RANDOM') {
    candidates = [...allCars];
  } 
  else if (gridType === 'SAME_CLASS') {
    candidates = allCars.filter(c => c.car_class === playerCar.car_class);
    // Si la classe est vide ou introuvable, on prend tout
    if (candidates.length <= 1) candidates = [...allCars]; 
  } 
  else if (gridType === 'SAME_BRAND') {
    candidates = allCars.filter(c => c.brand === playerCar.brand);
    if (candidates.length <= 1) candidates = [...allCars];
  }
  else if (gridType.includes('SIMILAR_POWER_')) {
    // Ex: SIMILAR_POWER_5 -> tolérance 5%
    const tolerance = parseInt(gridType.split('_').pop(), 10) / 100;
    const targetBhp = playerCar.bhp || 0;
    
    if (targetBhp > 0) {
      const min = targetBhp * (1 - tolerance);
      const max = targetBhp * (1 + tolerance);
      candidates = allCars.filter(c => c.bhp >= min && c.bhp <= max);
    } else {
      candidates = [...allCars];
    }
  }
  else if (gridType.includes('SIMILAR_PWRATIO_')) {
    // Ex: SIMILAR_PWRATIO_5 -> tolérance 5%
    const tolerance = parseInt(gridType.split('_').pop(), 10) / 100;
    const targetRatio = playerCar.pwratio || 0;
    
    if (targetRatio > 0) {
      const min = targetRatio * (1 - tolerance);
      const max = targetRatio * (1 + tolerance);
      // On exclut les voitures sans ratio connu pour éviter les faux positifs à 0
      candidates = allCars.filter(c => c.pwratio > 0 && c.pwratio >= min && c.pwratio <= max);
    } else {
      candidates = [...allCars];
    }
  }
  else if (gridType === 'SAME_ERA_CLASS') {
    // On cherche la même classe, à +/- 10 ans
    const targetYear = playerCar.year || 0;
    if (targetYear > 0) {
      candidates = allCars.filter(c => {
        const sameClass = c.car_class === playerCar.car_class;
        const closeYear = c.year > 0 && Math.abs(c.year - targetYear) <= 10;
        return sameClass && closeYear;
      });
    } else {
      candidates = allCars.filter(c => c.car_class === playerCar.car_class);
    }
    if (candidates.length <= 1) candidates = [...allCars];
  }

  // 2. Sécurité : si le filtre ne renvoie AUCUNE voiture (ou juste le joueur)
  if (candidates.length === 0) candidates = [...allCars];

  // 3. On génère la grille finale
  const grid = [];
  for (let i = 0; i < opponentsCount; i++) {
    // On pioche au hasard dans les candidats
    grid.push(candidates[Math.floor(Math.random() * candidates.length)]);
  }

  // 4. On formate pour l'affichage
  return grid.map(car => {
    let model = car.name;
    if (car.brand && car.brand !== 'Mod' && model.toLowerCase().startsWith(car.brand.toLowerCase())) {
      model = model.substring(car.brand.length).trim();
    }

    return {
      brand: car.brand || null,
      model: model,
      driver: getRandomName(),
      id: car.id
    };
  });
}