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
              <span v-if="currentSource.bpm && currentSource.bpm > 0" class="track-meta-item">
                <span class="meta-dot"></span>
                {{ Math.round(currentSource.bpm) }} BPM
              </span>
              <span v-if="currentSource.initial_key && currentSource.initial_key !== 'NONE'" class="track-meta-item">
                <span class="meta-dot"></span>
                {{ currentSource.initial_key }}
              </span>
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
          :class="{ seeking: isSeeking }"
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
      <!-- Playlist management for current track -->
      <div v-if="currentSource?.type === 'local'" class="playlist-mgmt">
        <button
          class="volume-btn"
          @click="togglePlaylistPanel"
          title="Manage playlists"
        >
          <Icon name="list" :size="18" />
        </button>
        <Transition name="panel-fade">
          <div v-if="playlistPanelOpen" class="playlist-panel">
            <div class="playlist-panel-header">
              <span>Playlists</span>
              <button class="panel-close" @click="playlistPanelOpen = false"><Icon name="x" :size="16" /></button>
            </div>
            <div v-if="loadingPlaylists" class="playlist-panel-loading">Loading&hellip;</div>
            <div v-else class="playlist-panel-list">
              <label
                v-for="pl in allPlaylists"
                :key="pl.id"
                class="playlist-check"
              >
                <input
                  type="checkbox"
                  :checked="trackPlaylistIds.has(pl.id)"
                  @change="handlePlaylistToggle(pl.id, !trackPlaylistIds.has(pl.id))"
                />
                <span>{{ pl.name }}</span>
              </label>
              <div v-if="allPlaylists.length === 0" class="playlist-panel-empty">No playlists yet</div>
            </div>
          </div>
        </Transition>
        <div v-if="playlistPanelOpen" class="playlist-backdrop" @click="playlistPanelOpen = false"></div>
      </div>

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
import { musicAPI } from '../../api/music'
import { formatTime } from '../../utils/audioHelpers'
import Icon from './Icons.vue'

const { state, stopPlayback: stop, setPlayingStatus, playPreviousTrack, playNextTrack, hasPreviousTrack, hasNextTrack, toggleAutoplay, toggleShuffle, cycleRepeat } = usePlayer()
const { error, success } = useToast()

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
  } else if (hasNext.value) {
    // Always advance to the next track in the queue — the queue works like a
    // regular sequential player.  The "autoplay" toggle is for continuing
    // playback once the queue runs out (e.g. radio-like behaviour); within
    // an existing queue we always advance.
    playNext()
  } else if (state.autoplay && state.repeat === 'all' && state.queue.length > 0) {
    // Repeat-all is handled inside playNextTrack, but we need autoplay on
    // to keep going when the queue is exhausted.
    playNext()
  }
}

const handleTimeUpdate = () => {
  if (audioRef.value && !isSeeking.value) {
    currentTime.value = audioRef.value.currentTime
    // Keep DJ Deck 1 position in sync while the regular player is active.
    // This runs ~4×/sec (browser fires timeupdate every 250ms) — lightweight.
    syncDeck1Position(audioRef.value.currentTime)
  }
}

/**
 * Sync DJ Deck 1's playback position to match the regular player.
 * Only fires if the same track is loaded in both. The deck stays paused;
 * we just keep its currentTime in sync so the user sees the right waveform
 * position if they switch to /decks.
 */
let lastDeck1SyncTime = 0
const syncDeck1Position = async (time: number) => {
  // Throttle to once per second to keep it cheap
  const now = Date.now()
  if (now - lastDeck1SyncTime < 1000) return
  lastDeck1SyncTime = now

  if (!state.currentSource || state.currentSource.type !== 'local') return
  const trackId = state.currentSource.id

  try {
    const { useDjStore } = await import('../../stores/djStore')
    const { useDjAudioEngine } = await import('../../composables/useDjAudioEngine')
    const store = useDjStore()
    const engine = useDjAudioEngine()

    const deck1 = store.getDeck(1)
    // Only sync if deck 1 has the same track and isn't actively playing
    if (deck1.track?.id !== trackId) return
    if (deck1.playState === 'playing') return

    engine.seekTo(1, time)
  } catch {
    // Non-critical
  }
}

const handleLoadedMetadata = () => {
  if (audioRef.value) {
    // Only update if the audio element gives us a valid duration
    const audioDuration = audioRef.value.duration
    if (audioDuration && isFinite(audioDuration) && audioDuration > 0) {
      duration.value = audioDuration
    }
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
// We track the seek target visually during drag but only commit the final
// position to the audio element on mouseup.  This avoids the "screechy"
// sound caused by rapidly updating currentTime while audio is playing.
const seekVisualTime = ref<number | null>(null) // non-null while dragging
let wasPlayingBeforeSeek = false

const startSeeking = (event: MouseEvent) => {
  if (!audioRef.value || !duration.value || !isFinite(duration.value)) return

  isSeeking.value = true
  wasPlayingBeforeSeek = isPlaying.value

  // Pause audio immediately so dragging doesn't produce sound
  if (wasPlayingBeforeSeek) {
    audioRef.value.pause()
  }

  updateSeekVisual(event)

  document.addEventListener('mousemove', handleSeekDrag)
  document.addEventListener('mouseup', stopSeeking)
}

const handleSeekDrag = (event: MouseEvent) => {
  if (!isSeeking.value) return
  updateSeekVisual(event)
}

const stopSeeking = () => {
  document.removeEventListener('mousemove', handleSeekDrag)
  document.removeEventListener('mouseup', stopSeeking)

  // Commit the final seek position
  if (audioRef.value && seekVisualTime.value !== null && duration.value && isFinite(duration.value)) {
    audioRef.value.currentTime = seekVisualTime.value
    currentTime.value = seekVisualTime.value
  }

  seekVisualTime.value = null
  isSeeking.value = false

  // Resume playback if it was playing before the seek
  if (wasPlayingBeforeSeek && audioRef.value) {
    audioRef.value.play().catch(() => {})
  }
}

/** Update only the visual position (currentTime ref) — no audio seek yet. */
const updateSeekVisual = (event: MouseEvent) => {
  if (!progressBarRef.value || !duration.value || !isFinite(duration.value)) return

  const rect = progressBarRef.value.getBoundingClientRect()
  let percentage = (event.clientX - rect.left) / rect.width
  percentage = Math.max(0, Math.min(1, percentage))

  const newTime = duration.value * percentage
  seekVisualTime.value = newTime
  currentTime.value = newTime // drives the visual progress bar
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

// Playlist management
const playlistPanelOpen = ref(false)
const loadingPlaylists = ref(false)
const allPlaylists = ref<{ id: string; name: string }[]>([])
const trackPlaylistIds = ref<Set<string>>(new Set())

const togglePlaylistPanel = async () => {
  if (playlistPanelOpen.value) {
    playlistPanelOpen.value = false
    return
  }
  playlistPanelOpen.value = true
  await loadPlaylistData()
}

const loadPlaylistData = async () => {
  if (!currentSource.value || currentSource.value.type !== 'local') return
  loadingPlaylists.value = true
  try {
    const [playlistsRes, trackPlRes] = await Promise.all([
      musicAPI.getPlaylists(),
      musicAPI.getTrackPlaylists(currentSource.value.id),
    ])
    allPlaylists.value = playlistsRes.data
    trackPlaylistIds.value = new Set(trackPlRes.data.map((p: any) => p.id))
  } catch (err) {
    console.warn('Failed to load playlist data', err)
  } finally {
    loadingPlaylists.value = false
  }
}

const handlePlaylistToggle = async (playlistId: string, add: boolean) => {
  if (!currentSource.value || currentSource.value.type !== 'local') return
  const trackId = currentSource.value.id
  try {
    if (add) {
      await musicAPI.addPlaylistTrack(playlistId, { music_file_id: trackId })
      trackPlaylistIds.value.add(playlistId)
      success('Added to playlist')
    } else {
      await musicAPI.removePlaylistTrack(playlistId, trackId)
      trackPlaylistIds.value.delete(playlistId)
      success('Removed from playlist')
    }
    // Force reactivity
    trackPlaylistIds.value = new Set(trackPlaylistIds.value)
  } catch (err: any) {
    error('Playlist update failed', err?.response?.data?.error || err?.message)
  }
}

// Sync audio source when state changes
const syncAudioSource = () => {
  if (!audioRef.value) return

  if (state.audioUrl) {
    audioRef.value.src = state.audioUrl
    audioRef.value.load()
    audioRef.value.volume = volumeLevel.value / 100
    
    // Reset duration and currentTime first
    duration.value = 0
    currentTime.value = 0
    
    // Set fallback duration from metadata (convert ms to s)
    if (state.currentSource?.type === 'local' && state.currentSource.duration) {
      duration.value = state.currentSource.duration / 1000
    }
    
    // Only auto-play if the player state says we should be playing.
    // This preserves pause state when navigating between views.
    if (state.isPlaying) {
      const playPromise = audioRef.value.play()
      if (playPromise !== undefined) {
        playPromise.catch((err) => {
          console.warn('Autoplay prevented:', err)
        })
      }
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

watch(() => state.isPlaying, (playing) => {
  if (!audioRef.value) return
  if (playing && audioRef.value.paused) {
    audioRef.value.play().catch(() => {})
  } else if (!playing && !audioRef.value.paused) {
    audioRef.value.pause()
  }
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
  z-index: 900;
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
  display: flex;
  align-items: center;
  flex-wrap: wrap;
  gap: 4px;
}

.track-meta-item {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  color: var(--text-tertiary);
  font-weight: 500;
}

.meta-dot {
  width: 3px;
  height: 3px;
  background: var(--text-tertiary);
  border-radius: 50%;
  opacity: 0.5;
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
  /* transition is toggled off during seeking via .seeking class */
  transition: width 0.1s linear;
}

.progress-bar.seeking .progress-fill {
  transition: none;
}

.progress-bar.seeking .progress-handle {
  transform: translate(-50%, -50%) scale(1);
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

/* Playlist management panel */
.playlist-mgmt {
  position: relative;
}

.playlist-backdrop {
  position: fixed;
  inset: 0;
  z-index: 99;
}

.playlist-panel {
  position: absolute;
  bottom: calc(100% + 12px);
  right: 0;
  width: 240px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  z-index: 100;
  overflow: hidden;
}

.playlist-panel-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 10px 14px;
  border-bottom: 1px solid var(--border-color);
  font-size: 13px;
  font-weight: 700;
  color: var(--text-color);
}

.panel-close {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  font-size: 14px;
  padding: 2px 6px;
  border-radius: 6px;
  transition: all 0.15s;
}

.panel-close:hover {
  background: var(--background-elevated);
  color: var(--text-color);
}

.playlist-panel-loading,
.playlist-panel-empty {
  padding: 16px 14px;
  font-size: 13px;
  color: var(--text-tertiary);
  text-align: center;
}

.playlist-panel-list {
  max-height: 200px;
  overflow-y: auto;
  padding: 6px;
}

.playlist-check {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 10px;
  border-radius: 8px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-color);
  transition: background 0.15s;
}

.playlist-check:hover {
  background: var(--background-elevated);
}

.playlist-check input[type="checkbox"] {
  accent-color: var(--primary-color);
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.panel-fade-enter-active,
.panel-fade-leave-active {
  transition: opacity 0.15s, transform 0.15s;
}

.panel-fade-enter-from,
.panel-fade-leave-to {
  opacity: 0;
  transform: translateY(8px);
}

/* Responsive Design */
@media (max-width: 768px) {
  .global-player {
    grid-template-columns: 1fr;
    gap: 12px;
    padding: 12px 16px 16px;
    bottom: 64px;
    border-radius: 16px 16px 0 0;
  }

  .player-info {
    justify-content: flex-start;
  }

  .track-thumbnail {
    width: 48px;
    height: 48px;
  }

  .player-controls {
    order: -1; /* Progress bar and main controls at top? No, maybe track info at top. */
  }

  .player-volume {
    justify-content: center;
    padding-top: 8px;
    border-top: 1px solid var(--border-color);
  }
}

@media (max-width: 480px) {
  .global-player {
    padding: 10px 12px 14px;
    gap: 8px;
  }

  .player-options {
    display: none; /* Hide shuffle/repeat on very small screens to save space */
  }

  .track-thumbnail {
    width: 40px;
    height: 40px;
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
