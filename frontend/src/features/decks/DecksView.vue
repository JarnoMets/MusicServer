<template>
  <div
    class="decks-view"
    @dragover.prevent
    @drop.prevent="handleDrop"
  >
    <!-- Autoplay Header/Status bar -->
    <div class="autoplay-bar" :class="{ active: store.autoPlay }">
      <div class="status-indicator">
        <Icon :name="store.autoPlay ? 'activity' : 'stop-circle'" :size="16" />
        <span>{{ store.autoPlay ? 'AUTOPLAY ACTIVE' : 'AUTOPLAY READY' }}</span>
        <span v-if="store.isTransitioning" class="transitioning-badge" :class="{ 'echo-fade': transitionType === 'echo-fade' }">
          {{ transitionType === 'echo-fade' ? 'ECHO FADE...' : 'BPM MATCH...' }}
        </span>
      </div>
      
      <div class="autoplay-controls">
        <!-- Settings toggle -->
        <button class="btn-settings" @click="showAutoplaySettings = !showAutoplaySettings" :class="{ active: showAutoplaySettings }">
          <Icon name="settings" :size="14" />
        </button>

        <button 
          v-if="store.autoPlayPlaylist.length > 0"
          class="btn-queue"
          @click="showQueueModal = true"
        >
          <Icon name="list" :size="14" />
          <span>View Queue ({{ store.autoPlayPlaylist.length }} tracks)</span>
        </button>

        <button 
          class="btn-autoplay" 
          :class="{ active: store.autoPlay }"
          @click="toggleAutoplay"
        >
          {{ store.autoPlay ? 'STOP AUTOPLAY' : 'START AUTOPLAY' }}
        </button>
        
        <div class="active-deck-info" v-if="store.autoPlay">
          <span>Active: Deck {{ currentActiveDeck }}</span>
          <Icon name="arrow-right" :size="12" />
          <span>Next: Deck {{ nextTargetDeck }}</span>
        </div>
      </div>
    </div>

    <!-- Autoplay Settings Panel -->
    <div v-if="showAutoplaySettings" class="autoplay-settings-panel">
      <div class="setting-row">
        <label>Match time</label>
        <div class="setting-control">
          <input
            type="range"
            :min="10"
            :max="180"
            :step="5"
            :value="store.autoplaySettings.matchTimeSeconds"
            @input="store.setAutoplaySetting('matchTimeSeconds', Number(($event.target as HTMLInputElement).value))"
          />
          <span class="setting-value">{{ store.autoplaySettings.matchTimeSeconds }}s</span>
        </div>
        <span class="setting-hint">Time before track end when next track begins to match BPM (must be >= Overlap)</span>
      </div>
      <div class="setting-row">
        <label>Overlap time</label>
        <div class="setting-control">
          <input
            type="range"
            :min="5"
            :max="180"
            :step="5"
            :value="store.autoplaySettings.overlapSeconds"
            @input="(e) => {
              const v = Number((e.target as HTMLInputElement).value);
              // ensure overlap <= matchTime
              const clamped = Math.min(v, store.autoplaySettings.matchTimeSeconds);
              store.setAutoplaySetting('overlapSeconds', clamped);
            }"
          />
          <span class="setting-value">{{ store.autoplaySettings.overlapSeconds }}s</span>
        </div>
        <span class="setting-hint">How long both songs play together (crossfade / overlap)</span>
      </div>
      <div class="setting-row">
        <label>Exit time</label>
        <div class="setting-control">
          <input
            type="range"
            :min="5"
            :max="120"
            :step="5"
            :value="store.autoplaySettings.exitTimeSeconds"
            @input="store.setAutoplaySetting('exitTimeSeconds', Number(($event.target as HTMLInputElement).value))"
          />
          <span class="setting-value">{{ store.autoplaySettings.exitTimeSeconds }}s</span>
        </div>
        <span class="setting-hint">Time the new song takes to return to its original BPM after transition</span>
      </div>
    </div>

    <!-- Main Layout: Decks + Mixer -->
    <div class="decks-layout">
      <!-- Left Column (1 & 2) -->
      <div class="decks-column">
        <DeckUnit :deck-id="1" :other-playing-key="otherPlayingKey(1)" @open-load="openLoadModal(1)" />
        <DeckUnit :deck-id="2" :other-playing-key="otherPlayingKey(2)" @open-load="openLoadModal(2)" />
      </div>

      <!-- Mixer -->
      <MixerUnit />

      <!-- Right Column (3 & 4) -->
      <div class="decks-column">
        <DeckUnit :deck-id="3" :other-playing-key="otherPlayingKey(3)" @open-load="openLoadModal(3)" />
        <DeckUnit :deck-id="4" :other-playing-key="otherPlayingKey(4)" @open-load="openLoadModal(4)" />
      </div>
    </div>

    <!-- Track Loading Modal -->
    <TrackSelectModal
      v-if="showLoadModal"
      :show="showLoadModal"
      :deck-id="loadTargetDeck"
      @close="closeLoadModal"
    />

    <!-- Queue Panel -->
    <QueuePanel
      v-if="showQueueModal"
      @close="showQueueModal = false"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useDjStore } from '../../stores/djStore'
import { useDjAudioEngine } from '../../composables/useDjAudioEngine'
import { useDjAutomation } from '../../composables/useDjAutomation'
import { usePlayer } from '../../composables/usePlayer'
import DeckUnit from './DeckUnit.vue'
import MixerUnit from './MixerUnit.vue'
import TrackSelectModal from './TrackSelectModal.vue'
import QueuePanel from './QueuePanel.vue'
import Icon from '../../shared/components/Icons.vue'
import type { DeckId } from '../../types/dj'
import type { MusicFile } from '../../types/index'

const store = useDjStore()
const engine = useDjAudioEngine()
const player = usePlayer()
const { startAutoplay, stopAutoplay, currentActiveDeck, nextTargetDeck, transitionType } = useDjAutomation()
const showLoadModal = ref(false)
const showQueueModal = ref(false)
const showAutoplaySettings = ref(false)
const loadTargetDeck = ref<DeckId>(1)

const openLoadModal = async (deckId: DeckId) => {
  if (!engine.isInitialized.value) {
    await engine.init()
  }
  loadTargetDeck.value = deckId
  showLoadModal.value = true
}

const closeLoadModal = () => {
  showLoadModal.value = false
}

/** Was the regular player active when we entered decks? (used by onUnmounted) */
const playerWasPlaying = ref(false)

onMounted(async () => {
  if (!engine.isInitialized.value) {
    await engine.init()
  }

  // If the regular player is currently playing a track that's mirrored on
  // Deck 1, pause the regular player and hand off to the deck.  The user
  // can then seamlessly hit Play on Deck 1 to continue.
  const src = player.state.currentSource
  if (src && src.type === 'local') {
    const deck1 = store.getDeck(1)
    playerWasPlaying.value = player.state.isPlaying

    if (deck1.track?.id === src.id) {
      // Pause the regular player (it's hidden on /decks anyway)
      if (player.state.isPlaying) {
        player.setPlayingStatus(false)
      }
    }
  }
})

onUnmounted(() => {
  // When leaving the decks view, if Deck 1 has a track that matches the
  // regular player's current source AND Deck 1 is paused, keep the regular
  // player paused too (honour the pause state).
  // If Deck 1 was playing, pause it and resume the regular player from Deck 1's position.
  const src = player.state.currentSource
  if (src && src.type === 'local') {
    const deck1 = store.getDeck(1)
    if (deck1.track?.id === src.id) {
      if (deck1.playState === 'playing') {
        // Deck 1 was playing → pause it and let the regular player resume
        engine.pause(1)
        player.setPlayingStatus(true)
      } else {
        // Deck 1 was paused/cued → keep regular player paused
        player.setPlayingStatus(false)
      }
    }
  }
})

const otherPlayingKey = (deckId: DeckId): string | null => {
  for (const d of store.decks) {
    if (d.id !== deckId && d.playState === 'playing' && d.track?.initial_key) {
      return d.track.initial_key
    }
  }
  return null
}

const handleDrop = async (e: DragEvent) => {
  try {
    const data = e.dataTransfer?.getData('application/json')
    if (!data) return
    const track: MusicFile = JSON.parse(data)

    if (!engine.isInitialized.value) {
      await engine.init()
    }

    const rect = (e.currentTarget as HTMLElement).getBoundingClientRect()
    const x = (e.clientX - rect.left) / rect.width
    const y = (e.clientY - rect.top) / rect.height

    let deckId: DeckId
    if (y < 0.5) {
      deckId = x < 0.5 ? 1 : 3
    } else {
      deckId = x < 0.5 ? 2 : 4
    }

    engine.loadTrackToDeck(deckId, track)
  } catch (err) {
    console.warn('Drop failed:', err)
  }
}

const toggleAutoplay = () => {
  if (store.autoPlay) stopAutoplay()
  else startAutoplay()
}
</script>

<style scoped>
.decks-view {
  height: 100%;
  display: flex;
  flex-direction: column;
  position: relative;
  min-height: 0;
}

.autoplay-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 8px 16px;
  background: rgba(0,0,0,0.4);
  border-bottom: 1px solid rgba(255,255,255,0.05);
  margin-bottom: 8px;
  border-radius: 8px;
  font-size: 11px;
  font-weight: 700;
  color: #aaa;
}

.autoplay-bar.active {
  background: rgba(79, 70, 229, 0.15);
  border-color: rgba(79, 70, 229, 0.3);
  color: #fff;
}

.status-indicator {
  display: flex;
  align-items: center;
  gap: 8px;
}

.transitioning-badge {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.15);
  padding: 2px 6px;
  border-radius: 4px;
  animation: pulse 1s infinite alternate;
}

.transitioning-badge.echo-fade {
  color: #a78bfa;
  background: rgba(167, 139, 250, 0.15);
}

@keyframes pulse {
  from { opacity: 1; }
  to { opacity: 0.5; }
}

.autoplay-controls {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-queue {
  display: flex;
  align-items: center;
  gap: 6px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #ccc;
  padding: 4px 10px;
  border-radius: 4px;
  cursor: pointer;
  font-size: 10px;
  transition: all 0.2s;
}

.btn-queue:hover {
  background: rgba(255, 255, 255, 0.1);
  border-color: rgba(255, 255, 255, 0.2);
  color: white;
}

.btn-settings {
  display: flex;
  align-items: center;
  justify-content: center;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.1);
  color: #888;
  width: 28px;
  height: 28px;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-settings:hover,
.btn-settings.active {
  background: rgba(79, 70, 229, 0.2);
  border-color: rgba(79, 70, 229, 0.4);
  color: #a5b4fc;
}

.autoplay-settings-panel {
  display: flex;
  gap: 24px;
  padding: 8px 16px;
  background: rgba(0, 0, 0, 0.3);
  border-bottom: 1px solid rgba(255, 255, 255, 0.05);
  border-radius: 0 0 8px 8px;
  margin-top: -8px;
  margin-bottom: 8px;
}

.setting-row {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
}

.setting-row label {
  color: #888;
  font-weight: 600;
  white-space: nowrap;
  min-width: 110px;
}

.setting-control {
  display: flex;
  align-items: center;
  gap: 6px;
}

.setting-control input[type="range"] {
  width: 100px;
  height: 4px;
  accent-color: #4f46e5;
  cursor: pointer;
}

.setting-value {
  color: #a5b4fc;
  font-weight: 700;
  font-size: 12px;
  min-width: 30px;
  text-align: right;
}

.setting-hint {
  color: #555;
  font-size: 9px;
  font-style: italic;
}

.btn-autoplay {
  background: rgba(255,255,255,0.1);
  border: 1px solid rgba(255,255,255,0.2);
  color: white;
  padding: 4px 12px;
  border-radius: 20px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-autoplay.active {
  background: #ef4444;
  border-color: #ef4444;
}

.active-deck-info {
  display: flex;
  align-items: center;
  gap: 6px;
  opacity: 0.8;
}

.decks-layout {
  display: flex;
  flex: 1;
  gap: 8px;
  min-height: 0;
  overflow: hidden;
  padding-bottom: 0;
}

.decks-column {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
}


</style>

