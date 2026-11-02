<script lang="ts">
  import { t } from 'svelte-i18n';
  import { appState } from '$lib/stores/appState.svelte.js';
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { 
    Rocket, BookOpen, HelpCircle, Link as LinkIcon, 
    Car, Map, Settings, CloudSun, 
    Package, Globe, ChevronDown, ExternalLink,
    Flag, Play, Palette, SunMoon, MonitorPlay
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

  <!-- 📖 COMPRENDRE LES ONGLETS -->
  <div class="help-section-title" style="margin-top: 30px;">
    <BookOpen size={18} /> {$t('settings.help.tabs_explanation.title')}
  </div>
  <div class="tabs-list">
    <div class="tab-help-item">
      <Car size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.tabs_explanation.cars_title')}</strong> - {$t('settings.help.tabs_explanation.cars')}
      </div>
    </div>
    <div class="tab-help-item">
      <Map size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.tabs_explanation.tracks_title')}</strong> - {$t('settings.help.tabs_explanation.tracks')}
      </div>
    </div>
    <div class="tab-help-item">
      <Settings size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.tabs_explanation.session_title')}</strong> - {$t('settings.help.tabs_explanation.session')}
      </div>
    </div>
    <div class="tab-help-item">
      <CloudSun size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.tabs_explanation.weather_title')}</strong> - {$t('settings.help.tabs_explanation.weather')}
      </div>
    </div>
    <div class="tab-help-item">
      <Package size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.tabs_explanation.mods_title')}</strong> - {$t('settings.help.tabs_explanation.mods')}
      </div>
    </div>
    <div class="tab-help-item">
      <Globe size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.tabs_explanation.online_title')}</strong> - {$t('settings.help.tabs_explanation.online')}
      </div>
    </div>
  </div>

  <!-- 🎨 INTERFACE & PERSONNALISATION -->
  <div class="help-section-title" style="margin-top: 30px;">
    <Palette size={18} /> {$t('settings.help.interface.title')}
  </div>
  <div class="tabs-list">
    <div class="tab-help-item">
      <SunMoon size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.interface.theme_title')}</strong> - {$t('settings.help.interface.theme')}
      </div>
    </div>
    <div class="tab-help-item">
      <Palette size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.interface.color_title')}</strong> - {$t('settings.help.interface.color')}
      </div>
    </div>
    <div class="tab-help-item">
      <MonitorPlay size={18} class="tab-help-icon" />
      <div class="tab-help-text">
        <strong>{$t('settings.help.interface.video_title')}</strong> - {$t('settings.help.interface.video')}
      </div>
    </div>
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

<style>
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

  /* Onglets - Effet de liste interactive */
  .tabs-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .tab-help-item {
    display: flex;
    align-items: center;
    gap: 14px;
    background: var(--bg-panel);
    padding: 12px 16px;
    border-radius: 6px;
    border-left: 3px solid transparent;
    transition: all 0.2s ease;
  }
  .tab-help-item:hover {
    background: var(--bg-elevated);
    border-left-color: var(--accent);
    transform: translateX(4px);
  }
  .tab-help-icon {
    color: var(--text-tertiary);
    flex-shrink: 0;
    transition: all 0.2s;
  }
  .tab-help-item:hover .tab-help-icon {
    color: var(--accent);
    filter: drop-shadow(0 0 6px var(--accent-glow));
  }
  .tab-help-text {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }
  .tab-help-text strong {
    color: var(--text-primary);
    font-weight: 700;
    margin-right: 4px;
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