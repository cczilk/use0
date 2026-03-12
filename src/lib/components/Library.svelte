<script>
  import { library } from '$lib/stores/library.svelte.js';
  import { player } from '$lib/stores/player.svelte.js';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { toast } from 'svelte-sonner';
  import EditMetadataModal from './EditMetadataModal.svelte';

  let { selectedPlaylist = null, onBackToLibrary = null } = $props();

  const theme = $derived(themeStore.theme);

  let tab            = $state('tracks');  // 'tracks' | 'albums' | 'artists'
  let editingTrack   = $state(null);
  let contextMenu    = $state(null);      // { x, y, track }
  let playlistMenu   = $state(false);
  let selectedAlbum  = $state(null);
  let selectedArtist = $state(null);

  // playlist mode
  let playlistTracks  = $state(null);
  let playlistName    = $state('');
  let loadingPlaylist = $state(false);

  $effect(() => {
    if (selectedPlaylist) {
      loadingPlaylist = true;
      const pl = library.playlists.find(p => p.id === selectedPlaylist);
      playlistName = pl?.name ?? 'Playlist';
      invoke('get_playlist_tracks', { playlistId: selectedPlaylist })
        .then(t => { playlistTracks = t; })
        .catch(() => { playlistTracks = []; })
        .finally(() => { loadingPlaylist = false; });
    } else {
      playlistTracks = null;
      playlistName   = '';
    }
  });

  const baseTracks = $derived(playlistTracks ?? library.tracks);

  const displayTracks = $derived(
    selectedAlbum  ? baseTracks.filter(t => (t.album  ?? 'Unknown Album')  === selectedAlbum)  :
    selectedArtist ? baseTracks.filter(t => (t.artist ?? 'Unknown Artist') === selectedArtist) :
    baseTracks
  );

  const albums = $derived.by(() => {
    const map = new Map();
    for (const t of baseTracks) {
      const key = t.album ?? 'Unknown Album';
      if (!map.has(key)) map.set(key, { name: key, tracks: [], art: null });
      const a = map.get(key);
      a.tracks.push(t);
      if (!a.art && t.thumbnail_path) a.art = t.thumbnail_path;
    }
    return [...map.values()].sort((a, b) => a.name.localeCompare(b.name));
  });

  const artists = $derived.by(() => {
    const map = new Map();
    for (const t of baseTracks) {
      const key = t.artist ?? 'Unknown Artist';
      if (!map.has(key)) map.set(key, { name: key, count: 0, art: null });
      const a = map.get(key);
      a.count++;
      if (!a.art && t.thumbnail_path) a.art = t.thumbnail_path;
    }
    return [...map.values()].sort((a, b) => a.name.localeCompare(b.name));
  });

  const headerLabel = $derived(
    selectedPlaylist ? `♫ ${playlistName}` :
    selectedAlbum   ? `💿 ${selectedAlbum}` :
    selectedArtist  ? `🎤 ${selectedArtist}` :
    '♫ library'
  );

  const headerCount = $derived(
    (tab === 'albums'  && !selectedAlbum && !selectedArtist) ? `${albums.length} albums` :
    (tab === 'artists' && !selectedAlbum && !selectedArtist) ? `${artists.length} artists` :
    `${displayTracks.length} tracks`
  );

  const canGoBack = $derived(!!(
    (selectedPlaylist && onBackToLibrary) || selectedAlbum || selectedArtist
  ));

  function goBack() {
    if (selectedArtist) { selectedArtist = null; return; }
    if (selectedAlbum)  { selectedAlbum  = null; return; }
    onBackToLibrary?.();
  }

  function openAlbum(name)  { selectedAlbum  = name; tab = 'tracks'; }
  function openArtist(name) { selectedArtist = name; tab = 'tracks'; }

  function playTrack(track) {
    const ids = displayTracks.map(t => t.id);
    const idx = ids.indexOf(track.id);
    player.setQueue(ids, idx);
    player.playTrack(track.id);
  }

  function playAlbum(album) {
    const ids = album.tracks.map(t => t.id);
    player.setQueue(ids, 0);
    player.playTrack(ids[0]);
  }

  function openContextMenu(e, track) {
    e.preventDefault();
    e.stopPropagation();
    playlistMenu = false;
    contextMenu = { x: e.clientX, y: e.clientY, track };
  }

  function closeContextMenu() { contextMenu = null; playlistMenu = false; }

  async function handleAddToPlaylist(playlistId) {
    try {
      await library.addToPlaylist(playlistId, contextMenu.track.id);
      toast.success('Added to playlist');
    } catch {
      toast.error('Failed to add to playlist');
    }
    closeContextMenu();
  }

  async function handleDelete(track) {
    closeContextMenu();
    if (!confirm(`Delete "${track.title}"?`)) return;
    try {
      await library.deleteTrack(track.id);
      toast.success('Track deleted');
    } catch {
      toast.error('Failed to delete track');
    }
  }

  async function handleSetArtwork(track) {
    closeContextMenu();
    try {
      const selected = await open({
        multiple: false,
        filters: [{ name: 'Image', extensions: ['png','jpg','jpeg','webp'] }]
      });
      if (!selected) return;
      const downloadDir = await invoke('get_downloads_dir');
      await invoke('update_track_artwork', { trackId: track.id, imagePath: selected, downloadDir });
      await library.fetchTracks();
      toast.success('Artwork updated');
    } catch {
      toast.error('Failed to set artwork');
    }
  }

  function handleEditMetadata(track) {
    closeContextMenu();
    editingTrack = track;
  }

  function formatDuration(secs) {
    if (!secs) return '--:--';
    return `${Math.floor(secs / 60)}:${String(Math.floor(secs % 60)).padStart(2, '0')}`;
  }

  const sortIcon = (field) => {
    if (library.sortBy !== field) return '';
    return library.sortOrder === 'asc' ? ' ↑' : ' ↓';
  };

  function artUrl(path) {
    if (!path) return null;
    return `asset://localhost/${encodeURIComponent(path).replace(/%2F/g, '/')}`;
  }

  const showTracks = $derived(tab === 'tracks' || !!selectedAlbum || !!selectedArtist);
</script>

{#if contextMenu}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <!-- svelte-ignore a11y_no_static_element_interactions -->
  <div style="position:fixed;inset:0;z-index:90" onclick={closeContextMenu}></div>

  <div style="position:fixed;left:{contextMenu.x}px;top:{contextMenu.y}px;
              z-index:100;background:{theme.bg};border:1px solid {theme.border};
              border-radius:8px;padding:4px;min-width:170px;
              box-shadow:0 8px 24px rgba(0,0,0,0.6);backdrop-filter:blur(12px)">

    {#each [
      { label: '▶  Play',          fn: () => { playTrack(contextMenu.track); closeContextMenu(); } },
      { label: '✏  Edit metadata', fn: () => handleEditMetadata(contextMenu.track) },
      { label: '🖼  Set artwork',   fn: () => handleSetArtwork(contextMenu.track) },
    ] as item (item.label)}
      <button onclick={item.fn}
        style="width:100%;text-align:left;padding:7px 12px;border:none;border-radius:5px;
               background:transparent;cursor:pointer;font-size:12px;color:{theme.text};font-family:inherit"
        onmouseenter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.08)'}
        onmouseleave={(e) => e.currentTarget.style.background = 'transparent'}
      >{item.label}</button>
    {/each}

    <!-- Add to playlist with submenu -->
    <div style="position:relative">
      <button onclick={() => playlistMenu = !playlistMenu}
        style="width:100%;text-align:left;padding:7px 12px;border:none;border-radius:5px;
               background:{playlistMenu ? 'rgba(255,255,255,0.08)' : 'transparent'};
               cursor:pointer;font-size:12px;display:flex;align-items:center;
               justify-content:space-between;color:{theme.text};font-family:inherit"
        onmouseenter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.08)'}
        onmouseleave={(e) => { if (!playlistMenu) e.currentTarget.style.background = 'transparent'; }}
      >
        <span>➕  Add to playlist</span>
        <span style="opacity:0.5;font-size:10px">▶</span>
      </button>

      {#if playlistMenu}
        <div style="position:absolute;left:100%;top:0;margin-left:4px;
                    background:{theme.bg};border:1px solid {theme.border};
                    border-radius:8px;padding:4px;min-width:150px;
                    box-shadow:0 8px 24px rgba(0,0,0,0.6);backdrop-filter:blur(12px);z-index:110;
                    max-height:200px;overflow-y:auto">
          {#if library.playlists.length === 0}
            <div style="padding:8px 12px;font-size:11px;color:{theme.textMuted}">No playlists yet</div>
          {:else}
            {#each library.playlists as pl (pl.id)}
              <button onclick={() => handleAddToPlaylist(pl.id)}
                style="width:100%;text-align:left;padding:7px 12px;border:none;border-radius:5px;
                       background:transparent;cursor:pointer;font-size:12px;color:{theme.text};
                       font-family:inherit;overflow:hidden;text-overflow:ellipsis;white-space:nowrap"
                onmouseenter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.08)'}
                onmouseleave={(e) => e.currentTarget.style.background = 'transparent'}
              >{pl.name}</button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>

    <div style="height:1px;background:{theme.border};margin:3px 0"></div>

    <button onclick={() => handleDelete(contextMenu.track)}
      style="width:100%;text-align:left;padding:7px 12px;border:none;border-radius:5px;
             background:transparent;cursor:pointer;font-size:12px;color:#ef4444;font-family:inherit"
      onmouseenter={(e) => e.currentTarget.style.background = 'rgba(239,68,68,0.15)'}
      onmouseleave={(e) => e.currentTarget.style.background = 'transparent'}
    >🗑  Delete</button>
  </div>
{/if}

<div style="display:flex;flex-direction:column;height:100%;
            background:{theme.bg};border-radius:8px;border:1px solid {theme.border};overflow:hidden">

  <div style="padding:10px 16px 0;border-bottom:1px solid {theme.border};flex-shrink:0">
    <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:6px">
      <div style="display:flex;align-items:center;gap:8px">
        {#if canGoBack}
          <button onclick={goBack}
            style="background:none;border:none;cursor:pointer;color:{theme.textMuted};
                   font-size:15px;padding:0;line-height:1"
            onmouseenter={(e) => e.currentTarget.style.color = theme.primary}
            onmouseleave={(e) => e.currentTarget.style.color = theme.textMuted}
            title="Back">←</button>
        {/if}
        <span style="color:{theme.primary};font-size:13px;font-weight:bold;
                     text-transform:uppercase;letter-spacing:0.05em">{headerLabel}</span>
      </div>
      <span style="font-size:11px;color:{theme.textMuted}">{headerCount}</span>
    </div>

    {#if !selectedPlaylist && !selectedAlbum && !selectedArtist}
      <div style="display:flex;gap:0">
        {#each [['tracks','Tracks'],['albums','Albums'],['artists','Artists']] as [id, label] (id)}
          <button onclick={() => tab = id}
            style="padding:5px 16px;border:none;cursor:pointer;font-size:12px;font-family:inherit;
                   font-weight:{tab===id ? 'bold' : 'normal'};
                   background:transparent;
                   color:{tab===id ? theme.primary : theme.textMuted};
                   border-bottom:{tab===id ? `2px solid ${theme.primary}` : '2px solid transparent'};
                   transition:all 0.15s">
            {label}
          </button>
        {/each}
      </div>
    {/if}

    {#if showTracks}
      <div style="padding:6px 0 8px">
        <input type="text" placeholder="Search tracks..."
          value={library.query}
          oninput={(e) => library.setQuery(e.currentTarget.value)}
          style="width:100%;padding:7px 12px;background:rgba(255,255,255,0.07);
                 border:1px solid {theme.border};border-radius:6px;color:{theme.text};
                 font-size:12px;box-sizing:border-box;outline:none;font-family:inherit"
        />
      </div>
    {:else}
      <div style="height:8px"></div>
    {/if}
  </div>

  {#if showTracks}
    <!-- Column headers -->
    <div style="display:grid;grid-template-columns:2fr 1fr 1fr 56px;gap:8px;
                padding:5px 16px;border-bottom:1px solid {theme.border};flex-shrink:0">
      {#each [['title','Title'],['artist','Artist'],['album','Album'],['duration','Time']] as [f, l] (f)}
        <button onclick={() => library.setSort(f)}
          style="text-align:left;background:none;border:none;cursor:pointer;font-size:10px;
                 font-weight:bold;text-transform:uppercase;letter-spacing:0.05em;font-family:inherit;
                 color:{library.sortBy===f ? theme.primary : theme.textMuted}">
          {l}{sortIcon(f)}
        </button>
      {/each}
    </div>

    <div style="flex:1;overflow-y:auto">
      {#if library.loading}
        <div style="display:flex;align-items:center;justify-content:center;height:100%;color:{theme.textMuted}">Loading...</div>
      {:else if displayTracks.length === 0}
        <div style="display:flex;align-items:center;justify-content:center;height:100%;color:{theme.textMuted}">
          <div style="text-align:center">
            <div style="font-size:36px;margin-bottom:8px">🎵</div>
            <p style="font-size:13px">{library.query ? 'No results' : 'No tracks here'}</p>
          </div>
        </div>
      {:else}
        {#each displayTracks as track (track.id)}
          {@const isActive = player.currentTrack?.id === track.id}
          <!-- svelte-ignore a11y_no_static_element_interactions -->
          <div
            onclick={() => playTrack(track)}
            oncontextmenu={(e) => openContextMenu(e, track)}
            onkeydown={(e) => e.key === 'Enter' && playTrack(track)}
            role="row" tabindex="0"
            style="display:grid;grid-template-columns:2fr 1fr 1fr 56px;gap:8px;
                   padding:6px 16px;cursor:pointer;
                   border-bottom:1px solid rgba(255,255,255,0.03);
                   background:{isActive ? `${theme.primary}18` : 'transparent'}"
            onmouseenter={(e) => { if (!isActive) e.currentTarget.style.background = 'rgba(255,255,255,0.04)'; }}
            onmouseleave={(e) => { if (!isActive) e.currentTarget.style.background = 'transparent'; }}
          >
            <div style="overflow:hidden;display:flex;align-items:center;gap:6px">
              {#if isActive}<span style="color:{theme.primary};font-size:9px;flex-shrink:0">▶</span>{/if}
              <span style="font-size:13px;color:{isActive ? theme.primary : theme.text};
                           overflow:hidden;text-overflow:ellipsis;white-space:nowrap;
                           font-weight:{isActive ? 'bold' : 'normal'}">{track.title}</span>
            </div>
            <div style="font-size:12px;color:{theme.textMuted};overflow:hidden;text-overflow:ellipsis;white-space:nowrap;display:flex;align-items:center">{track.artist ?? '—'}</div>
            <div style="font-size:12px;color:{theme.textMuted};overflow:hidden;text-overflow:ellipsis;white-space:nowrap;display:flex;align-items:center">{track.album ?? '—'}</div>
            <div style="font-size:12px;color:{theme.textMuted};display:flex;align-items:center">{formatDuration(track.duration)}</div>
          </div>
        {/each}
      {/if}
    </div>

  <!-- ── ALBUMS ───────────────────────────────────────────────── -->
  {:else if tab === 'albums'}
    <div style="flex:1;overflow-y:auto;padding:12px">
      {#if albums.length === 0}
        <div style="display:flex;align-items:center;justify-content:center;height:100%;color:{theme.textMuted}">
          <div style="text-align:center"><div style="font-size:36px;margin-bottom:8px">💿</div><p style="font-size:13px">No albums yet</p></div>
        </div>
      {:else}
        <div style="display:grid;grid-template-columns:repeat(auto-fill,minmax(130px,1fr));gap:12px">
          {#each albums as album (album.name)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div onclick={() => openAlbum(album.name)} role="button" tabindex="0"
              onkeydown={(e) => e.key==='Enter' && openAlbum(album.name)}
              style="cursor:pointer;border-radius:8px;overflow:hidden;
                     border:1px solid {theme.border};background:rgba(255,255,255,0.03);transition:all 0.15s"
              onmouseenter={(e) => { e.currentTarget.style.background='rgba(255,255,255,0.08)'; e.currentTarget.style.borderColor=theme.primary; }}
              onmouseleave={(e) => { e.currentTarget.style.background='rgba(255,255,255,0.03)'; e.currentTarget.style.borderColor=theme.border; }}
            >
              <div style="aspect-ratio:1;position:relative;overflow:hidden;
                           background:linear-gradient(135deg,{theme.primary}22,{theme.primary}44)">
                {#if album.art}
                  <img src={artUrl(album.art)} alt={album.name}
                    style="width:100%;height:100%;object-fit:cover;display:block" />
                {:else}
                  <div style="width:100%;height:100%;display:flex;align-items:center;justify-content:center;font-size:32px">💿</div>
                {/if}
                <!-- Play overlay on hover -->
                <!-- svelte-ignore a11y_no_static_element_interactions -->
                <div style="position:absolute;inset:0;display:flex;align-items:center;justify-content:center;
                             background:rgba(0,0,0,0);opacity:0;transition:all 0.15s"
                  onmouseenter={(e) => { e.currentTarget.style.opacity='1'; e.currentTarget.style.background='rgba(0,0,0,0.45)'; }}
                  onmouseleave={(e) => { e.currentTarget.style.opacity='0'; e.currentTarget.style.background='rgba(0,0,0,0)'; }}
                  onclick={(e) => { e.stopPropagation(); playAlbum(album); }}
                  role="button" tabindex="-1"
                  onkeydown={(e) => e.key==='Enter' && playAlbum(album)}
                >
                  <div style="width:36px;height:36px;border-radius:50%;background:{theme.primary};
                               display:flex;align-items:center;justify-content:center;font-size:13px;color:#000">▶</div>
                </div>
              </div>
              <div style="padding:8px">
                <div style="font-size:12px;font-weight:bold;color:{theme.text};
                             overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{album.name}</div>
                <div style="font-size:10px;color:{theme.textMuted};margin-top:2px">{album.tracks.length} tracks</div>
              </div>
            </div>
          {/each}
        </div>
      {/if}
    </div>

  <!-- ── ARTISTS ──────────────────────────────────────────────── -->
  {:else if tab === 'artists'}
    <div style="flex:1;overflow-y:auto;padding:8px">
      {#if artists.length === 0}
        <div style="display:flex;align-items:center;justify-content:center;height:100%;color:{theme.textMuted}">
          <div style="text-align:center"><div style="font-size:36px;margin-bottom:8px">🎤</div><p style="font-size:13px">No artists yet</p></div>
        </div>
      {:else}
        <div style="display:flex;flex-direction:column;gap:1px">
          {#each artists as artist (artist.name)}
            <!-- svelte-ignore a11y_no_static_element_interactions -->
            <div onclick={() => openArtist(artist.name)} role="button" tabindex="0"
              onkeydown={(e) => e.key==='Enter' && openArtist(artist.name)}
              style="display:flex;align-items:center;gap:12px;padding:8px 12px;
                     border-radius:6px;cursor:pointer;transition:background 0.1s"
              onmouseenter={(e) => e.currentTarget.style.background='rgba(255,255,255,0.06)'}
              onmouseleave={(e) => e.currentTarget.style.background='transparent'}
            >
              <div style="width:40px;height:40px;border-radius:50%;flex-shrink:0;overflow:hidden;
                           background:linear-gradient(135deg,{theme.primary}33,{theme.primary}66)">
                {#if artist.art}
                  <img src={artUrl(artist.art)} alt={artist.name} style="width:100%;height:100%;object-fit:cover" />
                {:else}
                  <div style="width:100%;height:100%;display:flex;align-items:center;justify-content:center;font-size:18px">🎤</div>
                {/if}
              </div>
              <div style="flex:1;min-width:0">
                <div style="font-size:13px;font-weight:500;color:{theme.text};
                             overflow:hidden;text-overflow:ellipsis;white-space:nowrap">{artist.name}</div>
                <div style="font-size:11px;color:{theme.textMuted}">{artist.count} track{artist.count !== 1 ? 's' : ''}</div>
              </div>
              <div style="color:{theme.textMuted};font-size:14px">›</div>
            </div>
          {/each}
        </div>
      {/if}
    </div>
  {/if}
</div>

{#if editingTrack}
  <EditMetadataModal track={editingTrack} onclose={() => editingTrack = null} />
{/if}