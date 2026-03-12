<script>
  import { library } from '$lib/stores/library.svelte.js';
  import { player } from '$lib/stores/player.svelte.js';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { invoke } from '@tauri-apps/api/core';
  import { ListMusic, Plus, Music, ChevronRight, ChevronDown, Play, X } from 'lucide-svelte';
  import { toast } from 'svelte-sonner';

  let { selectedPlaylist = null, onPlaylistSelect = null, onBackToLibrary = null } = $props();

  const t = $derived(themeStore.theme);

  let showCreateForm   = $state(false);
  let newPlaylistName  = $state('');
  let expandedPlaylist = $state(null);
  let playlistTracks   = $state({});

  async function loadPlaylistTracks(id) {
    try {
      const tracks = await invoke('get_playlist_tracks', { playlistId: id });
      playlistTracks = { ...playlistTracks, [id]: tracks };
    } catch (e) {
      console.error(e);
    }
  }

  function toggleExpand(id, e) {
    e.stopPropagation();
    if (expandedPlaylist === id) {
      expandedPlaylist = null;
    } else {
      expandedPlaylist = id;
      loadPlaylistTracks(id);
    }
  }

  function openInLibrary(id) {
    onPlaylistSelect?.(id);
  }

  async function handleCreate() {
    if (!newPlaylistName.trim()) return;
    try {
      await library.createPlaylist(newPlaylistName.trim());
      toast.success('Playlist created');
      newPlaylistName = '';
      showCreateForm = false;
    } catch {
      toast.error('Failed to create playlist');
    }
  }

  function playPlaylist(tracks) {
    if (!tracks?.length) return;
    const ids = tracks.map(t => t.id);
    player.setQueue(ids, 0);
    player.playTrack(ids[0]);
  }
</script>

<div style="border:1px solid {t.border};border-radius:8px;padding:10px;background:{t.bg};display:flex;flex-direction:column;gap:6px">

  <div style="display:flex;align-items:center;justify-content:space-between;padding:0 2px">
    <div style="display:flex;align-items:center;gap:6px">
      <ListMusic size={14} style="color:{t.primary}" />
      <span style="font-size:13px;font-weight:bold;color:{t.primary}">Playlists</span>
    </div>
    <button onclick={() => showCreateForm = !showCreateForm}
      style="padding:3px;border-radius:4px;background:transparent;border:none;cursor:pointer;color:{t.primary}">
      <Plus size={14} />
    </button>
  </div>

  {#if showCreateForm}
    <div style="display:flex;flex-direction:column;gap:6px;padding:4px 0">
      <input
        type="text"
        bind:value={newPlaylistName}
        onkeydown={(e) => {
          if (e.key === 'Enter') handleCreate();
          else if (e.key === 'Escape') { showCreateForm = false; newPlaylistName = ''; }
        }}
        placeholder="Playlist name..."
        autofocus
        style="width:100%;padding:6px 8px;font-size:12px;border-radius:4px;border:1px solid {t.border};
               background:rgba(255,255,255,0.06);color:{t.text};outline:none;box-sizing:border-box;font-family:inherit"
      />
      <div style="display:flex;gap:6px">
        <button onclick={handleCreate}
          style="flex:1;padding:4px;border-radius:4px;font-size:12px;font-weight:bold;background:{t.primary};border:none;cursor:pointer;color:#000;font-family:inherit">
          Create
        </button>
        <button onclick={() => { showCreateForm = false; newPlaylistName = ''; }}
          style="flex:1;padding:4px;border-radius:4px;font-size:12px;background:transparent;border:1px solid {t.border};cursor:pointer;color:{t.text};font-family:inherit">
          Cancel
        </button>
      </div>
    </div>
  {/if}

  <!-- Playlist list -->
  {#if library.playlists.length === 0}
    <div style="text-align:center;padding:16px 0;color:{t.textMuted}">
      <Music size={20} style="margin:0 auto 6px;opacity:0.4" />
      <p style="font-size:11px">No playlists yet</p>
    </div>
  {:else}
    {#each library.playlists as pl (pl.id)}
      <div>
        <div style="display:flex;align-items:center;border-radius:4px;overflow:hidden;
                    background:{selectedPlaylist === pl.id ? `${t.primary}22` : 'transparent'}">

          <!-- Chevron to expand inline tracks -->
          <button onclick={(e) => toggleExpand(pl.id, e)}
            style="padding:5px 4px;background:transparent;border:none;cursor:pointer;display:flex;align-items:center;color:{t.textMuted};flex-shrink:0">
            {#if expandedPlaylist === pl.id}
              <ChevronDown size={13} />
            {:else}
              <ChevronRight size={13} />
            {/if}
          </button>

          <!-- Playlist name — click to open in library -->
          <button
            onclick={() => openInLibrary(pl.id)}
            style="flex:1;min-width:0;text-align:left;background:transparent;border:none;cursor:pointer;
                   padding:5px 4px;color:{selectedPlaylist === pl.id ? t.primary : t.text};font-family:inherit"
            onmouseenter={(e) => { if (selectedPlaylist !== pl.id) e.currentTarget.style.color = t.primary; }}
            onmouseleave={(e) => { if (selectedPlaylist !== pl.id) e.currentTarget.style.color = t.text; }}
          >
            <div style="font-size:12px;font-weight:{selectedPlaylist===pl.id?'bold':'normal'};overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
              {pl.name}
            </div>
            <div style="font-size:10px;color:{t.textMuted}">{pl.track_count || 0} tracks</div>
          </button>

          <!-- Play all button -->
          {#if expandedPlaylist === pl.id && playlistTracks[pl.id]?.length}
            <button
              onclick={() => playPlaylist(playlistTracks[pl.id])}
              title="Play all"
              style="padding:5px 6px;background:transparent;border:none;cursor:pointer;color:{t.primary};flex-shrink:0">
              <Play size={11} fill="currentColor" />
            </button>
          {/if}
        </div>

        <!-- Inline track list when expanded -->
        {#if expandedPlaylist === pl.id}
          <div style="margin-left:20px;margin-top:2px;display:flex;flex-direction:column;gap:1px">
            {#if !playlistTracks[pl.id]}
              <div style="font-size:11px;padding:4px 8px;color:{t.textMuted}">Loading...</div>
            {:else if playlistTracks[pl.id].length === 0}
              <div style="font-size:11px;padding:4px 8px;color:{t.textMuted}">Empty playlist</div>
            {:else}
              {#each playlistTracks[pl.id] as track (track.id)}
                <div
                  style="font-size:11px;padding:3px 6px;border-radius:3px;display:flex;align-items:center;gap:5px;color:{t.text}"
                  onmouseenter={(e) => e.currentTarget.style.background = 'rgba(255,255,255,0.06)'}
                  onmouseleave={(e) => e.currentTarget.style.background = 'transparent'}
                >
                  <button onclick={() => player.playTrack(track.id)}
                    style="background:transparent;border:none;cursor:pointer;padding:0;flex-shrink:0;display:flex;align-items:center">
                    <Play size={9} style="color:{t.primary}" fill="currentColor" />
                  </button>
                  <div onclick={() => player.playTrack(track.id)}
                    style="flex:1;overflow:hidden;text-overflow:ellipsis;white-space:nowrap;cursor:pointer"
                    title={track.title}>{track.title}</div>
                  <div style="font-size:10px;color:{t.textMuted};flex-shrink:0;max-width:55px;overflow:hidden;text-overflow:ellipsis;white-space:nowrap">
                    {track.artist ?? ''}
                  </div>
                  <button
                    onclick={async () => { await library.removeFromPlaylist(pl.id, track.id); await loadPlaylistTracks(pl.id); }}
                    title="Remove from playlist"
                    style="background:transparent;border:none;cursor:pointer;padding:0;flex-shrink:0;
                           display:flex;align-items:center;color:{t.textMuted};opacity:0.5"
                    onmouseenter={(e) => { e.currentTarget.style.color='#ef4444'; e.currentTarget.style.opacity='1'; }}
                    onmouseleave={(e) => { e.currentTarget.style.color=t.textMuted; e.currentTarget.style.opacity='0.5'; }}
                  ><X size={9} /></button>
                </div>
              {/each}
            {/if}
          </div>
        {/if}
      </div>
    {/each}
  {/if}
</div>