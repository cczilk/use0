<script>
  import { invoke } from '@tauri-apps/api/core';
  import { library } from '$lib/stores/library.svelte.js';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { X, Save } from 'lucide-svelte';
  import { toast } from 'svelte-sonner';

  let { track, onclose } = $props();

  const t = $derived(themeStore.theme);

  let title  = $state(track?.title  || '');
  let artist = $state(track?.artist || '');
  let album  = $state(track?.album  || '');
  let saving = $state(false);

  async function handleSave() {
    if (!title.trim()) { toast.error('Title cannot be empty'); return; }
    saving = true;
    try {
      await invoke('update_track_metadata', {
        trackId: track.id,
        title:  title.trim(),
        artist: artist.trim() || null,
        album:  album.trim()  || null,
      });
      await library.fetchTracks();
      toast.success('Metadata updated!');
      onclose?.();
    } catch (e) {
      console.error(e);
      toast.error('Failed to update metadata');
    } finally {
      saving = false;
    }
  }

  function handleKeyDown(e) {
    if ((e.metaKey || e.ctrlKey) && e.key === 'Enter') handleSave();
    else if (e.key === 'Escape') onclose?.();
  }
</script>

<div onclick={() => onclose?.()} style="position:fixed;inset:0;background:rgba(0,0,0,0.8);backdrop-filter:blur(4px);z-index:9998"></div>

<div style="position:fixed;top:50%;left:50%;transform:translate(-50%,-50%);z-index:9999;
            width:90%;max-width:480px;background:{t.bg};border:1px solid {t.border};
            border-radius:12px;padding:24px;box-shadow:0 8px 32px rgba(0,0,0,0.8)">

  <div style="display:flex;align-items:center;justify-content:space-between;margin-bottom:20px">
    <h2 style="font-size:16px;font-weight:bold;color:{t.primary};margin:0">Edit Metadata</h2>
    <button onclick={() => onclose?.()}
      style="background:transparent;border:none;cursor:pointer;padding:4px;display:flex;align-items:center;color:{t.textMuted}">
      <X size={18} />
    </button>
  </div>

  <div style="display:flex;flex-direction:column;gap:14px">
    {#each [
      { label: 'TITLE *',  bind: 'title',  val: title,  set: (v) => title  = v, ph: 'Track title...',  focus: true  },
      { label: 'ARTIST',   bind: 'artist', val: artist, set: (v) => artist = v, ph: 'Artist name...',  focus: false },
      { label: 'ALBUM',    bind: 'album',  val: album,  set: (v) => album  = v, ph: 'Album name...',   focus: false },
    ] as f (f.bind)}
      <div>
        <label style="display:block;font-size:11px;font-weight:bold;margin-bottom:6px;color:{t.textMuted}">{f.label}</label>
        <input
          type="text"
          value={f.val}
          oninput={(e) => f.set(e.currentTarget.value)}
          onkeydown={handleKeyDown}
          placeholder={f.ph}
          autofocus={f.focus}
          style="width:100%;padding:10px 12px;background:rgba(255,255,255,0.06);border:1px solid {t.border};
                 border-radius:6px;color:{t.text};font-size:13px;outline:none;box-sizing:border-box;font-family:inherit"
        />
      </div>
    {/each}

    <div style="display:flex;gap:10px;margin-top:4px">
      <button onclick={handleSave} disabled={saving || !title.trim()}
        style="flex:1;padding:10px;background:{saving||!title.trim() ? 'rgba(255,255,255,0.05)' : t.primary};
               color:{saving||!title.trim() ? t.textMuted : '#000'};border:none;border-radius:6px;
               font-size:13px;font-weight:bold;cursor:{saving||!title.trim() ? 'not-allowed' : 'pointer'};
               display:flex;align-items:center;justify-content:center;gap:6px;font-family:inherit">
        <Save size={14} />{saving ? 'Saving...' : 'Save'}
      </button>
      <button onclick={() => onclose?.()} disabled={saving}
        style="flex:1;padding:10px;background:transparent;color:{t.text};border:1px solid {t.border};
               border-radius:6px;font-size:13px;font-weight:bold;cursor:pointer;font-family:inherit">
        Cancel
      </button>
    </div>
    <div style="font-size:10px;color:{t.textMuted};text-align:center">Ctrl+Enter to save · Esc to cancel</div>
  </div>
</div>