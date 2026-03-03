import { ref, watch } from 'vue'
import { useDjStore } from '../stores/djStore'
import { useDjAudioEngine } from './useDjAudioEngine'
import { usePlayer } from './usePlayer'
import type { DeckId } from '../types/dj'

const POLL_INTERVAL = 500 // ms

/**
 * Maximum total BPM difference (%) before we switch from BPM-matched
 * transitions to Echo Fade transitions.  12% means each track would need
 * to shift at most ~6% — beyond that the pitch artifacts become audible.
 */
const BPM_MATCH_THRESHOLD = 12

/**
 * Maximum overlap (seconds) used during an Echo Fade transition.
 * Shorter than a beat-matched crossfade because the grids don't align.
 */
const ECHO_FADE_MAX_OVERLAP = 16

// ─── Module-level singleton state ───────────────────────────────
// These live outside the composable function so they survive Vue component
// unmount / route changes.  This is the same pattern used by useDjAudioEngine
// and usePlayer.

const timer = ref<number | null>(null)
const currentActiveDeck = ref<DeckId>(1)
const nextTargetDeck = ref<DeckId>(4)

/** Which transition type is currently in progress (exposed for UI). */
const transitionType = ref<'bpm-match' | 'echo-fade' | null>(null)

// Track original tempo ranges so we can restore them after a wide transition
const savedTempoRanges = ref<Map<DeckId, number>>(new Map())

// Hold cancel functions for active filter sweeps
const activeFilterSweeps = ref<Map<DeckId, () => void>>(new Map())

// Sequence: 1(A) -> 4(B) -> 2(A) -> 3(B) -> repeat
const SEQUENCE: DeckId[] = [1, 4, 2, 3]

// Guard: set up the library-player watcher only once
let libraryPlayerWatcherInstalled = false

/**
 * Stop DJ autoplay. Module-level so it can be called from the library-player
 * watcher without depending on composable-scoped variables.
 */
const stopAutoplay = () => {
  if (timer.value) {
    clearInterval(timer.value)
    timer.value = null
  }
  const store = useDjStore()
  store.autoPlay = false
  transitionType.value = null
}

/**
 * Composable for DJ Automation: Autoplay and Transitions.
 *
 * All automation state lives at **module level** so autoplay keeps running
 * even when the Decks view is unmounted (e.g. user navigates to Library).
 * Autoplay only stops when:
 *   • `stopAutoplay()` is called explicitly (manual stop button), or
 *   • A library track / internet stream starts playing via `usePlayer`.
 *
 * Two transition strategies are used depending on the BPM difference:
 *
 * ── BPM-Matched Transition (difference ≤ 12%) ──────────────────
 *   1. BPM Equalization  — drifts both decks toward average BPM
 *   2. Beat-Grid Phase Alignment — seeks incoming track to match outgoing beat grid
 *   3. Crossfade — smoothly moves crossfader while grids stay locked
 *   4. Finalize — pause outgoing deck, drift new active deck back to 0%
 *
 * ── Echo Fade Transition (difference > 12%) ─────────────────────
 *   Neither track changes tempo. Instead:
 *   1. A low-pass filter sweep + volume fade is applied to the outgoing deck
 *      (simulating an "echo tail" / room-fade effect)
 *   2. The incoming deck is seeked to a clean phrase boundary (bar 1 beat 1)
 *      and faded in at its natural BPM
 *   3. Overlap is capped at 16 s so the two unmatched rhythms don't clash
 *   4. The outgoing deck is paused and its filter is reset
 *
 *   This is the technique professional DJs use when mixing between genres
 *   or BPMs that are too far apart — a clean EQ/filter transition rather
 *   than forced tempo matching that sounds unnatural.
 */
export const useDjAutomation = () => {
  const store = useDjStore()
  const engine = useDjAudioEngine()

  // ─── Library player watcher (installed once) ──────────────────
  // When the user starts a track or stream from the Library player,
  // DJ autoplay should stop so the two audio sources don't compete.
  if (!libraryPlayerWatcherInstalled) {
    libraryPlayerWatcherInstalled = true
    const { state: playerState } = usePlayer()

    watch(
      () => playerState.updatedAt,
      () => {
        if (playerState.isPlaying && timer.value) {
          console.log('[Autoplay] Library player started — stopping DJ autoplay.')
          stopAutoplay()
        }
      },
    )
  }

  // ─── Helpers ──────────────────────────────────────────────────

  /** Get the configured transition trigger time (seconds before end). */
  const getMatchTimeSeconds = () => store.autoplaySettings.matchTimeSeconds

  /** Get the configured crossfade duration (seconds). */
  const getOverlapSeconds = () => store.autoplaySettings.overlapSeconds
  const getExitTimeSeconds = () => store.autoplaySettings.exitTimeSeconds

  /**
   * Calculate the BPM difference percentage between two BPM values,
   * after octave-matching them (so 70 vs 140 counts as 0%).
   */
  const bpmDifferencePercent = (bpmA: number, bpmB: number): number => {
    if (bpmA <= 0 || bpmB <= 0) return 0
    // Octave-match B to A
    let b = bpmB
    while (b < bpmA * 0.75) b *= 2
    while (b > bpmA * 1.5) b /= 2
    return Math.abs(bpmA - b) / Math.min(bpmA, b) * 100
  }

  /**
   * Find the nearest beat-grid-aligned time in a deck at or after `minTime`.
   * Uses the detailed beatMap when available, otherwise falls back to
   * beatGridOffset + BPM-derived grid.
   */
  const findNearestBeatTime = (
    beatMap: number[] | null,
    beatGridOffset: number,
    bpm: number,
    currentTime: number,
    alignBeats: number,  // e.g. 8 for 2-bar phrases
  ): number => {
    if (beatMap && beatMap.length > 1) {
      // Walk through the beat map in chunks of `alignBeats` beats.
      // Find the phrase boundary closest to currentTime.
      let bestTime = beatMap[0]
      for (let i = 0; i < beatMap.length; i += alignBeats) {
        if (beatMap[i] >= currentTime - 0.05) {
          bestTime = beatMap[i]
          break
        }
        bestTime = beatMap[i]
      }
      return bestTime
    }

    // Fallback: computed grid from offset + BPM
    if (bpm <= 0) return currentTime
    const beatLen = 60 / bpm
    const phraseLen = beatLen * alignBeats
    const offset = beatGridOffset || 0
    const elapsed = currentTime - offset
    const phraseIndex = Math.floor(elapsed / phraseLen)
    // Return start of the current or next phrase boundary
    const phraseStart = offset + phraseIndex * phraseLen
    return phraseStart >= currentTime - 0.05 ? phraseStart : phraseStart + phraseLen
  }

  /**
   * Compute where the source deck currently is within its phrase (0..phraseDuration).
   * Uses beatMap for precision when available.
   */
  const getSourcePhrasePhase = (
    beatMap: number[] | null,
    beatGridOffset: number,
    bpm: number,
    currentTime: number,
    alignBeats: number,
  ): number => {
    const beatLen = 60 / bpm
    const phraseLen = beatLen * alignBeats

    if (beatMap && beatMap.length > 1) {
      // Find the phrase boundary just before currentTime
      let lastPhraseStart = beatMap[0]
      for (let i = 0; i < beatMap.length; i += alignBeats) {
        if (beatMap[i] > currentTime + 0.01) break
        lastPhraseStart = beatMap[i]
      }
      return currentTime - lastPhraseStart
    }

    // Fallback: computed grid
    const offset = beatGridOffset || 0
    return ((currentTime - offset) % phraseLen + phraseLen) % phraseLen
  }

  /**
   * Widen the tempo range for a deck if the required tempo adjustment
   * exceeds the current range.  Saves the original range for later restore.
   */
  const ensureTempoRange = (deckId: DeckId, requiredPercent: number) => {
    const deck = store.getDeck(deckId)
    const needed = Math.abs(requiredPercent)
    if (needed > deck.tempoRange) {
      if (!savedTempoRanges.value.has(deckId)) {
        savedTempoRanges.value.set(deckId, deck.tempoRange)
      }
      // Pick a range that comfortably covers the needed adjustment (+2% headroom)
      const newRange = Math.ceil(needed + 2)
      store.setTempoRange(deckId, newRange)
      console.log(`[Autoplay] Widened deck ${deckId} tempo range from ±${deck.tempoRange}% to ±${newRange}% for transition`)
    }
  }

  /** Restore the original tempo range for a deck after transition completes. */
  const restoreTempoRange = (deckId: DeckId) => {
    const saved = savedTempoRanges.value.get(deckId)
    if (saved !== undefined) {
      store.setTempoRange(deckId, saved)
      savedTempoRanges.value.delete(deckId)
      console.log(`[Autoplay] Restored deck ${deckId} tempo range to ±${saved}%`)
    }
  }

  // ─── Autoplay lifecycle ───────────────────────────────────────

  const startAutoplay = async () => {
    if (timer.value) return
    store.autoPlay = true
    
    // Initial setup if empty
    await setupInitialDecks()
    
    timer.value = window.setInterval(checkTransition, POLL_INTERVAL)
  }

  const setupInitialDecks = async () => {
    // Determine which deck is currently playing
    let playingDeck = SEQUENCE.find(id => store.getDeck(id).playState === 'playing')
    
    if (!playingDeck) {
      // Find even a 'cued' deck
      playingDeck = SEQUENCE.find(id => store.getDeck(id).playState === 'cued')
    }

    if (!playingDeck) {
      playingDeck = 1
      // If nothing is playing, load the first track from the autoplay playlist if available
      if (!store.getDeck(1).track && store.autoPlayPlaylist.length > 0) {
        const track = store.autoPlayPlaylist[store.autoPlayIndex % store.autoPlayPlaylist.length]
        await engine.loadTrackToDeck(1, track)
        store.autoPlayIndex++
      }
      engine.play(1)
    }
    
    currentActiveDeck.value = playingDeck
    
    // Set next deck in sequence
    const idx = SEQUENCE.indexOf(playingDeck)
    nextTargetDeck.value = SEQUENCE[(idx + 1) % SEQUENCE.length]
    
    // Pre-load the nextTargetDeck if it's empty
    if (!store.getDeck(nextTargetDeck.value).track && store.autoPlayPlaylist.length > 0) {
      const track = store.autoPlayPlaylist[store.autoPlayIndex % store.autoPlayPlaylist.length]
      await engine.loadTrackToDeck(nextTargetDeck.value, track)
      store.autoPlayIndex++
    }
    
    // Ensure crossfader is correct for current active deck
    const assign = store.mixer.crossfaderAssign[playingDeck - 1]
    if (assign === 'A') engine.updateCrossfader(-1)
    if (assign === 'B') engine.updateCrossfader(1)
  }

  const checkTransition = () => {
    if (!store.autoPlay || store.isTransitioning) return

    const deck = store.getDeck(currentActiveDeck.value)
    if (!deck.track || deck.duration === 0) return

    const remaining = deck.duration - deck.currentTime
    const matchSec = getMatchTimeSeconds()
    if (remaining <= matchSec && remaining > 0) {
      triggerTransition()
    }
  }

  // ─── Transition Router ────────────────────────────────────────

  const triggerTransition = async () => {
    store.isTransitioning = true
    const fromDeckId = currentActiveDeck.value
    const toDeckId = nextTargetDeck.value
    
    const fromDeck = store.getDeck(fromDeckId)
    const toDeck = store.getDeck(toDeckId)

    // 1. Prepare toDeck (ensure track is loaded)
    if (!toDeck.track && store.autoPlayPlaylist.length > 0) {
      const track = store.autoPlayPlaylist[store.autoPlayIndex % store.autoPlayPlaylist.length]
      await engine.loadTrackToDeck(toDeckId, track)
      store.autoPlayIndex++
    }

    // 2. Determine transition strategy based on BPM difference
    const fromBpmBase = fromDeck.track?.bpm || 120
    const fromEffBpm = fromBpmBase * (1 + fromDeck.tempoPercent / 100)
    const toBpmBase = toDeck.track?.bpm || 120
    const diffPercent = bpmDifferencePercent(fromEffBpm, toBpmBase)

    if (diffPercent > BPM_MATCH_THRESHOLD) {
      console.log(
        `[Autoplay] BPM difference ${diffPercent.toFixed(1)}% exceeds ${BPM_MATCH_THRESHOLD}% threshold. ` +
        `Using Echo Fade transition (${fromEffBpm.toFixed(1)} → ${toBpmBase.toFixed(1)} BPM)`
      )
      transitionType.value = 'echo-fade'
      triggerEchoFadeTransition(fromDeckId, toDeckId)
    } else {
      console.log(
        `[Autoplay] BPM difference ${diffPercent.toFixed(1)}% within threshold. ` +
        `Using BPM-matched transition (${fromEffBpm.toFixed(1)} → ${toBpmBase.toFixed(1)} BPM)`
      )
      transitionType.value = 'bpm-match'
      triggerBpmMatchedTransition(fromDeckId, toDeckId)
    }
  }

  // ─── BPM-Matched Transition (≤ 12% difference) ───────────────

  const triggerBpmMatchedTransition = async (fromDeckId: DeckId, toDeckId: DeckId) => {
    const fromDeck = store.getDeck(fromDeckId)
    const toDeck = store.getDeck(toDeckId)

    // Start toDeck (inaudible — crossfader is on fromDeck's side)
    engine.play(toDeckId)

    // ── BPM Equalization Setup ──────────────────────────────────
    const fromBpmBase = fromDeck.track?.bpm || 120
    const fromEffBpmStart = fromBpmBase * (1 + fromDeck.tempoPercent / 100)
    const toBpmBase = toDeck.track?.bpm || 120
    
    // Octave match toBpm to fromEffBpm
    let toBpmOctave = toBpmBase
    while (toBpmOctave < fromEffBpmStart * 0.75) toBpmOctave *= 2
    while (toBpmOctave > fromEffBpmStart * 1.5) toBpmOctave /= 2
    
    const targetBpm = (fromEffBpmStart + toBpmOctave) / 2
    
    // Calculate targets for both decks relative to their original base BPM
    let fromBpmOctaveTarget = fromBpmBase
    while (fromBpmOctaveTarget < targetBpm * 0.75) fromBpmOctaveTarget *= 2
    while (fromBpmOctaveTarget > targetBpm * 1.5) fromBpmOctaveTarget /= 2
    const fromTargetPercent = ((targetBpm / fromBpmOctaveTarget) - 1) * 100

    let toBpmOctaveTarget = toBpmBase
    while (toBpmOctaveTarget < targetBpm * 0.75) toBpmOctaveTarget *= 2
    while (toBpmOctaveTarget > targetBpm * 1.5) toBpmOctaveTarget /= 2
    const toTargetPercent = ((targetBpm / toBpmOctaveTarget) - 1) * 100

    // Ensure tempo ranges can accommodate the needed adjustment
    ensureTempoRange(fromDeckId, fromTargetPercent)
    ensureTempoRange(toDeckId, toTargetPercent)

    const fromStartPercent = fromDeck.tempoPercent
    const toStartPercent = toDeck.tempoPercent

    // Phase 1 (BPM equalization) takes the time available before the overlap starts:
    // matchTime - overlap. If zero or negative, equalization must be instant.
    const matchSec = getMatchTimeSeconds()
    const overlapSec = getOverlapSeconds()
    const equalizeSec = Math.max(0, matchSec - overlapSec)
    const phase1Duration = equalizeSec * 1000
    const steps = Math.max(10, Math.round(phase1Duration / 200)) // ~200ms per step
    const interval = phase1Duration / steps
    
    let step = 0
    const phase1Timer = window.setInterval(() => {
      step++
      const progress = step / steps
      
      // Drift both tempos toward targets
      engine.updateTempo(fromDeckId, fromStartPercent + (fromTargetPercent - fromStartPercent) * progress)
      engine.updateTempo(toDeckId, toStartPercent + (toTargetPercent - toStartPercent) * progress)

      if (step >= steps) {
        clearInterval(phase1Timer)

        // ── Beat-Grid Phase Alignment ───────────────────────────
        // Now that BPMs are matched, align the incoming track's beat grid
        // to the outgoing track's beat grid.
        const fromDeckNow = store.getDeck(fromDeckId)
        const toDeckNow = store.getDeck(toDeckId)
        
        const alignBeats = 8 // 2 bars (8 beats in 4/4)
        
        // 1. Get the outgoing track's current phase within its phrase
        const sourcePhase = getSourcePhrasePhase(
          fromDeckNow.beatMap,
          fromDeckNow.beatGridOffset || 0,
          targetBpm,
          fromDeckNow.currentTime,
          alignBeats,
        )
        
        // 2. Find the nearest phrase boundary in the incoming track
        const nearestBoundary = findNearestBeatTime(
          toDeckNow.beatMap,
          toDeckNow.beatGridOffset || 0,
          targetBpm,
          toDeckNow.currentTime,
          alignBeats,
        )
        
        // 3. Seek the incoming track to the boundary + the same phrase phase
        let newTargetTime = nearestBoundary + sourcePhase
        
        // If we've jumped backwards too much, move to the next phrase boundary
        if (newTargetTime < toDeckNow.currentTime - 0.5) {
          const phraseLen = (60 / targetBpm) * alignBeats
          newTargetTime += phraseLen
        }
        
        engine.seekTo(toDeckId, newTargetTime)
        console.log(
          `[Autoplay] Beat-grid aligned at ${targetBpm.toFixed(2)} BPM. ` +
          `Source phase: ${sourcePhase.toFixed(3)}s, ` +
          `Target seeked to: ${newTargetTime.toFixed(3)}s ` +
          `(boundary: ${nearestBoundary.toFixed(3)}s, beatMap: ${toDeckNow.beatMap ? 'yes' : 'no'})`
        )

        // 4. Start crossfade phase
        startCrossfade(fromDeckId, toDeckId, false)
      }
    }, interval)
  }

  // ─── Echo Fade Transition (> 12% BPM difference) ─────────────
  //
  // Strategy: No tempo changes. The outgoing track gets a low-pass filter
  // sweep (20kHz → 200Hz) + volume fade that creates a warm "echo tail"
  // effect, while the incoming track fades in at its natural tempo from a
  // clean phrase boundary.  The overlap is short (capped at 16s) so the
  // two unsynchronised rhythms don't audibly clash.

  const triggerEchoFadeTransition = async (fromDeckId: DeckId, toDeckId: DeckId) => {
    const toDeck = store.getDeck(toDeckId)
    const toBpmBase = toDeck.track?.bpm || 120

    // 1. Seek incoming track to a clean phrase boundary (4-bar = 16 beats)
    //    This ensures the new track enters on a musically strong point.
    const alignBeats = 16 // 4 bars for a clean phrase entry
    const nearestPhrase = findNearestBeatTime(
      toDeck.beatMap,
      toDeck.beatGridOffset || 0,
      toBpmBase,
      toDeck.currentTime,
      alignBeats,
    )
    engine.seekTo(toDeckId, nearestPhrase)

    // 2. Calculate overlap duration — use configured overlap but cap at ECHO_FADE_MAX_OVERLAP
    const configuredOverlap = getOverlapSeconds()
    const overlapSec = Math.min(configuredOverlap, ECHO_FADE_MAX_OVERLAP)

    // 3. Start the low-pass filter sweep on the outgoing deck
    //    The sweep runs for the full overlap duration
    const cancelSweep = engine.sweepFilter(fromDeckId, overlapSec * 1000)
    activeFilterSweeps.value.set(fromDeckId, cancelSweep)

    console.log(
      `[Autoplay] Echo Fade: outgoing deck ${fromDeckId} filter sweep ${overlapSec}s, ` +
      `incoming deck ${toDeckId} seeked to phrase boundary ${nearestPhrase.toFixed(3)}s`
    )

    // 4. Start the incoming track and begin crossfade
    engine.play(toDeckId)
    startCrossfade(fromDeckId, toDeckId, true, overlapSec)
  }

  // ─── Crossfade ────────────────────────────────────────────────

  /**
   * @param isEchoFade If true, uses an equal-power curve (more musical for
   *   non-beat-matched transitions) instead of linear interpolation.
   * @param overrideDurationSec Override the crossfade duration (used by echo fade).
   */
  const startCrossfade = (
    fromDeckId: DeckId,
    toDeckId: DeckId,
    isEchoFade: boolean,
    overrideDurationSec?: number,
  ) => {
    const overlapSec = overrideDurationSec ?? getOverlapSeconds()
    const phase2Duration = overlapSec * 1000
    const steps = Math.max(10, Math.round(phase2Duration / 200))
    const interval = phase2Duration / steps
    
    const startCf = store.mixer.crossfader
    const fromAssign = store.mixer.crossfaderAssign[fromDeckId - 1]
    const toAssign = store.mixer.crossfaderAssign[toDeckId - 1]
    
    // Target CF depends on sides
    let targetCf = startCf
    if (fromAssign === 'A' && toAssign === 'B') targetCf = 1
    else if (fromAssign === 'B' && toAssign === 'A') targetCf = -1

    let step = 0
    const fadeTimer = window.setInterval(() => {
      step++
      const linearProgress = step / steps

      // For echo fade: use an equal-power (sinusoidal) curve so the outgoing
      // track fades out faster at the end while the incoming track comes in
      // gently at the start.  This minimises the time both unmatched rhythms
      // are audible simultaneously.
      const progress = isEchoFade
        ? Math.sin(linearProgress * Math.PI / 2) // fast-in, slow-out (incoming side)
        : linearProgress

      // Interpolate crossfader
      engine.updateCrossfader(startCf + (targetCf - startCf) * progress)

      if (step >= steps) {
        clearInterval(fadeTimer)

        // Clean up any active filter sweep on the outgoing deck
        const cancelSweep = activeFilterSweeps.value.get(fromDeckId)
        if (cancelSweep) {
          cancelSweep()
          activeFilterSweeps.value.delete(fromDeckId)
        }

        finalizeTransition(fromDeckId, toDeckId, isEchoFade)
      }
    }, interval)
  }

  // ─── Finalize ─────────────────────────────────────────────────

  const finalizeTransition = (fromDeckId: DeckId, toDeckId: DeckId, isEchoFade: boolean) => {
    engine.pause(fromDeckId)

    // Reset outgoing deck's filter (in case echo fade left it in LP mode)
    engine.resetFilter(fromDeckId)

    // Restore outgoing deck's tempo range immediately (it's paused now)
    restoreTempoRange(fromDeckId)
    
    // Set next active deck
    currentActiveDeck.value = toDeckId
    const idx = SEQUENCE.indexOf(toDeckId)
    nextTargetDeck.value = SEQUENCE[(idx + 1) % SEQUENCE.length]
    
    // For echo fade transitions, the incoming track never changed tempo,
    // so there's nothing to drift back — mark transition complete immediately.
    if (isEchoFade) {
      store.isTransitioning = false
      transitionType.value = null
      console.log(`[Autoplay] Echo Fade transition complete.`)
      preloadNext(nextTargetDeck.value)
      return
    }

    // ── BPM-matched: drift back to original BPM ─────────────────
    // Note: isTransitioning stays true until drift-back completes
    // so we don't trigger a new transition during drift

    // Slow drift back to original BPM (0%) over the configured exit time
    const toDeck = store.getDeck(toDeckId)
    const currentTempo = toDeck.tempoPercent
    const driftDuration = getExitTimeSeconds() * 1000
    const steps = Math.max(10, Math.round(driftDuration / 200))
    const driftInterval = driftDuration / steps
    let step = 0
    const driftTimer = window.setInterval(() => {
      step++
      const progress = step / steps
      engine.updateTempo(toDeckId, currentTempo * (1 - progress))
      if (step >= steps) {
        clearInterval(driftTimer)
        // Now the entire transition is truly complete — restore range & allow next transition
        restoreTempoRange(toDeckId)
        store.isTransitioning = false
        transitionType.value = null
        console.log(`[Autoplay] BPM-matched transition complete. Tempo ranges restored.`)
      }
    }, driftInterval)
    
    // Pre-load next-next track into the upcoming deck
    preloadNext(nextTargetDeck.value)
  }

  const preloadNext = async (deckId: DeckId) => {
    if (store.autoPlayPlaylist.length === 0) return
    const track = store.autoPlayPlaylist[store.autoPlayIndex % store.autoPlayPlaylist.length]
    await engine.loadTrackToDeck(deckId, track)
    store.autoPlayIndex++
  }

  return {
    startAutoplay,
    stopAutoplay,
    currentActiveDeck,
    nextTargetDeck,
    transitionType,
  }
}
