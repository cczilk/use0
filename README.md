<div align="center">

# use0

**a minimal, themeable music player built with tauri + svelte**

![demo](demo.gif)

</div>

---

## features

- **local library** — import folders or individual files, auto-extracts metadata and album art
- **youtube download** — paste a URL and download directly into your library via yt-dlp
- **visualizer** — waveform + bar visualizer, upload a custom gif/video as background
- **equalizer** — 8-band EQ with presets
- **playlists** — create and manage playlists with drag-and-drop
- **themes** — dark, red, blue, purple
- **mini player** — compact mode for background listening
- **cross-platform** — linux, windows, and macos (apple silicon)

## built with

- [tauri 2](https://tauri.app) — rust backend + webview frontend
- [svelte 5](https://svelte.dev) — reactive UI with runes
- [rodio 0.21](https://github.com/RustAudio/rodio) — audio engine
- [lofty](https://github.com/Serial-ATA/lofty-rs) — audio metadata + embedded art
- [rusqlite](https://github.com/rusqlite/rusqlite) — local library database

## install

### linux

grab the latest binary from [releases](../../releases) or build from source:

```bash
# arch
sudo cp use0 /usr/local/bin/use0

# add to app launcher (optional)
sudo cp use0.png /usr/share/pixmaps/use0.png
sudo cp use0.desktop /usr/share/applications/use0.desktop
```

### windows

download `use0_setup.exe` from [releases](../../releases) and run it.

## build from source

**prerequisites:** rust, node, pnpm, yt-dlp (for youtube downloads)

```bash
git clone https://github.com/yourname/use0
cd use0
pnpm install
pnpm tauri build
```

**windows (cross-compile from linux):**
```bash
rustup target add x86_64-pc-windows-msvc
cargo install cargo-xwin
pnpm tauri build --target x86_64-pc-windows-msvc
```

## dev

```bash
pnpm tauri dev
```

## license

mit