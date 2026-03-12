<!-- src/lib/components/DownloadQueue.svelte -->
<script>
  import { invoke } from '@tauri-apps/api/core';
  import { open } from '@tauri-apps/plugin-dialog';
  import { themeStore } from '$lib/stores/theme.svelte.js';
  import { Download, Loader2, Upload, FolderOpen } from 'lucide-svelte';
  import { toast } from 'svelte-sonner';

  let { onDownloadComplete = null } = $props();

  const t = $derived(themeStore.theme);

  let url         = $state('');
  let downloading = $state(false);
  let uploading   = $state(false);
  let busy        = $derived(downloading || uploading);

  async function handleUploadFiles() {
    try {
      const selected = await open({
        multiple: true,
        filters: [{ name: 'Audio', extensions: ['mp3','wav','flac','ogg','m4a','aac','opus','webm','wma','aiff'] }]
      });
      if (!selected) return;
      const files = Array.isArray(selected) ? selected : [selected];
      uploading = true;
      const tid = 'upload';
      toast.loading(`Importing ${files.length} file(s)...`, { id: tid });
      const downloadDir = await invoke('get_downloads_dir');
      let ok = 0, skip = 0, fail = 0;
      for (const filePath of files) {
        try {
          await invoke('import_local_file', { filePath, downloadDir });
          ok++;
        } catch (err) {
          if (String(err).includes('Already imported')) skip++;
          else fail++;
        }
      }
      const parts = [];
      if (ok)   parts.push(`${ok} imported`);
      if (skip) parts.push(`${skip} skipped`);
      if (fail) parts.push(`${fail} failed`);
      if (ok > 0) {
        toast.success(parts.join(', '), { id: tid });
        onDownloadComplete?.();
      } else {
        toast.warning(parts.join(', ') || 'Nothing imported', { id: tid });
      }
    } catch (err) {
      toast.error('Import failed');
    } finally {
      uploading = false;
    }
  }

  async function handleImportFolder() {
    try {
      const selected = await open({ directory: true, multiple: false });
      if (!selected) return;
      uploading = true;
      const tid = 'folder';
      toast.loading('Scanning folder...', { id: tid });
      const downloadDir = await invoke('get_downloads_dir');
      const result = await invoke('import_folder', {
        folderPath: selected,
        downloadDir,
      });
      const parts = [];
      if (result.imported) parts.push(`${result.imported} imported`);
      if (result.skipped)  parts.push(`${result.skipped} skipped`);
      if (result.failed)   parts.push(`${result.failed} failed`);
      if (result.imported > 0) {
        toast.success(parts.join(', '), { id: tid });
        onDownloadComplete?.();
      } else {
        toast.info(parts.join(', ') || 'No new tracks found', { id: tid });
      }
    } catch (err) {
      toast.error(String(err) || 'Folder import failed');
    } finally {
      uploading = false;
    }
  }

  async function handleDownload(e) {
    e.preventDefault();
    if (!url.trim()) { toast.error('Please enter a URL'); return; }
    if (!url.includes('youtube.com') && !url.includes('youtu.be') && !url.includes('soundcloud.com')) {
      toast.error('Only YouTube and SoundCloud URLs are supported');
      return;
    }
    try {
      downloading = true;
      toast.loading('Downloading...', { id: 'dl' });
      const downloadDir = await invoke('get_downloads_dir');
      const result = await invoke('download_from_youtube', { url, downloadDir });
      toast.success(`Downloaded: ${result.title}`, { id: 'dl' });
      url = '';
      onDownloadComplete?.();
    } catch (error) {
      toast.error(String(error) || 'Download failed', { id: 'dl' });
    } finally {
      downloading = false;
    }
  }
</script>

<div style="border:1px solid {t.border}; border-radius:8px; padding:14px; background:{t.bg}">

  <div style="display:flex; align-items:center; gap:8px; margin-bottom:12px">
    <Download size={16} style="color:{t.primary}" />
    <span style="color:{t.primary}; font-weight:bold; font-size:13px">Add Music</span>
  </div>

  <!-- Upload files -->
  <button
    onclick={handleUploadFiles}
    disabled={busy}
    style="width:100%; background:{busy ? 'rgba(255,255,255,0.04)' : `${t.primary}22`};
           border:1px solid {t.primary}; color:{t.primary}; padding:9px 12px; border-radius:6px;
           cursor:{busy ? 'not-allowed' : 'pointer'}; display:flex; align-items:center;
           justify-content:center; gap:8px; margin-bottom:8px; font-weight:bold;
           font-size:13px; font-family:inherit; transition:background 0.15s"
    onmouseenter={(e) => { if (!busy) e.currentTarget.style.background = `${t.primary}40`; }}
    onmouseleave={(e) => { if (!busy) e.currentTarget.style.background = `${t.primary}22`; }}
  >
    {#if uploading}
      <Loader2 size={14} /> Importing...
    {:else}
      <Upload size={14} /> Upload Local Files
    {/if}
  </button>

  <!-- Import folder -->
  <button
    onclick={handleImportFolder}
    disabled={busy}
    style="width:100%; background:transparent; border:1px solid {t.border};
           color:{t.textMuted}; padding:9px 12px; border-radius:6px;
           cursor:{busy ? 'not-allowed' : 'pointer'}; display:flex; align-items:center;
           justify-content:center; gap:8px; font-size:13px; font-family:inherit;
           transition:all 0.15s"
    onmouseenter={(e) => { if (!busy) { e.currentTarget.style.borderColor = t.primary; e.currentTarget.style.color = t.primary; } }}
    onmouseleave={(e) => { if (!busy) { e.currentTarget.style.borderColor = t.border; e.currentTarget.style.color = t.textMuted; } }}
  >
    <FolderOpen size={14} /> Import Folder
  </button>

  <!-- Divider -->
  <div style="display:flex; align-items:center; gap:8px; margin:12px 0; color:{t.textMuted}; font-size:11px">
    <div style="flex:1; height:1px; background:{t.border}"></div>
    OR
    <div style="flex:1; height:1px; background:{t.border}"></div>
  </div>

  <!-- YouTube/SoundCloud download -->
  <div style="display:flex; flex-direction:column; gap:8px">
    <input
      type="text"
      bind:value={url}
      placeholder="https://youtube.com/watch?v=..."
      disabled={busy}
      style="width:100%; background:rgba(255,255,255,0.05); border:1px solid {t.border};
             color:{t.text}; padding:9px 12px; border-radius:6px; outline:none;
             box-sizing:border-box; font-size:12px; font-family:inherit"
    />
    <p style="font-size:10px; color:{t.textMuted}; margin:0">Supports: YouTube, SoundCloud</p>

    <button
      onclick={handleDownload}
      disabled={busy}
      style="width:100%; background:{downloading ? 'rgba(255,255,255,0.04)' : t.primary};
             color:{downloading ? t.textMuted : '#000'}; padding:9px; border-radius:6px;
             border:none; cursor:{busy ? 'not-allowed' : 'pointer'}; display:flex;
             align-items:center; justify-content:center; gap:8px; font-weight:bold;
             font-size:13px; font-family:inherit"
    >
      {#if downloading}
        <Loader2 size={14} /> Downloading...
      {:else}
        <Download size={14} /> Download
      {/if}
    </button>
  </div>

  <div style="margin-top:10px; padding:8px 10px; background:rgba(255,255,255,0.03);
              border:1px solid {t.border}; border-radius:6px; font-size:10px; color:{t.textMuted}">
    <strong style="color:{t.primary}">Note:</strong> Requires yt-dlp.<br />
    <code style="color:{t.primary}">brew install yt-dlp</code>
  </div>
</div>