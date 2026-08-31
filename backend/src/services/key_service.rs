use std::path::Path;
use std::io;
use log::debug;
use thiserror::Error;
use rustfft::num_complex::Complex;
use rustfft::FftPlanner;

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::errors::Error as SymphoniaError;

#[derive(Error, Debug)]
pub enum KeyError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Decode error: {0}")]
    Decode(String),
    #[error("No audio track found")]
    NoAudioTrack,
}

impl From<SymphoniaError> for KeyError {
    fn from(e: SymphoniaError) -> Self {
        KeyError::Decode(e.to_string())
    }
}

/// Key profiles (Krumhansl-Schmuckler)
const MAJOR_PROFILE: [f32; 12] = [6.35, 2.23, 3.48, 2.33, 4.38, 4.09, 2.52, 5.19, 2.39, 3.66, 2.29, 2.88];
const MINOR_PROFILE: [f32; 12] = [6.33, 2.68, 3.52, 5.38, 2.60, 3.53, 2.54, 4.75, 3.98, 2.69, 3.34, 3.17];

const NOTE_NAMES: [&str; 12] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "A#", "B"];

/// Decode a segment of the audio file to mono f32 PCM samples (reusing logic from BPM service)
fn decode_to_mono_f32(file_path: &str, duration_secs: u64) -> Result<(Vec<f32>, u32), KeyError> {
    let file = std::fs::File::open(file_path).map_err(KeyError::Io)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = Path::new(file_path).extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &Default::default(), &Default::default())
        .map_err(|e| KeyError::Decode(format!("probe failed: {}", e)))?;

    let mut format = probed.format;
    let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or(KeyError::NoAudioTrack)?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.ok_or_else(|| KeyError::Decode("unknown sample rate".into()))?;
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);
    
    // Skip 30s or start if shorter
    let total_frames = track.codec_params.n_frames;
    let skip_frames = total_frames.map(|f| f / 10).unwrap_or(sample_rate as u64 * 30).min(sample_rate as u64 * 30);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &Default::default())
        .map_err(|e| KeyError::Decode(format!("codec init failed: {}", e)))?;

    let mut mono_samples: Vec<f32> = Vec::new();
    let mut current_frame: u64 = 0;
    let max_samples = duration_secs as usize * sample_rate as usize;

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id { continue; }
        if current_frame < skip_frames {
            current_frame += packet.dur();
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            _ => continue,
        };
        let n = decoded.frames();
        match &decoded {
            AudioBufferRef::F32(buf) => for i in 0..n {
                let mut s = 0.0; for c in 0..channels { s += buf.chan(c)[i]; }
                mono_samples.push(s / channels as f32);
            }
            AudioBufferRef::S16(buf) => for i in 0..n {
                let mut s = 0.0; for c in 0..channels { s += buf.chan(c)[i] as f32 / 32768.0; }
                mono_samples.push(s / channels as f32);
            }
            _ => {}
        }
        if mono_samples.len() >= max_samples { break; }
    }
    Ok((mono_samples, sample_rate))
}

pub fn detect_key(file_path: &str) -> Result<String, KeyError> {
    // 1. Decode ~30 seconds for key analysis (usually plenty)
    let (samples, sample_rate) = decode_to_mono_f32(file_path, 30)?;
    if samples.is_empty() {
        return Err(KeyError::Decode("No samples".into()));
    }

    // 2. Compute Chromagram
    let window_size = 8192; // High resolution for low frequencies
    let hop_size = window_size / 2;
    let mut chromagram = [0.0f32; 12];
    
    let mut planner = FftPlanner::new();
    let fft = planner.plan_fft_forward(window_size);
    
    let mut i = 0;
    while i + window_size <= samples.len() {
        let mut buffer: Vec<Complex<f32>> = samples[i..i+window_size]
            .iter()
            .enumerate()
            .map(|(idx, &s)| {
                // Hanning window
                let w = index_to_hanning(idx, window_size);
                Complex::new(s * w, 0.0)
            })
            .collect();

        fft.process(&mut buffer);

        // Analyze bins (first half only)
        for (bin, complex_val) in buffer.iter().enumerate().take(window_size / 2) {
            let freq = bin as f32 * sample_rate as f32 / window_size as f32;
            if !(20.0..=5000.0).contains(&freq) { continue; } // Human hearing range for musical notes
            
            let magnitude = (complex_val.re.powi(2) + complex_val.im.powi(2)).sqrt();
            let midi_note = 12.0 * (freq / 440.0).log2() + 69.0;
            let note_idx = (midi_note.round() as i32 % 12 + 12) % 12;
            chromagram[note_idx as usize] += magnitude;
        }

        i += hop_size;
    }

    // 3. Normalize chromagram
    let sum: f32 = chromagram.iter().sum();
    if sum > 0.0 {
        for val in chromagram.iter_mut() {
            *val /= sum;
        }
    }

    // 4. Correlate with profiles
    let mut best_score = -1.0;
    let mut best_key = "Unknown".to_string();

    for is_minor in [false, true] {
        let profile = if is_minor { &MINOR_PROFILE } else { &MAJOR_PROFILE };
        for (shift, root) in NOTE_NAMES.iter().enumerate() {
            let mut score = 0.0;
            for (i, &p_val) in profile.iter().enumerate().take(12) {
                let chrom_idx = (i + shift) % 12;
                score += chromagram[chrom_idx] * p_val;
            }
            
            if score > best_score {
                best_score = score;
                best_key = format!("{} {}", root, if is_minor { "Minor" } else { "Major" });
            }
        }
    }

    debug!("Key detection result for {}: {} (score: {:.4})", file_path, best_key, best_score);
    Ok(best_key)
}

fn index_to_hanning(idx: usize, len: usize) -> f32 {
    0.5 * (1.0 - (2.0 * std::f32::consts::PI * idx as f32 / (len - 1) as f32).cos())
}

pub fn to_camelot(key: &str) -> String {
    match key {
        "G# Minor" => "1A".to_string(), "B Major" => "1B".to_string(),
        "D# Minor" => "2A".to_string(), "F# Major" => "2B".to_string(),
        "A# Minor" => "3A".to_string(), "C# Major" => "3B".to_string(),
        "F Minor" => "4A".to_string(), "G# Major" => "4B".to_string(),
        "C Minor" => "5A".to_string(), "D# Major" => "5B".to_string(),
        "G Minor" => "6A".to_string(), "A# Major" => "6B".to_string(),
        "D Minor" => "7A".to_string(), "F Major" => "7B".to_string(),
        "A Minor" => "8A".to_string(), "C Major" => "8B".to_string(),
        "E Minor" => "9A".to_string(), "G Major" => "9B".to_string(),
        "B Minor" => "10A".to_string(), "D Major" => "10B".to_string(),
        "F# Minor" => "11A".to_string(), "A Major" => "11B".to_string(),
        "C# Minor" => "12A".to_string(), "E Major" => "12B".to_string(),
        _ => key.to_string(),
    }
}
