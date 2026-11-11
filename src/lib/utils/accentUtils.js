/**
 * Utilitaires centralisés pour la couleur d'accentuation et le fond.
 */

export const DEFAULT_ACCENT = '#319dd3';
export const DEFAULT_INTENSITY = 100;
export const DEFAULT_THEME = 'dark';

export function hexToRgb(hex) {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex);
  return result
    ? {
        r: parseInt(result[1], 16),
        g: parseInt(result[2], 16),
        b: parseInt(result[3], 16),
      }
    : { r: 49, g: 157, b: 211 };
}

export function rgbToHex(r, g, b) {
  const clamp = (v) => Math.max(0, Math.min(255, Math.round(v)));
  const toHex = (v) => clamp(v).toString(16).padStart(2, '0');
  return `#${toHex(r)}${toHex(g)}${toHex(b)}`;
}

function isLightTheme(theme) {
  return theme === 'light' || theme === 'office';
}

export function applyAccentColor(color, theme, intensity) {
  if (typeof document === 'undefined') return;

  if (!color || !/^#[0-9a-f]{6}$/i.test(color)) color = DEFAULT_ACCENT;
  theme = theme || DEFAULT_THEME;
  if (typeof intensity !== 'number' || isNaN(intensity)) intensity = DEFAULT_INTENSITY;
  intensity = Math.max(0, Math.min(100, intensity));

  const { r, g, b } = hexToRgb(color);

  // 1. Couleur principale
  document.documentElement.style.setProperty('--accent', color.toLowerCase());
  document.documentElement.style.setProperty('--accent-rgb', `${r}, ${g}, ${b}`);

  // 2. Hover
  const shift = isLightTheme(theme) ? -20 : 20;
  document.documentElement.style.setProperty('--accent-hover', rgbToHex(r + shift, g + shift, b + shift));

  // 3. Intensité
  const factor = intensity / 100;
  document.documentElement.style.setProperty('--accent-muted', `rgba(${r}, ${g}, ${b}, ${(0.12 * factor).toFixed(3)})`);
  document.documentElement.style.setProperty('--accent-glow', `rgba(${r}, ${g}, ${b}, ${(0.40 * factor).toFixed(3)})`);
}

export function loadAccentState() {
  if (typeof localStorage === 'undefined') {
    return { color: DEFAULT_ACCENT, intensity: DEFAULT_INTENSITY, theme: DEFAULT_THEME };
  }

  const theme = localStorage.getItem('qarl-theme') || localStorage.getItem('qarl_theme') || DEFAULT_THEME;
  const color = localStorage.getItem('qarl_accent') || DEFAULT_ACCENT;
  const rawIntensity = parseInt(localStorage.getItem('qarl_accent_intensity') || '', 10);
  const intensity = isNaN(rawIntensity) ? DEFAULT_INTENSITY : Math.max(0, Math.min(100, rawIntensity));

  return { color, intensity, theme };
}

// ⭐ NOUVEAU : Gestion du fond d'écran personnalisé (Nuances de gris)

export const GRAYSCALE_SHADES = [
  { id: 'auto', hex: 'auto', label: 'sidebar.bg_shade_auto' },
  { id: 'g1', hex: '#0a0a0a', label: 'sidebar.bg_shade_g1' },
  { id: 'g2', hex: '#1a1a1a', label: 'sidebar.bg_shade_g2' },
  { id: 'g3', hex: '#2b2b2b', label: 'sidebar.bg_shade_g3' },
  { id: 'g4', hex: '#404040', label: 'sidebar.bg_shade_g4' },
  { id: 'g5', hex: '#595959', label: 'sidebar.bg_shade_g5' },
  { id: 'g6', hex: '#737373', label: 'sidebar.bg_shade_g6' },
  { id: 'g7', hex: '#999999', label: 'sidebar.bg_shade_g7' },
  { id: 'g8', hex: '#c0c0c0', label: 'sidebar.bg_shade_g8' },
  { id: 'g9', hex: '#e6e6e6', label: 'sidebar.bg_shade_g9' },
  { id: 'g10', hex: '#ffffff', label: 'sidebar.bg_shade_g10' }
];

export function applyBackgroundShade(shadeId) {
  if (typeof document === 'undefined') return;
  const root = document.documentElement;

  if (!shadeId || shadeId === 'auto') {
    ['--bg-root', '--bg-panel', '--bg-elevated', '--bg-hover', 
     '--text-primary', '--text-secondary', '--text-tertiary', 
     '--border-subtle', '--border', '--border-strong'].forEach(v => root.style.removeProperty(v));
    return;
  }

  const shadeObj = GRAYSCALE_SHADES.find(s => s.id === shadeId);
  if (!shadeObj || shadeObj.hex === 'auto') return;

  const { r, g, b } = hexToRgb(shadeObj.hex);
  const luminance = (0.299 * r + 0.587 * g + 0.114 * b) / 255;
  const isLight = luminance > 0.5;

  root.style.setProperty('--bg-root', shadeObj.hex);

  const shift = isLight ? -20 : 20;
  root.style.setProperty('--bg-panel', rgbToHex(r + shift, g + shift, b + shift));
  root.style.setProperty('--bg-elevated', rgbToHex(r + shift * 1.5, g + shift * 1.5, b + shift * 1.5));
  root.style.setProperty('--bg-hover', rgbToHex(r + shift * 2, g + shift * 2, b + shift * 2));

  if (isLight) {
    root.style.setProperty('--text-primary', '#1a1d24');
    root.style.setProperty('--text-secondary', '#4a5165');
    root.style.setProperty('--text-tertiary', '#8087a0');
    root.style.setProperty('--border-subtle', 'rgba(0, 0, 0, 0.06)');
    root.style.setProperty('--border', 'rgba(0, 0, 0, 0.10)');
    root.style.setProperty('--border-strong', 'rgba(0, 0, 0, 0.18)');
  } else {
    root.style.setProperty('--text-primary', '#eef0f6');
    root.style.setProperty('--text-secondary', '#98a0b5');
    root.style.setProperty('--text-tertiary', '#656b82');
    root.style.setProperty('--border-subtle', 'rgba(255, 255, 255, 0.06)');
    root.style.setProperty('--border', 'rgba(255, 255, 255, 0.10)');
    root.style.setProperty('--border-strong', 'rgba(255, 255, 255, 0.16)');
  }
}