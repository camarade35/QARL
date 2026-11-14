// src/lib/utils/cspModules.js

export const CATEGORIES = [
  { id: 'graphics',   label: 'csp.categories.graphics' },
  { id: 'camera',     label: 'csp.categories.camera' },
  { id: 'interface',  label: 'csp.categories.interface' },
  { id: 'simulation', label: 'csp.categories.simulation' },
  { id: 'system',     label: 'csp.categories.system' },
];

export const MODULES = [
  // GRAPHISMES & RENDU
  { id: 'grass_fx',              label: 'csp.modules.grass_fx.label',              category: 'graphics', description: 'csp.modules.grass_fx.desc' },
  { id: 'lighting_fx',           label: 'csp.modules.lighting_fx.label',           category: 'graphics', description: 'csp.modules.lighting_fx.desc', disableable: false },
  { id: 'particles_fx',          label: 'csp.modules.particles_fx.label',          category: 'graphics', description: 'csp.modules.particles_fx.desc' },
  { id: 'reflections_fx',        label: 'csp.modules.reflections_fx.label',        category: 'graphics', description: 'csp.modules.reflections_fx.desc', disableable: false },
  { id: 'extra_fx',              label: 'csp.modules.extra_fx.label',              category: 'graphics', description: 'csp.modules.extra_fx.desc' },
  { id: 'rain_fx',               label: 'csp.modules.rain_fx.label',               category: 'graphics', description: 'csp.modules.rain_fx.desc' },
  { id: 'skidmarks_fx',          label: 'csp.modules.skidmarks_fx.label',          category: 'graphics', description: 'csp.modules.skidmarks_fx.desc' },
  { id: 'tyres_fx',              label: 'csp.modules.tyres_fx.label',              category: 'graphics', description: 'csp.modules.tyres_fx.desc' },
  { id: 'windscreen_fx',         label: 'csp.modules.windscreen_fx.label',         category: 'graphics', description: 'csp.modules.windscreen_fx.desc' },
  { id: 'smart_shadows',         label: 'csp.modules.smart_shadows.label',         category: 'graphics', description: 'csp.modules.smart_shadows.desc' },
  { id: 'smart_mirror',          label: 'csp.modules.smart_mirror.label',          category: 'graphics', description: 'csp.modules.smart_mirror.desc', disableable: false },
  { id: 'graphics_adjustments',  label: 'csp.modules.graphics_adjustments.label',  category: 'graphics', description: 'csp.modules.graphics_adjustments.desc', disableable: false },
  { id: 'weather_fx',            label: 'csp.modules.weather_fx.label',            category: 'graphics', description: 'csp.modules.weather_fx.desc' },

  // CAMÉRA & VUE
  { id: 'neck',          label: 'csp.modules.neck.label',          category: 'camera', description: 'csp.modules.neck.desc' },
  { id: 'chaser_camera', label: 'csp.modules.chaser_camera.label', category: 'camera', description: 'csp.modules.chaser_camera.desc' },
  { id: 'freer_camera',  label: 'csp.modules.freer_camera.label',  category: 'camera', description: 'csp.modules.freer_camera.desc' },

  // INTERFACE
  { id: 'gui',              label: 'csp.modules.gui.label',              category: 'interface', description: 'csp.modules.gui.desc', disableable: false },
  { id: 'taskbar',          label: 'csp.modules.taskbar.label',          category: 'interface', description: 'csp.modules.taskbar.desc' },
  { id: 'splashscreen',     label: 'csp.modules.splashscreen.label',     category: 'interface', description: 'csp.modules.splashscreen.desc' },
  { id: 'chat_shortcuts',   label: 'csp.modules.chat_shortcuts.label',   category: 'interface', description: 'csp.modules.chat_shortcuts.desc' },
  { id: 'car_instruments',  label: 'csp.modules.car_instruments.label',  category: 'interface', description: 'csp.modules.car_instruments.desc', disableable: false },
  { id: 'nice_screenshots', label: 'csp.modules.nice_screenshots.label', category: 'interface', description: 'csp.modules.nice_screenshots.desc' },
  { id: 'music',            label: 'csp.modules.music.label',            category: 'interface', description: 'csp.modules.music.desc', disableable: false },

  // SIMULATION & GAMEPLAY
  { id: 'new_behaviour',     label: 'csp.modules.new_behaviour.label',     category: 'simulation', description: 'csp.modules.new_behaviour.desc' },
  { id: 'surfaces_fx',       label: 'csp.modules.surfaces_fx.label',       category: 'simulation', description: 'csp.modules.surfaces_fx.desc' },
  { id: 'track_adjustments', label: 'csp.modules.track_adjustments.label', category: 'simulation', description: 'csp.modules.track_adjustments.desc', disableable: false },
  { id: 'gamepad_fx',        label: 'csp.modules.gamepad_fx.label',        category: 'simulation', description: 'csp.modules.gamepad_fx.desc' },
  { id: 'ffb_tweaks',        label: 'csp.modules.ffb_tweaks.label',        category: 'simulation', description: 'csp.modules.ffb_tweaks.desc' },
  { id: 'g27_lights',        label: 'csp.modules.g27_lights.label',        category: 'simulation', description: 'csp.modules.g27_lights.desc' },

  // SYSTÈME & COMPAT
  { id: 'dxgi_tweaks',             label: 'csp.modules.dxgi_tweaks.label',             category: 'system', description: 'csp.modules.dxgi_tweaks.desc', disableable: false },
  { id: 'general',                 label: 'csp.modules.general.label',                 category: 'system', description: 'csp.modules.general.desc' },
  { id: 'small_tweaks',            label: 'csp.modules.small_tweaks.label',            category: 'system', description: 'csp.modules.small_tweaks.desc' },
  { id: 'triple_custom',           label: 'csp.modules.triple_custom.label',           category: 'system', description: 'csp.modules.triple_custom.desc' },
  { id: 'vr_tweaks',               label: 'csp.modules.vr_tweaks.label',               category: 'system', description: 'csp.modules.vr_tweaks.desc', disableable: false },
  { id: 'custom_rendering_modes',  label: 'csp.modules.custom_rendering_modes.label',  category: 'system', description: 'csp.modules.custom_rendering_modes.desc', disableable: false },
];