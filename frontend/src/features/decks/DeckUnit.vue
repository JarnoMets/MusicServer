<template>
  <div class="deck-unit" :class="[`deck-${deck.id}`, { 'deck-playing': deck.playState === 'playing', 'deck-empty': deck.playState === 'empty' }]">
    <!-- Track Info Header -->
    <div class="deck-header">
      <div class="deck-label">DECK {{ deck.id }}</div>
      <div class="deck-track-info" v-if="deck.track">
        <div class="track-title">
          {{ deck.track.title }}
          <span class="header-bpm" v-if="deck.track.bpm"> - {{ Math.round(deck.track.bpm) }} BPM</span>
        </div>
        <div class="track-artist">{{ deck.track.artist || 'Unknown' }}</div>
      </div>
      <div class="deck-track-info deck-empty-msg" v-else>
        <div class="track-title">No Track Loaded</div>
        <div class="track-artist">Click Load to select a track</div>
      </div>
      <button class="btn-load" @click="$emit('open-load')" title="Load Track">
        LOAD
      </button>
      <button class="btn-eject" @click="handleEject" v-if="deck.track" title="Eject">
        <Icon name="x" :size="14" />
      </button>
    </div>

    <!-- Waveform Display (Main visual — scrolling waveform with beat markers & position bar) -->
    <div class="deck-spectrogram-container" @click="handleSpectrogramClick" @wheel.prevent="handleWaveformWheel">
      <canvas ref="spectrogramCanvas" class="deck-spectrogram" />

      <!-- Position indicator line (static in center) -->
      <div class="spectrogram-position-line" style="left: 50%"></div>

      <!-- Beat flash overlay -->
      <div v-if="showBeatFlash" class="beat-flash" :class="{ 'bar-flash': showBarFlash }"></div>

      <!-- Info overlay -->
      <div v-if="deck.track" class="spectrogram-info-bar">
        <span class="info-key" v-if="deck.track.initial_key" :class="{ 'key-compatible': keyCompatible }">
          {{ deck.track.initial_key }}
          <span class="info-camelot">({{ camelotKey }})</span>
        </span>
        <span class="info-bar" v-if="currentBar >= 0">
          BAR {{ currentBar + 1 }}.{{ currentBeat + 1 }}
        </span>
        <span
          class="info-bpm"
          v-if="effectiveBpmDisplay"
          :class="{
            'bpm-sped': deck.tempoPercent > 0.01,
            'bpm-slowed': deck.tempoPercent < -0.01
          }"
        >
          {{ effectiveBpmDisplay }}
        </span>
      </div>

      <!-- Time displays -->
      <div class="spectrogram-time-left" v-if="deck.track">{{ elapsedTime }}</div>
      <div class="spectrogram-time-right" v-if="deck.track">{{ remainingTime }}</div>

      <div class="waveform-overlay" v-if="deck.playState === 'loading'">
        <Icon name="loader" :size="20" class="animate-spin" />
        <span>Loading...</span>
      </div>
    </div>

    <!-- Overview Waveform (small, clickable for seeking) -->
    <div class="deck-overview-container">
      <div class="overview-wrapper">
        <canvas ref="overviewCanvas" class="deck-overview" @click="handleOverviewClick" />

        <!-- Cue point markers (on overview) -->
        <div
          v-for="cue in deck.cuePoints"
          :key="cue.id"
          class="cue-marker-mini"
          v-show="cue.position >= 0"
          :style="{ left: cueMarkerPosition(cue) + '%', backgroundColor: cue.color }"
          :title="cue.label"
          @click.stop="handleCueJump(cue)"
        />

        <!-- Loop region (on overview) -->
        <div
          v-if="deck.loop.inPoint !== null && deck.loop.outPoint !== null"
          class="loop-region-mini"
          :class="{ active: deck.loop.active }"
          :style="loopRegionStyle"
        />
      </div>
    </div>

    <!-- Jog Wheel + Controls Row -->
    <div class="deck-controls-row">
      <!-- Jog Wheel -->
      <div class="jog-wheel-area">
        <canvas
          ref="jogCanvas"
          class="jog-canvas"
          width="140"
          height="140"
          @mousedown="startJogScratch"
          @wheel.prevent="handleJogWheel"
        />
        <div class="jog-time-display">{{ elapsedTimePrecise }}</div>
      </div>

      <!-- Transport + Cue/Loop Controls -->
      <div class="deck-transport">
        <div class="transport-buttons">
          <button class="btn-transport btn-cue" @click="handleCue" :disabled="!deck.track" title="Cue">
            CUE
          </button>
          <button
            class="btn-transport btn-play"
            :class="{ active: deck.playState === 'playing' }"
            @click="handlePlayPause"
            :disabled="!deck.track"
            title="Play/Pause"
          >
            <Icon v-if="deck.playState === 'playing'" name="pause" :size="18" />
            <Icon v-else name="play" :size="18" />
          </button>
        </div>

        <!-- Loop Controls -->
        <div class="loop-controls">
          <button class="btn-loop" @click="handleLoopIn" :class="{ active: deck.loop.inPoint !== null }" title="Loop In">
            IN
          </button>
          <button class="btn-loop" @click="handleLoopOut" :class="{ active: deck.loop.outPoint !== null }" title="Loop Out">
            OUT
          </button>
          <button class="btn-loop" @click="store.toggleLoop(deck.id)" :class="{ active: deck.loop.active }" title="Toggle Loop">
            <Icon name="repeat" :size="14" />
          </button>
          <button class="btn-loop" @click="store.clearLoop(deck.id)" title="Clear Loop">
            <Icon name="x" :size="14" />
          </button>
        </div>

        <!-- Auto Loop Buttons -->
        <div class="auto-loop-buttons">
          <button v-for="beats in [1, 2, 4, 8]" :key="beats" class="btn-auto-loop" @click="store.setAutoLoop(deck.id, beats)" :disabled="!deck.track?.bpm">
            {{ beats }}
          </button>
        </div>

        <!-- Hot Cue Pads -->
        <div class="hot-cue-pads">
          <button
            v-for="i in 8"
            :key="i"
            class="btn-hot-cue"
            :class="{ set: deck.cuePoints[i - 1] && deck.cuePoints[i - 1].position >= 0 }"
            :style="deck.cuePoints[i - 1] && deck.cuePoints[i - 1].position >= 0 ? { backgroundColor: deck.cuePoints[i - 1].color + '33', borderColor: deck.cuePoints[i - 1].color } : {}"
            @click="handleHotCue(i - 1)"
            @contextmenu.prevent="handleDeleteCue(i - 1)"
            :title="`Hot Cue ${i} (right-click to delete)`"
          >
            {{ i }}
          </button>
        </div>
      </div>

      <!-- Tempo Slider (XDJ-1000 Style) -->
      <div class="tempo-area">
        <div class="tempo-fader-unit">
          <!-- Tempo percentage display -->
          <div class="tempo-display">{{ tempoDisplay }}</div>

          <!-- The fader housing -->
          <div class="tempo-fader-housing">
            <div class="tempo-fader-track">
              <!-- Center zero mark -->
              <div class="tempo-zero-mark"></div>
              <!-- Tick marks -->
              <div class="tempo-tick" v-for="t in 9" :key="t" :style="{ top: ((t - 1) / 8 * 100) + '%' }"></div>

              <input
                type="range"
                class="tempo-slider"
                :min="-deck.tempoRange"
                :max="deck.tempoRange"
                :step="0.01"
                :value="deck.tempoPercent"
                @input="handleTempoChange"
                @dblclick="handleTempoReset"
                orient="vertical"
              />
            </div>
          </div>

          <!-- Controls below fader -->
          <div class="tempo-controls-row">
            <button
              class="btn-master-tempo"
              :class="{ active: deck.masterTempo }"
              @click="store.toggleMasterTempo(deck.id)"
              title="Key Lock (Pitch Lock)"
            >
              MT
            </button>

            <button class="btn-tempo-zero" @click="handleTempoReset" title="Reset tempo to 0%">
              0
            </button>

            <button
              class="btn-sync"
              :class="{ active: deck.syncActive }"
              @click="handleSync"
              title="Sync BPM and Phase"
            >
              SYNC
            </button>
          </div>

          <div class="tempo-range-btns">
            <button
              v-for="range in [6, 10, 16, 50]"
              :key="range"
              class="btn-tempo-range"
              :class="{ active: deck.tempoRange === range }"
              @click="store.setTempoRange(deck.id, range)"
            >
              {{ range }}
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch, onMounted, onUnmounted } from 'vue'
import { useDjStore } from '../../stores/djStore'
import { useDjAudioEngine } from '../../composables/useDjAudioEngine'
import {
  effectiveBpm,
  tempoToPlaybackRate,
  formatTimePrecise,
  formatTimeRemaining,
  positionToBar,
  positionToBeat,
  areKeysCompatible,
  toCamelot
} from '../../utils/audioHelpers'
import {
  setupHiDpiCanvas,
  drawWaveform,
  drawPlayhead,
  drawJogWheel,
  drawBeatGrid,
} from '../../utils/canvasHelpers'
import type { DeckId, CuePoint } from '../../types/dj'
import Icon from '../../shared/components/Icons.vue'

const props = defineProps<{
  deckId: DeckId
  otherPlayingKey?: string | null
}>()

defineEmits<{
  'open-load': []
}>()

const store = useDjStore()
const engine = useDjAudioEngine()

const deck = computed(() => store.getDeck(props.deckId))

// Smoothing Time Display
const displayTime = ref(0)
const showBeatFlash = ref(false)
const showBarFlash = ref(false)

const elapsedTime = computed(() => formatTimePrecise(displayTime.value))
const elapsedTimePrecise = computed(() => formatTimePrecise(displayTime.value))
const remainingTime = computed(() => formatTimeRemaining(displayTime.value, deck.value.duration))

const tempoDisplay = computed(() => {
  const pct = deck.value.tempoPercent
  return `${pct >= 0 ? '+' : ''}${pct.toFixed(2)}%`
})

const loopRegionStyle = computed(() => {
  const loop = deck.value.loop
  if (loop.inPoint === null || loop.outPoint === null || deck.value.duration <= 0) return {}
  const left = (loop.inPoint / deck.value.duration) * 100
  const width = ((loop.outPoint - loop.inPoint) / deck.value.duration) * 100
  return { left: `${left}%`, width: `${width}%` }
})

const effectiveBpmDisplay = computed(() => {
  const bpm = deck.value.track?.bpm
  if (!bpm) return null
  return effectiveBpm(bpm, deck.value.tempoPercent).toFixed(1)
})

const camelotKey = computed(() => toCamelot(deck.value.track?.initial_key))

const keyCompatible = computed(() => {
  if (!props.otherPlayingKey || !deck.value.track?.initial_key) return false
  return areKeysCompatible(deck.value.track.initial_key, props.otherPlayingKey)
})

const currentBar = computed(() => {
  const bpm = deck.value.track?.bpm
  if (!bpm) return -1
  return positionToBar(deck.value.currentTime, effectiveBpm(bpm, deck.value.tempoPercent), deck.value.beatGridOffset)
})

const currentBeat = computed(() => {
  const bpm = deck.value.track?.bpm
  if (!bpm) return 0
  const beat = positionToBeat(deck.value.currentTime, effectiveBpm(bpm, deck.value.tempoPercent), deck.value.beatGridOffset)
  return beat % 4
})

// Zoom level for waveform (base seconds visible on screen, before tempo normalization)
const waveformZoom = ref(10)

/**
 * Effective visible seconds after tempo normalization.
 * When a track is pitched up (playbackRate > 1), we show fewer track-seconds
 * so that each beat occupies the same pixel width as on other decks at the
 * same effective BPM.  Two tracks with matching effective BPMs will have
 * identical beat-grid spacing and scroll at exactly the same visual speed.
 */
const normalizedVisibleSeconds = computed(() => {
  const rate = tempoToPlaybackRate(deck.value.tempoPercent)
  return waveformZoom.value / rate
})

// Offscreen canvas for pre-rendered waveform (rebuilt when peaks change)
let offscreenWaveform: OffscreenCanvas | null = null
let offscreenWaveformDirty = true
let offscreenWaveformWidth = 0
const OFFSCREEN_HEIGHT = 160 // fixed logical height for offscreen

const markOffscreenDirty = () => { offscreenWaveformDirty = true }

// Canvas refs
const spectrogramCanvas = ref<HTMLCanvasElement | null>(null)
const overviewCanvas = ref<HTMLCanvasElement | null>(null)
const jogCanvas = ref<HTMLCanvasElement | null>(null)
let spectrogramCtx: CanvasRenderingContext2D | null = null
let overviewCtx: CanvasRenderingContext2D | null = null
let jogCtx: CanvasRenderingContext2D | null = null
let animFrameId: number | null = null

// ─── Cue Helpers ─────────────────────────────────────────────────

const cueMarkerPosition = (cue: CuePoint): number => {
  if (deck.value.duration <= 0) return 0
  return (cue.position / deck.value.duration) * 100
}

// ─── Event Handlers ──────────────────────────────────────────────

const handlePlayPause = () => engine.togglePlay(props.deckId)

const handleCue = () => engine.cuePlay(props.deckId)

const handleEject = () => engine.ejectDeck(props.deckId)

const handleLoopIn = () => {
  store.setLoopIn(props.deckId, deck.value.currentTime)
}

const handleLoopOut = () => {
  store.setLoopOut(props.deckId, deck.value.currentTime)
}

const handleHotCue = (index: number) => {
  const cue = deck.value.cuePoints[index]
  if (cue && cue.position >= 0) {
    engine.seekTo(props.deckId, cue.position)
  } else {
    store.setCuePointAt(props.deckId, index, deck.value.currentTime)
  }
}

const handleDeleteCue = (index: number) => {
  if (deck.value.cuePoints[index]) {
    store.removeCuePoint(props.deckId, deck.value.cuePoints[index].id)
  }
}

const handleCueJump = (cue: CuePoint) => {
  engine.seekTo(props.deckId, cue.position)
}

const handleTempoChange = (e: Event) => {
  const value = parseFloat((e.target as HTMLInputElement).value)
  engine.updateTempo(props.deckId, value)
}

const handleTempoReset = () => {
  engine.updateTempo(props.deckId, 0)
}

const handleSync = () => {
  const otherDecks = store.decks.filter(d => d.id !== props.deckId && d.playState === 'playing' && d.track?.bpm)
  
  if (otherDecks.length === 0) {
    store.toggleSync(props.deckId)
    return
  }

  // Toggle sync first (so store includes/excludes us in BPM average calculation)
  store.toggleSync(props.deckId)
  
  // If we just enabled sync, perform phase alignment
  if (deck.value.syncActive) {
    const source = otherDecks[0]
    const target = deck.value
    
    if (source.track?.bpm && target.track?.bpm) {
      if (target.playState === 'playing') {
        // If playing, use smoothing nudge if possible
        engine.nudgeToSync(props.deckId, source.id)
      } else {
        // If not playing (e.g. cued), just jump
        const sourceEffBpm = effectiveBpm(source.track.bpm, source.tempoPercent)
        const alignPeriod = (60 / sourceEffBpm) * 8 // 2 bars
        
        const sourcePhase = ((source.currentTime - (source.beatGridOffset || 0)) % alignPeriod + alignPeriod) % alignPeriod
        const targetPhase = ((target.currentTime - (target.beatGridOffset || 0)) % alignPeriod + alignPeriod) % alignPeriod
        
        let phaseDiff = sourcePhase - targetPhase
        if (phaseDiff > alignPeriod / 2) phaseDiff -= alignPeriod
        if (phaseDiff < -alignPeriod / 2) phaseDiff += alignPeriod

        engine.seekTo(props.deckId, target.currentTime + phaseDiff)
      }
    }
  }
}

// Overview click to seek
const handleOverviewClick = (e: MouseEvent) => {
  if (!deck.value.duration || !overviewCanvas.value) return
  const rect = overviewCanvas.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  const progress = x / rect.width
  const newTime = progress * deck.value.duration
  engine.seekTo(props.deckId, newTime)
  resetSpectrogram()
}

// Spectrogram click to seek
const handleSpectrogramClick = (e: MouseEvent) => {
  if (!deck.value.duration || !spectrogramCanvas.value) return
  const rect = spectrogramCanvas.value.getBoundingClientRect()
  const x = e.clientX - rect.left
  const progress = x / rect.width
  
  // Playhead is always at center. Click x maps relative to center.
  const currentTime = engine.getCurrentTime(props.deckId)
  const visibleSeconds = normalizedVisibleSeconds.value
  const startTime = currentTime - (visibleSeconds / 2)
  const newTime = startTime + (progress * visibleSeconds)
  
  engine.seekTo(props.deckId, Math.max(0, Math.min(newTime, deck.value.duration)))
  markOffscreenDirty()
}

// Handle zoom with mouse wheel on waveform
const handleWaveformWheel = (e: WheelEvent) => {
  e.preventDefault()
  const zoomDelta = e.deltaY > 0 ? 1.2 : 0.8
  waveformZoom.value = Math.max(1, Math.min(60, waveformZoom.value * zoomDelta))
}

// Reset spectrogram data (on seek or track change)
const resetSpectrogram = () => {
  if (spectrogramCtx && spectrogramCanvas.value) {
    const w = parseInt(spectrogramCanvas.value.style.width || '400')
    const h = parseInt(spectrogramCanvas.value.style.height || '100')
    spectrogramCtx.fillStyle = '#050510'
    spectrogramCtx.fillRect(0, 0, w, h)
  }
  markOffscreenDirty()
}

// ─── Jog Wheel Scratching ────────────────────────────────────────

let isJogScratch = false
let jogStartAngle = 0

const startJogScratch = (e: MouseEvent) => {
  if (!deck.value.track) return
  isJogScratch = true
  jogStartAngle = getAngleFromEvent(e)
  document.addEventListener('mousemove', handleJogMove)
  document.addEventListener('mouseup', stopJogScratch)
}

const handleJogMove = (e: MouseEvent) => {
  if (!isJogScratch) return
  const angle = getAngleFromEvent(e)
  const delta = angle - jogStartAngle
  jogStartAngle = angle
  const timeNudge = (delta / (Math.PI * 2)) * 2
  const newTime = Math.max(0, deck.value.currentTime + timeNudge)
  engine.seekTo(props.deckId, newTime)
}

const stopJogScratch = () => {
  isJogScratch = false
  document.removeEventListener('mousemove', handleJogMove)
  document.removeEventListener('mouseup', stopJogScratch)
}

const handleJogWheel = (e: WheelEvent) => {
  if (!deck.value.track) return
  const nudge = e.deltaY > 0 ? 0.05 : -0.05
  engine.seekTo(props.deckId, Math.max(0, deck.value.currentTime + nudge))
}

const getAngleFromEvent = (e: MouseEvent): number => {
  if (!jogCanvas.value) return 0
  const rect = jogCanvas.value.getBoundingClientRect()
  const cx = rect.left + rect.width / 2
  const cy = rect.top + rect.height / 2
  return Math.atan2(e.clientY - cy, e.clientX - cx)
}

// ─── Canvas Rendering ────────────────────────────────────────────

const setupCanvases = () => {
  if (spectrogramCanvas.value) {
    const rect = spectrogramCanvas.value.parentElement?.getBoundingClientRect()
    const w = rect?.width || 400
    const h = rect?.height ? Math.max(60, rect.height - 4) : 100
    spectrogramCtx = setupHiDpiCanvas(spectrogramCanvas.value, w, h)
    if (spectrogramCtx) {
      spectrogramCtx.fillStyle = '#050510'
      spectrogramCtx.fillRect(0, 0, w, h)
    }
  }
  if (overviewCanvas.value) {
    const rect = overviewCanvas.value.parentElement?.getBoundingClientRect()
    const w = rect?.width || 400
    const h = 24
    overviewCtx = setupHiDpiCanvas(overviewCanvas.value, w, h)
  }
  if (jogCanvas.value) {
    jogCtx = jogCanvas.value.getContext('2d')
  }
}

const colors: Record<number, string> = { 1: '#4f46e5', 2: '#ef4444', 3: '#22c55e', 4: '#f59e0b' }
const getDeckColor = (id: DeckId): string => colors[id]
const getDeckPlayedColor = (id: DeckId): string => {
  const playedColors: Record<number, string> = { 1: '#818cf8', 2: '#fca5a5', 3: '#86efac', 4: '#fcd34d' }
  return playedColors[id]
}


// ─── Offscreen waveform tile pre-render ─────────────────────────
// Renders the entire waveform at high density to an OffscreenCanvas so
// renderFrame just does a fast drawImage crop instead of looping peaks.

const rebuildOffscreenWaveform = (peaks: Float32Array, width: number) => {
  const dpr = window.devicePixelRatio || 1
  const ow = Math.ceil(width * dpr)
  const oh = OFFSCREEN_HEIGHT
  offscreenWaveformWidth = width

  if (!offscreenWaveform || offscreenWaveform.width !== ow || offscreenWaveform.height !== oh) {
    offscreenWaveform = new OffscreenCanvas(ow, oh)
  }

  const ctx = offscreenWaveform.getContext('2d') as OffscreenCanvasRenderingContext2D
  ctx.clearRect(0, 0, ow, oh)

  // Draw peaks: one column per pixel (at physical resolution)
  const midY = oh / 2
  const maxAmp = midY * 0.88
  const totalPeaks = peaks.length

  // Colour gradient: pre-played portion uses lighter shade (drawn later per frame)
  // Offscreen just draws all peaks in neutral unplayed colour
  const deckColor = getDeckColor(props.deckId)

  ctx.fillStyle = deckColor

  for (let px = 0; px < ow; px++) {
    const peakIdx = Math.floor((px / ow) * totalPeaks)
    // Max of a small window for higher detail
    let peak = 0
    const windowEnd = Math.min(totalPeaks, Math.floor(((px + 1) / ow) * totalPeaks))
    for (let k = peakIdx; k < windowEnd; k++) {
      if (peaks[k] > peak) peak = peaks[k]
    }
    const h = peak * maxAmp
    ctx.fillRect(px, midY - h, 1, h * 2)
  }

  offscreenWaveformDirty = false
}

// ─── Main waveform render (called every animation frame) ─────────
const drawScrollingWaveformFast = (ctx: CanvasRenderingContext2D, currentTime: number, w: number, h: number) => {
  const d = deck.value
  const peaks = d.waveformPeaks
  if (!peaks || peaks.length === 0 || d.duration <= 0) {
    ctx.clearRect(0, 0, w, h)
    return
  }

  // Rebuild offscreen if stale
  if (offscreenWaveformDirty || offscreenWaveformWidth !== w) {
    rebuildOffscreenWaveform(peaks, w)
  }

  // Use tempo-normalized visible seconds so that two decks at the same
  // effective BPM show identical beat-grid spacing and scroll in lockstep.
  const visibleSeconds = normalizedVisibleSeconds.value
  const halfVisible = visibleSeconds / 2
  const startTime = currentTime - halfVisible

  ctx.clearRect(0, 0, w, h)

  // ── Draw waveform from offscreen tile ───────────────────────
  if (offscreenWaveform) {
    const ow = offscreenWaveform.width
    // which fraction of the full track is visible (tempo-adjusted window)
    const srcX = ((startTime / d.duration) * ow)
    const srcW = ((visibleSeconds / d.duration) * ow)

    // Clip to valid range
    const clampedSrcX = Math.max(0, srcX)
    const clampedSrcW = Math.min(ow - clampedSrcX, srcW)
    const destX = ((clampedSrcX - srcX) / srcW) * w
    const destW = (clampedSrcW / srcW) * w

    if (clampedSrcW > 0 && destW > 0) {
      // Draw the waveform section
      ctx.drawImage(offscreenWaveform,
        clampedSrcX, 0, clampedSrcW, offscreenWaveform.height,
        destX, 0, destW, h)

      // Tint played portion (everything left of center playhead) with lighter colour
      const currentX = w / 2 // playhead is always center
      if (currentX > destX) {
        ctx.save()
        ctx.globalCompositeOperation = 'source-atop'
        const playedColor = getDeckPlayedColor(props.deckId)
        ctx.fillStyle = playedColor + 'aa' // semi-transparent tint
        ctx.fillRect(destX, 0, Math.min(currentX, destX + destW) - destX, h)
        ctx.restore()
      }
    }
  }

  // ── Draw beat grid ON TOP of waveform ───────────────────────
  const bpmValue = d.track?.bpm || 0
  if (bpmValue || (d.beatMap && d.beatMap.length > 0)) {
    // The visible window is tempo-normalised: when pitched up, fewer track-
    // seconds are shown so beat lines are spaced wider — matching any other
    // deck that plays at the same effective BPM.
    drawBeatGrid(ctx, bpmValue, d.duration, w, h, startTime, visibleSeconds, d.beatGridOffset || 0, d.beatMap)
  }

  // ── Draw cue point markers ───────────────────────────────────
  for (const cue of d.cuePoints) {
    if (cue.position < 0) continue
    const cueX = ((cue.position - startTime) / visibleSeconds) * w
    if (cueX < 0 || cueX > w) continue
    ctx.strokeStyle = cue.color
    ctx.lineWidth = 1.5
    ctx.beginPath()
    ctx.moveTo(cueX, 0)
    ctx.lineTo(cueX, h)
    ctx.stroke()
    // Triangle marker at top
    ctx.fillStyle = cue.color
    ctx.beginPath()
    ctx.moveTo(cueX - 4, 0)
    ctx.lineTo(cueX + 4, 0)
    ctx.lineTo(cueX, 6)
    ctx.closePath()
    ctx.fill()
  }

  // ── Draw loop region ────────────────────────────────────────
  if (d.loop.inPoint !== null && d.loop.outPoint !== null) {
    const loopX1 = ((d.loop.inPoint - startTime) / visibleSeconds) * w
    const loopX2 = ((d.loop.outPoint - startTime) / visibleSeconds) * w
    ctx.fillStyle = d.loop.active ? 'rgba(34, 197, 94, 0.25)' : 'rgba(34, 197, 94, 0.10)'
    ctx.fillRect(loopX1, 0, loopX2 - loopX1, h)
    ctx.strokeStyle = '#22c55e'
    ctx.lineWidth = 1.5
    ctx.strokeRect(loopX1, 0, loopX2 - loopX1, h)
  }

  // ── Center playhead line ─────────────────────────────────────
  const cx = w / 2
  ctx.strokeStyle = 'rgba(255,255,255,0.95)'
  ctx.lineWidth = 2
  ctx.shadowColor = '#ffffff'
  ctx.shadowBlur = 8
  ctx.beginPath()
  ctx.moveTo(cx, 0)
  ctx.lineTo(cx, h)
  ctx.stroke()
  ctx.shadowBlur = 0

  // Small triangle at bottom
  ctx.fillStyle = '#ffffff'
  ctx.beginPath()
  ctx.moveTo(cx - 6, h)
  ctx.lineTo(cx + 6, h)
  ctx.lineTo(cx, h - 8)
  ctx.closePath()
  ctx.fill()
}

const renderFrame = () => {
  // Always get time directly from audio engine for smooth sub-frame accuracy
  const currentTime = engine.getCurrentTime(props.deckId)
  displayTime.value = currentTime

  const d = deck.value

  // Beat flash detection (using real audio time)
  if (d.track?.bpm && d.playState === 'playing') {
    const bpm = effectiveBpm(d.track.bpm, d.tempoPercent)
    const beatInterval = 60 / bpm
    const beatIndex = Math.floor((currentTime - (d.beatGridOffset || 0)) / beatInterval)
    const beatIndexRef = (renderFrame as unknown as { _lastBeat?: number })
    if (beatIndex !== beatIndexRef._lastBeat) {
      beatIndexRef._lastBeat = beatIndex
      showBeatFlash.value = true
      showBarFlash.value = beatIndex % 4 === 0
      setTimeout(() => { showBeatFlash.value = false; showBarFlash.value = false }, 70)
    }
  }

  // Draw scrolling waveform
  if (spectrogramCtx && spectrogramCanvas.value) {
    const w = parseInt(spectrogramCanvas.value.style.width || '400')
    const h = parseInt(spectrogramCanvas.value.style.height || '100')
    drawScrollingWaveformFast(spectrogramCtx, currentTime, w, h)
  }

  // Draw STATIC Overview
  if (overviewCtx && overviewCanvas.value && d.waveformPeaks) {
    const w = parseInt(overviewCanvas.value.style.width)
    const h = parseInt(overviewCanvas.value.style.height)
    const progress = d.duration > 0 ? currentTime / d.duration : 0

    drawWaveform(overviewCtx, d.waveformPeaks, w, h, progress, {
      color: getDeckColor(props.deckId),
      playedColor: getDeckPlayedColor(props.deckId),
    })

    drawPlayhead(overviewCtx, progress * w, h, '#ffffff', 1.5)
  }

  // Draw jog wheel
  if (jogCtx && jogCanvas.value) {
    const size = 140
    jogCtx.clearRect(0, 0, size, size)

    let rotation = 0
    if (d.track) {
      const radPerSec = (33.33 / 60) * Math.PI * 2
      rotation = (currentTime * radPerSec) % (Math.PI * 2)
    }

    drawJogWheel(
      jogCtx,
      size / 2, size / 2, size / 2 - 4,
      rotation,
      d.playState === 'playing',
      getDeckColor(props.deckId)
    )
  }

  animFrameId = requestAnimationFrame(renderFrame)
}

// Initial setup
onMounted(() => {
  setupCanvases()
  renderFrame()
})

// Cleanup on unmount
onUnmounted(() => {
  if (animFrameId !== null) cancelAnimationFrame(animFrameId)
  stopJogScratch()
})

// Re-setup canvases when track loads
watch(() => deck.value.track, () => {
  setupCanvases()
  resetSpectrogram()
})

// When peaks arrive, mark offscreen dirty so it gets rebuilt
watch(() => deck.value.waveformPeaks, () => {
  markOffscreenDirty()
})

// When zoom changes, rebuild offscreen at new resolution density
watch(waveformZoom, () => {
  markOffscreenDirty()
})

// Watch for tempo changes from the store (e.g. from sync)
watch(() => deck.value.tempoPercent, (newPct) => {
  engine.updateTempo(props.deckId, newPct)
})

// Handle window resize
const handleResize = () => setupCanvases()
onMounted(() => window.addEventListener('resize', handleResize))
onUnmounted(() => window.removeEventListener('resize', handleResize))
</script>

<style scoped>
.deck-unit {
  position: relative;
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  background: linear-gradient(180deg, #0a0a14 0%, #060610 100%);
  border: 1px solid #1a1a2e;
  border-radius: 6px;
  overflow: hidden;
}

.deck-playing {
  border-color: rgba(255, 255, 255, 0.12);
}

.deck-header {
  padding: 6px 10px;
  border-bottom: 1px solid #1a1a2e;
  display: flex;
  align-items: center;
  justify-content: space-between;
  background: rgba(0, 0, 0, 0.3);
  flex-shrink: 0;
}

.deck-label {
  font-size: 11px;
  font-weight: 800;
  color: #666;
  letter-spacing: 0.05em;
}

.deck-1 .deck-label { color: #818cf8; }
.deck-2 .deck-label { color: #fca5a5; }
.deck-3 .deck-label { color: #86efac; }
.deck-4 .deck-label { color: #fcd34d; }

.deck-track-info {
  flex: 1;
  margin-left: 8px;
  color: #fff;
  min-width: 0;
}

.track-title {
  font-size: 13px;
  font-weight: 700;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.header-bpm {
  color: #888;
  font-weight: 500;
  font-size: 11px;
}

.track-artist {
  font-size: 10px;
  color: #888;
}

.deck-empty-msg .track-title {
  color: #555;
}

.deck-empty-msg .track-artist {
  color: #444;
}

.btn-load, .btn-eject {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 2px 6px;
  font-size: 11px;
  font-weight: 700;
  letter-spacing: 0.05em;
}

.btn-load:hover { color: #fff; }
.btn-eject:hover { color: #ef4444; }

/* ─── Spectrogram (main display) ────────────────────────────── */
.deck-spectrogram-container {
  position: relative;
  height: 80px; /* Fixed height instead of flex grow to keep it compact */
  flex-shrink: 0;
  background: #050510;
  cursor: crosshair;
  overflow: hidden;
  border-top: 1px solid rgba(255, 255, 255, 0.04);
  border-bottom: 1px solid rgba(255, 255, 255, 0.04);
}

.deck-spectrogram {
  width: 100%;
  height: 100%;
  display: block;
}

.spectrogram-position-line {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  background: rgba(255, 255, 255, 0.9);
  box-shadow: 0 0 8px rgba(255, 255, 255, 0.5), 0 0 2px #fff;
  z-index: 10;
  pointer-events: none;
  transition: left 0.08s linear;
}

.beat-flash {
  position: absolute;
  inset: 0;
  background: rgba(255, 255, 255, 0.04);
  pointer-events: none;
  z-index: 5;
  animation: flashFade 80ms ease-out;
}

.beat-flash.bar-flash {
  background: rgba(255, 255, 255, 0.08);
}

@keyframes flashFade {
  from { opacity: 1; }
  to { opacity: 0; }
}

.spectrogram-info-bar {
  position: absolute;
  top: 4px;
  left: 6px;
  right: 6px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  pointer-events: none;
  z-index: 20;
  font-size: 10px;
  font-weight: 700;
  color: rgba(255, 255, 255, 0.7);
}

.info-key {
  display: flex;
  align-items: center;
  gap: 3px;
}

.info-key.key-compatible {
  color: #22c55e;
}

.info-camelot {
  font-size: 9px;
  opacity: 0.6;
}

.info-bar {
  flex: 1;
  text-align: center;
  font-family: 'Courier New', monospace;
}

.info-bpm {
  min-width: 50px;
  text-align: right;
  font-family: 'Courier New', monospace;
}

.info-bpm.bpm-sped { color: #ef4444; }
.info-bpm.bpm-slowed { color: #3b82f6; }

.spectrogram-time-left,
.spectrogram-time-right {
  position: absolute;
  bottom: 4px;
  font-family: 'Courier New', monospace;
  font-size: 12px;
  font-weight: 700;
  color: #fff;
  background: rgba(0, 0, 0, 0.6);
  padding: 1px 4px;
  border-radius: 3px;
  pointer-events: none;
  z-index: 20;
}

.spectrogram-time-left { left: 6px; }
.spectrogram-time-right { right: 6px; }

.waveform-overlay {
  position: absolute;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 6px;
  z-index: 30;
  pointer-events: all;
  color: #888;
  font-size: 11px;
}

/* ─── Overview ──────────────────────────────────────────────── */
.deck-overview-container {
  flex-shrink: 0;
  height: 28px;
  padding: 2px 0;
  background: rgba(0, 0, 0, 0.4);
}

.overview-wrapper {
  position: relative;
  height: 24px;
  width: 100%;
}

.deck-overview {
  width: 100%;
  height: 24px;
  display: block;
  cursor: crosshair;
  background: rgba(0, 0, 0, 0.5);
}

.cue-marker-mini {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 2px;
  pointer-events: auto;
  cursor: pointer;
  z-index: 5;
  box-shadow: 0 0 2px rgba(0, 0, 0, 1);
}

.loop-region-mini {
  position: absolute;
  top: 0;
  bottom: 0;
  background: rgba(34, 197, 94, 0.2);
  border-left: 1px solid #22c55e;
  border-right: 1px solid #22c55e;
  z-index: 2;
  pointer-events: none;
}

.loop-region-mini.active {
  background: rgba(34, 197, 94, 0.4);
}

/* ─── Controls Row ──────────────────────────────────────────── */
.deck-controls-row {
  padding: 6px 8px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  min-height: 140px; /* Use min-height to ensure controls fit */
  flex: 1; /* Allow controls to expand and fill available space */
  gap: 8px;
  background: rgba(0, 0, 0, 0.2);
  overflow: hidden;
}

.jog-wheel-area {
  position: relative;
  width: 130px; /* Slightly larger base */
  height: 130px;
  flex-shrink: 0;
}

.jog-canvas {
  width: 100%;
  height: 100%;
}

.jog-time-display {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 10px;
  font-family: 'Courier New', monospace;
  color: #fff;
  background: rgba(0, 0, 0, 0.5);
  padding: 1px 3px;
  border-radius: 3px;
  pointer-events: none;
}

.deck-transport {
  flex: 1; /* Take up remaining space */
  display: flex;
  flex-direction: column;
  justify-content: space-evenly;
  height: 100%;
  gap: 2px;
  min-width: 0;
  padding: 0 4px;
}

.transport-buttons {
  display: flex;
  gap: 12px;
  justify-content: center;
  margin-bottom: 4px;
}

.btn-transport {
  width: 48px;
  height: 48px;
  background: linear-gradient(145deg, #1a1a1a, #0a0a0a);
  border: 1px solid #333;
  color: #ccc;
  border-radius: 50%;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  box-shadow: 2px 2px 5px rgba(0, 0, 0, 0.5), -1px -1px 2px rgba(255, 255, 255, 0.05);
  transition: all 0.1s;
}

.btn-transport:active {
  transform: scale(0.96);
  background: #050505;
  box-shadow: inset 2px 2px 5px rgba(0, 0, 0, 0.5);
}

.btn-transport:disabled {
  opacity: 0.4;
  cursor: not-allowed;
  box-shadow: none;
}

.btn-cue {
  border-color: #f59e0b;
  color: #f59e0b;
}

.btn-play.active {
  background: radial-gradient(circle, #22c55e 0%, #15803d 100%);
  border-color: #4ade80;
  color: #fff;
  box-shadow: 0 0 15px rgba(34, 197, 94, 0.4);
}

.loop-controls {
  display: flex;
  gap: 4px;
  justify-content: center;
  margin-bottom: 4px;
}

.btn-loop {
  padding: 4px 8px;
  font-size: 10px;
  background: #111;
  border: 1px solid #333;
  color: #888;
  border-radius: 3px;
  cursor: pointer;
  font-weight: 700;
  text-transform: uppercase;
}

.btn-loop:hover {
  background: #222;
  color: #aaa;
}

.btn-loop.active {
  background: #f59e0b;
  border-color: #f59e0b;
  color: #000;
}

.auto-loop-buttons {
  display: flex;
  gap: 4px;
  justify-content: center;
  margin-bottom: 4px;
}

.btn-auto-loop {
  width: 24px;
  height: 20px;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 10px;
  background: #111;
  border: 1px solid #222;
  color: #666;
  border-radius: 2px;
  cursor: pointer;
  font-weight: 700;
}

.btn-auto-loop:hover:not(:disabled) {
  background: #222;
  color: #ddd;
  border-color: #444;
}

.hot-cue-pads {
  display: grid;
  grid-template-columns: repeat(4, 1fr);
  gap: 4px;
  padding: 0 10px;
}

.btn-hot-cue {
  height: 24px;
  font-size: 10px;
  font-weight: 700;
  background: #0a0a0a;
  border: 1px solid #222;
  color: #444;
  border-radius: 2px;
  cursor: pointer;
  transition: all 0.1s;
}

.btn-hot-cue:hover {
  border-color: #444;
}

.btn-hot-cue.set {
  color: #fff;
  box-shadow: 0 0 4px currentColor;
  border-color: currentColor;
}

/* ─── Tempo Area (XDJ-1000 style) ──────────────────────────── */
.tempo-area {
  display: flex;
  flex-direction: column;
  width: 60px;
  height: 100%;
  flex-shrink: 0;
  background: #08080c;
  border-left: 1px solid #1a1a2e;
  padding: 4px;
  align-items: center;
}

.tempo-fader-unit {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
  width: 100%;
  height: 100%;
}

.tempo-display {
  font-size: 12px;
  font-family: 'Courier New', monospace;
  color: #fff;
  font-weight: 700;
  text-align: center;
  background: #000;
  padding: 2px 4px;
  border-radius: 2px;
  width: 100%;
  margin-bottom: 2px;
}

.tempo-fader-housing {
  flex: 1 1 auto; /* Allow grow and shrink */
  width: 100%;
  min-height: 60px; /* Ensure at least some height for slider */
  height: auto; /* Let flex handle it */
  position: relative;
  background: #111;
  border: 1px solid #333;
  border-radius: 2px;
  display: flex;
  justify-content: center;
  margin-bottom: 2px;
}

.tempo-fader-track {
  position: relative;
  width: 100%;
  height: 100%;
  overflow: hidden; /* Ensure slider doesn't spill out */
}

/* Custom Range Input Styling for Vertical Slider */
input[type=range][orient=vertical] {
  writing-mode: bt-lr; /* IE/Edge */
  -webkit-appearance: slider-vertical; /* WebKit */
  appearance: slider-vertical; /* Standard */
  width: 100%;
  height: 100%;
  padding: 0;
  margin: 0;
  background: transparent;
  cursor: pointer;
}

/* For Firefox/Standard, we might need a transform if orient=vertical isn't supported */
@supports not selector(input[type=range][orient=vertical]) {
  .tempo-slider {
    transform: rotate(270deg);
    width: 100%; 
    height: 100%;
  }
}

.tempo-zero-mark {
  position: absolute;
  top: 50%;
  left: 0;
  right: 0;
  height: 1px;
  background: #fff;
  opacity: 0.5;
  pointer-events: none;
  z-index: 2;
}

.tempo-controls-row {
  display: flex;
  flex-wrap: wrap;
  gap: 2px;
  justify-content: center;
  width: 100%;
}

.btn-master-tempo,
.btn-tempo-zero,
.btn-sync {
  flex: 1;
  padding: 2px 0;
  border: 1px solid #333;
  background: #111;
  color: #666;
  font-size: 9px;
  font-weight: 700;
  border-radius: 2px;
  cursor: pointer;
  text-align: center;
  min-width: 24px;
}

.btn-master-tempo.active {
  color: #10b981;
  border-color: #10b981;
  background: rgba(16, 185, 129, 0.1);
}

.btn-sync.active {
  background: #f59e0b;
  color: #000;
  border-color: #f59e0b;
}

.tempo-range-btns {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 2px;
  width: 100%;
}

.btn-tempo-range {
  padding: 2px 0;
  font-size: 9px;
  font-weight: 700;
  background: #050505;
  border: 1px solid #222;
  color: #555;
  cursor: pointer;
  border-radius: 2px;
  text-align: center;
}

.btn-tempo-range.active {
  color: #fff;
  border-color: #666;
  background: #222;
}
</style>
