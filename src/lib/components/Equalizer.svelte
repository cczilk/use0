<script>
  import { invoke } from '@tauri-apps/api/core';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { Sliders, RotateCcw, Save, Trash2 } from 'lucide-svelte';
  import { toast } from 'svelte-sonner';

  const t = $derived(themeStore.theme);

  const BANDS = [
    { freq: 32,    label: '32'  },
    { freq: 64,    label: '64'  },
    { freq: 125,   label: '125' },
    { freq: 250,   label: '250' },
    { freq: 500,   label: '500' },
    { freq: 1000,  label: '1k'  },
    { freq: 2000,  label: '2k'  },
    { freq: 4000,  label: '4k'  },
    { freq: 8000,  label: '8k'  },
    { freq: 16000, label: '16k' },
  ];

  const DEFAULT_PRESETS = {
    flat:       { name: 'Flat',          gains: [0,0,0,0,0,0,0,0,0,0] },
    bass:       { name: 'Bass Boost',    gains: [8,6,4,2,0,0,0,0,0,0] },
    treble:     { name: 'Treble Boost',  gains: [0,0,0,0,0,0,2,4,6,8] },
    vocal:      { name: 'Vocal',         gains: [-2,-1,0,2,4,4,2,0,-1,-2] },
    electronic: { name: 'Electronic',    gains: [4,3,0,-2,2,0,2,4,5,6] },
    rock:       { name: 'Rock',          gains: [4,3,2,-1,-2,-1,2,3,4,4] },
    jazz:       { name: 'Jazz',          gains: [3,2,1,2,-1,-1,0,1,2,3] },
    classical:  { name: 'Classical',     gains: [4,3,2,0,-1,-1,0,2,3,4] },
  };

  let gains         = $state([0,0,0,0,0,0,0,0,0,0]);
  let customPresets = $state([]);
  let showSave      = $state(false);
  let presetName    = $state('');
  let isExpanded    = $state(false);

  $effect(() => {
    invoke('get_eq_settings').then(saved => {
      if (saved?.gains) gains = saved.gains;
    }).catch(() => {
      const saved = localStorage.getItem('eq_last_settings');
      if (saved) { try { gains = JSON.parse(saved); } catch {} }
    });
    invoke('get_eq_presets').then(p => {
      if (p) customPresets = p;
    }).catch(() => {
      const saved = localStorage.getItem('eq_custom_presets');
      if (saved) { try { customPresets = JSON.parse(saved); } catch {} }
    });
  });

  function applyGains(newGains) {
    gains = newGains;
    invoke('apply_eq_gains', { gains: newGains }).catch(console.error);
    invoke('save_eq_settings', { gains: newGains }).catch(() => {
      localStorage.setItem('eq_last_settings', JSON.stringify(newGains));
    });
  }

  function handleSlider(index, value) {
    const next = [...gains];
    next[index] = parseFloat(value);
    applyGains(next);
  }

  function loadPreset(presetGains) {
    applyGains(presetGains);
    toast.success('Preset loaded');
  }

  function reset() {
    applyGains([0,0,0,0,0,0,0,0,0,0]);
    toast.success('EQ reset');
  }

  async function saveCustomPreset() {
    if (!presetName.trim()) { toast.error('Enter a preset name'); return; }
    const newPreset = { id: Date.now(), name: presetName.trim(), gains: [...gains] };
    const updated = [...customPresets, newPreset];
    customPresets = updated;
    invoke('save_eq_presets', { presets: updated }).catch(() => {
      localStorage.setItem('eq_custom_presets', JSON.stringify(updated));
    });
    toast.success(`Preset "${presetName}" saved!`);
    presetName = '';
    showSave = false;
  }

  function deletePreset(id) {
    const updated = customPresets.filter(p => p.id !== id);
    customPresets = updated;
    invoke('save_eq_presets', { presets: updated }).catch(() => {
      localStorage.setItem('eq_custom_presets', JSON.stringify(updated));
    });
    toast.success('Preset deleted');
  }
</script>

<div style="backdrop-filter:blur(12px); border:1px solid {t.border}; border-radius:8px; padding:12px; background:{t.bg}">
  <div onclick={() => isExpanded = !isExpanded}
    style="display:flex; align-items:center; justify-content:space-between; cursor:pointer; padding:4px; border-radius:4px; background:rgba(255,0,255,0.2)"
    onmouseenter={(e) => e.currentTarget.style.background = 'rgba(255,0,255,0.3)'}
    onmouseleave={(e) => e.currentTarget.style.background = 'rgba(255,0,255,0.2)'}
  >
    <div style="display:flex; align-items:center; gap:8px">
      <Sliders size={16} style="color:{t.primary}" />
      <span style="font-weight:bold; font-size:14px; color:{t.primary}">Equalizer {isExpanded ? '▼' : '▶'}</span>
    </div>
  </div>

  {#if isExpanded}
    <div style="margin-top:12px; margin-bottom:16px">
      <div style="font-size:10px; font-weight:bold; margin-bottom:8px; color:{t.textMuted}">PRESETS</div>
      <div style="display:flex; flex-wrap:wrap; gap:6px; margin-bottom:8px">
        {#each Object.entries(DEFAULT_PRESETS) as [key, preset] (key)}
          <button onclick={() => loadPreset(preset.gains)}
            style="padding:4px 10px; font-size:11px; border-radius:4px; background:transparent; border:1px solid {t.border}; color:{t.text}; cursor:pointer"
            onmouseenter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.1)'}
            onmouseleave={(e) => e.currentTarget.style.background = 'transparent'}>
            {preset.name}
          </button>
        {/each}
      </div>

      {#if customPresets.length > 0}
        <div style="display:flex; flex-wrap:wrap; gap:6px; margin-bottom:8px">
          {#each customPresets as preset (preset.id)}
            <button onclick={() => loadPreset(preset.gains)}
              style="padding:4px 10px; font-size:11px; border-radius:4px; background:{t.primary}40; border:1px solid {t.primary}; color:{t.text}; cursor:pointer; display:flex; align-items:center; gap:6px">
              {preset.name}
              <span onclick={(e) => { e.stopPropagation(); deletePreset(preset.id); }} style="display:flex; align-items:center; color:{t.textMuted}">
                <Trash2 size={10} />
              </span>
            </button>
          {/each}
        </div>
      {/if}

      <div style="display:flex; gap:6px">
        <button onclick={() => showSave = !showSave}
          style="padding:4px 10px; font-size:11px; border-radius:4px; background:transparent; border:1px dashed {t.primary}; color:{t.primary}; cursor:pointer; display:flex; align-items:center; gap:4px">
          <Save size={10} /> Save Custom
        </button>
        <button onclick={reset}
          style="padding:4px 10px; font-size:11px; border-radius:4px; background:transparent; border:1px solid {t.border}; color:{t.text}; cursor:pointer; display:flex; align-items:center; gap:4px">
          <RotateCcw size={10} /> Reset
        </button>
      </div>

      {#if showSave}
        <div style="margin-top:8px; padding:12px; background:{t.bgMuted}80; border-radius:8px; border:1px solid {t.border}">
          <input type="text" bind:value={presetName} onkeypress={(e) => e.key === 'Enter' && saveCustomPreset()}
            placeholder="Preset name..."
            style="width:100%; padding:8px; background:{t.bg}; border:1px solid {t.border}; border-radius:4px; color:{t.text}; font-size:12px; margin-bottom:8px; outline:none; box-sizing:border-box" />
          <div style="display:flex; gap:8px">
            <button onclick={saveCustomPreset} style="flex:1; padding:6px; font-size:11px; border-radius:4px; background:{t.primary}; color:#000; border:none; cursor:pointer; font-weight:bold">Save</button>
            <button onclick={() => { showSave = false; presetName = ''; }} style="flex:1; padding:6px; font-size:11px; border-radius:4px; background:transparent; color:{t.text}; border:1px solid {t.border}; cursor:pointer">Cancel</button>
          </div>
        </div>
      {/if}
    </div>

    <div style="display:flex; gap:12px; justify-content:space-between">
      {#each BANDS as band, i (band.freq)}
        <div style="display:flex; flex-direction:column; align-items:center; flex:1">
          <div style="font-size:10px; margin-bottom:4px; color:{gains[i] === 0 ? t.textMuted : t.primary}; font-weight:{gains[i] === 0 ? 'normal' : 'bold'}">
            {gains[i] > 0 ? '+' : ''}{gains[i].toFixed(1)}
          </div>
          <input type="range" min="-12" max="12" step="0.5"
            value={gains[i]}
            oninput={(e) => handleSlider(i, e.target.value)}
            style="width:100%; height:80px; writing-mode:bt-lr; -webkit-appearance:slider-vertical; appearance:slider-vertical" />
          <div style="font-size:9px; margin-top:4px; color:{t.textMuted}">{band.label}</div>
        </div>
      {/each}
    </div>

    <div style="margin-top:12px; font-size:10px; color:{t.textMuted}; text-align:center">Range: -12dB to +12dB</div>
  {/if}
</div>