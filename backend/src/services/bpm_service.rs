use std::path::Path;
use std::io;
use log::debug;
use thiserror::Error;
use aubio::{Tempo, OnsetMode};

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::probe::Hint;
use symphonia::core::errors::Error as SymphoniaError;

#[derive(Error, Debug)]
pub enum BpmError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),
    #[error("Decode error: {0}")]
    Decode(String),
    #[allow(dead_code)]
    #[error("Parse error: {0}")]
    Parse(String),
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
    pub offset: f64, // first beat in seconds
    pub beats: Vec<f64>, // list of all detected beat timestamps
}

/// Detect BPM and Grid Offset of an audio file using aubio-rs.
pub fn detect_bpm(file_path: &str) -> Result<BpmResult, BpmError> {
    let path = Path::new(file_path);
    if !path.exists() {
        return Err(BpmError::NotFound);
    }

    debug!("BPM detection (aubio) starting for: {}", file_path);

    let file = std::fs::File::open(file_path).map_err(BpmError::Io)?;
    let mss = MediaSourceStream::new(Box::new(file), Default::default());

    let mut hint = Hint::new();
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        hint.with_extension(ext);
    }

    let probed = symphonia::default::get_probe()
        .format(&hint, mss, &Default::default(), &Default::default())
        .map_err(|e| BpmError::Decode(format!("probe failed: {}", e)))?;

    let mut format = probed.format;
    let track = format.tracks().iter()
        .find(|t| t.codec_params.codec != symphonia::core::codecs::CODEC_TYPE_NULL)
        .ok_or(BpmError::NoAudioTrack)?;

    let track_id = track.id;
    let sample_rate = track.codec_params.sample_rate.ok_or_else(|| BpmError::Decode("unknown sample rate".into()))?;
    let channels = track.codec_params.channels.map(|c| c.count()).unwrap_or(1);

    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &Default::default())
        .map_err(|e| BpmError::Decode(format!("codec init failed: {}", e)))?;

    // Optimized settings for modern high-tempo music (DnB, Techno)
    let hop_size = 512;
    let buf_size = 1024;

    // Try multiple onset detection modes to be more robust across genres
    // (keep list conservative to match available variants in the aubio crate)
    let onset_modes = [OnsetMode::SpecFlux, OnsetMode::Hfc];
    let mut tempo_opt: Option<Tempo> = None;
    for mode in onset_modes.iter() {
        match Tempo::new(*mode, buf_size, hop_size, sample_rate) {
            Ok(mut t) => {
                // Slightly less aggressive silence threshold to avoid dropping quiet beats
                t.set_silence(-60.0);
                tempo_opt = Some(t);
                debug!("BPM detector using onset mode: {:?}", mode);
                break;
            }
            Err(e) => {
                debug!("Tempo init failed for mode {:?}: {:?}", mode, e);
                continue;
            }
        }
    }

    let mut tempo = tempo_opt.ok_or_else(|| BpmError::Aubio("failed to initialize aubio tempo detector".into()))?;

    let mut first_beat_seconds: Option<f64> = None;
    let mut all_beats = Vec::new();
    let mut current_frame: u64 = 0;
    
    // Analyze up to 10 minutes for building a comprehensive beat map
    let max_analyze_frames = sample_rate as u64 * 600;
    let mut processing_buffer: Vec<f32> = Vec::with_capacity(hop_size);

    while let Ok(packet) = format.next_packet() {
        if packet.track_id() != track_id { continue; }
        
        let decoded = match decoder.decode(&packet) {
            Ok(d) => d,
            Err(_) => continue,
        };

        let n = decoded.frames();
        
        match &decoded {
            AudioBufferRef::F32(buf) => {
                for i in 0..n {
                    let mut s = 0.0;
                    for c in 0..channels { s += buf.chan(c)[i]; }
                    processing_buffer.push(s / channels as f32);

                    if processing_buffer.len() == hop_size {
                        if let Ok(is_beat) = tempo.do_result(&processing_buffer) {
                            if is_beat > 0.0 {
                                let ts = tempo.get_last_s() as f64;
                                if first_beat_seconds.is_none() {
                                    first_beat_seconds = Some(ts);
                                }
                                all_beats.push(ts);
                            }
                        }
                        processing_buffer.clear();
                    }
                }
            }
            AudioBufferRef::S16(buf) => {
                for i in 0..n {
                    let mut s = 0.0;
                    for c in 0..channels { s += buf.chan(c)[i] as f32 / 32768.0; }
                    processing_buffer.push(s / channels as f32);

                    if processing_buffer.len() == hop_size {
                        if let Ok(is_beat) = tempo.do_result(&processing_buffer) {
                            if is_beat > 0.0 {
                                let ts = tempo.get_last_s() as f64;
                                if first_beat_seconds.is_none() {
                                    first_beat_seconds = Some(ts);
                                }
                                all_beats.push(ts);
                            }
                        }
                        processing_buffer.clear();
                    }
                }
            }
            AudioBufferRef::U8(buf) => {
                for i in 0..n {
                    let mut s = 0.0;
                    for c in 0..channels { s += (buf.chan(c)[i] as f32 - 128.0) / 128.0; }
                    processing_buffer.push(s / channels as f32);
                    if processing_buffer.len() == hop_size {
                        if let Ok(is_beat) = tempo.do_result(&processing_buffer) {
                            if is_beat > 0.0 {
                                let ts = tempo.get_last_s() as f64;
                                if first_beat_seconds.is_none() {
                                    first_beat_seconds = Some(ts);
                                }
                                all_beats.push(ts);
                            }
                        }
                        processing_buffer.clear();
                    }
                }
            }
            AudioBufferRef::S32(buf) => {
                for i in 0..n {
                    let mut s = 0.0;
                    for c in 0..channels { s += buf.chan(c)[i] as f32 / 2147483648.0; }
                    processing_buffer.push(s / channels as f32);
                    if processing_buffer.len() == hop_size {
                        if let Ok(is_beat) = tempo.do_result(&processing_buffer) {
                            if is_beat > 0.0 {
                                let ts = tempo.get_last_s() as f64;
                                if first_beat_seconds.is_none() {
                                    first_beat_seconds = Some(ts);
                                }
                                all_beats.push(ts);
                            }
                        }
                        processing_buffer.clear();
                    }
                }
            }
            AudioBufferRef::F64(buf) => {
                for i in 0..n {
                    let mut s = 0.0;
                    for c in 0..channels { s += buf.chan(c)[i] as f32; }
                    processing_buffer.push(s / channels as f32);
                    if processing_buffer.len() == hop_size {
                        if let Ok(is_beat) = tempo.do_result(&processing_buffer) {
                            if is_beat > 0.0 {
                                let ts = tempo.get_last_s() as f64;
                                if first_beat_seconds.is_none() {
                                    first_beat_seconds = Some(ts);
                                }
                                all_beats.push(ts);
                            }
                        }
                        processing_buffer.clear();
                    }
                }
            }
            _ => {
                // Unhandled audio buffer type - skip
            }
        }
        
        let frames_processed = decoded.frames() as u64;
        current_frame += frames_processed;

        if current_frame > max_analyze_frames {
            break;
        }
    }

    let bpm = tempo.get_bpm();
    let confidence = tempo.get_confidence();

    if confidence < 0.01 && all_beats.is_empty() {
        return Err(BpmError::Inconclusive);
    }

    // Round to 1 decimal place (standard for BPM)
    let bpm_rounded = (bpm * 10.0).round() / 10.0;
    let offset = first_beat_seconds.unwrap_or(0.0);
    
    debug!("BPM detection complete: {:.1} BPM, {} beats (conf: {:.4})", 
          bpm_rounded, all_beats.len(), confidence);

    Ok(BpmResult {
        bpm: bpm_rounded as f64,
        offset,
        beats: all_beats,
    })
}
