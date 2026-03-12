<script>
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { themeStore } from '$lib/stores/theme.svelte.js';

  const t = $derived(themeStore.theme);
  const win = getCurrentWindow();

  let isMaximized = $state(false);

  async function toggleMaximize() {
    await win.toggleMaximize();
    isMaximized = await win.isMaximized();
  }
</script>

<div
  data-tauri-drag-region
  style="
    height:32px; flex-shrink:0;
    display:flex; align-items:center; justify-content:space-between;
    padding:0 12px 0 16px;
    background:linear-gradient(135deg, {t.bg} 0%, {t.primary}0d 100%);
    border-bottom:1px solid {t.primary}33;
    user-select:none;
  "
>
  <div data-tauri-drag-region style="display:flex; align-items:center; gap:8px; pointer-events:none">
    <div style="width:18px; height:18px; border-radius:5px; background:{t.primary}22;
                border:1px solid {t.primary}44; display:flex; align-items:center;
                justify-content:center; font-size:10px">🎵</div>
    <span style="font-size:11px; font-weight:800; color:{t.primary}; letter-spacing:0.08em">USE0</span>
  </div>

  <div style="display:flex; align-items:center; gap:2px">
    <!-- Minimize -->
    <button
      onclick={() => win.minimize()}
      title="Minimize"
      style="
        width:28px; height:22px; border:none; border-radius:4px; cursor:pointer;
        background:transparent; color:{t.textMuted}; font-size:12px;
        display:flex; align-items:center; justify-content:center;
        transition:background 0.1s, color 0.1s;
      "
      onmouseenter={(e) => { e.currentTarget.style.background = t.primary + '22'; e.currentTarget.style.color = t.primary; }}
      onmouseleave={(e) => { e.currentTarget.style.background = 'transparent'; e.currentTarget.style.color = t.textMuted; }}
    >─</button>

    <button
      onclick={toggleMaximize}
      title={isMaximized ? 'Restore' : 'Maximize'}
      style="
        width:28px; height:22px; border:none; border-radius:4px; cursor:pointer;
        background:transparent; color:{t.textMuted}; font-size:11px;
        display:flex; align-items:center; justify-content:center;
        transition:background 0.1s, color 0.1s;
      "
      onmouseenter={(e) => { e.currentTarget.style.background = t.primary + '22'; e.currentTarget.style.color = t.primary; }}
      onmouseleave={(e) => { e.currentTarget.style.background = 'transparent'; e.currentTarget.style.color = t.textMuted; }}
    >{isMaximized ? '❐' : '□'}</button>

    <button
      onclick={() => win.close()}
      title="Close"
      style="
        width:28px; height:22px; border:none; border-radius:4px; cursor:pointer;
        background:transparent; color:{t.textMuted}; font-size:14px;
        display:flex; align-items:center; justify-content:center;
        transition:background 0.1s, color 0.1s;
      "
      onmouseenter={(e) => { e.currentTarget.style.background = '#ef444444'; e.currentTarget.style.color = '#ef4444'; }}
      onmouseleave={(e) => { e.currentTarget.style.background = 'transparent'; e.currentTarget.style.color = t.textMuted; }}
    >✕</button>
  </div>
</div>