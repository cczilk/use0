<!-- src/lib/components/ThemeSelector.svelte -->
<script>
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { Palette } from 'lucide-svelte';
  const t = $derived(themeStore.theme);
  let isOpen = $state(false);
  let btnEl = $state(null);
  let dropdownPos = $state({ top: 0, left: 0 });
  const colors = { dark: '#4ade80', red: '#ef4444', blue: '#60a5fa', purple: '#a78bfa' };

  function toggle() {
    if (!isOpen && btnEl) {
      const r = btnEl.getBoundingClientRect();
      dropdownPos = { top: r.bottom + 6, left: r.left };
    }
    isOpen = !isOpen;
  }
</script>

<div>
  <button
    bind:this={btnEl}
    onclick={toggle}
    style="padding:6px; border-radius:4px; background:transparent; border:none; cursor:pointer; color:{t.primary}; display:flex; align-items:center"
    onmouseenter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.1)'}
    onmouseleave={(e) => e.currentTarget.style.background = 'transparent'}
  >
    <Palette size={18} />
  </button>

  {#if isOpen}
    <!-- svelte-ignore a11y_click_events_have_key_events -->
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div onclick={() => isOpen = false} style="position:fixed; inset:0; z-index:9998" />
    <div style="position:fixed; top:{dropdownPos.top}px; left:{dropdownPos.left}px;
                padding:8px; border-radius:8px; border:1px solid {t.border};
                backdrop-filter:blur(12px); z-index:9999; min-width:150px;
                background:{t.bg}; box-shadow:0 4px 20px rgba(0,0,0,0.8)">
      <div style="font-size:10px; font-weight:bold; margin-bottom:8px; padding:0 8px; color:{t.textMuted}">THEME</div>
      {#each themeStore.available as name (name)}
        <button
          onclick={() => { themeStore.change(name); isOpen = false; }}
          style="width:100%; text-align:left; padding:8px 12px; border-radius:4px; display:flex; align-items:center; gap:8px;
                 background:{themeStore.current === name ? 'rgba(255,255,255,0.2)' : 'transparent'};
                 border:none; cursor:pointer; color:{t.text}; margin-bottom:2px"
          onmouseenter={(e) => { if (themeStore.current !== name) e.currentTarget.style.background = 'rgba(255,255,255,0.1)'; }}
          onmouseleave={(e) => { if (themeStore.current !== name) e.currentTarget.style.background = 'transparent'; }}
        >
          <div style="width:16px; height:16px; border-radius:50%; background:{colors[name]};
                      border:2px solid {themeStore.current === name ? t.primary : 'transparent'}; flex-shrink:0"></div>
          <span style="text-transform:capitalize; font-size:13px">{name}</span>
        </button>
      {/each}
    </div>
  {/if}
</div>