import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

class LibraryStore {
  tracks    = $state([]);
  playlists = $state([]);
  loading   = $state(true);
  query     = $state('');
  sortBy    = $state('title');
  sortOrder = $state('asc');

  byArtist = $derived(
    this.tracks.reduce((acc, t) => {
      const key = t.artist || 'Unknown Artist';
      if (!acc[key]) acc[key] = [];
      acc[key].push(t);
      return acc;
    }, {})
  );

  byAlbum = $derived(
    this.tracks.reduce((acc, t) => {
      const key = t.album || 'Unknown Album';
      if (!acc[key]) acc[key] = [];
      acc[key].push(t);
      return acc;
    }, {})
  );

  trackCount = $derived(this.tracks.length);
  #searchTimeout = null;

  async init() {
    this.loading = true;
    await Promise.all([this.fetchTracks(), this.fetchPlaylists()]);
    this.loading = false;

    await listen('playlists://updated', () => this.fetchPlaylists());
    await listen('library://refreshed', () => {
      this.fetchTracks();
      this.fetchPlaylists();
    });
  }

  async fetchTracks() {
    try {
      this.tracks = await invoke('search_tracks', {
        query:     this.query || null,
        sortBy:    this.sortBy,
        sortOrder: this.sortOrder,
      });
    } catch (e) {
      console.error('search_tracks error:', e);
    }
  }

  async importLocalFile(filePath, downloadDir) {
    const result = await invoke('import_local_file', { filePath, downloadDir });
    await this.fetchTracks();
    return result;
  }

  async fetchPlaylists() {
    try {
      this.playlists = await invoke('get_all_playlists');
    } catch (e) {
      console.error('get_all_playlists error:', e);
    }
  }

  setQuery(value) {
    this.query = value;
    clearTimeout(this.#searchTimeout);
    this.#searchTimeout = setTimeout(() => this.fetchTracks(), 150);
  }

  setSort(field) {
    if (this.sortBy === field) {
      this.sortOrder = this.sortOrder === 'asc' ? 'desc' : 'asc';
    } else {
      this.sortBy = field;
      this.sortOrder = 'asc';
    }
    this.fetchTracks();
  }

  async deleteTrack(trackId) {
    await invoke('delete_track', { trackId });
    this.tracks = this.tracks.filter(t => t.id !== trackId);
  }

  async addToPlaylist(playlistId, trackId) {
    await invoke('add_track_to_playlist', { playlistId, trackId });
    await this.fetchPlaylists();
  }

  async removeFromPlaylist(playlistId, trackId) {
    await invoke('remove_from_playlist', { playlistId, trackId });
    await this.fetchPlaylists();
  }

  async createPlaylist(name) {
    await invoke('create_playlist', { name });
  }

  updateTrackLocally(updated) {
    this.tracks = this.tracks.map(t => t.id === updated.id ? { ...t, ...updated } : t);
    this.fetchTracks();
  }
}

export const library = new LibraryStore();