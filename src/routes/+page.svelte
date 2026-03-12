<!-- src/routes/+page.svelte -->
<script>
  import { onMount } from 'svelte';
  import { player } from '$lib/stores/player.svelte.js';
  import { library } from '$lib/stores/library.svelte.js';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { audioAnalyser } from '$lib/stores/analyser.svelte.js';
  import { keyboard } from '$lib/actions/keyboard.js';
  import Library from '$lib/components/Library.svelte';
  import NowPlaying from '$lib/components/NowPlaying.svelte';
  import PlaylistPanel from '$lib/components/PlaylistPanel.svelte';
  import Visualizer from '$lib/components/Visualizer.svelte';
  import DownloadQueue from '$lib/components/DownloadQueue.svelte';
  import Equalizer from '$lib/components/Equalizer.svelte';
  import ThemeSelector from '$lib/components/ThemeSelector.svelte';
  import { Toaster } from 'svelte-sonner';
  import TitleBar from '$lib/components/TitleBar.svelte';
  import MiniPlayer from '$lib/components/MiniPlayer.svelte';

  const t = $derived(themeStore.theme);

  let appReady         = $state(false);
  let selectedPlaylist = $state(null);
  let showEQ           = $state(false);
  let miniMode         = $state(false);

  onMount(async () => {
    await Promise.all([player.init(), library.init(), audioAnalyser.init()]);
    appReady = true;
  });

  async function handleDownloadComplete() {
    await library.fetchTracks();
  }
</script>

<Toaster position="bottom-right" />

<!-- svelte-ignore a11y_no_static_element_interactions -->
<div use:keyboard style="
  width:100vw; height:100vh;
  background:{t.bg}; color:{t.text};
  font-family:'JetBrains Mono','Fira Code',ui-monospace,monospace;
  display:flex; flex-direction:column; overflow:hidden; user-select:none;
">

  <TitleBar />

  {#if !appReady}
    <!-- Boot screen -->
    <div style="flex:1; display:flex; align-items:center; justify-content:center; flex-direction:column; gap:16px">
      <div style="font-size:32px">🎵</div>
      <div style="font-size:14px; color:{t.primary}">Loading Tabi...</div>
      <div style="width:200px; height:2px; background:rgba(255,255,255,0.1); border-radius:1px; overflow:hidden">
        <div style="height:100%; background:{t.primary}; animation:loading 1.2s ease-in-out infinite" />
      </div>
    </div>
  {:else}

    <!-- ── Top bar ── -->
    <header style="
      display:flex; align-items:center; justify-content:space-between;
      padding:0 16px; height:48px; flex-shrink:0; gap:12px;
      background:linear-gradient(135deg, {t.bg} 0%, {t.primary}0d 100%);
      border-bottom:1px solid {t.primary}33;
    ">
<!-- Right controls -->
      <div style="display:flex; align-items:center; gap:4px">
        <button onclick={() => miniMode = !miniMode}
          title={miniMode ? 'Full view' : 'Mini player'}
          style="padding:5px 10px; border-radius:6px; font-size:11px; font-weight:bold; border:none; cursor:pointer; font-family:inherit;
                 background:{miniMode ? t.primary : t.primary+'1a'}; color:{miniMode ? '#000' : t.primary}">
          ⛶
        </button>
        <button onclick={() => showEQ = !showEQ}
          style="padding:5px 10px; border-radius:6px; font-size:11px; font-weight:bold; border:none; cursor:pointer; font-family:inherit;
                 background:{showEQ ? t.primary : t.primary+'1a'}; color:{showEQ ? '#000' : t.primary}">
          🎚
        </button>
        <ThemeSelector />
      </div>
    </header>

    <!-- ── Main 3-column layout ── -->
    {#if miniMode}
      <MiniPlayer onExpand={() => miniMode = false} />
    {:else}
    <div style="flex:1; display:flex; overflow:hidden; min-height:0">

      <!-- LEFT SIDEBAR: Add Music + Playlists -->
      <aside style="
        width:280px; flex-shrink:0;
        display:flex; flex-direction:column; gap:8px;
        padding:8px; border-right:1px solid {t.border};
        overflow-y:auto;
      ">
        <DownloadQueue onDownloadComplete={handleDownloadComplete} />
        <PlaylistPanel
          {selectedPlaylist}
          onPlaylistSelect={(id) => selectedPlaylist = id}
          onBackToLibrary={() => selectedPlaylist = null}
        />
        {#if showEQ}
          <Equalizer />
        {/if}
      </aside>

      <!-- CENTER: Library (top) + Visualizer (bottom) -->
      <main style="flex:1; min-width:0; display:flex; flex-direction:column; overflow:hidden">

        <!-- Library takes most of the space -->
        <div style="flex:1; min-height:0; padding:8px 8px 4px 8px; overflow:hidden">
          <Library playlistMode={!!selectedPlaylist} onBackToLibrary={() => selectedPlaylist = null} />
        </div>

        <!-- Visualizer pinned at bottom of center column -->
        <div style="height:220px; flex-shrink:0; padding:4px 8px 8px 8px">
          <Visualizer
            isPlaying={player.isPlaying}
            getFrequencyData={() => audioAnalyser.getFrequencyData()}
            getTimeDomainData={() => audioAnalyser.getTimeDomainData()}
          />
        </div>
      </main>

      <!-- RIGHT PANEL: NowPlaying -->
      <aside style="
        width:320px; flex-shrink:0;
        border-left:1px solid {t.border};
        overflow:hidden; display:flex; flex-direction:column;
      ">
        <NowPlaying />
      </aside>

    </div>
  {/if}
    {/if}
</div>

<style>
  @keyframes loading {
    from { transform: translateX(-100%); }
    to   { transform: translateX(250%); }
  }
  :global(body) { margin:0; padding:0; overflow:hidden; }
  :global(*) { box-sizing:border-box; }
  :global(::-webkit-scrollbar) { width:4px; height:4px; }
  :global(::-webkit-scrollbar-track) { background:transparent; }
  :global(::-webkit-scrollbar-thumb) { background:rgba(255,255,255,0.15); border-radius:2px; }
  :global(::-webkit-scrollbar-thumb:hover) { background:rgba(255,255,255,0.3); }
</style>