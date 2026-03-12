// analyser.svelte.js
// Uses fetch via Tauri's asset protocol with readBinaryFile fallback,
// decodes audio into an OfflineAudioContext for analysis.
// No hidden <audio> element = no CORS/403 issues.

import { listen } from '@tauri-apps/api/event';
import { convertFileSrc } from '@tauri-apps/api/core';
import { readFile } from '@tauri-apps/plugin-fs';

class AudioAnalyserStore {
  isReady    = $state(false);
  #ctx       = null;
  #analyser  = null;
  #source    = null;
  #buffer    = null;
  #startTime = 0;
  #startOffset = 0;
  #isPlaying = false;
  #filePath  = null;

  async init() {
    await listen('player://track-changed', ({ payload }) => {
      if (payload.file_path) this.#loadFile(payload.file_path);
    });

    await listen('player://state-changed', ({ payload }) => {
      if (!this.#ctx) return;
      if (payload.is_playing && !this.#isPlaying) {
        this.#resumeAt(payload.position_secs ?? 0);
      } else if (!payload.is_playing && this.#isPlaying) {
        this.#pause(payload.position_secs ?? 0);
      }
    });
  }

  #ensureContext() {
    if (this.#ctx) return;
    this.#ctx = new AudioContext();
    this.#analyser = this.#ctx.createAnalyser();
    this.#analyser.fftSize = 2048;
    this.#analyser.smoothingTimeConstant = 0.8;
    // silent output — we only want analysis
    const gain = this.#ctx.createGain();
    gain.gain.value = 0;
    this.#analyser.connect(gain);
    gain.connect(this.#ctx.destination);
    this.isReady = true;
  }

  async #loadFile(filePath) {
    this.#ensureContext();
    this.#filePath = filePath;
    this.#stopSource();

    try {
      // Read raw bytes via Tauri FS plugin — bypasses asset:// 403
      const bytes = await readFile(filePath);
      const arrayBuffer = bytes.buffer;
      this.#buffer = await this.#ctx.decodeAudioData(arrayBuffer);
      console.log('[analyser] decoded:', filePath, this.#buffer.duration.toFixed(1) + 's');
      this.#resumeAt(0);
    } catch (e) {
      console.warn('[analyser] load failed:', e);
      this.#buffer = null;
    }
  }

  #stopSource() {
    if (this.#source) {
      try { this.#source.stop(); } catch {}
      this.#source.disconnect();
      this.#source = null;
    }
    this.#isPlaying = false;
  }

  #resumeAt(offsetSecs) {
    if (!this.#buffer || !this.#ctx) return;
    this.#stopSource();
    if (this.#ctx.state === 'suspended') this.#ctx.resume().catch(() => {});
    this.#source = this.#ctx.createBufferSource();
    this.#source.buffer = this.#buffer;
    this.#source.connect(this.#analyser);
    this.#source.start(0, Math.min(offsetSecs, this.#buffer.duration - 0.01));
    this.#startTime = this.#ctx.currentTime;
    this.#startOffset = offsetSecs;
    this.#isPlaying = true;
  }

  #pause(offsetSecs) {
    this.#stopSource();
    this.#startOffset = offsetSecs;
  }

  getFrequencyData() {
    if (!this.#analyser) return null;
    const data = new Uint8Array(this.#analyser.frequencyBinCount);
    this.#analyser.getByteFrequencyData(data);
    return data;
  }

  getTimeDomainData() {
    if (!this.#analyser) return null;
    const data = new Uint8Array(this.#analyser.fftSize);
    this.#analyser.getByteTimeDomainData(data);
    return data;
  }
}

export const audioAnalyser = new AudioAnalyserStore();
