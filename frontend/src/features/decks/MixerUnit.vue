<template>
  <div class="mixer-unit">
    <!-- Channel Strips -->
    <div class="channel-strips">
      <div
        v-for="ch in mixer.channels"
        :key="ch.deckId"
        class="channel-strip"
        :class="[`channel-${ch.deckId}`]"
      >
        <!-- Deck Label -->
        <div class="channel-label" :style="{ color: getDeckColor(ch.deckId) }">{{ ch.deckId }}</div>

        <!-- Gain Knob -->
        <div class="knob-group">
          <label class="knob-label">GAIN</label>
          <div
            class="knob"
            @mousedown="startKnobDrag('gain', ch.deckId, $event)"
            @dblclick="resetKnob('gain', ch.deckId)"
            :title="`Gain: ${ch.gain.toFixed(1)} dB`"
          >
            <div class="knob-indicator" :style="{ transform: `rotate(${gainToAngle(ch.gain)}deg)` }">
              <div class="knob-dot"></div>
            </div>
          </div>
        </div>

        <!-- EQ Section -->
        <div class="eq-section">
          <div class="knob-group">
            <label class="knob-label">HI</label>
            <div
              class="knob knob-small"
              @mousedown="startKnobDrag('hi', ch.deckId, $event)"
              @dblclick="resetKnob('hi', ch.deckId)"
              :title="`Hi EQ: ${ch.eq.hi.toFixed(1)} dB`"
            >
              <div class="knob-indicator" :style="{ transform: `rotate(${eqToAngle(ch.eq.hi)}deg)` }">
                <div class="knob-dot"></div>
              </div>
            </div>
          </div>
          <div class="knob-group">
            <label class="knob-label">MID</label>
            <div
              class="knob knob-small"
              @mousedown="startKnobDrag('mid', ch.deckId, $event)"
              @dblclick="resetKnob('mid', ch.deckId)"
              :title="`Mid EQ: ${ch.eq.mid.toFixed(1)} dB`"
            >
              <div class="knob-indicator" :style="{ transform: `rotate(${eqToAngle(ch.eq.mid)}deg)` }">
                <div class="knob-dot"></div>
              </div>
            </div>
          </div>
          <div class="knob-group">
            <label class="knob-label">LOW</label>
            <div
              class="knob knob-small"
              @mousedown="startKnobDrag('low', ch.deckId, $event)"
              @dblclick="resetKnob('low', ch.deckId)"
              :title="`Low EQ: ${ch.eq.low.toFixed(1)} dB`"
            >
              <div class="knob-indicator" :style="{ transform: `rotate(${eqToAngle(ch.eq.low)}deg)` }">
                <div class="knob-dot"></div>
              </div>
            </div>
          </div>
        </div>

        <!-- Filter Knob -->
        <div class="knob-group">
          <label class="knob-label">FILTER</label>
          <div
            class="knob knob-small"
            @mousedown="startKnobDrag('filter', ch.deckId, $event)"
            @dblclick="resetKnob('filter', ch.deckId)"
            :title="`Filter: ${ch.filter.toFixed(2)}`"
          >
            <div class="knob-indicator" :style="{ transform: `rotate(${filterToAngle(ch.filter)}deg)` }">
              <div class="knob-dot"></div>
            </div>
          </div>
        </div>

        <!-- CUE (Headphone) Button -->
        <button
          class="btn-cue-mix"
          :class="{ active: ch.cueMix }"
          @click="engine.toggleMute(ch.deckId); store.toggleChannelCue(ch.deckId)"
          title="Headphone Cue"
        >
          <Icon name="headphones" :size="14" />
        </button>

        <!-- Level Meter -->
        <div class="level-meter-container">
          <canvas
            :ref="el => setMeterRef(ch.deckId, el as HTMLCanvasElement)"
            class="level-meter-canvas"
            width="12"
            height="120"
          />
        </div>

        <!-- Channel Fader -->
        <div class="channel-fader-container">
          <input
            type="range"
            class="channel-fader"
            min="0"
            max="1"
            step="0.005"
            :value="ch.volume"
            @input="handleFaderChange(ch.deckId, $event)"
            @dblclick="handleFaderReset(ch.deckId)"
            orient="vertical"
          />
        </div>

        <!-- Crossfader Assign -->
        <div class="cf-assign">
          <button
            v-for="assign in (['A', 'THRU', 'B'] as const)"
            :key="assign"
            class="btn-cf-assign"
            :class="{ active: mixer.crossfaderAssign[ch.deckId - 1] === assign }"
            @click="handleCrossfaderAssign(ch.deckId, assign)"
          >
            {{ assign === 'THRU' ? '—' : assign }}
          </button>
        </div>
      </div>
    </div>

    <!-- Master Section -->
    <div class="master-section">
      <!-- Master Level Meter -->
      <div class="master-meter-group">
        <label class="knob-label">MASTER</label>
        <canvas
          ref="masterMeterCanvas"
          class="level-meter-canvas master-meter"
          width="16"
          height="120"
        />
        <div class="master-level-db">{{ masterLevelDb }}</div>
      </div>

      <!-- Master Volume -->
      <div class="knob-group">
        <label class="knob-label">VOL</label>
        <div
          class="knob"
          @mousedown="startKnobDrag('masterVol', 0 as any, $event)"
          @dblclick="resetKnob('masterVol', 0 as any)"
          :title="`Master: ${Math.round(mixer.masterVolume * 100)}%`"
        >
          <div class="knob-indicator" :style="{ transform: `rotate(${volumeToAngle(mixer.masterVolume)}deg)` }">
            <div class="knob-dot"></div>
          </div>
        </div>
      </div>

      <!-- Headphone Section -->
      <div class="headphone-section">
        <div class="knob-group">
          <label class="knob-label">HP VOL</label>
          <div
            class="knob knob-small"
            @mousedown="startKnobDrag('hpVol', 0 as any, $event)"
            @dblclick="resetKnob('hpVol', 0 as any)"
            :title="`HP Vol: ${Math.round(mixer.headphoneVolume * 100)}%`"
          >
            <div class="knob-indicator" :style="{ transform: `rotate(${volumeToAngle(mixer.headphoneVolume)}deg)` }">
              <div class="knob-dot"></div>
            </div>
          </div>
        </div>
        <div class="knob-group">
          <label class="knob-label">MIX</label>
          <div
            class="knob knob-small"
            @mousedown="startKnobDrag('hpMix', 0 as any, $event)"
            @dblclick="resetKnob('hpMix', 0 as any)"
            :title="`HP Mix: CUE ${Math.round((1 - mixer.headphoneMix) * 100)}% / MASTER ${Math.round(mixer.headphoneMix * 100)}%`"
          >
            <div class="knob-indicator" :style="{ transform: `rotate(${volumeToAngle(mixer.headphoneMix)}deg)` }">
              <div class="knob-dot"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- Crossfader Curve -->
      <div class="cf-curve-section">
        <label class="knob-label">X-FADE</label>
        <div class="cf-curve-btns">
          <button
            v-for="curve in (['smooth', 'sharp', 'through'] as const)"
            :key="curve"
            class="btn-cf-curve"
            :class="{ active: mixer.crossfaderCurve === curve }"
            @click="store.setCrossfaderCurve(curve)"
          >
            {{ curve === 'smooth' ? '╲╱' : curve === 'sharp' ? '⌐¬' : '╱╲' }}
          </button>
        </div>
      </div>
    </div>

    <!-- Crossfader -->
    <div class="crossfader-area">
      <div class="crossfader-labels">
        <span>A</span>
        <span>B</span>
      </div>
      <input
        type="range"
        class="crossfader"
        min="-1"
        max="1"
        step="0.01"
        :value="mixer.crossfader"
        @input="handleCrossfaderChange"
        @dblclick="handleCrossfaderReset"
      />
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, onMounted, onUnmounted } from 'vue'
import { useDjStore } from '../../stores/djStore'
import { useDjAudioEngine } from '../../composables/useDjAudioEngine'
import { drawMeter } from '../../utils/canvasHelpers'
import { linearToDb, formatDb } from '../../utils/audioHelpers'
import type { DeckId, CrossfaderAssign } from '../../types/dj'
import Icon from '../../shared/components/Icons.vue'

const store = useDjStore()
const engine = useDjAudioEngine()

const mixer = computed(() => store.mixer)

// Meter canvas refs
const meterRefs = new Map<DeckId, HTMLCanvasElement>()
const masterMeterCanvas = ref<HTMLCanvasElement | null>(null)
let meterFrameId: number | null = null

const setMeterRef = (deckId: DeckId, el: HTMLCanvasElement | null) => {
  if (el) meterRefs.set(deckId, el)
}

// ─── Knob Angle Calculations ────────────────────────────────────

const KNOB_MIN = -135
const KNOB_MAX = 135
const KNOB_RANGE = KNOB_MAX - KNOB_MIN

// Ensure neutral/center is 12 o'clock (0 degrees)
const gainToAngle = (gain: number): number => {
  // -12 to +12 dB → -135 to +135 degrees. 0 is center.
  const ratio = (gain + 12) / 24
  return Math.round(KNOB_MIN + ratio * KNOB_RANGE)
}

const eqToAngle = (eq: number): number => {
  // -26 to +6 dB. Use piecewise mapping to keep 0dB at 12 o'clock (0 degrees).
  if (eq >= 0) {
    // 0..6 dB maps to 0..135 degrees
    return Math.round((eq / 6) * 135)
  } else {
    // -26..0 dB maps to -135..0 degrees
    return Math.round((eq / 26) * 135)
  }
}

const filterToAngle = (filter: number): number => {
  // -1 to +1 → -135 to +135 degrees. 0 is center.
  const ratio = (filter + 1) / 2
  return Math.round(KNOB_MIN + ratio * KNOB_RANGE)
}

const volumeToAngle = (vol: number): number => {
  // 0 to 1 → -135 to +135 degrees
  return Math.round(KNOB_MIN + vol * KNOB_RANGE)
}

// ─── Knob Dragging ──────────────────────────────────────────────

type KnobType = 'gain' | 'hi' | 'mid' | 'low' | 'filter' | 'masterVol' | 'hpVol' | 'hpMix'

let activeKnob: { type: KnobType; deckId: DeckId; startY: number; startValue: number } | null = null

const startKnobDrag = (type: KnobType, deckId: DeckId, e: MouseEvent) => {
  e.preventDefault()
  let startValue = 0
  const ch = deckId > 0 ? store.getChannel(deckId) : null

  switch (type) {
    case 'gain': startValue = ch?.gain ?? 0; break
    case 'hi': startValue = ch?.eq.hi ?? 0; break
    case 'mid': startValue = ch?.eq.mid ?? 0; break
    case 'low': startValue = ch?.eq.low ?? 0; break
    case 'filter': startValue = ch?.filter ?? 0; break
    case 'masterVol': startValue = mixer.value.masterVolume; break
    case 'hpVol': startValue = mixer.value.headphoneVolume; break
    case 'hpMix': startValue = mixer.value.headphoneMix; break
  }

  activeKnob = { type, deckId, startY: e.clientY, startValue }
  document.addEventListener('mousemove', handleKnobDrag)
  document.addEventListener('mouseup', stopKnobDrag)
}

const handleKnobDrag = (e: MouseEvent) => {
  if (!activeKnob) return
  const delta = (activeKnob.startY - e.clientY) / 100 // Increased sensitivity (150 -> 100)
  const { type, deckId, startValue } = activeKnob

  switch (type) {
    case 'gain': {
      const val = Math.max(-12, Math.min(12, startValue + delta * 24))
      engine.updateChannelGain(deckId, val)
      break
    }
    case 'hi':
    case 'mid':
    case 'low': {
      const val = Math.max(-26, Math.min(6, startValue + delta * 32))
      engine.updateEq(deckId, type, val)
      break
    }
    case 'filter': {
      const val = Math.max(-1, Math.min(1, startValue + delta * 2))
      engine.updateFilter(deckId, val)
      break
    }
    case 'masterVol': {
      const val = Math.max(0, Math.min(1, startValue + delta))
      engine.updateMasterVolume(val)
      break
    }
    case 'hpVol': {
      const val = Math.max(0, Math.min(1, startValue + delta))
      engine.updateHeadphoneVolume(val)
      break
    }
    case 'hpMix': {
      const val = Math.max(0, Math.min(1, startValue + delta))
      store.setHeadphoneMix(val)
      break
    }
  }
}

const stopKnobDrag = () => {
  activeKnob = null
  document.removeEventListener('mousemove', handleKnobDrag)
  document.removeEventListener('mouseup', stopKnobDrag)
}

const resetKnob = (type: KnobType, deckId: DeckId) => {
  switch (type) {
    case 'gain': engine.updateChannelGain(deckId, 0); break
    case 'hi':
    case 'mid':
    case 'low': engine.updateEq(deckId, 'hi', 0); engine.updateEq(deckId, 'mid', 0); engine.updateEq(deckId, 'low', 0); break
    case 'filter': engine.updateFilter(deckId, 0); break
    case 'masterVol': engine.updateMasterVolume(0.8); break
    case 'hpVol': engine.updateHeadphoneVolume(0.5); break
    case 'hpMix': store.setHeadphoneMix(0.5); break
  }
}

// ─── Event Handlers ─────────────────────────────────────────────

const handleFaderChange = (deckId: DeckId, e: Event) => {
  const value = parseFloat((e.target as HTMLInputElement).value)
  engine.updateChannelVolume(deckId, value)
}

const handleFaderReset = (deckId: DeckId) => {
  engine.updateChannelVolume(deckId, 0.8)
}

const handleCrossfaderChange = (e: Event) => {
  const value = parseFloat((e.target as HTMLInputElement).value)
  engine.updateCrossfader(value)
}

const handleCrossfaderReset = () => {
  engine.updateCrossfader(0)
}

const handleCrossfaderAssign = (deckId: DeckId, assign: CrossfaderAssign) => {
  store.setCrossfaderAssign(deckId, assign)
  engine.applyCrossfader()
}

// ─── Level Meter Rendering ──────────────────────────────────────

const renderMeters = () => {
  for (const [deckId, canvas] of meterRefs) {
    const ctx = canvas.getContext('2d')
    if (!ctx) continue
    const ch = store.getChannel(deckId)
    drawMeter(ctx, ch.peakLevel, ch.peakHold, {
      width: 12,
      height: 120,
    })
  }

  // Master meter
  if (masterMeterCanvas.value) {
    const ctx = masterMeterCanvas.value.getContext('2d')
    if (ctx) {
      drawMeter(ctx, mixer.value.masterLevel, mixer.value.masterPeakHold, {
        width: 16,
        height: 120,
      })
    }
  }

  meterFrameId = requestAnimationFrame(renderMeters)
}

const masterLevelDb = computed(() => {
  const db = linearToDb(mixer.value.masterLevel)
  return formatDb(db)
})

// ─── Helpers ────────────────────────────────────────────────────

const getDeckColor = (id: DeckId): string => {
  const colors: Record<DeckId, string> = { 1: '#4f46e5', 2: '#ef4444', 3: '#22c55e', 4: '#f59e0b' }
  return colors[id]
}

// ─── Lifecycle ──────────────────────────────────────────────────

onMounted(() => {
  meterFrameId = requestAnimationFrame(renderMeters)
})

onUnmounted(() => {
  if (meterFrameId !== null) cancelAnimationFrame(meterFrameId)
  stopKnobDrag()
})
</script>

<style scoped>
.mixer-unit {
  background: var(--surface-color, #1a1a2e);
  border: 1px solid var(--border-color, #2a2a3e);
  border-radius: 12px;
  padding: 12px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.channel-strips {
  display: flex;
  gap: 8px;
  justify-content: center;
}

.channel-strip {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 6px;
  padding: 8px 6px;
  background: rgba(0,0,0,0.2);
  border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.04);
  min-width: 56px;
}

.channel-label {
  font-size: 11px;
  font-weight: 800;
  letter-spacing: 0.05em;
}

/* Knob Styles */
.knob-group {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 2px;
}

.knob-label {
  font-size: 8px;
  font-weight: 700;
  color: var(--text-tertiary, #666);
  letter-spacing: 0.08em;
  text-transform: uppercase;
}

.knob {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: radial-gradient(circle at 30% 30%, #3a3a4e, #1a1a2e);
  border: 2px solid rgba(255,255,255,0.1);
  cursor: grab;
  position: relative;
  box-shadow: 0 2px 6px rgba(0,0,0,0.3), inset 0 1px 2px rgba(255,255,255,0.05);
}

.knob:hover {
  border-color: rgba(255,255,255,0.2);
}

.knob:active {
  cursor: grabbing;
}

.knob-small {
  width: 30px;
  height: 30px;
}

.knob-indicator {
  position: absolute;
  inset: 3px;
  border-radius: 50%;
}

.knob-dot {
  position: absolute;
  top: 2px;
  left: 50%;
  transform: translateX(-50%);
  width: 3px;
  height: 3px;
  background: white;
  border-radius: 50%;
}

.knob-small .knob-dot {
  width: 2px;
  height: 2px;
}

/* EQ Section */
.eq-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
  padding: 4px 0;
  border-top: 1px solid rgba(255,255,255,0.04);
  border-bottom: 1px solid rgba(255,255,255,0.04);
}

/* CUE Button */
.btn-cue-mix {
  width: 30px;
  height: 24px;
  border-radius: 4px;
  border: 1px solid rgba(255,255,255,0.08);
  background: rgba(255,255,255,0.03);
  color: var(--text-tertiary, #666);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.btn-cue-mix:hover {
  background: rgba(255,255,255,0.08);
}

.btn-cue-mix.active {
  background: rgba(245, 158, 11, 0.3);
  border-color: #f59e0b;
  color: #f59e0b;
}

/* Level Meter */
.level-meter-container {
  height: 120px;
}

.level-meter-canvas {
  display: block;
  border-radius: 3px;
}

.master-meter {
  width: 16px;
}

/* Channel Fader */
.channel-fader-container {
  height: 110px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.channel-fader {
  -webkit-appearance: slider-vertical;
  appearance: slider-vertical;
  writing-mode: vertical-lr;
  direction: rtl;
  width: 28px;
  height: 110px;
  background: transparent;
  cursor: pointer;
}

.channel-fader::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 20px;
  height: 6px;
  background: #ddd;
  border-radius: 2px;
  cursor: pointer;
  box-shadow: 0 1px 4px rgba(0,0,0,0.4);
}

.channel-fader::-webkit-slider-runnable-track {
  width: 4px;
  background: rgba(255,255,255,0.08);
  border-radius: 2px;
}

/* Crossfader Assign */
.cf-assign {
  display: flex;
  gap: 2px;
}

.btn-cf-assign {
  padding: 2px 4px;
  border-radius: 2px;
  border: 1px solid rgba(255,255,255,0.06);
  background: rgba(255,255,255,0.02);
  color: var(--text-tertiary, #555);
  font-size: 8px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-cf-assign.active {
  background: rgba(255,255,255,0.15);
  color: white;
  border-color: rgba(255,255,255,0.2);
}

/* Master Section */
.master-section {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  justify-content: center;
  padding: 10px;
  background: rgba(0,0,0,0.15);
  border-radius: 8px;
  border: 1px solid rgba(255,255,255,0.04);
  flex-wrap: wrap;
}

.master-meter-group {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.master-level-db {
  font-size: 10px;
  font-family: 'Courier New', monospace;
  color: var(--text-secondary, #aaa);
  font-weight: 600;
}

.headphone-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.cf-curve-section {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 4px;
}

.cf-curve-btns {
  display: flex;
  gap: 2px;
}

.btn-cf-curve {
  padding: 3px 6px;
  border-radius: 3px;
  border: 1px solid rgba(255,255,255,0.06);
  background: rgba(255,255,255,0.02);
  color: var(--text-tertiary, #666);
  font-size: 10px;
  cursor: pointer;
  transition: all 0.15s;
}

.btn-cf-curve.active {
  background: rgba(255,255,255,0.15);
  color: white;
  border-color: rgba(255,255,255,0.2);
}

/* Crossfader */
.crossfader-area {
  padding: 8px 12px;
  background: rgba(0,0,0,0.1);
  border-radius: 8px;
}

.crossfader-labels {
  display: flex;
  justify-content: space-between;
  font-size: 10px;
  font-weight: 700;
  color: var(--text-tertiary, #666);
  padding: 0 4px;
  margin-bottom: 4px;
}

.crossfader {
  width: 100%;
  height: 24px;
  -webkit-appearance: none;
  appearance: none;
  background: transparent;
  cursor: pointer;
}

.crossfader::-webkit-slider-track {
  height: 6px;
  background: rgba(255,255,255,0.08);
  border-radius: 3px;
}

.crossfader::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 40px;
  height: 18px;
  background: linear-gradient(to bottom, #555, #333);
  border: 1px solid #666;
  border-radius: 4px;
  cursor: pointer;
  margin-top: -6px;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
}

.crossfader::-moz-range-thumb {
  width: 40px;
  height: 18px;
  background: linear-gradient(to bottom, #555, #333);
  border: 1px solid #666;
  border-radius: 4px;
  cursor: pointer;
  box-shadow: 0 2px 4px rgba(0,0,0,0.3);
}
</style>
