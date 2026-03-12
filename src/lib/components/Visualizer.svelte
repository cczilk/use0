<script>
  import { onMount } from 'svelte';
  import { themeStore } from '$lib/stores/theme.svelte.js';

  let { getFrequencyData = null, getTimeDomainData = null, isPlaying = false } = $props();

  const appTheme = $derived(themeStore.theme);

  const COLOR_THEMES = {
    neonBlue:  { name: 'Neon Blue',  bar: (i,v) => `rgba(0,200,255,${0.3+v})`,              wave: '#00e5ff' },
    neonGreen: { name: 'Neon Green', bar: (i,v) => `rgba(0,255,100,${0.3+v})`,              wave: '#4ade80' },
    purple:    { name: 'Purple',     bar: (i,v) => `rgba(180,80,255,${0.3+v})`,             wave: '#a855f7' },
    sunset:    { name: 'Sunset',     bar: (i,v) => `hsl(${(i/64)*40+10},90%,${50+v*20}%)`, wave: '#fb923c' },
  };

  let visualType = $state('bars');
  let colorTheme = $state('neonGreen');
  let background = $state('transparent');
  let canvasEl   = $state(null);
  let animId     = null;

  onMount(() => {
    try {
      const saved = JSON.parse(localStorage.getItem('visualizer_last_settings') || '{}');
      if (saved.visualType && saved.visualType !== 'circular') visualType = saved.visualType;
      if (saved.colorTheme) colorTheme = saved.colorTheme;
      if (saved.background) background = saved.background;
    } catch {}
  });

  $effect(() => {
    try { localStorage.setItem('visualizer_last_settings', JSON.stringify({ visualType, colorTheme, background })); } catch {}
  });

  $effect(() => {
    const _deps = [visualType, colorTheme, background, isPlaying, getFrequencyData, getTimeDomainData];
    if (!canvasEl) return;
    return startDraw(canvasEl);
  });

  function startDraw(canvas) {
    if (animId) { cancelAnimationFrame(animId); animId = null; }
    const ctx = canvas.getContext('2d');
    const dpr = window.devicePixelRatio || 1;

    const resize = () => {
      canvas.width  = canvas.offsetWidth  * dpr;
      canvas.height = canvas.offsetHeight * dpr;
      ctx.scale(dpr, dpr);
    };
    resize();
    window.addEventListener('resize', resize);

    const draw = () => {
      const w = canvas.offsetWidth, h = canvas.offsetHeight;

      if (background === 'transparent') ctx.clearRect(0, 0, w, h);
      else if (background === 'tinted') { ctx.fillStyle = 'rgba(0,0,0,0.25)'; ctx.fillRect(0,0,w,h); }
      else { ctx.fillStyle = 'rgba(0,0,0,0.08)'; ctx.fillRect(0,0,w,h); }

      if (isPlaying && getFrequencyData && getTimeDomainData) {
        const freq = getFrequencyData();
        const time = getTimeDomainData();
        if (freq && time) {
          if      (visualType === 'bars')     drawBars(ctx, w, h, freq);
          else if (visualType === 'waveform') drawWave(ctx, w, h, time);
        }
      }
      animId = requestAnimationFrame(draw);
    };
    draw();

    return () => {
      window.removeEventListener('resize', resize);
      if (animId) { cancelAnimationFrame(animId); animId = null; }
    };
  }

  function drawBars(ctx, w, h, freq) {
    const bars = 64, bw = w / bars;
    for (let i = 0; i < bars; i++) {
      const v = freq[Math.floor((i / bars) * freq.length)] / 255;
      ctx.fillStyle = COLOR_THEMES[colorTheme].bar(i, v);
      ctx.fillRect(i * bw + 2, h - v * h * 0.9, bw - 4, v * h * 0.9);
    }
  }

  function drawWave(ctx, w, h, time) {
    ctx.strokeStyle = COLOR_THEMES[colorTheme].wave;
    ctx.lineWidth = 2;
    ctx.beginPath();
    const slice = w / time.length;
    for (let i = 0; i < time.length; i++) {
      const y = (time[i] / 128.0) * (h / 2);
      i === 0 ? ctx.moveTo(0, y) : ctx.lineTo(i * slice, y);
    }
    ctx.stroke();
  }
</script>

<div style="background:{appTheme.bg}; border-radius:8px; border:1px solid {appTheme.border}; padding:12px;
            display:flex; flex-direction:column; height:100%; backdrop-filter:blur(12px); box-sizing:border-box">

  <div style="display:flex; align-items:center; justify-content:space-between; margin-bottom:10px; flex-wrap:wrap; gap:8px">
    <span style="color:{appTheme.primary}; font-size:13px; font-weight:bold">Visualizer</span>
    <div style="display:flex; gap:6px">
      {#each [['bars','Bars'],['waveform','Wave']] as [val, label] (val)}
        <button onclick={() => visualType = val}
          style="padding:4px 10px; font-size:11px; border-radius:4px; border:none; cursor:pointer; white-space:nowrap;
                 background:{visualType===val ? appTheme.primary : 'transparent'};
                 color:{visualType===val ? '#000' : appTheme.text};
                 font-weight:{visualType===val ? 'bold' : 'normal'}">{label}</button>
      {/each}
    </div>
  </div>

  <div style="display:flex; gap:6px; margin-bottom:10px; flex-wrap:wrap">
    {#each Object.entries(COLOR_THEMES) as [key, val] (key)}
      <button onclick={() => colorTheme = key}
        style="padding:4px 10px; font-size:11px; border-radius:4px; border:none; cursor:pointer; white-space:nowrap;
               background:{colorTheme===key ? appTheme.primary : 'transparent'};
               color:{colorTheme===key ? '#000' : appTheme.text};
               font-weight:{colorTheme===key ? 'bold' : 'normal'}">{val.name}</button>
    {/each}
  </div>

  <div style="display:flex; gap:6px; margin-bottom:10px; flex-wrap:wrap">
    {#each [['transparent','Clear'],['tinted','Tinted'],['blur','Blur']] as [val, label] (val)}
      <button onclick={() => background = val}
        style="padding:4px 10px; font-size:11px; border-radius:4px; border:none; cursor:pointer; white-space:nowrap;
               background:{background===val ? appTheme.primary : 'transparent'};
               color:{background===val ? '#000' : appTheme.text};
               font-weight:{background===val ? 'bold' : 'normal'}">{label}</button>
    {/each}
  </div>

  <div style="flex:1; min-height:60px">
    <canvas bind:this={canvasEl} style="width:100%; height:100%; border-radius:4px" />
  </div>
</div>