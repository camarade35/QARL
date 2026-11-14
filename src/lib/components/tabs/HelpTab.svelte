<script lang="ts">
  import { t } from 'svelte-i18n';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { 
    Rocket, HelpCircle, Link as LinkIcon, 
    Car, Settings, ChevronDown, ExternalLink,
    Flag, Play, MonitorPlay
  } from 'lucide-svelte';

  // État pour gérer l'ouverture des accordéons (FAQ)
  let openFaq = $state<Record<number, boolean>>({});

  function toggleFaq(index: number) {
    openFaq[index] = !openFaq[index];
  }

  // Fonction pour changer d'onglet depuis l'aide
  function goToTab(tabId: string) {
    appState.activeTab = tabId;
    if (tabId === 'tab-settings') {
      window.dispatchEvent(new CustomEvent('open-general-tab'));
    }
  }

  // ⭐ LOGIQUE DU GUIDE VISUEL
  let activeModal = $state<number | null>(null);
  let slideIndex = $state(0);
  let isZoomed = $state(false);

  const HELP_DATA = [
    {
      icon: '🖥️',
      titleKey: 'settings.help.guide.cards.sidebar_title',
      descKey: 'settings.help.guide.cards.sidebar_desc',
      targetTab: 'tab-cars',
      slides: [
        { img: '/assets/help/sidebar/sidebar_complete.png', textKey: 'settings.help.guide.slides.sidebar_1_complete' },
        { img: '/assets/help/sidebar/sidebar_theme_settings.png', textKey: 'settings.help.guide.slides.sidebar_2_theme' },
        { img: '/assets/help/sidebar/sidebar_color.png', textKey: 'settings.help.guide.slides.sidebar_3_color' },
        { img: '/assets/help/sidebar/sidebar_cars_tracks.png', textKey: 'settings.help.guide.slides.sidebar_4_cards' },
        { img: '/assets/help/sidebar/sidebar_session_weather_start.png', textKey: 'settings.help.guide.slides.sidebar_5_session' }
      ]
    },
    {
      icon: '🏎️',
      titleKey: 'settings.help.guide.cards.cars_title',
      descKey: 'settings.help.guide.cards.cars_desc',
      targetTab: 'tab-cars',
      slides: [
        { img: '/assets/help/cars/filters.png', textKey: 'settings.help.guide.slides.cars_filters' },
        { img: '/assets/help/cars/grid.png', textKey: 'settings.help.guide.slides.cars_grid' },
        { img: '/assets/help/cars/skins_modal.png', textKey: 'settings.help.guide.slides.cars_skins' },
        { img: '/assets/help/cars/folder.png', textKey: 'settings.help.guide.slides.cars_folder' }
      ]
    },
    {
      icon: '🏁',
      titleKey: 'settings.help.guide.cards.tracks_title',
      descKey: 'settings.help.guide.cards.tracks_desc',
      targetTab: 'tab-tracks',
      slides: [
        { img: '/assets/help/tracks/filters.png', textKey: 'settings.help.guide.slides.tracks_filters' },
        { img: '/assets/help/tracks/layouts.png', textKey: 'settings.help.guide.slides.tracks_layouts' },
        { img: '/assets/help/tracks/folder.png', textKey: 'settings.help.guide.slides.tracks_folder' }
      ]
    },
    {
      icon: '⏱️',
      titleKey: 'settings.help.guide.cards.session_title',
      descKey: 'settings.help.guide.cards.session_desc',
      targetTab: 'tab-session',
      slides: [
        { img: '/assets/help/session/1_tabs.png', textKey: 'settings.help.guide.slides.session_tabs' },
        { img: '/assets/help/session/2_setup_practice.png', textKey: 'settings.help.guide.slides.session_setup_practice' },
        { img: '/assets/help/session/3_setup_race.png', textKey: 'settings.help.guide.slides.session_setup_race' },
        { img: '/assets/help/session/4_setup_hotlap.png', textKey: 'settings.help.guide.slides.session_setup_hotlap' },
        { img: '/assets/help/session/5_race_ia.png', textKey: 'settings.help.guide.slides.session_race_ia' },
        { img: '/assets/help/session/6_ai_grid.png', textKey: 'settings.help.guide.slides.session_ai_grid' },
        { img: '/assets/help/session/7_ai_pos.png', textKey: 'settings.help.guide.slides.session_ai_pos' },
        { img: '/assets/help/session/8_ai_penalty.png', textKey: 'settings.help.guide.slides.session_ai_penalty' },
        { img: '/assets/help/session/9_pilot.png', textKey: 'settings.help.guide.slides.session_pilot' },
        { img: '/assets/help/session/10_pilot_limits.png', textKey: 'settings.help.guide.slides.session_pilot_limits' },
        { img: '/assets/help/session/11_pilot_grip.png', textKey: 'settings.help.guide.slides.session_pilot_grip' },
        { img: '/assets/help/session/12_presets.png', textKey: 'settings.help.guide.slides.session_presets' },
        { img: '/assets/help/session/13_history.png', textKey: 'settings.help.guide.slides.session_history' }
      ]
    },
    {
      icon: '🌤️',
      titleKey: 'settings.help.guide.cards.weather_title',
      descKey: 'settings.help.guide.cards.weather_desc',
      targetTab: 'tab-weather',
      slides: [
        { img: '/assets/help/weather/1_static.png', textKey: 'settings.help.guide.slides.weather_static' },
        { img: '/assets/help/weather/2_schedule.png', textKey: 'settings.help.guide.slides.weather_schedule' },
        { img: '/assets/help/weather/3_pure_plans.png', textKey: 'settings.help.guide.slides.weather_pure_plans' },
        { img: '/assets/help/weather/4_custom_plan.png', textKey: 'settings.help.guide.slides.weather_custom_plan' }
      ]
    },
    {
      icon: '📦',
      titleKey: 'settings.help.guide.cards.mods_title',
      descKey: 'settings.help.guide.cards.mods_desc',
      targetTab: 'tab-mods',
      slides: [
        { img: '/assets/help/mods/mods_1_overview.png', textKey: 'settings.help.guide.slides.mods_1_overview' },
        { img: '/assets/help/mods/mods_2_journal_inside.png', textKey: 'settings.help.guide.slides.mods_2_journal_inside' },
        { img: '/assets/help/mods/mods_3_drag_active.png', textKey: 'settings.help.guide.slides.mods_3_drag_active' },
        { img: '/assets/help/mods/mods_4_progress_1.png', textKey: 'settings.help.guide.slides.mods_4_progress_1' },
        { img: '/assets/help/mods/mods_5_progress_2.png', textKey: 'settings.help.guide.slides.mods_5_progress_2' },
        { img: '/assets/help/mods/mods_6_progress_success.png', textKey: 'settings.help.guide.slides.mods_6_progress_success' }
      ]
    },
    {
      icon: '🌐',
      titleKey: 'settings.help.guide.cards.online_title',
      descKey: 'settings.help.guide.cards.online_desc',
      targetTab: 'tab-online',
      slides: [
        { img: '/assets/help/online/1_general.png', textKey: 'settings.help.guide.slides.online_1_general' },
        { img: '/assets/help/online/2_manual.png', textKey: 'settings.help.guide.slides.online_2_manual' },
        { img: '/assets/help/online/3_history.png', textKey: 'settings.help.guide.slides.online_3_history' },
        { img: '/assets/help/online/4_public.png', textKey: 'settings.help.guide.slides.online_4_public' },
        { img: '/assets/help/online/5_modal.png', textKey: 'settings.help.guide.slides.online_5_modal' },
        { img: '/assets/help/online/6_skin.png', textKey: 'settings.help.guide.slides.online_6_skin' }
      ]
    },
    {
      icon: '📸',
      titleKey: 'settings.help.guide.cards.media_title',
      descKey: 'settings.help.guide.cards.media_desc',
      targetTab: 'tab-media',
      slides: [
        { img: '/assets/help/media/photo_1_general.png', textKey: 'settings.help.guide.slides.media_photo_general' },
        { img: '/assets/help/media/photo_2_settings.png', textKey: 'settings.help.guide.slides.media_photo_settings' },
        { img: '/assets/help/media/photo_3_lightbox.png', textKey: 'settings.help.guide.slides.media_photo_lightbox' },
        { img: '/assets/help/media/photo_4_collapsed.png', textKey: 'settings.help.guide.slides.media_photo_collapsed' },
        { img: '/assets/help/media/replay_1_general.png', textKey: 'settings.help.guide.slides.media_replay_general' },
        { img: '/assets/help/media/replay_2_settings.png', textKey: 'settings.help.guide.slides.media_replay_settings' },
        { img: '/assets/help/media/replay_3_selection.png', textKey: 'settings.help.guide.slides.media_replay_selection' },
        { img: '/assets/help/media/replay_4_collapsed.png', textKey: 'settings.help.guide.slides.media_replay_collapsed' }
      ]
    },
    {
      icon: '⚙️',
      titleKey: 'settings.help.guide.cards.settings_title',
      descKey: 'settings.help.guide.cards.settings_desc',
      targetTab: 'tab-settings',
      slides: [
        { img: '/assets/help/settings/settings_1_general.png', textKey: 'settings.help.guide.slides.settings_1_general' },
        { img: '/assets/help/settings/settings_2_general_actions.png', textKey: 'settings.help.guide.slides.settings_2_general_actions' },
        { img: '/assets/help/settings/settings_3_controls_cockpit.png', textKey: 'settings.help.guide.slides.settings_3_controls_cockpit' },
        { img: '/assets/help/settings/settings_4_controls_advanced.png', textKey: 'settings.help.guide.slides.settings_4_controls_advanced' },
        { img: '/assets/help/settings/settings_5_video.png', textKey: 'settings.help.guide.slides.settings_5_video' },
        { img: '/assets/help/settings/settings_6_audio.png', textKey: 'settings.help.guide.slides.settings_6_audio' },
        { img: '/assets/help/settings/settings_7_csp.png', textKey: 'settings.help.guide.slides.settings_7_csp' },
        { img: '/assets/help/settings/settings_8_help.png', textKey: 'settings.help.guide.slides.settings_8_help' }
      ]
    }
  ];

  function openModal(index: number) {
    activeModal = index;
    slideIndex = 0;
    isZoomed = false;
  }

  function nextSlide() {
    if (activeModal === null) return;
    if (slideIndex < HELP_DATA[activeModal].slides.length - 1) slideIndex++;
  }

  function prevSlide() {
    if (slideIndex > 0) slideIndex--;
  }
</script>

<div class="group">
  <div class="group-title" style="display: flex; align-items: center; gap: 8px;">
    <HelpCircle size={20} color="var(--accent)" />
    {$t('settings.help.title')}
  </div>
  <p style="color: var(--text-secondary); margin-bottom: 20px; font-size: 0.9rem;">
    {$t('settings.help.intro')}
  </p>

  <!-- 🚀 DÉMARRAGE RAPIDE -->
  <div class="help-section-title">
    <Rocket size={18} /> {$t('settings.help.quick_start.title')}
  </div>
  <div class="quick-grid">
    <div class="quick-card">
      <div class="step-num">1</div>
      <div class="step-title">{$t('settings.help.quick_start.step1_title')}</div>
      <div class="step-desc">{$t('settings.help.quick_start.step1_desc')}</div>
      <button class="btn-secondary mini-btn" onclick={() => goToTab('tab-settings')}>
        <Settings size={14} /> {$t('settings.help.quick_start.go_settings')}
      </button>
    </div>
    <div class="quick-card">
      <div class="step-num">2</div>
      <div class="step-title">{$t('settings.help.quick_start.step2_title')}</div>
      <div class="step-desc">{$t('settings.help.quick_start.step2_desc')}</div>
      <button class="btn-secondary mini-btn" onclick={() => goToTab('tab-cars')}>
        <Car size={14} /> {$t('settings.help.quick_start.go_cars')}
      </button>
    </div>
    <div class="quick-card">
      <div class="step-num">3</div>
      <div class="step-title">{$t('settings.help.quick_start.step3_title')}</div>
      <div class="step-desc">{$t('settings.help.quick_start.step3_desc')}</div>
      <button class="btn-secondary mini-btn" onclick={() => goToTab('tab-tracks')}>
        <Flag size={14} /> {$t('settings.help.quick_start.go_tracks')}
      </button>
    </div>
    <div class="quick-card">
      <div class="step-num">4</div>
      <div class="step-title">{$t('settings.help.quick_start.step4_title')}</div>
      <div class="step-desc">{$t('settings.help.quick_start.step4_desc')}</div>
      <button class="btn-secondary mini-btn" onclick={() => goToTab('tab-session')}>
        <Play size={14} /> {$t('settings.help.quick_start.go_session')}
      </button>
    </div>
  </div>

  <!-- 🖼️ GUIDE VISUEL -->
  <div class="help-section-title" style="margin-top: 30px;">
    <MonitorPlay size={18} /> {$t('settings.help.guide.title')}
  </div>
  
  <div class="help-grid">
    {#each HELP_DATA as item, index}
      <div class="help-card" onclick={() => openModal(index)}>
        <div class="help-card-icon">{item.icon}</div>
        <div class="help-card-title">{$t(item.titleKey)}</div>
        <div class="help-card-desc">{$t(item.descKey)}</div>
        <div class="help-card-badge">{$t('settings.search.go')} →</div>
      </div>
    {/each}
  </div>

  <!-- ❓ FAQ -->
  <div class="help-section-title" style="margin-top: 30px;">
    <HelpCircle size={18} /> {$t('settings.help.faq.title')}
  </div>
  <div class="faq-list">
    <!-- FAQ 1 -->
    <div class="faq-item" class:open={openFaq[0]}>
      <button class="faq-question" onclick={() => toggleFaq(0)}>
        {$t('settings.help.faq.q_steering')}
        <ChevronDown size={16} class="faq-chevron" />
      </button>
      <div class="faq-answer">{$t('settings.help.faq.a_steering')}</div>
    </div>
    <!-- FAQ 2 -->
    <div class="faq-item" class:open={openFaq[1]}>
      <button class="faq-question" onclick={() => toggleFaq(1)}>
        {$t('settings.help.faq.q_files')}
        <ChevronDown size={16} class="faq-chevron" />
      </button>
      <div class="faq-answer">{$t('settings.help.faq.a_files')}</div>
    </div>
    <!-- FAQ 3 -->
    <div class="faq-item" class:open={openFaq[2]}>
      <button class="faq-question" onclick={() => toggleFaq(2)}>
        {$t('settings.help.faq.q_pure')}
        <ChevronDown size={16} class="faq-chevron" />
      </button>
      <div class="faq-answer">{$t('settings.help.faq.a_pure')}</div>
    </div>
    <!-- FAQ 4 -->
    <div class="faq-item" class:open={openFaq[3]}>
      <button class="faq-question" onclick={() => toggleFaq(3)}>
        {$t('settings.help.faq.q_steam')}
        <ChevronDown size={16} class="faq-chevron" />
      </button>
      <div class="faq-answer">{$t('settings.help.faq.a_steam')}</div>
    </div>
    <!-- FAQ 5 (CSP) -->
    <div class="faq-item" class:open={openFaq[4]}>
      <button class="faq-question" onclick={() => toggleFaq(4)}>
        {$t('settings.help.faq.q_csp')}
        <ChevronDown size={16} class="faq-chevron" />
      </button>
      <div class="faq-answer">{$t('settings.help.faq.a_csp')}</div>
    </div>
    <!-- FAQ 6 (CSP QARL) -->
    <div class="faq-item" class:open={openFaq[5]}>
      <button class="faq-question" onclick={() => toggleFaq(5)}>
        {$t('settings.help.faq.q_csp_qarl')}
        <ChevronDown size={16} class="faq-chevron" />
      </button>
      <div class="faq-answer">{$t('settings.help.faq.a_csp_qarl')}</div>
    </div>
  </div>

  <!-- 🌐 LIENS UTILES -->
  <div class="help-section-title" style="margin-top: 30px;">
    <LinkIcon size={18} /> {$t('settings.help.links.title')}
  </div>
  <div class="links-grid">
    <button class="filter-btn link-btn" onclick={() => openUrl('https://acstuff.ru/app/')}>
      <ExternalLink size={14} /> Content Manager
    </button>
    <button class="filter-btn link-btn" onclick={() => openUrl('https://www.patreon.com/pure')}>
      <ExternalLink size={14} /> Pure
    </button>
    <button class="filter-btn link-btn" onclick={() => openUrl('https://acstuff.ru/patch/')}>
      <ExternalLink size={14} /> CSP
    </button>
    <button class="filter-btn link-btn" onclick={() => openUrl('https://www.overtake.gg/')}>
      <ExternalLink size={14} /> Overtake.gg
    </button>
    <button class="filter-btn link-btn" onclick={() => openUrl('https://discord.gg/assettocorsa')}>
      <ExternalLink size={14} /> Discord AC
    </button>
    <button class="filter-btn link-btn" onclick={() => openUrl('https://www.assettocorsa.it/')}>
      <ExternalLink size={14} /> Kunos
    </button>
  </div>
</div>

<!-- ⭐ MODALE DU GUIDE VISUEL -->
{#if activeModal !== null}
  {@const item = HELP_DATA[activeModal]}
  {@const slide = item.slides[slideIndex]}
  
  <div class="guide-overlay" onclick={(e) => { if (e.target === e.currentTarget) activeModal = null; }}>
    <div class="guide-modal" class:zoomed={isZoomed}>
      
      <!-- ⭐ Header avec Titre et bouton Fermer -->
      <div class="guide-header">
        <h3>{$t(item.titleKey)}</h3>
        <button class="guide-close" onclick={() => activeModal = null}>✕</button>
      </div>
      
      <div class="guide-image-wrapper">
        <div class="guide-image-container" onclick={() => isZoomed = !isZoomed} title={isZoomed ? "Réduire" : "Agrandir"}>
          <img src={slide.img} alt={$t(item.titleKey)} class:zoomed-img={isZoomed} />
        </div>
        
        <!-- ⭐ Pagination par points -->
        {#if item.slides.length > 1}
          <div class="guide-dots">
            {#each item.slides as _, i}
              <button 
                class="guide-dot" 
                class:active={i === slideIndex} 
                onclick={(e) => { e.stopPropagation(); slideIndex = i; }}
                title="Slide {i + 1}"
              ></button>
            {/each}
          </div>
        {/if}
      </div>
      
      <div class="guide-text-container">
        <p>{$t(slide.textKey)}</p>
        {#if item.targetTab}
          <button class="btn-secondary mini-btn" style="margin: 15px auto 0;" onclick={() => { goToTab(item.targetTab); activeModal = null; }}>
            <LinkIcon size={14} /> {$t('settings.help.guide.open_tab')}
          </button>
        {/if}
      </div>
    </div>
  </div>
{/if}

<style>
  /* ⭐ Styles pour la grille et la modale du guide */
  .help-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(240px, 1fr));
    gap: 16px;
    margin-bottom: 20px;
  }
  .help-card {
    background: var(--bg-panel);
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    padding: 20px;
    cursor: pointer;
    transition: all 0.3s ease;
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .help-card:hover {
    transform: translateY(-4px);
    border-color: var(--accent);
    box-shadow: 0 8px 24px var(--accent-muted);
  }
  .help-card-icon { font-size: 2rem; }
  .help-card-title { font-weight: 700; font-size: 1rem; color: var(--text-primary); }
  .help-card-desc { font-size: 0.85rem; color: var(--text-secondary); line-height: 1.4; flex-grow: 1; }
  .help-card-badge { font-size: 0.75rem; color: var(--accent); font-weight: 700; }

  /* Modale */
  .guide-overlay {
    position: fixed; top: 0; left: 0; width: 100%; height: 100%;
    background: rgba(0, 0, 0, 0.85); backdrop-filter: blur(4px);
    display: flex; justify-content: center; align-items: center; z-index: 9999;
  }
  
  .guide-modal {
    background: var(--bg-panel);
    border: 1px solid var(--border-subtle);
    border-radius: 12px;
    width: 90%; max-width: 900px;
    box-shadow: 0 20px 50px rgba(0,0,0,0.5);
    overflow: hidden;
    display: flex; flex-direction: column;
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  }
  
  .guide-modal.zoomed {
    width: 95vw;
    max-width: 95vw;
    height: 95vh;
    max-height: 95vh;
  }

  .guide-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 15px 20px;
    background: var(--bg-panel);
    border-bottom: 1px solid var(--border-subtle);
  }
  
  .guide-header h3 {
    margin: 0;
    font-size: 1.1rem;
    color: var(--accent);
    font-weight: 700;
  }

  .guide-close {
    background: transparent; border: none; color: var(--text-secondary);
    width: 32px; height: 32px; border-radius: 50%;
    font-size: 1.2rem; cursor: pointer;
    display: flex; align-items: center; justify-content: center;
    transition: all 0.2s;
  }
  .guide-close:hover { background: var(--accent-muted); color: var(--accent); }
  
  .guide-image-wrapper {
    display: flex;
    flex-direction: column;
    background: #000;
    position: relative;
  }

  .guide-image-container {
    position: relative;
    width: 100%;
    max-height: 60vh;
    min-height: 300px;
    display: flex;
    justify-content: center;
    align-items: center;
    cursor: zoom-in;
    transition: max-height 0.3s ease;
  }
  
  .guide-modal.zoomed .guide-image-container {
    max-height: calc(95vh - 150px); /* Laisse la place pour le header et les dots */
    flex-grow: 1;
    cursor: zoom-out;
  }
  
  .guide-image-container img {
    max-width: 100%;
    max-height: 60vh;
    object-fit: contain;
    display: block;
    transition: max-height 0.3s ease;
  }

  .guide-modal.zoomed .guide-image-container img {
    max-height: calc(95vh - 160px);
  }
  
  /* ⭐ Pagination points */
  .guide-dots {
    display: flex;
    justify-content: center;
    gap: 8px;
    padding: 12px 0;
    background: #000;
  }
  
  .guide-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: rgba(255,255,255,0.3);
    border: none;
    cursor: pointer;
    padding: 0;
    transition: all 0.2s;
  }
  .guide-dot:hover {
    background: rgba(255,255,255,0.6);
  }
  .guide-dot.active {
    background: var(--accent);
    transform: scale(1.2);
    box-shadow: 0 0 8px var(--accent-muted);
  }

  .guide-text-container {
    padding: 20px 30px;
    text-align: center;
    background: var(--bg-elevated);
    border-top: 1px solid var(--border-subtle);
  }
  .guide-text-container p {
    color: var(--text-secondary); font-size: 0.95rem; line-height: 1.5;
    margin: 0;
  }

  /* ... (Le reste de tes styles existants pour FAQ, liens, etc.) ... */
  /* Titres de section avec effet lumineux */
  .help-section-title {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 1rem;
    font-weight: 700;
    color: var(--text-primary);
    margin-bottom: 16px;
    padding-bottom: 8px;
    border-bottom: 1px solid var(--border-subtle);
    position: relative;
  }
  .help-section-title::after {
    content: '';
    position: absolute;
    bottom: -1px;
    left: 0;
    width: 50px;
    height: 2px;
    background: var(--accent);
    box-shadow: 0 0 8px var(--accent-glow);
    border-radius: 2px;
  }
  .help-section-title svg {
    color: var(--accent);
    filter: drop-shadow(0 0 5px var(--accent-muted));
  }

  /* Démarrage rapide - Effet de carte flottante */
  .quick-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    padding-top: 10px; /* Pour laisser de la place au badge numéro */
  }
  .quick-card {
    background: linear-gradient(145deg, var(--bg-panel), var(--bg-elevated));
    border: 1px solid var(--border-subtle);
    border-radius: 10px;
    padding: 20px 16px 16px;
    position: relative;
    display: flex;
    flex-direction: column;
    gap: 8px;
    transition: all 0.3s cubic-bezier(0.25, 0.8, 0.25, 1);
  }
  .quick-card:hover {
    transform: translateY(-4px);
    border-color: var(--accent);
    box-shadow: 0 8px 24px var(--accent-muted), inset 0 0 20px rgba(255,255,255,0.02);
  }
  .step-num {
    position: absolute;
    top: -12px;
    left: 16px;
    background: var(--bg-root);
    color: var(--accent);
    border: 2px solid var(--accent);
    box-shadow: 0 0 12px var(--accent-glow);
    width: 28px;
    height: 28px;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.9rem;
    font-weight: 800;
    z-index: 2;
  }
  .step-title {
    font-weight: 700;
    font-size: 0.95rem;
    color: var(--text-primary);
    margin-top: 4px;
  }
  .step-desc {
    font-size: 0.8rem;
    color: var(--text-secondary);
    flex-grow: 1;
    line-height: 1.4;
  }
  .mini-btn {
    padding: 6px 10px;
    font-size: 0.75rem;
    align-self: flex-start;
    margin-top: 8px;
    background: var(--bg-panel);
    border: 1px solid var(--border-subtle);
    color: var(--text-primary);
    border-radius: 6px;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .quick-card:hover .mini-btn {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-muted);
  }

  /* FAQ - Accordéons lumineux */
  .faq-list {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }
  .faq-item {
    background: var(--bg-panel);
    border: 1px solid var(--border-subtle);
    border-radius: 8px;
    overflow: hidden;
    transition: all 0.2s;
  }
  .faq-item:hover {
    border-color: rgba(255, 255, 255, 0.15);
  }
  .faq-item.open {
    border-color: var(--accent);
    box-shadow: 0 0 15px var(--accent-muted);
  }
  .faq-question {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 14px 16px;
    background: transparent;
    border: none;
    color: var(--text-primary);
    font-weight: 600;
    font-size: 0.85rem;
    cursor: pointer;
    text-align: left;
    transition: color 0.2s;
  }
  .faq-item.open .faq-question {
    color: var(--accent);
  }
  .faq-chevron {
    transition: transform 0.3s cubic-bezier(0.4, 0, 0.2, 1);
    color: var(--text-tertiary);
  }
  .faq-item.open .faq-chevron {
    transform: rotate(180deg);
    color: var(--accent);
  }
  .faq-answer {
    max-height: 0;
    overflow: hidden;
    padding: 0 16px;
    color: var(--text-secondary);
    font-size: 0.82rem;
    line-height: 1.6;
    transition: all 0.3s ease-in-out;
    opacity: 0;
  }
  .faq-item.open .faq-answer {
    max-height: 200px;
    padding: 0 16px 16px 16px;
    opacity: 1;
  }

  /* Liens utiles - Boutons vivants */
  .links-grid {
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
  }
  .link-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 14px;
    font-size: 0.8rem;
    background: var(--bg-panel);
    border: 1px solid var(--border-subtle);
    border-radius: 6px;
    color: var(--text-primary);
    cursor: pointer;
    transition: all 0.2s;
  }
  .link-btn:hover {
    border-color: var(--accent);
    color: var(--accent);
    background: var(--accent-muted);
    box-shadow: 0 0 10px var(--accent-muted);
    transform: translateY(-2px);
  }
</style>