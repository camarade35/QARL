import { get } from 'svelte/store';
import { t } from 'svelte-i18n';

export const WEATHER_ICONS = {
  'clear': '☀️', 'mid_clear': '🌤️', 'light_clouds': '🌤️', 'few_clouds': '🌤️',
  'mid_clouds': '⛅', 'scattered_clouds': '⛅', 'heavy_clouds': '☁️', 'broken_clouds': '☁️',
  'overcast': '☁️', 'overcast_clouds': '☁️', 'light_fog': '🌫️', 'mid_fog': '🌫️',
  'heavy_fog': '🌁', 'fog': '🌁', 'mist': '🌫️', 'light_rain': '🌧️', 'rain': '🌧️',
  'heavy_rain': '⛈️', 'light_drizzle': '🌦️', 'drizzle': '🌦️', 'heavy_drizzle': '🌧️',
  'light_thunderstorm': '🌩️', 'thunderstorm': '⛈️', 'heavy_thunderstorm': '⛈️',
  'light_snow': '🌨️', 'snow': '❄️', 'heavy_snow': '❄️', 'light_sleet': '🌨️',
  'sleet': '🌨️', 'heavy_sleet': '🌨️', 'hail': '🧊', 'windy': '💨', 'squalls': '💨',
  'tornado': '🌪️', 'hurricane': '🌀', 'sand': '🏜️', 'dust': '🏜️', 'smoke': '💨',
  'haze': '🌫️', 'cold': '🥶', 'hot': '🥵', 'no_clouds': '☀️'
};

export function getWeatherIcon(id) {
  if (!id) return '🌡️';
  const cleanId = id.toLowerCase().replace(/^\d+_/, '').replace(/^(sol|pure)_\d*_?/, '');
  return WEATHER_ICONS[cleanId] || '🌡️';
}

export function translateWeatherName(id, defaultName, currentLocale = 'fr') {
  if (!id) return defaultName;
  const lowerId = id.toLowerCase();

  // 1. On extrait la clé de base (ex: "clear", "heavy_clouds")
  let key = null;
  const kunosMatch = lowerId.match(/^\d+_([a-z_]+)$/);
  const pureMatch = lowerId.match(/^pure_([a-z_]+)$/);
  const solMatch = lowerId.match(/^sol_\d+_([a-z_]+)$/);

  if (kunosMatch) key = kunosMatch[1];
  else if (pureMatch) key = pureMatch[1];
  else if (solMatch) key = solMatch[1];

  // 2. On cherche la traduction dynamique
  if (key) {
    // On passe currentLocale pour forcer la réactivité
    const translated = get(t)(`weather_names.${key}`, { locale: currentLocale });
    // Si la traduction existe (ce n'est pas juste la clé qui a été retournée)
    if (translated && translated !== `weather_names.${key}`) {
      return translated;
    }
  }

  // 3. Fallback : nettoyage basique du nom par défaut
  const clean = defaultName.replace(/_/g, ' ');
  return clean.charAt(0).toUpperCase() + clean.slice(1);
}