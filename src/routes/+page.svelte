<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { onMount } from "svelte";

  // State Runes
  let folderPath = $state("/home/cc/Downloads/music"); 
  let library = $state<any[]>([]);
  let loading = $state(false);
  let currentSong = $state<any>(null);
  let isPaused = $state(false);
  let volume = $state(70);
  let currentView = $state("albums");

  onMount(async () => {
    try {
      const saved = await invoke<any[]>("load_library");
      if (saved && saved.length > 0) {
        library = saved;
      } else {
        // No saved library, auto-scan default folder
        loading = true;
        try {
          const result = await invoke<any[]>("scan_library", { path: folderPath });
          library = result;
          await invoke("save_library", { songs: result });
        } catch (e) {
          console.log("Auto-scan failed: " + e);
        } finally {
          loading = false;
        }
      }
    } catch (e) {
      console.log("No saved library, starting fresh");
    }
  });

  // --- Grouping Logic ---
  let albums = $derived(
    library.reduce((acc, song) => {
      const name = song.album || "Unknown Album";
      if (!acc[name]) acc[name] = [];
      acc[name].push(song);
      return acc;
    }, {} as Record<string, any[]>)
  );

  let artists = $derived(
    library.reduce((acc, song) => {
      const name = song.artist || "Unknown Artist";
      if (!acc[name]) acc[name] = [];
      acc[name].push(song);
      return acc;
    }, {} as Record<string, any[]>)
  );

  let displayGroups = $derived(currentView === "albums" ? albums : artists);

  // --- Actions ---
  async function startScan() {
    if (!folderPath) return alert("Please enter a folder path!");
    loading = true;
    try {
      const result = await invoke<any[]>("scan_library", { path: folderPath });
      library = result;
      await invoke("save_library", { songs: result });
    } catch (e) {
      alert("Scan failed: " + e);
    } finally {
      loading = false;
    }
  }

  async function playSong(song: any) {
    currentSong = song;
    isPaused = false;
    await invoke("play_song", { path: song.path });
  }

  async function togglePlay() {
    isPaused = await invoke("toggle_pause");
  }

  async function updateVolume() {
    await invoke("set_volume", { volume: volume / 100 });
  }
</script>

<div class="app-container">
  <aside class="sidebar">
    <h2 class="logo">Tabi Rust</h2>
    
    <div class="search-box">
      <input bind:value={folderPath} placeholder="Music directory..." />
      <button class="btn-primary" onclick={startScan} disabled={loading}>
        {loading ? "Scanning..." : "Sync Library"}
      </button>
    </div>

    <nav class="nav">
      <div class="nav-label">YOUR LIBRARY</div>
      <button 
        class="nav-link" 
        class:active={currentView === "albums"} 
        onclick={() => currentView = "albums"}
      >
        Albums
      </button>
      <button 
        class="nav-link" 
        class:active={currentView === "artists"} 
        onclick={() => currentView = "artists"}
      >
        Artists
      </button>
    </nav>
  </aside>

  <main class="content">
    {#if loading}
      <div class="empty-state">
        <p>Scanning your music library...</p>
      </div>
    {:else if library.length === 0}
      <div class="empty-state">
        <p>Your library is empty. Sync a folder to begin.</p>
      </div>
    {/if}

    {#each Object.entries(displayGroups) as [groupName, songs]}
      <div class="album-group">
        <h3 class="group-title">{groupName}</h3>
        <div class="song-grid">
          {#each songs as song}
            <button 
              class="song-card" 
              onclick={() => playSong(song)} 
              class:active={currentSong?.path === song.path}
            >
              {#if song.cover}
                <img src={song.cover} alt="art" />
              {:else}
                <div class="no-cover">♪</div>
              {/if}
              <div class="info">
                <span class="title">{song.title}</span>
                <span class="artist">{song.artist}</span>
              </div>
            </button>
          {/each}
        </div>
      </div>
    {/each}
  </main>

  <footer class="player-bar">
    <div class="current-track">
      {#if currentSong}
        <img src={currentSong.cover} alt="" class="mini-art" />
        <div class="track-meta">
          <span class="t-name">{currentSong.title}</span>
          <span class="a-name">{currentSong.artist}</span>
        </div>
      {/if}
    </div>

    <div class="player-controls">
      <button class="play-btn" onclick={togglePlay}>
        {isPaused ? "▶" : "⏸"}
      </button>
    </div>

    <div class="volume-box">
      <span class="icon">Vol</span>
      <input 
        type="range" 
        min="0" 
        max="100" 
        value={volume}
        oninput={(e) => {
          volume = Number((e.target as HTMLInputElement).value);
          updateVolume();
        }}
      />
    </div>
  </footer>
</div>

<style>
  :global(body) { margin: 0; background: #000; color: white; font-family: 'Inter', sans-serif; overflow: hidden; }
  
  .app-container { display: flex; flex-direction: row; height: 100vh; width: 100vw; }
  
  .sidebar { width: 260px; background: #000; padding: 1.5rem; border-right: 1px solid #282828; display: flex; flex-direction: column; }
  .logo { font-size: 1.5rem; margin-bottom: 2rem; color: #1db954; font-weight: 900; }
  .search-box { display: flex; flex-direction: column; gap: 0.7rem; margin-bottom: 2rem; }
  
  input[type="text"] { background: #282828; border: none; border-radius: 4px; padding: 12px; color: white; width: calc(100% - 24px); }
  .btn-primary { background: #fff; color: #000; font-weight: bold; border: none; padding: 12px; border-radius: 25px; cursor: pointer; transition: transform 0.1s; }
  .btn-primary:active { transform: scale(0.95); }
  
  .nav { display: flex; flex-direction: column; gap: 0.2rem; }
  .nav-label { font-size: 0.7rem; color: #b3b3b3; letter-spacing: 1px; margin-bottom: 0.8rem; font-weight: bold; }
  .nav-link { background: none; border: none; color: #b3b3b3; text-align: left; padding: 10px 12px; font-size: 0.9rem; cursor: pointer; border-radius: 4px; transition: 0.2s; }
  .nav-link:hover { color: #fff; background: #1a1a1a; }
  .nav-link.active { color: #fff; background: #282828; font-weight: bold; }

  .content { flex: 1; overflow-y: auto; padding: 2rem; padding-bottom: 120px; background: linear-gradient(to bottom, #181818, #121212); }
  .album-group { margin-bottom: 3.5rem; }
  .group-title { border-bottom: 1px solid #333; padding-bottom: 0.5rem; margin-bottom: 1.5rem; color: #fff; font-size: 1.4rem; }

  .song-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(170px, 1fr)); gap: 1.5rem; }
  .song-card { background: #181818; border: none; color: white; padding: 15px; border-radius: 8px; cursor: pointer; transition: 0.3s; text-align: left; }
  .song-card:hover { background: #282828; transform: translateY(-5px); }
  .song-card.active { background: #333; }
  .song-card img { width: 100%; aspect-ratio: 1; border-radius: 4px; object-fit: cover; margin-bottom: 10px; box-shadow: 0 8px 16px rgba(0,0,0,0.3); }
  .no-cover { width: 100%; aspect-ratio: 1; background: #333; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 2.5rem; margin-bottom: 10px; }

  .info .title { display: block; font-weight: bold; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; margin-bottom: 4px; }
  .info .artist { font-size: 0.8rem; color: #b3b3b3; }

  .player-bar { 
    position: fixed; bottom: 0; left: 0; right: 0; height: 90px; 
    background: #121212; border-top: 1px solid #282828; 
    display: flex; align-items: center; justify-content: space-between; padding: 0 1.5rem;
    z-index: 100;
  }
  .current-track { display: flex; align-items: center; gap: 1rem; width: 30%; min-width: 200px; }
  .mini-art { width: 56px; height: 56px; border-radius: 4px; box-shadow: 0 0 10px rgba(0,0,0,0.5); }
  .track-meta { display: flex; flex-direction: column; overflow: hidden; }
  .t-name { font-weight: bold; font-size: 0.9rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
  .a-name { font-size: 0.75rem; color: #b3b3b3; }

  .player-controls { display: flex; align-items: center; gap: 1.5rem; }
  .play-btn { background: #fff; border: none; width: 42px; height: 42px; border-radius: 50%; cursor: pointer; font-size: 1rem; transition: transform 0.1s; }
  .play-btn:hover { transform: scale(1.05); }

  .volume-box { display: flex; align-items: center; gap: 12px; width: 30%; justify-content: flex-end; }
  input[type="range"] { accent-color: #1db954; cursor: pointer; width: 100px; }

  .empty-state { text-align: center; margin-top: 15vh; color: #b3b3b3; font-size: 1.1rem; }
</style>