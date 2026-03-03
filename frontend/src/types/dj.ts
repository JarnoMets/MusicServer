/**
 * Types for the DJ Decks feature.
 * Shared across components, stores, and composables.
 */

import type { MusicFile } from './index'

// ─── Deck Types ─────────────────────────────────────────────────

export type DeckId = 1 | 2 | 3 | 4

export interface CuePoint {
  id: string
  position: number // seconds
  color: string
  label: string
}

export interface LoopState {
  active: boolean
  inPoint: number | null  // seconds
  outPoint: number | null // seconds
}

export type DeckPlayState = 'empty' | 'loading' | 'playing' | 'paused' | 'cued'

export interface DeckState {
  id: DeckId
  track: MusicFile | null
  playState: DeckPlayState
  currentTime: number    // seconds
  duration: number       // seconds
  tempoPercent: number   // -50 to +50 percent
  tempoRange: number     // ±6, ±10, ±16, ±50
  masterTempo: boolean   // key lock (pitch remains same while tempo changes)
  waveformPeaks: Float32Array | null
  cuePoints: CuePoint[]
  loop: LoopState
  slip: boolean          // slip mode: continues playing underneath while scratching/looping
  beatGridOffset: number // first downbeat position in seconds
  beatMap: number[] | null // detailed beat positions in seconds
  jogWheelRotation: number // radians
  syncActive: boolean     // whether sync is currently engaged for this deck
}

// ─── Mixer Types ─────────────────────────────────────────────────

export interface EqState {
  hi: number   // -26 to +6 dB (mapped to 0-1 for fader display)
  mid: number  // -26 to +6 dB
  low: number  // -26 to +6 dB
}

export interface ChannelState {
  deckId: DeckId
  volume: number        // 0-1 (channel fader)
  eq: EqState
  gain: number          // trim/gain knob: -12 to +12 dB
  filter: number        // -1 to +1 (left = LP, 0 = off, right = HP)
  cueMix: boolean       // headphone cue active
  mute: boolean
  peakLevel: number     // 0-1 current output level (for meter)
  peakHold: number      // 0-1 peak hold level (for meter)
}

export type CrossfaderCurve = 'smooth' | 'sharp' | 'through'
export type CrossfaderAssign = 'A' | 'B' | 'THRU'

export interface MixerState {
  channels: [ChannelState, ChannelState, ChannelState, ChannelState]
  crossfader: number    // -1 (A) to +1 (B)
  crossfaderCurve: CrossfaderCurve
  crossfaderAssign: [CrossfaderAssign, CrossfaderAssign, CrossfaderAssign, CrossfaderAssign]
  masterVolume: number  // 0-1
  boothVolume: number   // 0-1
  headphoneVolume: number // 0-1
  headphoneMix: number  // 0 (cue) to 1 (master)
  masterLevel: number   // 0-1 current output level
  masterPeakHold: number
}

// ─── Autoplay Settings ───────────────────────────────────────────

export interface AutoplaySettings {
  matchTimeSeconds: number  // time before end of track both songs start playing to match BPM
  overlapSeconds: number    // time both songs play together (crossfade / overlap)
  exitTimeSeconds: number   // time the new song takes to return to its original BPM after transition
}

export const createDefaultAutoplaySettings = (): AutoplaySettings => ({
  matchTimeSeconds: 60,
  overlapSeconds: 45,
  exitTimeSeconds: 30,
})

// ─── Session Types ───────────────────────────────────────────────

export interface DjSession {
  decks: [DeckState, DeckState, DeckState, DeckState]
  mixer: MixerState
  masterBpm: number | null // if sync is used, the master BPM
  syncEnabled: boolean
  autoplaySettings: AutoplaySettings
}

// ─── Browser Types ───────────────────────────────────────────────

export type BrowserSource = 'all' | 'playlist' | 'genre' | 'search'

export interface BrowserState {
  source: BrowserSource
  playlistId: string | null
  genreFilter: string
  searchQuery: string
}

// ─── Helper factories ────────────────────────────────────────────

export const createDefaultEq = (): EqState => ({
  hi: 0,
  mid: 0,
  low: 0,
})

export const createDefaultLoop = (): LoopState => ({
  active: false,
  inPoint: null,
  outPoint: null,
})

export const createDefaultDeck = (id: DeckId): DeckState => ({
  id,
  track: null,
  playState: 'empty',
  currentTime: 0,
  duration: 0,
  tempoPercent: 0,
  tempoRange: 6,
  masterTempo: false,
  waveformPeaks: null,
  cuePoints: [],
  loop: createDefaultLoop(),
  slip: false,
  beatGridOffset: 0,
  beatMap: null,
  jogWheelRotation: 0,
  syncActive: false,
})

export const createDefaultChannel = (deckId: DeckId): ChannelState => ({
  deckId,
  volume: 0.8,
  eq: createDefaultEq(),
  gain: 0,
  filter: 0,
  cueMix: false,
  mute: false,
  peakLevel: 0,
  peakHold: 0,
})

export const createDefaultMixer = (): MixerState => ({
  channels: [
    createDefaultChannel(1),
    createDefaultChannel(2),
    createDefaultChannel(3),
    createDefaultChannel(4),
  ],
  crossfader: 0,
  crossfaderCurve: 'smooth',
  crossfaderAssign: ['A', 'A', 'B', 'B'],
  masterVolume: 0.8,
  boothVolume: 0.5,
  headphoneVolume: 0.5,
  headphoneMix: 0.5,
  masterLevel: 0,
  masterPeakHold: 0,
})
