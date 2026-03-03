/**
 * Pinia store for DJ Decks state.
 * Manages 4 decks + mixer state, persists nothing (session-only).
 */
import { defineStore } from 'pinia'
import { reactive, ref, watch } from 'vue'
import type {
  DeckId,
  DeckState,
  MixerState,
  CuePoint,
  CrossfaderAssign,
  CrossfaderCurve,
  EqState,
  BrowserState,
  AutoplaySettings,
} from '../types/dj'
import {
  createDefaultDeck,
  createDefaultMixer,
  createDefaultLoop,
  createDefaultAutoplaySettings,
} from '../types/dj'
import type { MusicFile } from '../types/index'
import { musicAPI } from '../api/music'

export const useDjStore = defineStore('dj', () => {
  // ─── State ────────────────────────────────────────────────────
  const decks = reactive<[DeckState, DeckState, DeckState, DeckState]>([
    createDefaultDeck(1),
    createDefaultDeck(2),
    createDefaultDeck(3),
    createDefaultDeck(4),
  ])

  const mixer = reactive<MixerState>(createDefaultMixer())

  const masterBpm = ref<number | null>(null)
  const syncEnabled = ref(false)
  const autoPlay = ref(false)
  const autoPlayPlaylist = ref<MusicFile[]>([])
  const autoPlayIndex = ref(0) // Next track to load
  const isTransitioning = ref(false)

  // Autoplay settings (persisted)
  const autoplaySettings = reactive<AutoplaySettings>(createDefaultAutoplaySettings())
  const savedAutoplaySettings = localStorage.getItem('dj_autoplay_settings')
  if (savedAutoplaySettings) {
    try {
      const data = JSON.parse(savedAutoplaySettings)
      Object.assign(autoplaySettings, data)
    } catch { /* */ }
  }

  // Ensure logical constraint: overlapSeconds cannot exceed matchTimeSeconds
  const clampAutoplaySettings = () => {
    if (autoplaySettings.overlapSeconds > autoplaySettings.matchTimeSeconds) {
      autoplaySettings.overlapSeconds = autoplaySettings.matchTimeSeconds
    }
  }

  clampAutoplaySettings()

  watch(autoplaySettings, (val) => {
    clampAutoplaySettings()
    localStorage.setItem('dj_autoplay_settings', JSON.stringify(val))

    // Persist to backend (best-effort). Do not block store updates.
    musicAPI.updateAutoplayConfig(
      autoplaySettings.matchTimeSeconds,
      autoplaySettings.overlapSeconds,
      autoplaySettings.exitTimeSeconds
    ).catch(() => {})
  }, { deep: true })

  // Attempt to load persisted server-side settings on startup (non-blocking)
  ;(async () => {
    try {
      const res = await musicAPI.getAutoplayConfig()
      const json = res.data
      if (json && typeof json.match_time_seconds === 'number') {
        autoplaySettings.matchTimeSeconds = json.match_time_seconds
        autoplaySettings.overlapSeconds = json.overlap_seconds
        autoplaySettings.exitTimeSeconds = json.exit_time_seconds
        clampAutoplaySettings()
      }
    } catch (e) {
      // ignore network errors — fallback to local settings
    }
  })()

  // Browser State persistence
  const browserState = reactive<BrowserState>({
    source: 'all',
    playlistId: null,
    genreFilter: '',
    searchQuery: '',
  })

  // Persistence for browserState
  const savedBrowser = localStorage.getItem('dj_browser_state')
  if (savedBrowser) {
    try {
      const data = JSON.parse(savedBrowser)
      Object.assign(browserState, data)
    } catch { /* */ }
  }

  watch(browserState, (val) => {
    localStorage.setItem('dj_browser_state', JSON.stringify(val))
  }, { deep: true })

  // ─── Deck Helpers ─────────────────────────────────────────────

  const getDeck = (id: DeckId): DeckState => decks[id - 1]
  const getChannel = (id: DeckId) => mixer.channels[id - 1]

  // ─── Deck Actions ─────────────────────────────────────────────

  const loadTrack = (deckId: DeckId, track: MusicFile) => {
    const deck = getDeck(deckId)
    deck.track = track
    deck.playState = 'loading'
    deck.currentTime = 0
    deck.duration = (track.duration || 0) / 1000 // backend stores ms
    deck.tempoPercent = 0
    deck.cuePoints = []
    deck.beatGridOffset = track.beat_grid_offset || 0
    deck.beatMap = track.beat_map || null

    // Try to find BPM/Key from metadata
    const bpm = track.bpm || null
    if (bpm) {
      deck.track.bpm = bpm
      deck.tempoPercent = 0
    } else {
      deck.track.bpm = null
    }

    const key = track.initial_key || null
    if (deck.track) {
      deck.track.initial_key = key ? (key.startsWith('?') ? key : `/${key}`) : null
    }

    // Reset waveform and Cue Points
    deck.waveformPeaks = null
    // Restore any saved cue points for this track
    restoreCuePoints(deckId)

    // Ensure Cue Point 1 at 0.00 always exists
    if (!deck.cuePoints.some(c => c.position === 0)) {
      deck.cuePoints.unshift({
        id: `cue-auto-0-${Date.now()}`,
        position: 0,
        color: '#3b82f6', // blue
        label: 'CUE 1',
      })
    }
  }

  const ejectTrack = (deckId: DeckId) => {
    const deck = getDeck(deckId)
    deck.track = null
    deck.playState = 'empty'
    deck.currentTime = 0
    deck.duration = 0
    deck.waveformPeaks = null
    deck.cuePoints = []
    deck.loop = createDefaultLoop()
    deck.beatGridOffset = 0
    deck.beatMap = null
    deck.jogWheelRotation = 0
  }

  const setPlayState = (deckId: DeckId, state: DeckState['playState']) => {
    getDeck(deckId).playState = state
  }

  const setCurrentTime = (deckId: DeckId, time: number) => {
    getDeck(deckId).currentTime = time
  }

  const setDuration = (deckId: DeckId, duration: number) => {
    getDeck(deckId).duration = duration
  }

  const setWaveformPeaks = (deckId: DeckId, peaks: Float32Array) => {
    getDeck(deckId).waveformPeaks = peaks
  }

  const setTempo = (deckId: DeckId, percent: number) => {
    const deck = getDeck(deckId)
    const range = deck.tempoRange
    deck.tempoPercent = Math.max(-range, Math.min(range, percent))
  }

  const setTempoRange = (deckId: DeckId, range: number) => {
    const deck = getDeck(deckId)
    deck.tempoRange = range
    // Clamp current tempo to new range
    deck.tempoPercent = Math.max(-range, Math.min(range, deck.tempoPercent))
  }

  const toggleMasterTempo = (deckId: DeckId) => {
    getDeck(deckId).masterTempo = !getDeck(deckId).masterTempo
  }

  const setJogRotation = (deckId: DeckId, rotation: number) => {
    getDeck(deckId).jogWheelRotation = rotation
  }

  const setBeatGridOffset = (deckId: DeckId, offset: number) => {
    getDeck(deckId).beatGridOffset = offset
  }

  const toggleSync = (deckId: DeckId) => {
    const deck = getDeck(deckId)
    deck.syncActive = !deck.syncActive
    
    if (deck.syncActive) {
      equalizeSyncedBpm()
    }
  }

  /**
   * Calculate the average BPM of all synced decks and update them all.
   * Handles 2x/0.5x multiples as well.
   */
  const equalizeSyncedBpm = () => {
    const syncedDecks = decks.filter(d => !!(d.syncActive && d.track?.bpm))
    if (syncedDecks.length < 2) return

    // 1. Pick a reference BPM (first synced deck's current effective BPM)
    const refDeck = syncedDecks[0]
    const refTrackBpm = refDeck.track?.bpm
    if (!refTrackBpm) return

    const refBpm = refTrackBpm * (1 + refDeck.tempoPercent / 100)

    // 2. Sum up normalized effective BPMs
    let totalBpm = 0
    syncedDecks.forEach(d => {
      const trackBpm = d.track?.bpm
      if (!trackBpm) return

      let b = trackBpm * (1 + d.tempoPercent / 100)
      // Normalize to be in the same octave as refBpm
      while (b < refBpm * 0.75) b *= 2
      while (b > refBpm * 1.5) b /= 2
      totalBpm += b
    })

    const avgBpm = totalBpm / syncedDecks.length

    // 3. Update all synced decks to match this average
    syncedDecks.forEach(d => {
      const base = d.track?.bpm
      if (!base) return

      // Find which multiple of base BPM is closest to avgBpm
      let target = base
      while (target < avgBpm * 0.75) target *= 2
      while (target > avgBpm * 1.5) target /= 2
      
      const neededPercent = ((avgBpm / target) - 1) * 100
      setTempo(d.id, neededPercent)
    })
  }

  // ─── Cue Points ───────────────────────────────────────────────

  const CUE_COLORS = ['#ff3b30', '#ff9500', '#ffcc00', '#34c759', '#007aff', '#af52de', '#ff2d55', '#5856d6']

  const addCuePoint = (deckId: DeckId, position: number, label?: string) => {
    const deck = getDeck(deckId)
    const id = `cue-${Date.now()}`
    const colorIndex = deck.cuePoints.length % CUE_COLORS.length
    const cue: CuePoint = {
      id,
      position,
      color: CUE_COLORS[colorIndex],
      label: label || `CUE ${deck.cuePoints.length + 1}`,
    }
    deck.cuePoints.push(cue)
    persistCuePoints(deckId)
    return cue
  }

  /**
   * Set a cue point at a specific pad index (0-7).
   * If the index is beyond current length, fill gaps with null-like entries.
   */
  const setCuePointAt = (deckId: DeckId, index: number, position: number) => {
    const deck = getDeck(deckId)
    // Extend array if needed
    while (deck.cuePoints.length < index) {
      deck.cuePoints.push({
        id: `cue-empty-${deck.cuePoints.length}`,
        position: -1,
        color: CUE_COLORS[deck.cuePoints.length % CUE_COLORS.length],
        label: '',
      })
    }
    const cue: CuePoint = {
      id: `cue-${Date.now()}-${index}`,
      position,
      color: CUE_COLORS[index % CUE_COLORS.length],
      label: `CUE ${index + 1}`,
    }
    if (index < deck.cuePoints.length) {
      deck.cuePoints[index] = cue
    } else {
      deck.cuePoints.push(cue)
    }
    persistCuePoints(deckId)
    return cue
  }

  const removeCuePoint = (deckId: DeckId, cueId: string) => {
    const deck = getDeck(deckId)
    const idx = deck.cuePoints.findIndex(c => c.id === cueId)
    if (idx >= 0) {
      // Replace with empty slot to preserve indices
      deck.cuePoints[idx] = {
        id: '',
        position: -1,
        color: CUE_COLORS[idx % CUE_COLORS.length],
        label: '',
      }
    }
    persistCuePoints(deckId)
  }

  /**
   * Persist cue points to localStorage keyed by track ID.
   */
  const persistCuePoints = (deckId: DeckId) => {
    const deck = getDeck(deckId)
    if (!deck.track) return
    const key = `dj_cues_${deck.track.id}`
    const data = deck.cuePoints
      .filter(c => c.position >= 0)
      .map(c => ({ position: c.position, color: c.color, label: c.label }))
    try {
      localStorage.setItem(key, JSON.stringify(data))
    } catch { /* ignore */ }
  }

  /**
   * Restore cue points from localStorage when a track is loaded.
   */
  const restoreCuePoints = (deckId: DeckId) => {
    const deck = getDeck(deckId)
    if (!deck.track) return
    const key = `dj_cues_${deck.track.id}`
    try {
      const saved = localStorage.getItem(key)
      if (saved) {
        const data = JSON.parse(saved) as { position: number; color: string; label: string }[]
        deck.cuePoints = data.map((c, i) => ({
          id: `cue-restored-${i}-${Date.now()}`,
          position: c.position,
          color: c.color || CUE_COLORS[i % CUE_COLORS.length],
          label: c.label || `CUE ${i + 1}`,
        }))
      }
    } catch { /* ignore */ }
  }

  // ─── Loop ─────────────────────────────────────────────────────

  const setLoopIn = (deckId: DeckId, position: number) => {
    getDeck(deckId).loop.inPoint = position
  }

  const setLoopOut = (deckId: DeckId, position: number) => {
    const deck = getDeck(deckId)
    deck.loop.outPoint = position
    if (deck.loop.inPoint !== null) {
      deck.loop.active = true
    }
  }

  const toggleLoop = (deckId: DeckId) => {
    const deck = getDeck(deckId)
    if (deck.loop.inPoint !== null && deck.loop.outPoint !== null) {
      deck.loop.active = !deck.loop.active
    }
  }

  const clearLoop = (deckId: DeckId) => {
    getDeck(deckId).loop = createDefaultLoop()
  }

  const setAutoLoop = (deckId: DeckId, beats: number) => {
    const deck = getDeck(deckId)
    const bpm = deck.track?.bpm
    if (!bpm || bpm <= 0) return
    const beatLen = 60 / bpm
    const loopLen = beatLen * beats
    deck.loop.inPoint = deck.currentTime
    deck.loop.outPoint = deck.currentTime + loopLen
    deck.loop.active = true
  }

  // ─── Mixer Actions ────────────────────────────────────────────

  const setChannelVolume = (deckId: DeckId, volume: number) => {
    getChannel(deckId).volume = Math.max(0, Math.min(1, volume))
  }

  const setChannelGain = (deckId: DeckId, gainDb: number) => {
    getChannel(deckId).gain = Math.max(-12, Math.min(12, gainDb))
  }

  const setChannelEq = (deckId: DeckId, band: keyof EqState, value: number) => {
    getChannel(deckId).eq[band] = Math.max(-26, Math.min(6, value))
  }

  const setChannelFilter = (deckId: DeckId, value: number) => {
    getChannel(deckId).filter = Math.max(-1, Math.min(1, value))
  }

  const toggleChannelCue = (deckId: DeckId) => {
    getChannel(deckId).cueMix = !getChannel(deckId).cueMix
  }

  const toggleChannelMute = (deckId: DeckId) => {
    getChannel(deckId).mute = !getChannel(deckId).mute
  }

  const setChannelLevel = (deckId: DeckId, level: number, peak: number) => {
    const ch = getChannel(deckId)
    ch.peakLevel = level
    ch.peakHold = Math.max(ch.peakHold, peak)
  }

  const setCrossfader = (value: number) => {
    mixer.crossfader = Math.max(-1, Math.min(1, value))
  }

  const setCrossfaderCurve = (curve: CrossfaderCurve) => {
    mixer.crossfaderCurve = curve
  }

  const setCrossfaderAssign = (deckId: DeckId, assign: CrossfaderAssign) => {
    mixer.crossfaderAssign[deckId - 1] = assign
  }

  const setMasterVolume = (volume: number) => {
    mixer.masterVolume = Math.max(0, Math.min(1, volume))
  }

  const setHeadphoneVolume = (volume: number) => {
    mixer.headphoneVolume = Math.max(0, Math.min(1, volume))
  }

  const setHeadphoneMix = (mix: number) => {
    mixer.headphoneMix = Math.max(0, Math.min(1, mix))
  }

  const setMasterLevel = (level: number, peak: number) => {
    mixer.masterLevel = level
    mixer.masterPeakHold = Math.max(mixer.masterPeakHold, peak)
  }

  const setAutoplaySetting = <K extends keyof AutoplaySettings>(key: K, value: AutoplaySettings[K]) => {
    autoplaySettings[key] = value
  }

  // ─── Queue Management ─────────────────────────────────────────

  /**
   * Add a single track to the end of the autoplay queue.
   */
  const addToQueue = (track: MusicFile) => {
    autoPlayPlaylist.value.push(track)
  }

  /**
   * Add multiple tracks to the end of the autoplay queue.
   */
  const addTracksToQueue = (tracks: MusicFile[]) => {
    autoPlayPlaylist.value.push(...tracks)
  }

  /**
   * Remove a track from the queue by index.
   * Only allows removing tracks that haven't been played yet and aren't currently loaded.
   */
  const removeFromQueue = (index: number) => {
    if (index < 0 || index >= autoPlayPlaylist.value.length) return
    autoPlayPlaylist.value.splice(index, 1)
    // Adjust autoPlayIndex if we removed something before the current index
    if (index < autoPlayIndex.value) {
      autoPlayIndex.value = Math.max(0, autoPlayIndex.value - 1)
    }
  }

  /**
   * Move a track in the queue from one position to another.
   * Only allows reordering tracks that are pending (not played, not loaded).
   */
  const reorderQueue = (fromIndex: number, toIndex: number) => {
    if (fromIndex === toIndex) return
    if (fromIndex < 0 || fromIndex >= autoPlayPlaylist.value.length) return
    if (toIndex < 0 || toIndex >= autoPlayPlaylist.value.length) return
    const [item] = autoPlayPlaylist.value.splice(fromIndex, 1)
    autoPlayPlaylist.value.splice(toIndex, 0, item)
  }

  /**
   * Clear all pending (unplayed, unloaded) tracks from the queue.
   */
  const clearPendingQueue = () => {
    // Keep tracks up to autoPlayIndex, remove the rest
    autoPlayPlaylist.value.splice(autoPlayIndex.value)
  }

  /**
   * Get the status of a track in the queue by index.
   * Returns 'played' | 'loaded' | 'pending'
   */
  const getTrackQueueStatus = (index: number): 'played' | 'loaded' | 'pending' => {
    // Tracks before autoPlayIndex have been consumed (played or loaded to a deck)
    if (index < autoPlayIndex.value) {
      // Check if this track is currently loaded in a deck
      const track = autoPlayPlaylist.value[index]
      if (track && decks.some(d => d.track?.id === track.id)) {
        return 'loaded'
      }
      return 'played'
    }
    // Track at autoPlayIndex is the next to be loaded
    // Check if it's currently loaded in a deck
    const track = autoPlayPlaylist.value[index]
    if (track && decks.some(d => d.track?.id === track.id)) {
      return 'loaded'
    }
    return 'pending'
  }

  // Peak hold decay (call on animation frame)
  const decayPeakHold = () => {
    const decayRate = 0.005
    for (const ch of mixer.channels) {
      if (ch.peakHold > ch.peakLevel) {
        ch.peakHold = Math.max(ch.peakLevel, ch.peakHold - decayRate)
      }
    }
    if (mixer.masterPeakHold > mixer.masterLevel) {
      mixer.masterPeakHold = Math.max(mixer.masterLevel, mixer.masterPeakHold - decayRate)
    }
  }

  // ─── Sync ─────────────────────────────────────────────────────

  const syncToDeck = (targetDeckId: DeckId, sourceDeckId: DeckId) => {
    const source = getDeck(sourceDeckId)
    const target = getDeck(targetDeckId)
    if (!source.track?.bpm || !target.track?.bpm) return

    // 1. Match BPM
    const sourceEffectiveBpm = source.track.bpm * (1 + source.tempoPercent / 100)
    const neededPercent = ((sourceEffectiveBpm / target.track.bpm) - 1) * 100
    setTempo(targetDeckId, neededPercent)

    // 2. Align Phase (Beat Grids)
    // use beatMap for higher accuracy if available
    let sourceTimeInBeat = 0
    let sourceBeatInterval = 0

    if (source.beatMap && source.beatMap.length > 1) {
      // Find where we are in source's beat map
      let bIdx = 0
      while (bIdx < source.beatMap.length - 1 && source.beatMap[bIdx + 1] < source.currentTime) {
        bIdx++
      }
      const lastBeat = source.beatMap[bIdx]
      const nextBeat = source.beatMap[bIdx + 1]
      sourceBeatInterval = nextBeat - lastBeat
      sourceTimeInBeat = source.currentTime - lastBeat
    } else {
      sourceBeatInterval = 60 / sourceEffectiveBpm
      sourceTimeInBeat = (source.currentTime - (source.beatGridOffset || 0)) % sourceBeatInterval
    }
    
    // Jump target to the same relative position in its beat grid
    let targetTimeInBeat = 0
    let targetBeatInterval = 0
    
    if (target.beatMap && target.beatMap.length > 1) {
      // Similar mapping for target
      let bIdx = 0
      while (bIdx < target.beatMap.length - 1 && target.beatMap[bIdx + 1] < target.currentTime) {
        bIdx++
      }
      const lastBeat = target.beatMap[bIdx]
      const nextBeat = target.beatMap[bIdx + 1]
      targetBeatInterval = nextBeat - lastBeat
      targetTimeInBeat = target.currentTime - lastBeat
    } else {
      targetBeatInterval = 60 / sourceEffectiveBpm // Same BPM now
      targetTimeInBeat = (target.currentTime - (target.beatGridOffset || 0)) % targetBeatInterval
    }

    // Adjust target's currentTime to match phase
    // This is a simplified jump;traktor/rekordbox do more nuanced phase matching
    const phaseOffset = sourceTimeInBeat - targetTimeInBeat
    setCurrentTime(targetDeckId, target.currentTime + phaseOffset)
  }

  return {
    // State
    decks,
    mixer,
    masterBpm,
    syncEnabled,
    autoPlay,
    autoPlayPlaylist,
    autoPlayIndex,
    isTransitioning,
    autoplaySettings,
    browserState,
    // Getters
    getDeck,
    getChannel,
    // Deck actions
    loadTrack,
    ejectTrack,
    setPlayState,
    setCurrentTime,
    setDuration,
    setWaveformPeaks,
    setTempo,
    setTempoRange,
    toggleMasterTempo,
    setJogRotation,
    setBeatGridOffset,
    toggleSync,
    // Cue
    addCuePoint,
    setCuePointAt,
    removeCuePoint,
    restoreCuePoints,
    persistCuePoints,
    // Loop
    setLoopIn,
    setLoopOut,
    toggleLoop,
    clearLoop,
    setAutoLoop,
    // Mixer
    setChannelVolume,
    setChannelGain,
    setChannelEq,
    setChannelFilter,
    toggleChannelCue,
    toggleChannelMute,
    setChannelLevel,
    setCrossfader,
    setCrossfaderCurve,
    setCrossfaderAssign,
    setMasterVolume,
    setHeadphoneVolume,
    setHeadphoneMix,
    setMasterLevel,
    decayPeakHold,
    // Autoplay settings
    setAutoplaySetting,
    // Queue management
    addToQueue,
    addTracksToQueue,
    removeFromQueue,
    reorderQueue,
    clearPendingQueue,
    getTrackQueueStatus,
    // Sync
    syncToDeck,
    equalizeSyncedBpm,
  }
})
