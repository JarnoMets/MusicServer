<template>
  <div class="global-player" :class="{ 'player-active': currentSource, 'player-idle': !currentSource }">
    <audio
      ref="audioRef"
      @play="handlePlay"
      @pause="handlePause"
      @ended="handleEnded"
      @timeupdate="handleTimeUpdate"
      @loadedmetadata="handleLoadedMetadata"
      @volumechange="handleVolumeChange"
      @error="handleError"
    />

    <!-- Track Info -->
    <div class="player-section player-info">
      <div class="track-thumbnail" :class="{ playing: isPlaying }">
        <Icon name="music" :size="26" />
        <div v-if="isPlaying" class="thumbnail-pulse"></div>
      </div>
      <div class="track-details">
        <div class="track-title">
          {{ currentSource?.title || 'No track playing' }}
        </div>
        <div class="track-artist">
          <template v-if="currentSource">
            <span v-if="currentSource.type === 'local'">
              {{ currentSource.artist || 'Unknown Artist' }}
            </span>
            <span v-else class="stream-badge">
              <span class="live-dot"></span>
              {{ currentSource.genre || 'Internet Stream' }}
            </span>
          </template>
          <template v-else>
            Select a track to start listening
          </template>
        </div>
      </div>
    </div>

    <!-- Player Controls -->
    <div class="player-section player-controls">
      <div class="control-buttons">
        <button
          class="control-btn"
          @click="playPrevious"
          :disabled="!hasPrevious"
          title="Previous track"
        >
          <Icon name="skip-back" :size="18" />
        </button>
        <button
          class="control-btn control-btn-main"
          @click="togglePlayPause"
          :disabled="!currentSource"
          :title="isPlaying ? 'Pause' : 'Play'"
        >
          <Icon v-if="isPlaying" name="pause" :size="20" />
          <Icon v-else name="play" :size="20" />
        </button>
        <button
          class="control-btn"
          @click="playNext"
          :disabled="!hasNext"
          title="Next track"
        >
          <Icon name="skip-forward" :size="18" />
        </button>
        <button
          class="control-btn control-btn-small"
          @click="stopPlayback"
          :disabled="!currentSource"
          title="Stop"
        >
          <Icon name="x" :size="16" />
        </button>
      </div>

      <div class="progress-section">
        <span class="time-display">{{ formatTime(currentTime) }}</span>
        <div 
          class="progress-bar" 
          ref="progressBarRef"
          @mousedown="startSeeking"
          @mouseenter="showHandle = true" 
          @mouseleave="showHandle = false"
        >
          <div class="progress-track">
            <div class="progress-fill" :style="{ width: progressPercentage + '%' }"></div>
            <div 
              class="progress-handle" 
              :class="{ visible: showHandle || isSeeking }" 
              :style="{ left: progressPercentage + '%' }"
            ></div>
          </div>
        </div>
        <span class="time-display">{{ formatTime(duration) }}</span>
      </div>

      <div class="player-options">
        <button
          class="option-btn"
          :class="{ active: state.autoplay }"
          @click="toggleAutoplay"
          title="Toggle autoplay"
        >
          <Icon name="play-circle" :size="16" />
        </button>
        <button
          class="option-btn"
          :class="{ active: state.shuffle }"
          @click="toggleShuffle"
          title="Toggle shuffle"
        >
          <Icon name="shuffle" :size="16" />
        </button>
        <button
          class="option-btn"
          :class="{ active: state.repeat !== 'off', 'repeat-one': state.repeat === 'one' }"
          @click="cycleRepeat"
          title="Cycle repeat mode"
        >
          <Icon name="repeat" :size="16" />
          <span v-if="state.repeat === 'one'" class="repeat-indicator">1</span>
        </button>
      </div>
    </div>

    <!-- Volume Controls -->
    <div class="player-section player-volume">
      <button
        class="volume-btn"
        @click="toggleMute"
        :title="isMuted ? 'Unmute' : 'Mute'"
      >
        <Icon v-if="isMuted || volumeLevel === 0" name="volume-x" :size="20" />
        <Icon v-else name="volume-2" :size="20" />
      </button>
      <div class="volume-slider">
        <input
          type="range"
          min="0"
          max="100"
          v-model="volumeLevel"
          @input="updateVolume"
          class="volume-input"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import Icon from './Icons.vue'

const { state, stopPlayback: stop, setPlayingStatus, playPreviousTrack, playNextTrack, hasPreviousTrack, hasNextTrack, toggleAutoplay, toggleShuffle, cycleRepeat } = usePlayer()
const { error } = useToast()

const audioRef = ref<HTMLAudioElement | null>(null)
const progressBarRef = ref<HTMLElement | null>(null)
const currentTime = ref(0)
const duration = ref(0)
const volumeLevel = ref(100)
const isMuted = ref(false)
const isPlaying = ref(false)
const showHandle = ref(false)
const isSeeking = ref(false)

const currentSource = computed(() => state.currentSource)
const hasPrevious = computed(() => hasPreviousTrack())
const hasNext = computed(() => hasNextTrack())

const progressPercentage = computed(() => {
  if (!duration.value || !isFinite(duration.value)) return 0
  return Math.min((currentTime.value / duration.value) * 100, 100)
})

// Audio event handlers
const handlePlay = () => {
  isPlaying.value = true
  setPlayingStatus(true)
}

const handlePause = () => {
  isPlaying.value = false
  setPlayingStatus(false)
}

const handleEnded = () => {
  isPlaying.value = false
  setPlayingStatus(false)
  currentTime.value = 0

  // Handle repeat one mode
  if (state.repeat === 'one' && state.currentSource?.type === 'local') {
    const playPromise = audioRef.value?.play()
    if (playPromise !== undefined) {
      playPromise.catch(err => {
        console.warn('Replay failed:', err)
      })
    }
  } else if (state.autoplay) {
    // Auto-play next track if autoplay is enabled
    if (hasNext.value) {
      playNext()
    }
  }
}

const handleTimeUpdate = () => {
  if (audioRef.value && !isSeeking.value) {
    currentTime.value = audioRef.value.currentTime
  }
}

const handleLoadedMetadata = () => {
  if (audioRef.value) {
    duration.value = audioRef.value.duration
  }
}

const handleVolumeChange = () => {
  if (audioRef.value) {
    isMuted.value = audioRef.value.muted
  }
}

const handleError = () => {
  if (currentSource.value) {
    error('Playback failed', `Could not play "${currentSource.value.title}"`)
  }
}

// Control functions
const togglePlayPause = () => {
  if (!audioRef.value) return
  if (isPlaying.value) {
    audioRef.value.pause()
  } else {
    audioRef.value.play().catch(err => {
      console.warn('Playback failed:', err)
    })
  }
}

const stopPlayback = () => {
  stop()
  if (audioRef.value) {
    audioRef.value.pause()
    audioRef.value.currentTime = 0
    audioRef.value.removeAttribute('src')
  }
  currentTime.value = 0
  duration.value = 0
  isPlaying.value = false
}

const playPrevious = () => {
  playPreviousTrack()
}

const playNext = () => {
  playNextTrack()
}

// Seeking with drag support
const startSeeking = (event: MouseEvent) => {
  if (!audioRef.value || !duration.value || !isFinite(duration.value)) return
  
  isSeeking.value = true
  seekToPosition(event)
  
  // Add document-level listeners for drag
  document.addEventListener('mousemove', handleSeekDrag)
  document.addEventListener('mouseup', stopSeeking)
}

const handleSeekDrag = (event: MouseEvent) => {
  if (!isSeeking.value) return
  seekToPosition(event)
}

const stopSeeking = () => {
  isSeeking.value = false
  document.removeEventListener('mousemove', handleSeekDrag)
  document.removeEventListener('mouseup', stopSeeking)
}

const seekToPosition = (event: MouseEvent) => {
  if (!audioRef.value || !progressBarRef.value || !duration.value || !isFinite(duration.value)) return
  
  const rect = progressBarRef.value.getBoundingClientRect()
  let percentage = (event.clientX - rect.left) / rect.width
  percentage = Math.max(0, Math.min(1, percentage))
  
  const newTime = duration.value * percentage
  audioRef.value.currentTime = newTime
  currentTime.value = newTime
}

const updateVolume = () => {
  if (audioRef.value) {
    audioRef.value.volume = volumeLevel.value / 100
    if (volumeLevel.value > 0 && isMuted.value) {
      audioRef.value.muted = false
    }
  }
}

const toggleMute = () => {
  if (audioRef.value) {
    audioRef.value.muted = !audioRef.value.muted
  }
}

const formatTime = (seconds: number): string => {
  if (!isFinite(seconds) || seconds < 0) return '0:00'
  const mins = Math.floor(seconds / 60)
  const secs = Math.floor(seconds % 60)
  return `${mins}:${secs.toString().padStart(2, '0')}`
}

// Sync audio source when state changes
const syncAudioSource = () => {
  if (!audioRef.value) return

  if (state.audioUrl) {
    audioRef.value.src = state.audioUrl
    audioRef.value.load()
    audioRef.value.volume = volumeLevel.value / 100
    const playPromise = audioRef.value.play()
    if (playPromise !== undefined) {
      playPromise.catch((err) => {
        console.warn('Autoplay prevented:', err)
      })
    }
  } else {
    audioRef.value.pause()
    audioRef.value.removeAttribute('src')
    currentTime.value = 0
    duration.value = 0
  }
}

watch(() => state.updatedAt, () => {
  syncAudioSource()
})

onMounted(() => {
  syncAudioSource()
  if (audioRef.value) {
    audioRef.value.volume = volumeLevel.value / 100
  }
})

onUnmounted(() => {
  // Clean up any event listeners
  document.removeEventListener('mousemove', handleSeekDrag)
  document.removeEventListener('mouseup', stopSeeking)
})
</script>

<style scoped>
.global-player {
  display: grid;
  grid-template-columns: minmax(200px, 1fr) 2fr minmax(140px, 1fr);
  gap: 24px;
  align-items: center;
  padding: 16px 28px;
  background: var(--header-gradient);
  border-top: 1px solid var(--border-color);
  box-shadow: 0 -8px 32px rgba(0, 0, 0, 0.3);
  min-height: 88px;
  position: fixed;
  bottom: 0;
  left: 0;
  right: 0;
  z-index: 1000;
  backdrop-filter: var(--glass-blur);
  transition: all var(--transition-base);
}

.global-player.player-idle {
  opacity: 0.85;
}

.global-player.player-active {
  border-top-color: var(--primary-color);
}

.player-section {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* Track Info Section */
.player-info {
  justify-content: flex-start;
  min-width: 0;
}

.track-thumbnail {
  width: 56px;
  height: 56px;
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--accent-color) 100%);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  box-shadow: 0 4px 20px var(--accent-muted);
  position: relative;
  overflow: hidden;
  transition: all var(--transition-base);
}

.track-thumbnail.playing {
  animation: pulse-glow 2s ease-in-out infinite;
}

@keyframes pulse-glow {
  0%, 100% {
    box-shadow: 0 4px 20px var(--accent-muted);
  }
  50% {
    box-shadow: 0 4px 30px var(--accent-muted), 0 0 40px var(--accent-muted);
  }
}

.track-thumbnail .icon {
  color: white;
}

.thumbnail-pulse {
  position: absolute;
  inset: 0;
  background: radial-gradient(circle, rgba(255,255,255,0.1) 0%, transparent 70%);
  animation: pulse 1.5s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% { transform: scale(1); opacity: 0.5; }
  50% { transform: scale(1.1); opacity: 0.8; }
}

.track-details {
  min-width: 0;
  flex: 1;
}

.track-title {
  font-weight: 700;
  font-size: 14px;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin-bottom: 4px;
}

.track-artist {
  font-size: 12px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.stream-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
}

.live-dot {
  width: 6px;
  height: 6px;
  background: var(--error-color);
  border-radius: 50%;
  animation: blink 1s ease-in-out infinite;
}

@keyframes blink {
  0%, 100% { opacity: 1; }
  50% { opacity: 0.4; }
}

/* Player Controls Section */
.player-controls {
  flex-direction: column;
  justify-content: center;
  gap: 10px;
  flex: 1;
}

.control-buttons {
  display: flex;
  gap: 12px;
  align-items: center;
  justify-content: center;
}

.control-btn {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  background: var(--surface-muted);
  border: 1px solid var(--border-color);
  color: var(--text-color);
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--transition-base);
}

.control-btn:hover:not(:disabled) {
  background: var(--surface-hover);
  border-color: var(--border-hover);
  transform: scale(1.05);
}

.control-btn:disabled {
  opacity: 0.3;
  cursor: not-allowed;
}

.control-btn-main {
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, var(--primary-color), var(--accent-color));
  border: none;
  box-shadow: 0 4px 20px var(--accent-muted);
  font-size: 18px;
}

.control-btn-main:hover:not(:disabled) {
  transform: scale(1.08);
  box-shadow: 0 6px 25px var(--accent-muted);
}

.control-btn-main .icon {
  color: white;
}

.control-btn-small {
  width: 32px;
  height: 32px;
  font-size: 14px;
}

.progress-section {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  max-width: 600px;
}

.time-display {
  font-size: 11px;
  color: var(--text-tertiary);
  font-variant-numeric: tabular-nums;
  min-width: 36px;
  text-align: center;
  font-weight: 500;
}

.progress-bar {
  flex: 1;
  height: 28px;
  cursor: pointer;
  display: flex;
  align-items: center;
  padding: 10px 0;
}

.progress-track {
  position: relative;
  width: 100%;
  height: 4px;
  background: var(--surface-muted);
  border-radius: var(--radius-full);
  overflow: visible;
  transition: height var(--transition-fast);
}

.progress-bar:hover .progress-track {
  height: 6px;
}

.progress-fill {
  position: absolute;
  top: 0;
  left: 0;
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color) 0%, var(--accent-color) 100%);
  border-radius: var(--radius-full);
  transition: width 0.1s linear;
}

.progress-handle {
  position: absolute;
  top: 50%;
  transform: translate(-50%, -50%) scale(0);
  width: 14px;
  height: 14px;
  background: white;
  border-radius: 50%;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.3);
  transition: transform var(--transition-base);
}

.progress-handle.visible,
.progress-bar:hover .progress-handle {
  transform: translate(-50%, -50%) scale(1);
}

/* Player Options Section */
.player-options {
  display: flex;
  gap: 8px;
  align-items: center;
  justify-content: flex-end;
}

.option-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
  font-size: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--transition-base);
  position: relative;
}

.option-btn:hover {
  background: var(--surface-muted);
  border-color: var(--border-hover);
  color: var(--text-color);
}

.option-btn.active {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
}

.repeat-indicator {
  position: absolute;
  font-size: 10px;
  font-weight: 700;
  top: 6px;
  right: 5px;
  background: var(--primary-color);
  color: white;
  width: 16px;
  height: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  border: 2px solid var(--surface-color);
}

/* Volume Controls Section */
.player-volume {
  justify-content: flex-end;
  gap: 10px;
}

.volume-btn {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  font-size: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all var(--transition-base);
}

.volume-btn:hover {
  color: var(--text-color);
  background: var(--surface-muted);
}

.volume-slider {
  width: 90px;
  height: 28px;
  display: flex;
  align-items: center;
}

.volume-input {
  width: 100%;
  height: 4px;
  background: var(--surface-muted);
  border-radius: var(--radius-full);
  outline: none;
  -webkit-appearance: none;
  appearance: none;
  cursor: pointer;
}

.volume-input::-webkit-slider-thumb {
  -webkit-appearance: none;
  appearance: none;
  width: 12px;
  height: 12px;
  background: var(--primary-color);
  border-radius: 50%;
  cursor: pointer;
  transition: all var(--transition-base);
  box-shadow: 0 2px 6px var(--accent-muted);
}

.volume-input::-webkit-slider-thumb:hover {
  transform: scale(1.2);
  box-shadow: 0 4px 12px var(--accent-muted);
}

.volume-input::-moz-range-thumb {
  width: 12px;
  height: 12px;
  background: var(--primary-color);
  border: none;
  border-radius: 50%;
  cursor: pointer;
  transition: all var(--transition-base);
  box-shadow: 0 2px 6px var(--accent-muted);
}

/* Responsive Design */
@media (max-width: 900px) {
  .global-player {
    grid-template-columns: 1fr;
    gap: 12px;
    padding: 12px 16px 16px;
  }

  .player-info {
    justify-content: center;
  }

  .track-thumbnail {
    width: 48px;
    height: 48px;
  }

  .thumbnail-icon {
    font-size: 22px;
  }

  .player-volume {
    justify-content: center;
  }
}

@media (max-width: 480px) {
  .global-player {
    padding: 10px 12px 14px;
  }

  .track-thumbnail {
    width: 44px;
    height: 44px;
  }

  .control-btn-main {
    width: 44px;
    height: 44px;
  }

  .control-btn {
    width: 36px;
    height: 36px;
    font-size: 14px;
  }

  .volume-slider {
    width: 70px;
  }
}
</style>
