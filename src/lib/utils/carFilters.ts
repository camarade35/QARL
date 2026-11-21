import { getCountryName } from '$lib/utils/assets.js';

export type CarFilterable = {
  id: string;
  name: string;
  brand: string;
  is_kunos: boolean;
  car_class: string;
  bhp: number;
  tags: string[];
};

type CarFilterOptions = {
  type: string;
  brand?: string;
  country?: string;
  carClass?: string;
  power?: string;
  tags?: string[];
  search?: string;
  favoriteIds?: string[];
  usedCars?: Record<string, number>;
};

export const CM_CATEGORIES = [
  { id: 'Classic', keywords: ['classic', 'historic', 'old'] },
  { id: 'Drift', keywords: ['drift'] },
  { id: 'Open-wheel', keywords: ['open-wheel', 'openwheel', 'formula', 'f1'] },
  { id: 'Prototype', keywords: ['prototype', 'lmp', 'hypercar'] },
  { id: 'Race', keywords: ['race', 'gt3', 'gt4', 'gt2', 'touring'] },
  { id: 'Rally', keywords: ['rally'] },
  { id: 'Sportscars', keywords: ['sportscar', 'sports car', 'supercar'] },
  { id: 'Street', keywords: ['street', 'road', 'production'] }
];

export function getCarCMCategory(car: Pick<CarFilterable, 'car_class' | 'tags'>) {
  const carClass = (car.car_class || '').toLowerCase();
  const carTags = (car.tags || []).map(tag => tag.toLowerCase());
  const values = [carClass, ...carTags];

  for (const category of CM_CATEGORIES) {
    if (category.keywords.some(keyword => values.some(value => value.includes(keyword)))) {
      return category.id;
    }
  }
  return null;
}

export function matchesCarFilters(car: CarFilterable, filters: CarFilterOptions) {
  if (filters.type === 'kunos' && !car.is_kunos) return false;
  if (filters.type === 'mods' && car.is_kunos) return false;
  if (filters.type === 'favorites' && !filters.favoriteIds?.includes(car.id)) return false;
  if (filters.type === 'used' && !filters.usedCars?.[car.id]) return false;

  if (filters.brand && car.brand !== filters.brand) return false;
  if (filters.country && getCountryName(car.tags) !== filters.country) return false;
  if (filters.carClass && getCarCMCategory(car) !== filters.carClass) return false;

  if (filters.power) {
    if (filters.power === 'under_200' && car.bhp >= 200) return false;
    if (filters.power === '200_400' && (car.bhp < 200 || car.bhp > 400)) return false;
    if (filters.power === '400_600' && (car.bhp < 400 || car.bhp > 600)) return false;
    if (filters.power === 'over_600' && car.bhp <= 600) return false;
  }

  if (filters.tags?.length && !filters.tags.every(tag => car.tags?.includes(tag))) return false;
  if (filters.search && !car.name.toLowerCase().includes(filters.search.toLowerCase())) return false;
  return true;
}
