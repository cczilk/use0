// src/lib/stores/theme.svelte.js

const THEMES = {
  dark:   { primary: '#4ade80', bg: 'rgba(10,10,10,0.95)',   bgMuted: 'rgba(255,255,255,0.05)', border: 'rgba(74,222,128,0.2)',  text: '#e5e7eb', textMuted: '#6b7280' },
  red:    { primary: '#ef4444', bg: 'rgba(10,5,5,0.95)',     bgMuted: 'rgba(255,255,255,0.05)', border: 'rgba(239,68,68,0.2)',   text: '#e5e7eb', textMuted: '#6b7280' },
  blue:   { primary: '#60a5fa', bg: 'rgba(5,8,15,0.95)',     bgMuted: 'rgba(255,255,255,0.05)', border: 'rgba(96,165,250,0.2)',  text: '#e5e7eb', textMuted: '#6b7280' },
  purple: { primary: '#a78bfa', bg: 'rgba(8,5,15,0.95)',     bgMuted: 'rgba(255,255,255,0.05)', border: 'rgba(167,139,250,0.2)', text: '#e5e7eb', textMuted: '#6b7280' },
};

class ThemeStore {
  // Guard localStorage access — not available during SSR/module analysis
  #name = $state(
    typeof localStorage !== 'undefined'
      ? (localStorage.getItem('tabi_theme') || 'dark')
      : 'dark'
  );

  theme     = $derived(THEMES[this.#name] ?? THEMES.dark);
  current   = $derived(this.#name);
  available = Object.keys(THEMES);

  change(name) {
    if (!THEMES[name]) return;
    this.#name = name;
    if (typeof localStorage !== 'undefined') {
      localStorage.setItem('tabi_theme', name);
    }
  }
}

export const themeStore = new ThemeStore();