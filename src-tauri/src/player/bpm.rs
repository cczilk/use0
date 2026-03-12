use std::path::Path;

#[derive(Debug, Clone)]
pub struct BpmResult {
    pub bpm: f32,
    pub energy: f32,   // RMS energy 0.0–1.0, used for visualizer intensity
}

/// Detect BPM from an audio file.
/// `sample_secs` — how many seconds of audio to analyse (20s is plenty, fast).
pub fn detect_bpm_from_file(path: &str, sample_secs: f32) -> Result<BpmResult, String> {
    use rodio::Decoder;
    use std::fs::File;
    use std::io::BufReader;

    let file = File::open(Path::new(path))
        .map_err(|e| format!("BPM: open error: {e}"))?;
    let decoder = Decoder::new(BufReader::new(file))
        .map_err(|e| format!("BPM: decode error: {e}"))?;

    let sample_rate = 44100u32; // we'll resample mentally; rodio gives us whatever
    let max_samples = (sample_secs * sample_rate as f32) as usize;

    let samples: Vec<f32> = decoder
        .take(max_samples * 2)
        .enumerate()
        .filter(|(i, _)| i % 2 == 0)
        .map(|(_, s)| s as f32 / i16::MAX as f32)
        .collect();

    if samples.len() < 512 {
        return Ok(BpmResult { bpm: 120.0, energy: 0.0 });
    }

    let energy = rms_energy(&samples);
    let bpm = detect_bpm(&samples, sample_rate as f32);

    Ok(BpmResult { bpm, energy })
}

fn rms_energy(samples: &[f32]) -> f32 {
    let sum_sq: f32 = samples.iter().map(|s| s * s).sum();
    (sum_sq / samples.len() as f32).sqrt()
}

fn detect_bpm(samples: &[f32], sample_rate: f32) -> f32 {
    let alpha: f32 = 1.0 - (-2.0 * std::f32::consts::PI * 200.0 / sample_rate).exp();
    let mut envelope = 0.0f32;
    let enveloped: Vec<f32> = samples.iter().map(|&s| {
        let abs = s.abs();
        if abs > envelope {
            envelope = abs;
        } else {
            envelope = envelope * (1.0 - alpha) + abs * alpha;
        }
        envelope
    }).collect();

    let window = (sample_rate * 0.010) as usize;
    let energies: Vec<f32> = enveloped.windows(window)
        .step_by(window / 2)
        .map(|w| rms_energy(w))
        .collect();

    if energies.len() < 8 {
        return 120.0;
    }

    // ── 3. Peak-picking ───────────────────────────────────────────────────
    let threshold = energies.iter().cloned().fold(0.0f32, f32::max) * 0.35;
    let min_gap_frames = (0.2 / (window as f32 / 2.0 / sample_rate)) as usize;

    let mut peaks: Vec<usize> = Vec::new();
    let mut last_peak = 0usize;

    for (i, &e) in energies.iter().enumerate() {
        if e > threshold {
            let is_local_max = {
                let lo = i.saturating_sub(3);
                let hi = (i + 4).min(energies.len());
                energies[lo..hi].iter().all(|&v| v <= e)
            };
            if is_local_max && i.saturating_sub(last_peak) >= min_gap_frames {
                peaks.push(i);
                last_peak = i;
            }
        }
    }

    if peaks.len() < 4 {
        return 120.0;
    }

    let frame_secs = window as f32 / 2.0 / sample_rate;
    let intervals: Vec<f32> = peaks.windows(2)
        .map(|w| (w[1] - w[0]) as f32 * frame_secs)
        .collect();

    // Histogram over 40–200 BPM range (0.3s–1.5s intervals)
    let bpm_min = 40.0f32;
    let bpm_max = 200.0f32;
    let bins = 160usize;
    let mut histogram = vec![0u32; bins];

    for &interval in &intervals {
        let bpm = 60.0 / interval;
        if bpm >= bpm_min && bpm <= bpm_max {
            let bin = ((bpm - bpm_min) / (bpm_max - bpm_min) * bins as f32) as usize;
            let bin = bin.min(bins - 1);
            histogram[bin] += 1;
            // Also weight harmonics (doublings/halvings)
            let half = bin / 2;
            if half > 0 { histogram[half] += 1; }
            let double = (bin * 2).min(bins - 1);
            histogram[double] += 1;
        }
    }

    let peak_bin = histogram.iter().enumerate()
        .max_by_key(|(_, &v)| v)
        .map(|(i, _)| i)
        .unwrap_or(80); // default 120 BPM bin

    let raw_bpm = bpm_min + peak_bin as f32 / bins as f32 * (bpm_max - bpm_min);

    // ── 5. Octave-fold to 60–200 BPM ──────────────────────────────────────
    let mut bpm = raw_bpm;
    while bpm < 60.0  { bpm *= 2.0; }
    while bpm > 200.0 { bpm /= 2.0; }

    bpm
}