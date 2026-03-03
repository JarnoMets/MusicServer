/**
 * DJ Audio Engine composable.
 *
 * Manages the Web Audio API graph for 4 decks + mixer:
 *
 *   [Deck 1 MediaElement] → GainNode (trim) → BiquadFilter (Hi) → BiquadFilter (Mid) → BiquadFilter (Lo) → BiquadFilter (filter) → GainNode (fader) → ─┐
 *   [Deck 2 MediaElement] → ...same chain... → ──────────────────────────────────────────────────────────────────────────────────────────────────────────┤
 *   [Deck 3 MediaElement] → ...same chain... → ──────────────────────────────────────────────────────────────────────────────────────────────────────────┼→ GainNode (master) → AnalyserNode → Destination
 *   [Deck 4 MediaElement] → ...same chain... → ──────────────────────────────────────────────────────────────────────────────────────────────────────────┘
 *
 * Also provides per-channel AnalyserNodes for metering and a headphone bus.
 */
import { ref, onUnmounted } from 'vue'
import type { DeckId } from '../types/dj'
import { useDjStore } from '../stores/djStore'
import { getAPIBaseURL } from '../utils/api'
import { tempoToPlaybackRate, dbToLinear, effectiveBpm } from '../utils/audioHelpers'
import { musicAPI } from '../api/music'

const API_BASE_URL = getAPIBaseURL()

interface DeckAudioNodes {
  audio: HTMLAudioElement
  source: MediaElementAudioSourceNode | null
  gainTrim: GainNode
  eqHi: BiquadFilterNode
  eqMid: BiquadFilterNode
  eqLow: BiquadFilterNode
  filterNode: BiquadFilterNode
  fader: GainNode
  analyser: AnalyserNode
  syncHandled?: boolean // To prevent multiple sync triggers for the same track load
}

// Singleton audio context (shared across all useDjAudioEngine calls)
let audioCtx: AudioContext | null = null
let masterGain: GainNode | null = null
let masterAnalyser: AnalyserNode | null = null
let headphoneGain: GainNode | null = null
const deckNodes: Map<DeckId, DeckAudioNodes> = new Map()
let animFrameId: number | null = null
let initialized = false

const getAudioContext = (): AudioContext => {
  if (!audioCtx) {
    audioCtx = new AudioContext()
  }
  return audioCtx
}

const buildStreamUrl = (musicId: string): string => {
  const base = `${API_BASE_URL}/music/${musicId}/stream`
  try {
    const token = localStorage.getItem('music_auth_token')
    if (token) return `${base}?token=${encodeURIComponent(token)}`
  } catch { /* */ }
  return base
}

/**
 * Create the Web Audio graph for one deck channel.
 */
const createDeckNodes = (_deckId: DeckId, ctx: AudioContext): DeckAudioNodes => {
  const audio = new Audio()
  audio.crossOrigin = 'anonymous'
  audio.preload = 'auto'

  const gainTrim = ctx.createGain()
  gainTrim.gain.value = 1

  // 3-band EQ using peaking filters
  const eqHi = ctx.createBiquadFilter()
  eqHi.type = 'highshelf'
  eqHi.frequency.value = 3200
  eqHi.gain.value = 0

  const eqMid = ctx.createBiquadFilter()
  eqMid.type = 'peaking'
  eqMid.frequency.value = 1000
  eqMid.Q.value = 0.7
  eqMid.gain.value = 0

  const eqLow = ctx.createBiquadFilter()
  eqLow.type = 'lowshelf'
  eqLow.frequency.value = 250
  eqLow.gain.value = 0

  // Channel filter (LP/HP sweep)
  const filterNode = ctx.createBiquadFilter()
  filterNode.type = 'allpass'
  filterNode.frequency.value = 1000
  filterNode.Q.value = 1

  // Channel fader
  const fader = ctx.createGain()
  fader.gain.value = 0.8 // Standard baseline (0.8)

  // Per-channel analyser for metering
  const analyser = ctx.createAnalyser()
  analyser.fftSize = 256 // Reduced for meter performance
  analyser.smoothingTimeConstant = 0.5 // Faster response for meters

  // Wire: trim → eqHi → eqMid → eqLow → filter → fader → analyser
  gainTrim.connect(eqHi)
  eqHi.connect(eqMid)
  eqMid.connect(eqLow)
  eqLow.connect(filterNode)
  filterNode.connect(fader)
  fader.connect(analyser)

  return {
    audio,
    source: null, // Will be created when audio loads
    gainTrim,
    eqHi,
    eqMid,
    eqLow,
    filterNode,
    fader,
    analyser,
  }
}

export const useDjAudioEngine = () => {
  const store = useDjStore()
  const isInitialized = ref(initialized)
  const waveformLoading = ref<Set<DeckId>>(new Set())

  /**
   * Initialize the entire audio engine. Must be called from a user gesture.
   */
  const init = async () => {
    if (initialized) return

    const ctx = getAudioContext()

    // Master bus
    masterGain = ctx.createGain()
    masterGain.gain.value = store.mixer.masterVolume

    const limiter = ctx.createDynamicsCompressor()
    limiter.threshold.value = -0.5 // Start at -0.5dB (soft clip avoidance)
    limiter.knee.value = 0
    limiter.ratio.value = 20
    limiter.attack.value = 0.003
    limiter.release.value = 0.1

    masterAnalyser = ctx.createAnalyser()
    masterAnalyser.fftSize = 1024
    masterAnalyser.smoothingTimeConstant = 0.6 // Slightly faster but still smooth

    headphoneGain = ctx.createGain()
    headphoneGain.gain.value = store.mixer.headphoneVolume

    masterGain.connect(limiter)
    limiter.connect(masterAnalyser)
    masterAnalyser.connect(ctx.destination)

    // Create 4 deck channels
    for (const deckId of [1, 2, 3, 4] as DeckId[]) {
      const nodes = createDeckNodes(deckId, ctx)
      nodes.analyser.connect(masterGain)
      deckNodes.set(deckId, nodes)
      setupAudioEvents(deckId, nodes)
    }

    initialized = true
    isInitialized.value = true

    // Resume context if suspended (may require gesture but nodes are now created)
    if (ctx.state === 'suspended') {
      try {
        await ctx.resume()
      } catch (e) {
        console.warn('AudioContext resume failed, user gesture needed:', e)
      }
    }

    startMeterLoop()
  }

  /**
   * Set up event listeners for a deck's audio element.
   */
  const setupAudioEvents = (deckId: DeckId, nodes: DeckAudioNodes) => {
    const { audio } = nodes

    audio.addEventListener('loadedmetadata', () => {
      store.setDuration(deckId, audio.duration)
      store.setPlayState(deckId, 'cued')

      // Create MediaElementSource once the audio is loaded
      if (!nodes.source) {
        const ctx = getAudioContext()
        nodes.source = ctx.createMediaElementSource(audio)
        nodes.source.connect(nodes.gainTrim)
      }
    })

    audio.addEventListener('canplaythrough', () => {
      // If sync is active, perform auto-sync on load
      // We use canplaythrough to ensure the browser is ready to play and seek instantly
      performAutoSyncOnLoad(deckId)
    }, { once: false }) // canplaythrough can fire again if src changes

    audio.addEventListener('timeupdate', () => {
      store.setCurrentTime(deckId, audio.currentTime)

      // Loop handling
      const deck = store.getDeck(deckId)
      if (deck.loop.active && deck.loop.outPoint !== null && audio.currentTime >= deck.loop.outPoint) {
        if (deck.loop.inPoint !== null) {
          audio.currentTime = deck.loop.inPoint
        }
      }
    })

    audio.addEventListener('ended', () => {
      store.setPlayState(deckId, 'cued')
      store.setCurrentTime(deckId, 0)
    })

    audio.addEventListener('playing', () => {
      store.setPlayState(deckId, 'playing')
    })

    audio.addEventListener('pause', () => {
      const deck = store.getDeck(deckId)
      if (deck.playState !== 'empty' && deck.playState !== 'loading') {
        store.setPlayState(deckId, 'paused')
      }
    })
  }

  /**
   * Automatically equalize BPM, align phase, and start playback if sync is active on load.
   */
  const performAutoSyncOnLoad = (deckId: DeckId) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes || nodes.syncHandled) return

    const deck = store.getDeck(deckId)
    // Only proceed if SYNC is active and we have BPM
    if (!deck.syncActive || !deck.track?.bpm) return

    // 1. Equalize BPM (this updates the store's tempoPercent for all synced decks)
    store.equalizeSyncedBpm()
    
    // We must immediately apply the new tempo to the hardware BEFORE playing
    // otherwise it might start at 1.0/wrong speed for a few frames.
    const updatedDeck = store.getDeck(deckId)
    const rate = tempoToPlaybackRate(updatedDeck.tempoPercent)
    nodes.audio.playbackRate = rate
    nodes.audio.preservesPitch = updatedDeck.masterTempo

    // 2. Align Phase and Play if others are playing
    const otherDecks = store.decks.filter(d => d.id !== deckId && d.playState === 'playing' && d.track?.bpm)
    if (otherDecks.length > 0) {
      const source = otherDecks[0]
      const sourceEffBpm = effectiveBpm(source.track?.bpm || 0, source.tempoPercent)
      
      // Use 2-bar (8 beat) alignment
      const alignPeriod = (60 / sourceEffBpm) * 8
      
      // Calculate current phase of the master deck
      const sourcePhase = ((source.currentTime - (source.beatGridOffset || 0)) % alignPeriod + alignPeriod) % alignPeriod
      
      // Jump to match the source's current phase
      nodes.audio.currentTime = sourcePhase
      
      // Now that BPM and Phase are set, start playing immediately
      nodes.audio.play().catch(e => console.warn('[DJ] Sync auto-play failed:', e))
      console.log(`[DJ] Auto-synced deck ${deckId} on load to ${sourceEffBpm.toFixed(2)} BPM at phase ${sourcePhase.toFixed(3)}s`)
    }

    nodes.syncHandled = true
  }

  // ─── Deck Controls ────────────────────────────────────────────

  const loadTrackToDeck = async (deckId: DeckId, track: import('../types/index').MusicFile) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return

    nodes.syncHandled = false // Reset sync lock for new track

    // Stop current playback
    nodes.audio.pause()
    nodes.audio.currentTime = 0

    store.loadTrack(deckId, track)

    // Set audio source
    const url = buildStreamUrl(track.id)
    nodes.audio.src = url
    nodes.audio.load()

    // Generate waveform peaks
    generateWaveform(deckId, url)

    // Auto-detect BPM if the track doesn't have one
    if (!track.bpm || track.bpm <= 0) {
      triggerBpmDetection(deckId, track.id)
    }
  }

  /**
   * Trigger server-side BPM detection for a track and update deck state when done.
   * After BPM is known, re-run beat grid offset detection.
   */
  const triggerBpmDetection = async (deckId: DeckId, trackId: string) => {
    try {
      const response = await musicAPI.detectBpm(trackId)
      const data = response.data as { bpm?: number; offset?: number; track?: { bpm?: number; beat_grid_offset?: number } }
      const detectedBpm = data.bpm || data.track?.bpm
      const detectedOffset = data.offset || data.track?.beat_grid_offset
      
      if (detectedBpm && detectedBpm > 0) {
        const deck = store.getDeck(deckId)
        if (deck.track && deck.track.id === trackId) {
          deck.track.bpm = detectedBpm
          if (detectedOffset !== undefined) {
            deck.track.beat_grid_offset = detectedOffset
            store.setBeatGridOffset(deckId, detectedOffset)
          }
          
          if (deck.syncActive) {
            store.equalizeSyncedBpm()
            // If we missed the on-load sync because BPM was missing, do it now
            const nodes = deckNodes.get(deckId)
            if (nodes && !nodes.syncHandled) {
              performAutoSyncOnLoad(deckId)
            }
          }

          // Re-run beat offset detection now that we have BPM
          // Fetch audio again to do the detection (only if peaks available)
          if (deck.waveformPeaks) {
            const nodes = deckNodes.get(deckId)
            if (nodes?.audio.src) {
              try {
                const resp2 = await fetch(nodes.audio.src)
                const buf = await resp2.arrayBuffer()
                const offCtx = new OfflineAudioContext(1, 1, 44100)
                const audioBuf = await offCtx.decodeAudioData(buf)
                const offset = detectBeatGridOffset(audioBuf, detectedBpm)
                store.setBeatGridOffset(deckId, offset)
                console.log(`[DJ] Beat grid offset (post-BPM) for deck ${deckId}: ${offset.toFixed(4)}s at ${detectedBpm} BPM`)
              } catch { /* non-critical */ }
            }
          }
        }
        console.log(`[DJ] Auto-detected BPM for deck ${deckId}: ${detectedBpm}`)
      }
    } catch (e) {
      console.warn(`[DJ] BPM detection failed for deck ${deckId}:`, e)
    }
  }

  /**
   * Sync a deck's BPM and phase to another playing deck.
   * Returns true if sync was applied.
   */
  const syncBpmAndPhase = (targetDeckId: DeckId): boolean => {
    const targetDeck = store.getDeck(targetDeckId)
    if (!targetDeck.track?.bpm) return false

    // Find the best source deck (playing, has BPM)
    const sourceDeck = store.decks.find(
      d => d.id !== targetDeckId && d.playState === 'playing' && d.track?.bpm
    )
    if (!sourceDeck || !sourceDeck.track?.bpm) return false

    // 1. Match BPM
    const sourceEffBpm = effectiveBpm(sourceDeck.track.bpm, sourceDeck.tempoPercent)
    const neededPercent = ((sourceEffBpm / targetDeck.track.bpm) - 1) * 100
    const clamped = Math.max(-targetDeck.tempoRange, Math.min(targetDeck.tempoRange, neededPercent))
    updateTempo(targetDeckId, clamped)

    // 2. Phase alignment
    const targetNodes = deckNodes.get(targetDeckId)
    const sourceNodes = deckNodes.get(sourceDeck.id)
    if (!targetNodes || !sourceNodes) return true

    const beatLen = 60 / sourceEffBpm
    const sourceTime = sourceNodes.audio.currentTime
    const targetTime = targetNodes.audio.currentTime

    const sourcePhase = ((sourceTime - (sourceDeck.beatGridOffset || 0)) % beatLen + beatLen) % beatLen
    const targetPhase = ((targetTime - (targetDeck.beatGridOffset || 0)) % beatLen + beatLen) % beatLen
    let phaseDiff = sourcePhase - targetPhase

    // Normalize to nearest beat boundary (-half beat to +half beat)
    if (phaseDiff > beatLen / 2) phaseDiff -= beatLen
    if (phaseDiff < -beatLen / 2) phaseDiff += beatLen

    // Only nudge if the difference is significant (> 5ms)
    if (Math.abs(phaseDiff) > 0.005) {
      targetNodes.audio.currentTime = Math.max(0, targetTime + phaseDiff)
    }

    return true
  }

  /**
   * Smoothly nudge a deck to align its phase with another deck.
   * Instead of jumping, it slightly speeds up or slows down for a short period.
   */
  const nudgeToSync = (deckId: DeckId, sourceDeckId: DeckId) => {
    const deck = store.getDeck(deckId)
    const source = store.getDeck(sourceDeckId)
    if (!deck.track?.bpm || !source.track?.bpm) return

    const sourceEffBpm = effectiveBpm(source.track.bpm, source.tempoPercent)
    // Use 2-bar (8 beat) alignment
    const alignPeriod = (60 / sourceEffBpm) * 8
    
    const sourcePhase = ((source.currentTime - (source.beatGridOffset || 0)) % alignPeriod + alignPeriod) % alignPeriod
    const targetPhase = ((deck.currentTime - (deck.beatGridOffset || 0)) % alignPeriod + alignPeriod) % alignPeriod
    
    let phaseDiff = sourcePhase - targetPhase
    if (phaseDiff > alignPeriod / 2) phaseDiff -= alignPeriod
    if (phaseDiff < -alignPeriod / 2) phaseDiff += alignPeriod

    if (Math.abs(phaseDiff) < 0.005) return // Already synced

    // Nudge: apply a ±1% tempo offset for for a duration that fixes the phase
    // Time saved/lost = duration * nudgePercent
    // duration = phaseDiff / nudgePercent
    const nudgeAmount = phaseDiff > 0 ? 0.02 : -0.02 // 2% nudge
    const nudgeDuration = Math.abs(phaseDiff / nudgeAmount) * 1000 // in ms

    const originalTempo = deck.tempoPercent
    updateTempo(deckId, originalTempo + (nudgeAmount * 100))

    setTimeout(() => {
      updateTempo(deckId, originalTempo)
    }, Math.min(nudgeDuration, 2000)) // Cap nudge at 2 seconds
  }

  const play = (deckId: DeckId) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes || !nodes.audio.src) return

    const ctx = getAudioContext()
    if (ctx.state === 'suspended') ctx.resume()

    nodes.audio.play().catch(e => console.warn('Play failed:', e))
  }

  const pause = (deckId: DeckId) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    nodes.audio.pause()
  }

  const togglePlay = (deckId: DeckId) => {
    const deck = store.getDeck(deckId)
    if (deck.playState === 'playing') {
      pause(deckId)
    } else {
      play(deckId)
    }
  }

  const seekTo = (deckId: DeckId, time: number) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    nodes.audio.currentTime = Math.max(0, Math.min(time, nodes.audio.duration || 0))
  }

  const cuePlay = (deckId: DeckId) => {
    const deck = store.getDeck(deckId)
    if (deck.playState === 'playing') {
      // If playing, jump back to cue point and pause
      pause(deckId)
      const firstCue = deck.cuePoints[0]
      seekTo(deckId, firstCue?.position ?? 0)
      store.setPlayState(deckId, 'cued')
    } else {
      // Set cue at current position and play
      if (deck.cuePoints.length === 0) {
        store.addCuePoint(deckId, deck.currentTime)
      }
      play(deckId)
    }
  }

  const ejectDeck = (deckId: DeckId) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    nodes.audio.pause()
    nodes.audio.removeAttribute('src')
    nodes.audio.load()
    store.ejectTrack(deckId)
  }

  // ─── Tempo ────────────────────────────────────────────────────

  const updateTempo = (deckId: DeckId, percent: number) => {
    store.setTempo(deckId, percent)
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    const deck = store.getDeck(deckId)
    const rate = tempoToPlaybackRate(deck.tempoPercent)
    nodes.audio.playbackRate = rate
    if (deck.masterTempo) {
      // Key lock: preserve pitch while changing tempo
      // Note: preservesPitch is supported in modern browsers
      nodes.audio.preservesPitch = true
    } else {
      nodes.audio.preservesPitch = false
    }
  }

  // ─── Mixer Controls ───────────────────────────────────────────

  const updateChannelVolume = (deckId: DeckId, volume: number) => {
    store.setChannelVolume(deckId, volume)
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    const ch = store.getChannel(deckId)
    nodes.fader.gain.value = ch.mute ? 0 : volume
  }

  const updateChannelGain = (deckId: DeckId, gainDb: number) => {
    store.setChannelGain(deckId, gainDb)
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    nodes.gainTrim.gain.value = dbToLinear(gainDb)
  }

  const updateEq = (deckId: DeckId, band: 'hi' | 'mid' | 'low', value: number) => {
    store.setChannelEq(deckId, band, value)
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    const map = { hi: nodes.eqHi, mid: nodes.eqMid, low: nodes.eqLow }
    map[band].gain.value = value
  }

  const updateFilter = (deckId: DeckId, value: number) => {
    store.setChannelFilter(deckId, value)
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    if (Math.abs(value) < 0.05) {
      nodes.filterNode.type = 'allpass'
      nodes.filterNode.frequency.value = 1000
    } else if (value < 0) {
      // Low-pass filter
      nodes.filterNode.type = 'lowpass'
      // Map -1..0 to 200Hz..20000Hz (logarithmic)
      const normalized = 1 + value // 0..1
      nodes.filterNode.frequency.value = 200 * Math.pow(100, normalized)
    } else {
      // High-pass filter
      nodes.filterNode.type = 'highpass'
      // Map 0..1 to 20Hz..8000Hz (logarithmic)
      nodes.filterNode.frequency.value = 20 * Math.pow(400, value)
    }
    nodes.filterNode.Q.value = 1
  }

  const updateMasterVolume = (volume: number) => {
    store.setMasterVolume(volume)
    if (masterGain) {
      masterGain.gain.value = volume
    }
  }

  const updateHeadphoneVolume = (volume: number) => {
    store.setHeadphoneVolume(volume)
    if (headphoneGain) {
      headphoneGain.gain.value = volume
    }
  }

  const toggleMute = (deckId: DeckId) => {
    store.toggleChannelMute(deckId)
    const ch = store.getChannel(deckId)
    const nodes = deckNodes.get(deckId)
    if (nodes) {
      nodes.fader.gain.value = ch.mute ? 0 : ch.volume
    }
  }

  // ─── Crossfader ───────────────────────────────────────────────

  const updateCrossfader = (value: number) => {
    store.setCrossfader(value)
    applyCrossfader()
  }

  const applyCrossfader = () => {
    const cf = store.mixer.crossfader
    const curve = store.mixer.crossfaderCurve

    for (let i = 0; i < 4; i++) {
      const deckId = (i + 1) as DeckId
      const assign = store.mixer.crossfaderAssign[i]
      const nodes = deckNodes.get(deckId)
      if (!nodes) continue

      const ch = store.getChannel(deckId)
      let cfGain = 1

      if (assign === 'THRU') {
        cfGain = 1
      } else if (assign === 'A') {
        // A side: full volume at cf=-1, fades as cf goes to +1
        const t = (cf + 1) / 2 // 0 (full A) to 1 (full B)
        cfGain = curve === 'sharp' ? (t < 0.5 ? 1 : 0) : (1 - t)
      } else if (assign === 'B') {
        const t = (cf + 1) / 2
        cfGain = curve === 'sharp' ? (t > 0.5 ? 1 : 0) : t
      }

      nodes.fader.gain.value = ch.mute ? 0 : ch.volume * cfGain
    }
  }

  // ─── Waveform Generation ──────────────────────────────────────

  /**
   * Detect beat grid offset using onset strength envelope + phase search.
   * Returns seconds of the first beat.
   */
  const detectBeatGridOffset = (audioBuffer: AudioBuffer, bpm: number): number => {
    if (bpm <= 0) return 0
    const sampleRate = audioBuffer.sampleRate
    const ch0 = audioBuffer.getChannelData(0)

    // Compute RMS energy envelope at 100 fps (hopSize = sampleRate / 100)
    const hopSize = Math.max(1, Math.floor(sampleRate / 100))
    const frameSize = hopSize * 2
    const numFrames = Math.floor(ch0.length / hopSize)
    if (numFrames < 4) return 0

    const envelope = new Float32Array(numFrames)
    let prevEnergy = 0
    for (let f = 0; f < numFrames; f++) {
      const start = f * hopSize
      let energy = 0
      const end = Math.min(ch0.length, start + frameSize)
      for (let i = start; i < end; i++) {
        energy += ch0[i] * ch0[i]
      }
      energy /= (end - start)
      // Half-wave rectified spectral flux (energy increase only)
      envelope[f] = Math.max(0, energy - prevEnergy)
      prevEnergy = energy
    }

    // Beat period in envelope frames (100 fps)
    const beatPeriodFrames = (60 / bpm) * 100

    // Phase search over one beat period to find offset with max onset energy
    const searchLen = Math.min(numFrames, Math.round(beatPeriodFrames))
    let bestOffset = 0
    let bestScore = -1
    for (let offset = 0; offset < searchLen; offset++) {
      let score = 0
      let k = 0
      while (true) {
        const frameIdx = Math.round(offset + k * beatPeriodFrames)
        if (frameIdx >= numFrames) break
        score += envelope[frameIdx]
        k++
      }
      if (score > bestScore) {
        bestScore = score
        bestOffset = offset
      }
    }

    return bestOffset / 100 // frames → seconds
  }

  const generateWaveform = async (deckId: DeckId, url: string) => {
    waveformLoading.value.add(deckId)
    try {
      const response = await fetch(url)
      const arrayBuffer = await response.arrayBuffer()
      // Decode at native sample rate (stereo if available)
      const offlineCtx = new OfflineAudioContext(1, 1, 44100)
      const audioBuffer = await offlineCtx.decodeAudioData(arrayBuffer)

      const ch0 = audioBuffer.getChannelData(0)
      const ch1 = audioBuffer.numberOfChannels > 1 ? audioBuffer.getChannelData(1) : null

      // 100 peaks per second for high-resolution zoom
      const samples = Math.max(1, Math.floor(audioBuffer.duration * 100))
      const blockSize = Math.max(1, Math.floor(ch0.length / samples))
      const peaks = new Float32Array(samples)

      for (let i = 0; i < samples; i++) {
        let peak = 0
        const start = i * blockSize
        const end = Math.min(ch0.length, start + blockSize)
        for (let j = start; j < end; j++) {
          const val = ch1
            ? Math.max(Math.abs(ch0[j]), Math.abs(ch1[j]))
            : Math.abs(ch0[j])
          if (val > peak) peak = val
        }
        peaks[i] = peak
      }

      // Normalize to 0-1 (use 99th percentile to avoid outlier spikes flattening the rest)
      const sorted = Float32Array.from(peaks).sort()
      const p99 = sorted[Math.floor(sorted.length * 0.99)] || 1
      const norm = p99 > 0 ? 1 / p99 : 1
      for (let i = 0; i < peaks.length; i++) {
        peaks[i] = Math.min(1, peaks[i] * norm)
      }

      store.setWaveformPeaks(deckId, peaks)

      // Detect beat grid offset if the track has a BPM but no offset yet
      const deck = store.getDeck(deckId)
      if (deck.track?.bpm && deck.track.bpm > 0) {
        if (deck.track.beat_grid_offset && deck.track.beat_grid_offset > 0) {
          store.setBeatGridOffset(deckId, deck.track.beat_grid_offset)
        } else {
          // Only perform local detection if the backend didn't provide it
          const offset = detectBeatGridOffset(audioBuffer, deck.track.bpm)
          store.setBeatGridOffset(deckId, offset)
          console.log(`[DJ] Beat grid offset (local) for deck ${deckId}: ${offset.toFixed(4)}s at ${deck.track.bpm} BPM`)
        }
      }
    } catch (e) {
      console.warn(`Failed to generate waveform for deck ${deckId}:`, e)
    } finally {
      waveformLoading.value.delete(deckId)
    }
  }

  // ─── Analyser / Metering ──────────────────────────────────────

  /**
   * Get frequency data for a deck's analyser (for spectrogram).
   */
  const getFrequencyData = (deckId: DeckId): Uint8Array | null => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return null
    const data = new Uint8Array(nodes.analyser.frequencyBinCount)
    nodes.analyser.getByteFrequencyData(data)
    return data
  }

  /**
   * Get time-domain data for a deck's analyser (for oscilloscope).
   */
  const getTimeDomainData = (deckId: DeckId): Uint8Array | null => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return null
    const data = new Uint8Array(nodes.analyser.frequencyBinCount)
    nodes.analyser.getByteTimeDomainData(data)
    return data
  }

  /**
   * Get master frequency data for master meter.
   */
  const getMasterFrequencyData = (): Uint8Array | null => {
    if (!masterAnalyser) return null
    const data = new Uint8Array(masterAnalyser.frequencyBinCount)
    masterAnalyser.getByteFrequencyData(data)
    return data
  }

  /**
   * Calculate level (RMS or Peak) from frequency or time data.
   * Using peak from time domain for the VU meters is more responsive and "standard".
   */
  const calculateLevel = (data: Uint8Array, type: 'peak' | 'rms' = 'peak'): number => {
    if (type === 'peak') {
      let maxVal = 0
      for (let i = 0; i < data.length; i++) {
        // time domain is 128-centered. Map 0..255 to 0..1 peak amplitude
        const val = Math.abs(data[i] - 128) / 128
        if (val > maxVal) maxVal = val
      }
      return maxVal
    } else {
      // RMS for frequency magnitude
      let sum = 0
      for (let i = 0; i < data.length; i++) {
        const val = data[i] / 255
        sum += val * val
      }
      return Math.sqrt(sum / data.length)
    }
  }

  /**
   * Animation loop for updating meter levels in the store.
   */
  const startMeterLoop = () => {
    const tick = () => {
      for (const deckId of [1, 2, 3, 4] as DeckId[]) {
        const data = getTimeDomainData(deckId)
        if (data) {
          const level = calculateLevel(data, 'peak')
          // Using a slight boost to help "orange lining" feel correct for pop music
          const boostedLevel = Math.min(1.0, level * 1.15)
          store.setChannelLevel(deckId, boostedLevel, boostedLevel)
        }
      }
      // Master level
      if (masterAnalyser) {
        const data = new Uint8Array(masterAnalyser.frequencyBinCount)
        masterAnalyser.getByteTimeDomainData(data)
        const masterLevel = calculateLevel(data, 'peak')
        const boostedMasterLevel = Math.min(1.0, masterLevel * 1.15)
        store.setMasterLevel(boostedMasterLevel, boostedMasterLevel)
      }
      store.decayPeakHold()
      animFrameId = requestAnimationFrame(tick)
    }
    animFrameId = requestAnimationFrame(tick)
  }

  // ─── Filter Sweep (for Echo Fade transitions) ────────────────

  /**
   * Smoothly sweep a deck's filter from open (allpass) to a tight low-pass
   * over `durationMs` milliseconds. Used during Echo Fade transitions when
   * the BPM difference is too large for tempo matching.
   *
   * @returns a cancel function that stops the sweep and resets the filter.
   */
  const sweepFilter = (deckId: DeckId, durationMs: number): (() => void) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return () => {}

    const ctx = getAudioContext()
    const now = ctx.currentTime
    const endTime = now + durationMs / 1000

    // Switch to lowpass and sweep frequency from 20kHz down to 200Hz
    nodes.filterNode.type = 'lowpass'
    nodes.filterNode.Q.value = 2 // gentle resonance for a warm sweep
    nodes.filterNode.frequency.cancelScheduledValues(now)
    nodes.filterNode.frequency.setValueAtTime(20000, now)
    nodes.filterNode.frequency.exponentialRampToValueAtTime(200, endTime)

    let cancelled = false
    const cancel = () => {
      if (cancelled) return
      cancelled = true
      resetFilter(deckId)
    }
    return cancel
  }

  /**
   * Reset a deck's filter back to allpass (fully open).
   */
  const resetFilter = (deckId: DeckId) => {
    const nodes = deckNodes.get(deckId)
    if (!nodes) return
    const ctx = getAudioContext()
    nodes.filterNode.frequency.cancelScheduledValues(ctx.currentTime)
    nodes.filterNode.type = 'allpass'
    nodes.filterNode.frequency.value = 1000
    nodes.filterNode.Q.value = 1
    store.setChannelFilter(deckId, 0)
  }

  const getCurrentTime = (deckId: DeckId): number => {
    const nodes = deckNodes.get(deckId)
    return nodes ? nodes.audio.currentTime : 0
  }

  // ─── Cleanup ──────────────────────────────────────────────────

  const destroy = () => {
    if (animFrameId !== null) {
      cancelAnimationFrame(animFrameId)
      animFrameId = null
    }
    for (const [, nodes] of deckNodes) {
      nodes.audio.pause()
      nodes.audio.removeAttribute('src')
    }
    // Don't close the AudioContext - it's singleton and may be reused
  }

  onUnmounted(() => {
    // We do NOT destroy on unmount because the engine is a singleton.
    // The DecksView will manage the lifecycle explicitly.
  })

  return {
    isInitialized,
    waveformLoading,
    init,
    destroy,
    // Deck controls
    loadTrackToDeck,
    play,
    pause,
    togglePlay,
    seekTo,
    cuePlay,
    ejectDeck,
    updateTempo,
    // Mixer
    updateChannelVolume,
    updateChannelGain,
    updateEq,
    updateFilter,
    updateMasterVolume,
    updateHeadphoneVolume,
    toggleMute,
    updateCrossfader,
    applyCrossfader,
    // Sync
    syncBpmAndPhase,
    nudgeToSync,
    // Filter sweep (for echo fade transitions)
    sweepFilter,
    resetFilter: resetFilter,
    // Analysis
    getFrequencyData,
    getTimeDomainData,
    getMasterFrequencyData,
    calculateLevel,
    getCurrentTime,
  }
}
