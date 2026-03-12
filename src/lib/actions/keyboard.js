import { invoke } from '@tauri-apps/api/core';
import { player } from '$lib/stores/player.svelte.js';

export function keyboard(node) {
  function handleKeyDown(e) {
    if (e.target.tagName === 'INPUT' || e.target.tagName === 'TEXTAREA') return;

    switch (e.key) {
      case ' ':
        e.preventDefault();
        player.isPlaying ? player.pause() : player.play();
        break;

      case 'ArrowRight':
        e.preventDefault();
        player.next();
        break;

      case 'ArrowLeft':
        e.preventDefault();
        player.prev();
        break;

      case 'ArrowUp':
        e.preventDefault();
        player.setVolume(Math.min(100, player.volume + 5));
        break;

      case 'ArrowDown':
        e.preventDefault();
        player.setVolume(Math.max(0, player.volume - 5));
        break;

      case 'm':
      case 'M':
        e.preventDefault();
        player.setVolume(player.volume > 0 ? 0 : 70);
        break;

      case 's':
      case 'S':
        e.preventDefault();
        player.toggleShuffle();
        break;

      case 'a':
      case 'A':
        e.preventDefault();
        player.toggleAutoplay();
        break;
    }
  }

  window.addEventListener('keydown', handleKeyDown);

  return {
    destroy() {
      window.removeEventListener('keydown', handleKeyDown);
    }
  };
}