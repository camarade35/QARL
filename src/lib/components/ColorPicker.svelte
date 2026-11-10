<script lang="ts">
  import { t } from 'svelte-i18n';

  let { value = $bindable('#319dd3') } = $props();

  let isOpen = $state(false);
  let hexInput = $state(value);
  
  // ⭐ Position du popover pour le mode fixed
  let popoverX = $state(0);
  let popoverY = $state(0);

  function togglePopover(e: MouseEvent) {
    const btn = e.currentTarget as HTMLElement;
    const rect = btn.getBoundingClientRect();
    
    // On centre le popover (190px de large) sous le bouton
    let x = rect.left + (rect.width / 2) - 95;
    // On s'assure qu'il ne sorte pas de l'écran à gauche
    if (x < 10) x = 10;
    
    popoverX = x;
    popoverY = rect.bottom + 8;
    
    isOpen = !isOpen;
  }

  // Sync l'input hex quand la valeur change
  $effect(() => {
    hexInput = value;
  });

  function handleHexChange(e: Event & { currentTarget: HTMLInputElement }) {
    let val = e.currentTarget.value;
    if (!val.startsWith('#')) val = '#' + val;
    hexInput = val;
    if (/^#[0-9A-F]{6}$/i.test(val)) {
      value = val;
    }
  }

  function handleColorChange(e: Event & { currentTarget: HTMLInputElement }) {
    value = e.currentTarget.value;
  }

  const presetColors = [
    '#c29b00', // Jaune
    '#E67E22', // Orange
    '#9b1203', // Rouge
    '#b8286e', // Rose
    '#762895', // Violet
    '#084CBA', // Bleu (R8 G76 B186)
    '#17974c', // Vert
    '#08A5BA'  // Ton bleu (R8 G165 B186)
  ];
</script>

<div class="custom-color-picker">
  <button 
    class="theme-picker" 
    style="background: {value}" 
    onclick={(e) => { e.stopPropagation(); togglePopover(e); }} 
    title={$t('sidebar.accent_color_tooltip')}
  >
    <svg class="theme-picker-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
      <path d="M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z"></path>
    </svg>
  </button>

  {#if isOpen}
    <div class="color-popover" style="left: {popoverX}px; top: {popoverY}px;">
      <div class="color-popover-header">
        <span>Couleur</span>
        <button class="color-popover-close" onclick={() => isOpen = false}>✕</button>
      </div>
      
      <div class="color-popover-body">
        <div class="native-picker-wrap">
          <input type="color" value={value} oninput={handleColorChange} />
        </div>
        
        <div class="color-inputs">
          <input 
            type="text" 
            class="hex-input" 
            value={hexInput} 
            oninput={handleHexChange} 
            placeholder="#RRGGBB" 
            maxlength="7" 
          />
        </div>

        <div class="preset-swatches">
          {#each presetColors as color}
            <button 
              class="swatch" 
              class:active={value === color}
              style="background-color: {color}" 
              onclick={() => { value = color; isOpen = false; }}
              title={color}
            ></button>
          {/each}
        </div>
      </div>
    </div>
  {/if}
</div>

<svelte:window onclick={(e) => {
  const target = e.target as HTMLElement;
  if (!target.closest('.custom-color-picker')) {
    isOpen = false;
  }
}} />