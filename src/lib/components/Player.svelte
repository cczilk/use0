<!-- src/lib/components/Player.svelte -->
<script>
  import { player } from '$lib/stores/player.svelte.js';
  import { themeStore } from '$lib/stores/theme.svelte.js';

  let { compact = false, showVolume = false } = $props();
  const theme = $derived(themeStore.theme);

  // Optimistic seek — update UI instantly, send to Rust only on release
  let dragging      = $state(false);
  let dragPercent   = $state(0);

  const displayPercent = $derived(dragging ? dragPercent : player.progressPercent);
  const displayTime    = $derived(dragging
    ? formatTime((dragPercent / 100) * player.duration)
    : player.formattedTime
  );

  function formatTime(secs) {
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

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
    const pct = getPct(e, e.currentTarget);
    player.seek(player.duration * pct);
  }
</script>

<div style="display:flex; flex-direction:column; gap:{compact ? '6px' : '12px'}">

  <!-- Progress bar -->
  <div>
    <!-- svelte-ignore a11y_no_static_element_interactions -->
    <div
      role="slider"
      tabindex="0"
      aria-valuemin="0"
      aria-valuemax="100"
      aria-valuenow={Math.round(displayPercent)}
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={onPointerUp}
      onkeydown={(e) => {
        if (e.key === 'ArrowRight') player.seek(Math.min(player.duration, player.currentTime + 5));
        if (e.key === 'ArrowLeft')  player.seek(Math.max(0, player.currentTime - 5));
      }}
      style="height:6px; background:rgba(255,255,255,0.1); border-radius:3px; cursor:pointer;
             position:relative; touch-action:none; user-select:none;"
    >
      <div style="width:{displayPercent}%; height:100%; background:{theme.primary};
                  border-radius:3px; pointer-events:none; position:relative;">
        <!-- Scrubber dot -->
        <div style="position:absolute; right:-5px; top:50%; transform:translateY(-50%);
                    width:10px; height:10px; border-radius:50%; background:{theme.primary};
                    opacity:{dragging ? 1 : 0}; transition:opacity 0.1s;
                    box-shadow:0 0 4px {theme.primary};" />
      </div>
    </div>
    <div style="display:flex; justify-content:space-between; margin-top:3px">
      <span style="font-size:10px; color:{theme.textMuted}">{displayTime}</span>
      <span style="font-size:10px; color:{theme.textMuted}">{player.formattedDuration}</span>
    </div>
  </div>

  <!-- Controls -->
  <div style="display:flex; align-items:center; justify-content:center; gap:{compact ? '10px' : '20px'}">
    <button
      onclick={() => player.prev()}
      style="background:none; border:none; cursor:pointer; color:{theme.textMuted};
             font-size:{compact ? '15px' : '20px'}; padding:4px; transition:color 0.1s"
      onmouseenter={(e) => e.currentTarget.style.color = theme.primary}
      onmouseleave={(e) => e.currentTarget.style.color = theme.textMuted}
      title="Previous">⏮</button>

    <button
      onclick={() => player.isPlaying ? player.pause() : player.play()}
      style="background:{theme.primary}; border:none; border-radius:50%; cursor:pointer;
             width:{compact ? '34px' : '44px'}; height:{compact ? '34px' : '44px'};
             font-size:{compact ? '13px' : '18px'}; display:flex; align-items:center;
             justify-content:center; color:#000; transition:transform 0.1s; flex-shrink:0"
      onmouseenter={(e) => e.currentTarget.style.transform = 'scale(1.08)'}
      onmouseleave={(e) => e.currentTarget.style.transform = 'scale(1)'}
      title={player.isPlaying ? 'Pause' : 'Play'}>
      {player.isPlaying ? '⏸' : '▶'}
    </button>

    <button
      onclick={() => player.next()}
      style="background:none; border:none; cursor:pointer; color:{theme.textMuted};
             font-size:{compact ? '15px' : '20px'}; padding:4px; transition:color 0.1s"
      onmouseenter={(e) => e.currentTarget.style.color = theme.primary}
      onmouseleave={(e) => e.currentTarget.style.color = theme.textMuted}
      title="Next">⏭</button>
  </div>

  <!-- Volume -->
  {#if !compact || showVolume}
    <div style="display:flex; align-items:center; gap:8px">
      <button
        onclick={() => player.setVolume(player.volume > 0 ? 0 : 70)}
        style="background:none; border:none; cursor:pointer; font-size:12px;
               color:{theme.textMuted}; flex-shrink:0; padding:0"
      >{player.volume === 0 ? '🔇' : '🔈'}</button>
      <input
        type="range" min="0" max="100" value={player.volume}
        oninput={(e) => player.setVolume(Number(e.currentTarget.value))}
        style="flex:1; accent-color:{theme.primary}; cursor:pointer; height:3px"
      />
      <span style="font-size:10px; color:{theme.textMuted}; min-width:28px; text-align:right">
        {player.volume}%
      </span>
    </div>
  {/if}
</div>