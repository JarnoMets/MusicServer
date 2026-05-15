use std::path::Path;
use std::io;
use log::debug;
use thiserror::Error;
use aubio::{Tempo, OnsetMode};

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::errors::Error as SymphoniaError;

// ── tunables ─────────────────────────────────────────────────────────────────

/// aubio hop size (samples fed per call to do_result).
const HOP_SIZE: usize = 512;
/// aubio internal window size — must be >= 2 * HOP_SIZE.
const BUF_SIZE: usize = 1024;
/// Collect a BPM snapshot from aubio every this many hops.
/// At 44 100 Hz / 512 = ~86 hops/s → sample every ~2 s.
const SAMPLE_EVERY_N_HOPS: usize = 172;
/// Analyse at most this many seconds of audio (5 min is ample for any track).
const MAX_ANALYSE_SECS: u64 = 300;
/// Require at least this many detected beats to trust timestamps.
const MIN_BEATS: usize = 8;
/// Width of each BPM histogram bucket (in BPM).
const HIST_BUCKET: f64 = 0.5;

// ─────────────────────────────────────────────────────────────────────────────

#[derive(Error, Debug)]
pub enum BpmError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("File not found")]
    NotFound,
    #[error("No audio track found")]
    NoAudioTrack,
    #[error("BPM detection inconclusive")]
    Inconclusive,
    #[error("Aubio error: {0}")]
    Aubio(String),
}

impl From<SymphoniaError> for BpmError {
    fn from(e: SymphoniaError) -> Self {
        BpmError::Decode(e.to_string())
    }
}

#[derive(serde::Serialize, Clone, Debug)]
pub struct BpmResult {
    pub bpm: f64,
    pub offset: f64,     // first beat timestamp in seconds
    pub beats: Vec<f64>, // all detected beat timestamps in seconds
}

// ─── audio decoding ───────────────────────────────────────────────────────────

struct AudioInfo {
    samples: Vec<f32>,
    sample_rate: u32,
}

/// Decode the audio file to mono f32 PCM, up to MAX_ANALYSE_SECS of audio.
fn decode_audio(file_path: &str) -> Result<AudioInfo, BpmError> {
    let file = std::fs::File::open(file_path).map_err(BpmError::Io)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(file_path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &Default::default(), &Default::default())
        .map_err(|e| BpmError::Decode(format!("probe failed: {e}")))?;

    let mut format = probed.format;
    let track = format
        .tracks()
        .iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or(BpmError::NoAudioTrack)?;

    let track_id = track.id;
    let sample_rate = track
        .codec_params
        .sample_rate
        .ok_or_else(|| BpmError::Decode("unknown sample rate".into()))?;
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);
    let max_samples = MAX_ANALYSE_SECS as usize * sample_rate as usize;

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &Default::default())
        .map_err(|e| BpmError::Decode(format!("codec init failed: {e}")))?;

    let mut mono: Vec<f32> = Vec::with_capacity(max_samples.min(sample_rate as usize * 60));

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };
        let n = decoded.frames();
        push_mono(&decoded, n, channels, &mut mono);
        if mono.len() >= max_samples {
            break;
        }
    }

    Ok(AudioInfo { samples: mono, sample_rate })
}

/// Downmix a decoded audio buffer to mono f32 and append to `out`.
#[inline]
fn push_mono(decoded: &AudioBufferRef<'_>, n: usize, ch: usize, out: &mut Vec<f32>) {
    let inv = 1.0_f32 / ch as f32;
    match decoded {
        AudioBufferRef::F32(buf) => {
            for i in 0..n {
                let mut s = 0.0_f32;
                for c in 0..ch { s += buf.chan(c)[i]; }
                out.push(s * inv);
            }
        }
        AudioBufferRef::F64(buf) => {
            for i in 0..n {
                let mut s = 0.0_f64;
                for c in 0..ch { s += buf.chan(c)[i]; }
                out.push((s * inv as f64) as f32);
            }
        }
        AudioBufferRef::S32(buf) => {
            for i in 0..n {
                let mut s = 0.0_f32;
                for c in 0..ch { s += buf.chan(c)[i] as f32 / 2_147_483_648.0; }
                out.push(s * inv);
            }
        }
        AudioBufferRef::S16(buf) => {
            for i in 0..n {
                let mut s = 0.0_f32;
                for c in 0..ch { s += buf.chan(c)[i] as f32 / 32_768.0; }
                out.push(s * inv);
            }
        }
        AudioBufferRef::U8(buf) => {
            for i in 0..n {
                let mut s = 0.0_f32;
                for c in 0..ch { s += (buf.chan(c)[i] as f32 - 128.0) / 128.0; }
                out.push(s * inv);
            }
        }
        _ => {} // uncommon formats (S24, U16, …) — skip silently
    }
}

// ─── BPM estimation helpers ───────────────────────────────────────────────────

/// Find the most-voted bucket in a BPM histogram and return its centre value.
///
/// Each sample is rounded to the nearest HIST_BUCKET-wide bin. The bin with
/// the highest vote count wins. Returns `None` when `samples` is empty.
fn histogram_peak(samples: &[f64]) -> Option<f64> {
    if samples.is_empty() {
        return None;
    }
    let mut counts: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();
    for &b in samples {
        if b > 0.0 {
            let key = (b / HIST_BUCKET).round() as i64;
            *counts.entry(key).or_default() += 1;
        }
    }
    counts
        .into_iter()
        .max_by_key(|&(_, c)| c)
        .map(|(k, _)| k as f64 * HIST_BUCKET)
}

/// Derive BPM from the median inter-beat interval of the detected beat list.
///
/// Using the median makes this robust to a handful of spurious or missed beats.
/// Returns `None` when there are too few valid intervals to be meaningful.
fn ibi_bpm(beats: &[f64]) -> Option<f64> {
    let mut ibis: Vec<f64> = beats
        .windows(2)
        .map(|w| w[1] - w[0])
        // Valid range covers 24 BPM (2.5 s) up to 400 BPM (0.15 s)
        .filter(|&ibi| ibi > 0.15 && ibi < 2.5)
        .collect();

    if ibis.len() < 4 {
        return None;
    }
    ibis.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let median = ibis[ibis.len() / 2];
    if median > 0.0 { Some(60.0 / median) } else { None }
}

#[inline]
fn round1(v: f64) -> f64 {
    (v * 10.0).round() / 10.0
}

// ─── public API ───────────────────────────────────────────────────────────────

/// Detect BPM, beat-grid offset, and full beat map for an audio file.
///
/// Strategy:
/// 1. Decode up to MAX_ANALYSE_SECS of mono f32 PCM via Symphonia.
/// 2. Feed HOP_SIZE chunks into an aubio SpecFlux Tempo detector.
/// 3. Every ~2 s, snapshot `get_bpm()` into a histogram; also record every
///    detected beat timestamp (absolute, from aubio's internal frame counter).
/// 4. Pick the histogram peak as the primary BPM estimate.
/// 5. Cross-validate with the median inter-beat interval (IBI) derived from
///    the beat timestamps.  If the two estimates agree within 3 %, prefer the
///    IBI value (higher precision); otherwise trust the histogram.
/// 6. Return the result — callers must not rely on a specific rounding beyond
///    one decimal place.
pub fn detect_bpm(file_path: &str) -> Result<BpmResult, BpmError> {
    if !Path::new(file_path).exists() {
        return Err(BpmError::NotFound);
    }

    debug!("BPM detection starting for: {}", file_path);

    let AudioInfo { samples, sample_rate } = decode_audio(file_path)?;
    if samples.is_empty() {
        return Err(BpmError::Inconclusive);
    }

    let mut tempo = Tempo::new(OnsetMode::SpecFlux, BUF_SIZE, HOP_SIZE, sample_rate)
        .map_err(|e| BpmError::Aubio(format!("{e:?}")))?;
    // -60 dBFS silence gate — quiet sections still get analysed
    tempo.set_silence(-60.0);
    // Default aubio threshold is 0.3; keep it explicit
    tempo.set_threshold(0.3);

    let mut beat_timestamps: Vec<f64> = Vec::new();
    // Periodic BPM snapshots — this is the histogram input
    let mut bpm_snapshots: Vec<f64> = Vec::new();
    let mut hop_count: usize = 0;

    for chunk in samples.chunks_exact(HOP_SIZE) {
        if let Ok(is_beat) = tempo.do_result(chunk) {
            if is_beat > 0.0 {
                // get_last_s() returns the absolute beat position in seconds
                // (aubio tracks total frames internally — no manual offset needed)
                let ts = tempo.get_last_s() as f64;
                if ts > 0.0 {
                    beat_timestamps.push(ts);
                }
            }
        }

        hop_count += 1;
        if hop_count % SAMPLE_EVERY_N_HOPS == 0 {
            let snap = tempo.get_bpm() as f64;
            if snap > 0.0 {
                bpm_snapshots.push(snap);
            }
        }
    }

    // Final snapshot at end-of-file
    let snap = tempo.get_bpm() as f64;
    if snap > 0.0 {
        bpm_snapshots.push(snap);
    }

    debug!(
        "aubio pass: {} beats detected, {} BPM snapshots collected",
        beat_timestamps.len(),
        bpm_snapshots.len()
    );

    if beat_timestamps.len() < MIN_BEATS && bpm_snapshots.len() < 3 {
        return Err(BpmError::Inconclusive);
    }

    // Primary estimate: histogram vote over periodically sampled BPM values
    let hist = histogram_peak(&bpm_snapshots);

    // Secondary estimate: median inter-beat interval converted to BPM
    let ibi = if beat_timestamps.len() >= MIN_BEATS {
        ibi_bpm(&beat_timestamps)
    } else {
        None
    };

    let raw_bpm = match (hist, ibi) {
        (Some(h), Some(i)) => {
            // Both agree within 3 % → prefer IBI (derived from actual beat events)
            if (h - i).abs() / h.max(1.0) < 0.03 { i } else { h }
        }
        (Some(h), None) => h,
        (None, Some(i)) => i,
        (None, None) => return Err(BpmError::Inconclusive),
    };

    let bpm = round1(raw_bpm);
    let offset = beat_timestamps.first().copied().unwrap_or(0.0);

    debug!(
        "BPM result: {:.1} (hist={:?}, ibi={:?}, beats={})",
        bpm, hist, ibi, beat_timestamps.len()
    );

    Ok(BpmResult { bpm, offset, beats: beat_timestamps })
}
