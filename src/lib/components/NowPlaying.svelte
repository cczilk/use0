<script>
  import { Sparkles, Upload } from 'lucide-svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { invoke, convertFileSrc } from '@tauri-apps/api/core';
  import { player } from '$lib/stores/player.svelte.js';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import Player from './Player.svelte';

  const theme = $derived(themeStore.theme);

  const DEFAULT_VISUALIZERS = [
    { id: 'none',      name: 'None' },
    { id: 'gradient',  name: 'Gradient Pulse' },
    { id: 'particles', name: 'Particles' },
    { id: 'waves',     name: 'Waves' },
  ];

  let showVisualizerMenu = $state(false);
  let selectedVisualizer = $state('none');
  let customVideoPath    = $state(null);
  let customVideoMime    = $state('image/gif');
  let particlesCanvas    = $state(null);
  let artworkEl          = $state(null);
  let artworkUrl         = $state(null);

  // Load artwork as base64 whenever track changes
  $effect(() => {
    const id = player.currentTrack?.id;
    if (!id) { artworkUrl = null; return; }
    invoke('get_thumbnail_base64', { trackId: id })
      .then(data => { artworkUrl = data ?? null; })
      .catch(() => { artworkUrl = null; });
  });

  const visualizerStyle = $derived.by(() => {
    if (selectedVisualizer === 'none' || selectedVisualizer === 'custom' || selectedVisualizer === 'particles') return '';
    const beatDuration = 60 / (player.bpm || 120);
    switch (selectedVisualizer) {
      case 'gradient':
        return `background: radial-gradient(ellipse at 30% 50%, ${theme.primary}66 6%, transparent 30%),
                               radial-gradient(ellipse at 70% 50%, ${theme.primary}44 6%, transparent 30%);
                background-size: 180% 140%;
                animation: gradientMove ${beatDuration * 2}s ease-in-out infinite;`;
      case 'waves':
        return `background: linear-gradient(90deg, rgba(0,0,0,0) 0%, ${theme.primary}30 50%, rgba(0,0,0,0) 100%);
                background-size: 200% 100%;
                animation: waveMove ${beatDuration * 4}s linear infinite;`;
      default: return '';
    }
  });

  $effect(() => {
    if (selectedVisualizer !== 'particles' || !particlesCanvas || !artworkEl) return;
    const canvas = particlesCanvas, container = artworkEl;
    const ctx = canvas.getContext('2d'), dpr = window.devicePixelRatio || 1;
    let rafId = null, particles = [];
    const rgb = hexToRgb(theme.primary || '#ffffff');
    const resize = () => {
      const w = container.clientWidth, h = container.clientHeight;
      canvas.width = Math.floor(w*dpr); canvas.height = Math.floor(h*dpr);
      canvas.style.width = `${w}px`; canvas.style.height = `${h}px`;
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    };
    const initParticles = () => {
      particles = [];
      for (let i = 0; i < 40; i++) particles.push({ x: Math.random()*container.clientWidth, y: Math.random()*container.clientHeight, vx: (Math.random()-0.5)*0.5, vy: (Math.random()-0.5)*0.5, base: 1+Math.random()*2 });
    };
    const draw = () => {
      ctx.clearRect(0,0,canvas.width,canvas.height);
      for (const p of particles) {
        p.x += p.vx; p.y += p.vy;
        if (p.x<0) p.x=container.clientWidth; if (p.x>container.clientWidth) p.x=0;
        const grad=ctx.createRadialGradient(p.x,p.y,0,p.x,p.y,p.base*4);
        grad.addColorStop(0,`rgba(${rgb.r},${rgb.g},${rgb.b},0.6)`);
        grad.addColorStop(1,`rgba(${rgb.r},${rgb.g},${rgb.b},0)`);
        ctx.fillStyle=grad; ctx.beginPath(); ctx.arc(p.x,p.y,p.base*2,0,Math.PI*2); ctx.fill();
      }
      rafId = requestAnimationFrame(draw);
    };
    resize(); initParticles(); rafId = requestAnimationFrame(draw);
    return () => cancelAnimationFrame(rafId);
  });

  async function handleUploadVideo() {
    const selected = await open({ filters: [{ name: 'Video', extensions: ['mp4','webm','gif'] }] });
    if (selected) {
      const ext = selected.split('.').pop().toLowerCase();
      customVideoMime = ext === 'gif' ? 'image/gif' : ext === 'webm' ? 'video/webm' : 'video/mp4';
      // convertFileSrc creates a proper asset:// URL Tauri's webview can serve
      customVideoPath = convertFileSrc(selected);
      selectedVisualizer = 'custom';
      showVisualizerMenu = false;
    }
  }

  function hexToRgb(hex) {
    const h = hex.replace('#','');
    const n = parseInt(h.length===3 ? h.split('').map(c=>c+c).join('') : h, 16);
    return { r:(n>>16)&255, g:(n>>8)&255, b:n&255 };
  }
</script>

<style>
  @keyframes gradientMove { 0%,100% { background-position:0% 50%; } 50% { background-position:100% 50%; } }
  @keyframes waveMove { 0% { background-position:0% 50%; } 100% { background-position:200% 50%; } }
</style>

<div style="height:100%; display:flex; flex-direction:column; background:{theme.bg}; overflow:hidden;">
  <div style="padding:12px 16px; border-bottom:1px solid {theme.border}; flex-shrink:0;">
    <span style="font-size:10px; font-weight:800; letter-spacing:0.1em; color:{theme.textMuted}; text-transform:uppercase;">Player Engine</span>
  </div>

  {#if player.currentTrack}
    <div style="padding:20px; flex-shrink:0;">
      <div style="width:100%; aspect-ratio:1/1; position:relative; border-radius:16px; overflow:hidden; border:1px solid {theme.border}; box-shadow:0 12px 40px rgba(0,0,0,0.4)">
        {#if artworkUrl}
          <img src={artworkUrl} alt="art" style="width:100%; height:100%; object-fit:cover;" />
        {:else}
          <div style="width:100%; height:100%; display:flex; align-items:center; justify-content:center; background:{theme.bgMuted}; font-size:40px;">🎵</div>
        {/if}
      </div>
    </div>

    <div bind:this={artworkEl} style="flex:1; margin:0 12px 12px 12px; position:relative; border-radius:20px; border:1px solid {theme.border}; background:{theme.bgMuted}">
      <div style="position:absolute; inset:0; z-index:0; pointer-events:none; border-radius:20px; overflow:hidden;">
        {#if customVideoPath && selectedVisualizer === 'custom'}
          {#if customVideoMime.startsWith('video/')}
            <video src={customVideoPath} autoplay loop muted playsinline
              style="width:100%; height:100%; object-fit:cover; opacity:0.4;"></video>
          {:else}
            <img src={customVideoPath} alt="bg" style="width:100%; height:100%; object-fit:cover; opacity:0.4;" />
          {/if}
        {:else if artworkUrl}
          <img src={artworkUrl} alt="blur" style="width:100%; height:100%; object-fit:cover; filter:blur(30px) brightness(0.4); opacity:0.6;" />
        {/if}
        {#if selectedVisualizer === 'particles'}
          <canvas bind:this={particlesCanvas} style="position:absolute; inset:0; width:100%; height:100%;"></canvas>
        {/if}
        {#if visualizerStyle}
          <div style="position:absolute; inset:0; {visualizerStyle} opacity:0.4;"></div>
        {/if}
      </div>

      <div style="position:relative; z-index:1; height:100%; display:flex; flex-direction:column; padding:20px;">
        <div style="display:flex; justify-content:space-between; align-items:center; margin-bottom:12px;">
          <span style="font-size:11px; font-weight:bold; color:{theme.primary}; text-transform:uppercase; letter-spacing:0.05em">Now Playing</span>
          <div style="position:relative;">
            <button onclick={(e) => { e.stopPropagation(); showVisualizerMenu = !showVisualizerMenu; }}
              style="background:rgba(255,255,255,0.1); border:none; padding:6px; border-radius:8px; color:{theme.primary}; cursor:pointer; display:flex; position:relative; z-index:101;">
              <Sparkles size={14} />
            </button>
            {#if showVisualizerMenu}
              <!-- svelte-ignore a11y_click_events_have_key_events -->
              <!-- svelte-ignore a11y_no_static_element_interactions -->
              <div onclick={() => showVisualizerMenu = false} style="position:fixed; inset:0; z-index:99;"></div>
              <div style="position:absolute; bottom:100%; right:0; margin-bottom:10px; background:{theme.bg}; border:1px solid {theme.border}; border-radius:12px; padding:6px; min-width:160px; box-shadow:0 8px 32px rgba(0,0,0,0.8); z-index:100">
                <div style="font-size:9px; font-weight:bold; color:{theme.textMuted}; padding:4px 10px; text-transform:uppercase;">Visualizer</div>
                {#each DEFAULT_VISUALIZERS as v}
                  <button onclick={() => {selectedVisualizer=v.id; showVisualizerMenu=false;}}
                    style="width:100%; text-align:left; background:{selectedVisualizer===v.id ? theme.primary+'33' : 'transparent'}; border:none; color:{selectedVisualizer===v.id ? theme.primary : theme.text}; padding:8px 10px; border-radius:6px; font-size:12px; cursor:pointer;">
                    {v.name}
                  </button>
                {/each}
                <div style="height:1px; background:{theme.border}; margin:4px 6px;"></div>
                <button onclick={handleUploadVideo} style="width:100%; text-align:left; background:transparent; border:none; color:{theme.primary}; padding:8px 10px; font-size:12px; cursor:pointer; display:flex; align-items:center; gap:8px;">
                  <Upload size={12} /> {customVideoPath ? 'Change Video' : 'Upload Video'}
                </button>
              </div>
            {/if}
          </div>
        </div>

        <div style="overflow:hidden; margin-bottom:12px;">
          <h1 style="font-size:18px; font-weight:800; color:{theme.text}; margin:0; white-space:nowrap; text-overflow:ellipsis; overflow:hidden;">{player.currentTrack.title}</h1>
          <p style="font-size:14px; font-weight:600; color:{theme.primary}; margin:4px 0 0;">{player.currentTrack.artist ?? 'Unknown'}</p>
        </div>

        <div style="margin-top:auto; display:flex; flex-direction:column; gap:20px;">
          <Player compact={true} showVolume={true} />
          <div style="display:flex; gap:8px;">
            <button onclick={() => player.toggleShuffle()} style="flex:1; background:rgba(255,255,255,0.05); border:1px solid {theme.border}; border-radius:10px; padding:10px; color:{player.shuffle ? theme.primary : theme.textMuted}; font-size:11px; font-weight:bold; cursor:pointer;">
              SHUFFLE: {player.shuffle ? 'ON' : 'OFF'}
            </button>
            <button onclick={() => player.toggleAutoplay()} style="flex:1; background:rgba(255,255,255,0.05); border:1px solid {theme.border}; border-radius:10px; padding:10px; color:{player.autoplay ? theme.primary : theme.textMuted}; font-size:11px; font-weight:bold; cursor:pointer;">
              AUTO: {player.autoplay ? 'ON' : 'OFF'}
            </button>
          </div>
        </div>
      </div>
    </div>
  {:else}
    <div style="flex:1; display:flex; align-items:center; justify-content:center; color:{theme.textMuted}; font-size:13px; font-weight:500;">
      Drop some musicc to start
    </div>
  {/if}
</div>
