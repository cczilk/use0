import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
class PlayerStore {
  isPlaying     = $state(false);
  currentTime   = $state(0);
  duration      = $state(0);
  volume        = $state(70);
  currentTrack  = $state(null);
  queue         = $state([]);
  currentIndex  = $state(-1);
  autoplay      = $state(true);
  shuffle       = $state(false);
  bpm           = $state(120);
  progressPercent   = $derived(this.duration > 0 ? (this.currentTime / this.duration) * 100 : 0);
  formattedTime     = $derived(this.#fmt(this.currentTime));
  formattedDuration = $derived(this.#fmt(this.duration));

  #fmt(secs) {
    const m = Math.floor(secs / 60);
    const s = Math.floor(secs % 60);
    return `${m}:${s.toString().padStart(2, '0')}`;
  }

  async init() {
    try {
      const snap = await invoke('player_get_state');
      this.isPlaying   = snap.is_playing;
      this.currentTime = snap.position_secs;
      this.duration    = snap.duration_secs;
      this.volume      = snap.volume;
      this.shuffle     = snap.shuffle;
      this.autoplay    = snap.autoplay;
    } catch (e) {
      console.warn('player_get_state failed:', e);
    }

    await listen('player://state-changed', ({ payload }) => {
      this.isPlaying   = payload.is_playing;
      this.currentTime = payload.position_secs;
      this.volume      = Math.round(payload.volume * 100);
    });

    await listen('player://track-changed', ({ payload }) => {
      this.currentTrack = {
        id:            payload.track_id,
        title:         payload.title,
        artist:        payload.artist,
        album:         payload.album,
        thumbnailPath: payload.thumbnail_path ?? null,
      };
      this.duration    = payload.duration_secs;
      this.currentTime = 0;
    });

    await listen('player://queue-updated', ({ payload }) => {
      this.queue        = payload.queue;
      this.currentIndex = payload.current_index;
    });

    await listen('player://bpm-detected', ({ payload }) => {
      this.bpm = payload.bpm;
    });

    // Autoplay: Rust emits this when a track finishes and autoplay is on
    await listen('player://autoplay-next', ({ payload: trackId }) => {
      this.playTrack(trackId);
    });
  }

  playTrack(trackId)    { return invoke('player_play_track', { trackId }); }
  play()                { return invoke('player_resume'); }
  pause()               { return invoke('player_pause'); }
  stop()                { return invoke('player_stop'); }
  next()                { return invoke('player_next'); }
  prev()                { return invoke('player_previous'); }

  seek(positionSecs) {
    if (!this.currentTrack) return;
    return invoke('player_seek', { trackId: this.currentTrack.id, positionSecs });
  }

  setVolume(vol) {
    this.volume = vol;
    return invoke('player_set_volume', { volume: Math.round(vol) });
  }

  setQueue(trackIds, startIndex = 0) {
    return invoke('player_set_queue', { trackIds, startIndex });
  }

  toggleShuffle() {
    this.shuffle = !this.shuffle;
    return invoke('player_set_shuffle', { enabled: this.shuffle });
  }

  toggleAutoplay() {
    this.autoplay = !this.autoplay;
    return invoke('player_set_autoplay', { enabled: this.autoplay });
  }
}
export const player = new PlayerStore();