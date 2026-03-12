use rodio::{Decoder, OutputStream, OutputStreamBuilder, Sink, Source};
use std::{
    fs::File,
    io::BufReader,
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
    thread,
};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

#[derive(Clone, Serialize)]
pub struct TrackChangedPayload {
    pub track_id: i64,
    pub title: String,
    pub artist: String,
    pub album: Option<String>,
    pub duration_secs: f64,
    pub file_path: String,
    pub thumbnail_path: Option<String>,
}

#[derive(Clone, Serialize)]
pub struct StateChangedPayload {
    pub is_playing: bool,
    pub position_secs: f64,
    pub volume: f32,
}

#[derive(Clone, Serialize)]
pub struct QueueUpdatedPayload {
    pub queue: Vec<i64>,
    pub current_index: usize,
}

#[derive(Clone, Serialize)]
pub struct BpmDetectedPayload {
    pub bpm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerConfig {
    pub volume: f32,
    pub shuffle: bool,
    pub autoplay: bool,
    pub fade_duration_ms: u64,
}

impl Default for PlayerConfig {
    fn default() -> Self {
        Self { volume: 0.7, shuffle: false, autoplay: true, fade_duration_ms: 80 }
    }
}

struct PlayerInner {
    sink: Sink,
    _stream: OutputStream,
    queue: Vec<i64>,
    current_index: Option<usize>,
    config: PlayerConfig,
    downloads_dir: PathBuf,
    position_secs: f64,
    duration_secs: f64,
    shuffle_history: Vec<i64>,
}

impl PlayerInner {
    fn new(downloads_dir: PathBuf) -> Result<Self, String> {
        let stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("Failed to open audio device: {e}"))?;
        let sink = Sink::connect_new(&stream.mixer());
        Ok(Self {
            sink,
            _stream: stream,
            queue: Vec::new(),
            current_index: None,
            config: PlayerConfig::default(),
            downloads_dir,
            position_secs: 0.0,
            duration_secs: 0.0,
            shuffle_history: Vec::new(),
        })
    }

    fn stop_with_fade(&mut self) {
        if !self.sink.empty() {
            let steps = 8u32;
            let step_ms = self.config.fade_duration_ms / steps as u64;
            let initial_vol = self.sink.volume();
            for i in 0..steps {
                self.sink.set_volume(initial_vol * (1.0 - i as f32 / steps as f32));
                thread::sleep(Duration::from_millis(step_ms));
            }
            self.sink.stop();
            self.sink.set_volume(self.config.volume);
        }
    }

    fn load_and_append(&mut self, file_path: &str) -> Result<f64, String> {
        let path = PathBuf::from(file_path);
        let file = File::open(&path).map_err(|e| format!("Cannot open {path:?}: {e}"))?;
        let source = Decoder::new(BufReader::new(file))
            .map_err(|e| format!("Cannot decode {path:?}: {e}"))?;
        let duration = source.total_duration().map(|d| d.as_secs_f64()).unwrap_or(0.0);
        let fade_dur = Duration::from_millis(self.config.fade_duration_ms);
        self.sink.append(source.fade_in(fade_dur));
        Ok(duration)
    }
}

#[derive(Clone)]
pub struct Player(Arc<Mutex<PlayerInner>>);

impl Player {
    pub fn new(downloads_dir: PathBuf) -> Result<Self, String> {
        Ok(Self(Arc::new(Mutex::new(PlayerInner::new(downloads_dir)?))))
    }

    pub fn start_ticker(&self, app: AppHandle) {
        let app_state = app.clone();
        let p1 = self.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(250));
            if let Ok(g) = p1.0.lock() {
                let _ = app_state.emit("player://state-changed", StateChangedPayload {
                    is_playing: !g.sink.is_paused() && !g.sink.empty(),
                    position_secs: g.position_secs,
                    volume: g.config.volume,
                });
            }
        });

        let app_pos = app.clone();
        let p2 = self.clone();
        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(100));
            let autoplay_id: Option<i64> = {
                if let Ok(mut g) = p2.0.lock() {
                    if !g.sink.is_paused() && !g.sink.empty() {
                        g.position_secs += 0.1;
                        if g.duration_secs > 0.0 {
                            g.position_secs = g.position_secs.min(g.duration_secs);
                        }
                        None
                    } else if g.sink.empty()
                        && g.duration_secs > 0.0
                        && g.position_secs >= g.duration_secs - 0.6
                        && g.config.autoplay
                        && !g.queue.is_empty()
                    {
                        g.position_secs = 0.0;
                        g.duration_secs = 0.0;
                        if g.config.shuffle {
                            let unplayed: Vec<i64> = g.queue.iter().copied()
                                .filter(|id| !g.shuffle_history.contains(id)).collect();
                            let pool = if unplayed.is_empty() {
                                g.shuffle_history.clear(); g.queue.clone()
                            } else { unplayed };
                            if pool.is_empty() { None } else {
                                let chosen = pool[fastrand::usize(..pool.len())];
                                g.shuffle_history.push(chosen);
                                g.current_index = g.queue.iter().position(|&id| id == chosen);
                                Some(chosen)
                            }
                        } else {
                            let next_idx = g.current_index.map(|i| i + 1).unwrap_or(0);
                            if next_idx < g.queue.len() {
                                g.current_index = Some(next_idx);
                                Some(g.queue[next_idx])
                            } else { None }
                        }
                    } else { None }
                } else { None }
            };
            if let Some(tid) = autoplay_id {
                let _ = app_pos.emit("player://autoplay-next", tid);
            }
        });
    }

    pub fn play_track(&self, file_path: &str, _track_id: i64, _app: &AppHandle) -> Result<f64, String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        g.stop_with_fade();
        g.position_secs = 0.0;
        let duration = g.load_and_append(file_path)?;
        g.duration_secs = duration;
        g.sink.play();
        Ok(duration)
    }

    pub fn pause(&self) -> Result<(), String> {
        self.0.lock().map_err(|_| "Lock poisoned")?.sink.pause();
        Ok(())
    }

    pub fn resume(&self) -> Result<(), String> {
        self.0.lock().map_err(|_| "Lock poisoned")?.sink.play();
        Ok(())
    }

    pub fn stop(&self) -> Result<(), String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        g.stop_with_fade();
        g.position_secs = 0.0;
        Ok(())
    }

    pub fn seek(&self, file_path: &str, position_secs: f64) -> Result<(), String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        let target = Duration::from_secs_f64(position_secs);
        match g.sink.try_seek(target) {
            Ok(()) => {
                g.position_secs = position_secs;
                Ok(())
            }
            Err(_) => {
                let was_paused = g.sink.is_paused();
                g.sink.stop();
                let file = File::open(file_path).map_err(|e| format!("Seek open: {e}"))?;
                let source = Decoder::new(BufReader::new(file))
                    .map_err(|e| format!("Seek decode: {e}"))?;
                g.sink.append(source.skip_duration(target));
                g.position_secs = position_secs;
                if was_paused { g.sink.pause(); }
                Ok(())
            }
        }
    }

    pub fn set_volume(&self, volume: f32) -> Result<(), String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        let vol = volume.clamp(0.0, 1.0);
        g.sink.set_volume(vol);
        g.config.volume = vol;
        Ok(())
    }

    pub fn set_queue(&self, track_ids: Vec<i64>, start_index: usize) -> Result<(), String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        g.queue = track_ids;
        g.current_index = Some(start_index);
        g.shuffle_history.clear();
        Ok(())
    }

    pub fn current_queue(&self) -> Result<(Vec<i64>, Option<usize>), String> {
        let g = self.0.lock().map_err(|_| "Lock poisoned")?;
        Ok((g.queue.clone(), g.current_index))
    }

    pub fn next_track_id(&self) -> Result<Option<i64>, String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        if g.queue.is_empty() { return Ok(None); }
        if g.config.shuffle {
            let unplayed: Vec<i64> = g.queue.iter().copied()
                .filter(|id| !g.shuffle_history.contains(id)).collect();
            let pool = if unplayed.is_empty() {
                g.shuffle_history.clear(); g.queue.clone()
            } else { unplayed };
            let chosen = pool[fastrand::usize(..pool.len())];
            g.shuffle_history.push(chosen);
            g.current_index = g.queue.iter().position(|&id| id == chosen);
            Ok(Some(chosen))
        } else {
            let next = g.current_index.map(|i| i + 1).unwrap_or(0);
            if next < g.queue.len() { g.current_index = Some(next); Ok(Some(g.queue[next])) }
            else { Ok(None) }
        }
    }

    pub fn prev_track_id(&self) -> Result<Option<i64>, String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        if let Some(i) = g.current_index.and_then(|i| i.checked_sub(1)) {
            g.current_index = Some(i);
            Ok(Some(g.queue[i]))
        } else { Ok(None) }
    }

    pub fn set_shuffle(&self, enabled: bool) -> Result<(), String> {
        let mut g = self.0.lock().map_err(|_| "Lock poisoned")?;
        g.config.shuffle = enabled;
        if enabled { g.shuffle_history.clear(); }
        Ok(())
    }

    pub fn set_autoplay(&self, enabled: bool) -> Result<(), String> {
        self.0.lock().map_err(|_| "Lock poisoned")?.config.autoplay = enabled;
        Ok(())
    }

    pub fn config(&self) -> Result<PlayerConfig, String> {
        Ok(self.0.lock().map_err(|_| "Lock poisoned")?.config.clone())
    }

    pub fn is_playing(&self) -> bool {
        self.0.lock().map(|g| !g.sink.is_paused() && !g.sink.empty()).unwrap_or(false)
    }

    pub fn position(&self) -> f64 {
        self.0.lock().map(|g| g.position_secs).unwrap_or(0.0)
    }

    pub fn duration(&self) -> f64 {
        self.0.lock().map(|g| g.duration_secs).unwrap_or(0.0)
    }
}