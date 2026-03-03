/**
 * Shared audio utility helpers.
 * Used by the DJ decks, the global player, waveform editors, etc.
 */

// ─── Time Formatting ─────────────────────────────────────────────

/**
 * Format seconds to mm:ss or hh:mm:ss string.
 */
export const formatTime = (seconds: number): string => {
  if (!isFinite(seconds) || seconds < 0) return '0:00'
  const hrs = Math.floor(seconds / 3600)
  const mins = Math.floor((seconds % 3600) / 60)
  const secs = Math.floor(seconds % 60)
  if (hrs > 0) {
    return `${hrs}:${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

/**
 * Format seconds to mm:ss.ms with centisecond precision (DJ-style display).
 */
export const formatTimePrecise = (seconds: number): string => {
  if (!isFinite(seconds) || seconds < 0) return '00:00.00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  const cs = Math.floor((seconds % 1) * 100)
  return `${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}.${String(cs).padStart(2, '0')}`
}

/**
 * Format remaining time with a minus sign (CDJ style: -mm:ss).
 */
export const formatTimeRemaining = (current: number, total: number): string => {
  const remaining = Math.max(0, total - current)
  return `-${formatTimePrecise(remaining)}`
}

// ─── BPM / Tempo ─────────────────────────────────────────────────

/**
 * Calculate the effective BPM given a base BPM and a tempo percentage adjustment.
 * @param baseBpm Original BPM of the track
 * @param tempoPercent Adjustment in percent (e.g. +3.5 means 3.5% faster)
 */
export const effectiveBpm = (baseBpm: number, tempoPercent: number): number => {
  return baseBpm * (1 + tempoPercent / 100)
}

/**
 * Calculate the playback rate for a given tempo adjustment.
 * @param tempoPercent Adjustment in percent (e.g. +3.5 means 3.5% faster)
 */
export const tempoToPlaybackRate = (tempoPercent: number): number => {
  return 1 + tempoPercent / 100
}

/**
 * Clamp a value between min and max.
 */
export const clamp = (value: number, min: number, max: number): number => {
  return Math.min(Math.max(value, min), max)
}

/**
 * Snap a tempo value to 0 if within a threshold (for tempo faders).
 */
export const snapToZero = (value: number, threshold = 0.05): number => {
  return Math.abs(value) < threshold ? 0 : value
}

/**
 * Calculate pitch percentage from semitones.
 */
export const semitonesToPercent = (semitones: number): number => {
  return (Math.pow(2, semitones / 12) - 1) * 100
}

/**
 * Calculate beats per bar based on time signature (default 4/4).
 */
export const beatsPerBar = (timeSignature = 4): number => timeSignature

/**
 * Calculate the duration (in seconds) of one beat at a given BPM.
 */
export const beatDuration = (bpm: number): number => {
  if (bpm <= 0) return 0
  return 60 / bpm
}

/**
 * Calculate the duration (in seconds) of one bar at a given BPM.
 */
export const barDuration = (bpm: number, beatsPerBar = 4): number => {
  return beatDuration(bpm) * beatsPerBar
}

/**
 * Given a position in seconds and a BPM, calculate which beat number we're on.
 */
export const positionToBeat = (positionSeconds: number, bpm: number, offsetSeconds = 0): number => {
  if (bpm <= 0) return 0
  return Math.floor((positionSeconds - offsetSeconds) / beatDuration(bpm))
}

/**
 * Given a position in seconds and a BPM, calculate which bar number we're on.
 */
export const positionToBar = (positionSeconds: number, bpm: number, offsetSeconds = 0, beats = 4): number => {
  if (bpm <= 0) return 0
  return Math.floor((positionSeconds - offsetSeconds) / barDuration(bpm, beats))
}

/**
 * Calculate the phase (0-1) within the current beat.
 */
export const beatPhase = (positionSeconds: number, bpm: number, offsetSeconds = 0): number => {
  if (bpm <= 0) return 0
  const bd = beatDuration(bpm)
  return ((positionSeconds - offsetSeconds) % bd) / bd
}

// ─── Musical Key Helpers ─────────────────────────────────────────

const CAMELOT_MAP: Record<string, string> = {
  'C Major': '8B', 'C': '8B', 'Am': '8A', 'A Minor': '8A',
  'G Major': '9B', 'G': '9B', 'Em': '9A', 'E Minor': '9A',
  'D Major': '10B', 'D': '10B', 'Bm': '10A', 'B Minor': '10A',
  'A Major': '11B', 'A': '11B', 'F#m': '11A', 'F# Minor': '11A',
  'E Major': '12B', 'E': '12B', 'C#m': '12A', 'C# Minor': '12A',
  'B Major': '1B', 'B': '1B', 'G#m': '1A', 'G# Minor': '1A',
  'F# Major': '2B', 'F#': '2B', 'Ebm': '2A', 'D# Minor': '2A',
  'Db Major': '3B', 'Db': '3B', 'C# Major': '3B', 'Bbm': '3A', 'A# Minor': '3A',
  'Ab Major': '4B', 'Ab': '4B', 'G# Major': '4B', 'Fm': '4A', 'F Minor': '4A',
  'Eb Major': '5B', 'Eb': '5B', 'D# Major': '5B', 'Cm': '5A', 'C Minor': '5A',
  'Bb Major': '6B', 'Bb': '6B', 'A# Major': '6B', 'Gm': '6A', 'G Minor': '6A',
  'F Major': '7B', 'F': '7B', 'Dm': '7A', 'D Minor': '7A',
}

/**
 * Convert a musical key to Camelot notation.
 */
export const toCamelot = (key: string | null | undefined): string => {
  if (!key) return ''
  return CAMELOT_MAP[key] || key
}

/**
 * Check if two keys are harmonically compatible (Camelot wheel adjacency).
 */
export const areKeysCompatible = (key1: string | null | undefined, key2: string | null | undefined): boolean => {
  const c1 = toCamelot(key1)
  const c2 = toCamelot(key2)
  if (!c1 || !c2) return false
  if (c1 === c2) return true
  const n1 = parseInt(c1), l1 = c1.slice(-1)
  const n2 = parseInt(c2), l2 = c2.slice(-1)
  if (isNaN(n1) || isNaN(n2)) return false
  // Same number, different letter (relative major/minor)
  if (n1 === n2) return true
  // Same letter, adjacent numbers (wrapping 1-12)
  if (l1 === l2) {
    const diff = Math.abs(n1 - n2)
    return diff === 1 || diff === 11
  }
  return false
}

// ─── Color Helpers (for waveforms/spectrograms) ──────────────────

/**
 * Map a frequency bin (0-1 normalized) to an RGB color for spectrogram display.
 */
export const frequencyToColor = (normalizedFreq: number, intensity: number): string => {
  // Low freq = red/warm, Mid = green/yellow, High = blue/cyan
  const h = (1 - normalizedFreq) * 270 // hue from blue(270) to red(0)
  const s = 80 + intensity * 20
  const l = 10 + intensity * 50
  return `hsl(${h}, ${s}%, ${l}%)`
}

/**
 * Map amplitude (0-1) to a waveform color using a gradient.
 */
export const amplitudeToColor = (amplitude: number, baseColor = '#4f46e5', peakColor = '#22d3ee'): string => {
  // Simple interpolation
  const t = clamp(amplitude, 0, 1)
  if (t < 0.7) return baseColor
  return peakColor
}

// ─── dB Conversion ──────────────────────────────────────────────

/**
 * Convert a linear gain (0-1+) to decibels.
 */
export const linearToDb = (gain: number): number => {
  if (gain <= 0) return -Infinity
  return 20 * Math.log10(gain)
}

/**
 * Convert decibels to linear gain.
 */
export const dbToLinear = (db: number): number => {
  return Math.pow(10, db / 20)
}

/**
 * Format a dB value for display.
 */
export const formatDb = (db: number): string => {
  if (db <= -60) return '-∞'
  return `${db >= 0 ? '+' : ''}${db.toFixed(1)}`
}
