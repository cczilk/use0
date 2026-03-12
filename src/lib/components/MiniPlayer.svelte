<script>
  import { player } from '$lib/stores/player.svelte.js';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { audioAnalyser } from '$lib/stores/analyser.svelte.js';
  import { convertFileSrc } from '@tauri-apps/api/core';
  import Visualizer from './Visualizer.svelte';

  let { onExpand } = $props();
  const t = $derived(themeStore.theme);

  const artworkUrl = $derived(
    player.currentTrack?.thumbnailPath
      ? convertFileSrc(player.currentTrack.thumbnailPath)
      : null
  );

  // Seek drag
  let dragging    = $state(false);
  let dragPercent = $state(0);
  const displayPercent = $derived(dragging ? dragPercent : player.progressPercent);

  function getPct(e, el) {
    const rect = el.getBoundingClientRect();
    return Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
  }
  function onPointerDown(e) {
    e.currentTarget.setPointerCapture(e.pointerId);
    dragging = true;
    dragPercent = getPct(e, e.currentTarget) * 100;
  }
  function onPointerMove(e) {
    if (!dragging) return;
    dragPercent = getPct(e, e.currentTarget) * 100;
  }
  function onPointerUp(e) {
    if (!dragging) return;
    dragging = false;
    player.seek(player.duration * getPct(e, e.currentTarget));
  }
</script>

<div style="flex:1; display:flex; flex-direction:column; overflow:hidden; position:relative">

  {#if artworkUrl}
    <div style="position:absolute; inset:0; z-index:0; overflow:hidden">
      <img src={artworkUrl} alt="" style="width:100%; height:100%; object-fit:cover;
        filter:blur(40px) brightness(0.25) saturate(1.4); transform:scale(1.1)" />
    </div>
  {/if}

  <!-- Main mini layout -->
  <div style="position:relative; z-index:1; flex:1; display:flex; flex-direction:column; overflow:hidden">

    <!-- Top row: art + track info + controls -->
    <div style="display:flex; align-items:stretch; gap:0; flex-shrink:0; height:100px; border-bottom:1px solid {t.border}22">

      <!-- Album art -->
      <div style="width:100px; height:100px; flex-shrink:0; overflow:hidden">
        {#if artworkUrl}
          <img src={artworkUrl} alt="art" style="width:100%; height:100%; object-fit:cover" />
        {:else}
          <div style="width:100%; height:100%; display:flex; align-items:center; justify-content:center;
                      background:{t.bgMuted}; font-size:32px">🎵</div>
        {/if}
      </div>

      <!-- Track info -->
      <div style="flex:1; min-width:0; padding:16px 20px; display:flex; flex-direction:column; justify-content:center; gap:4px">
        {#if player.currentTrack}
          <div style="font-size:16px; font-weight:800; color:{t.text};
                      white-space:nowrap; overflow:hidden; text-overflow:ellipsis">
            {player.currentTrack.title}
          </div>
          <div style="font-size:12px; color:{t.primary}; font-weight:600">
            {player.currentTrack.artist ?? 'Unknown Artist'}
          </div>
          {#if player.currentTrack.album}
            <div style="font-size:11px; color:{t.textMuted}">
              {player.currentTrack.album}
            </div>
          {/if}
        {:else}
          <div style="font-size:14px; color:{t.textMuted}">No track playing</div>
        {/if}
      </div>

      <!-- Controls + volume -->
      <div style="display:flex; flex-direction:column; justify-content:center; align-items:center;
                  gap:12px; padding:16px 20px; flex-shrink:0">

        <!-- Playback buttons -->
        <div style="display:flex; align-items:center; gap:16px">
          <button onclick={() => player.prev()}
            style="background:none; border:none; cursor:pointer; color:{t.textMuted}; font-size:18px; padding:4px"
            onmouseenter={(e) => e.currentTarget.style.color = t.primary}
            onmouseleave={(e) => e.currentTarget.style.color = t.textMuted}>⏮</button>

          <button onclick={() => player.isPlaying ? player.pause() : player.play()}
            style="background:{t.primary}; border:none; border-radius:50%; cursor:pointer;
                   width:40px; height:40px; font-size:16px; display:flex; align-items:center;
                   justify-content:center; color:#000; flex-shrink:0"
            onmouseenter={(e) => e.currentTarget.style.transform = 'scale(1.08)'}
            onmouseleave={(e) => e.currentTarget.style.transform = 'scale(1)'}>
            {player.isPlaying ? '⏸' : '▶'}
          </button>

          <button onclick={() => player.next()}
            style="background:none; border:none; cursor:pointer; color:{t.textMuted}; font-size:18px; padding:4px"
            onmouseenter={(e) => e.currentTarget.style.color = t.primary}
            onmouseleave={(e) => e.currentTarget.style.color = t.textMuted}>⏭</button>
        </div>

        <!-- Volume -->
        <div style="display:flex; align-items:center; gap:8px">
          <span style="font-size:12px; color:{t.textMuted}">🔈</span>
          <input type="range" min="0" max="100" value={player.volume}
            oninput={(e) => player.setVolume(Number(e.currentTarget.value))}
            style="width:100px; accent-color:{t.primary}; cursor:pointer; height:3px" />
          <span style="font-size:10px; color:{t.textMuted}; min-width:28px">{player.volume}%</span>
        </div>
      </div>

      <!-- Shuffle / Auto / Expand buttons -->
      <div style="display:flex; flex-direction:column; justify-content:center; gap:6px;
                  padding:16px 16px 16px 0; flex-shrink:0; border-left:1px solid {t.border}22; padding-left:16px">
        <button onclick={() => player.toggleShuffle()}
          style="padding:5px 10px; border-radius:6px; font-size:10px; font-weight:bold;
                 border:1px solid {t.border}; cursor:pointer; font-family:inherit;
                 background:{player.shuffle ? t.primary+'22' : 'transparent'};
                 color:{player.shuffle ? t.primary : t.textMuted}">
          ⇄ SHUFFLE
        </button>
        <button onclick={() => player.toggleAutoplay()}
          style="padding:5px 10px; border-radius:6px; font-size:10px; font-weight:bold;
                 border:1px solid {t.border}; cursor:pointer; font-family:inherit;
                 background:{player.autoplay ? t.primary+'22' : 'transparent'};
                 color:{player.autoplay ? t.primary : t.textMuted}">
          ↺ AUTO
        </button>
        <button onclick={onExpand}
          title="Full view"
          style="padding:5px 10px; border-radius:6px; font-size:10px; font-weight:bold;
                 border:1px solid {t.border}; cursor:pointer; font-family:inherit;
                 background:transparent; color:{t.textMuted}"
          onmouseenter={(e) => { e.currentTarget.style.color = t.primary; e.currentTarget.style.borderColor = t.primary; }}
          onmouseleave={(e) => { e.currentTarget.style.color = t.textMuted; e.currentTarget.style.borderColor = t.border; }}>
          ⛶ EXPAND
        </button>
      </div>
    </div>

    <!-- Progress bar -->
    <div
      role="slider"
      tabindex="0"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={Math.round(displayPercent)}
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={onPointerUp}
      style="height:4px; background:rgba(255,255,255,0.08); cursor:pointer;
             touch-action:none; user-select:none; flex-shrink:0"
    >
      <div style="width:{displayPercent}%; height:100%; background:{t.primary};
                  pointer-events:none; transition:{dragging ? 'none' : 'width 0.1s linear'}">
      </div>
    </div>

    <!-- Time labels -->
    <div style="display:flex; justify-content:space-between; padding:4px 12px;
                flex-shrink:0; border-bottom:1px solid {t.border}22">
      <span style="font-size:10px; color:{t.textMuted}">{player.formattedTime}</span>
      <span style="font-size:10px; color:{t.textMuted}">{player.formattedDuration}</span>
    </div>

    <!-- Visualizer fills remaining space -->
    <div style="flex:1; min-height:0; padding:8px">
      <Visualizer
        isPlaying={player.isPlaying}
        getFrequencyData={() => audioAnalyser.getFrequencyData()}
        getTimeDomainData={() => audioAnalyser.getTimeDomainData()}
      />
    </div>
  </div>
</div>