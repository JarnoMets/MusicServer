<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import type { MusicFile, PlaylistSummary } from '../../types/MusicTab'
import { formatDuration, formatDate } from '../../utils/musicFormatters'
import Icon from '../../shared/components/Icons.vue'
import { usePlayer } from '../../composables/usePlayer'

interface Props {
  tracks: MusicFile[]
  playlists: PlaylistSummary[]
  loading: boolean
  playlistMenuOpen: string | null
  canEdit?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  canEdit: false
})

const emit = defineEmits<{
  'track:play': [track: MusicFile]
  'track:edit': [track: MusicFile]
  'track:delete': [track: MusicFile]
  'track:confirm-genre': [track: MusicFile]
  'playlist:toggle': [trackId: string]
  'playlist:add': [trackId: string, playlistId: string]
  reset: []
}>()

// Get player state to highlight currently playing track
const { state: playerState, setPlayingStatus } = usePlayer()

// Computed helpers for player state
const currentTrack = computed(() => {
  const source = playerState.currentSource
  return source?.type === 'local' ? source : null
})

const isPlaying = computed(() => playerState.isPlaying)

// Toggle play/pause
const togglePlayPause = () => {
  setPlayingStatus(!playerState.isPlaying)
}

// Keyboard navigation
const selectedIndex = ref(-1)

const handleKeydown = (e: KeyboardEvent) => {
  // Don't handle if user is in an input
  if ((e.target as HTMLElement).tagName === 'INPUT' || (e.target as HTMLElement).tagName === 'TEXTAREA') {
    return
  }
  
  if (props.tracks.length === 0) return
  
  switch (e.key) {
    case 'ArrowDown':
      e.preventDefault()
      selectedIndex.value = Math.min(selectedIndex.value + 1, props.tracks.length - 1)
      break
    case 'ArrowUp':
      e.preventDefault()
      selectedIndex.value = Math.max(selectedIndex.value - 1, 0)
      break
    case 'Enter':
      if (selectedIndex.value >= 0 && selectedIndex.value < props.tracks.length) {
        emit('track:play', props.tracks[selectedIndex.value])
      }
      break
    case ' ':
      // Space to play/pause current track
      if (currentTrack.value) {
        e.preventDefault()
        togglePlayPause()
      } else if (selectedIndex.value >= 0) {
        e.preventDefault()
        emit('track:play', props.tracks[selectedIndex.value])
      }
      break
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})

// Check if track is currently playing
const isCurrentTrack = (track: MusicFile) => {
  return currentTrack.value?.id === track.id
}

// Double-click to play
const handleRowDoubleClick = (track: MusicFile) => {
  emit('track:play', track)
}

// Click to select row
const handleRowClick = (index: number) => {
  selectedIndex.value = index
}
</script>

<template>
  <div class="music-container">
    <div v-if="loading" class="loading-skeleton">
      <div class="skeleton-header">
        <div class="skeleton-bar"></div>
        <div class="skeleton-bar"></div>
        <div class="skeleton-bar"></div>
        <div class="skeleton-bar"></div>
        <div class="skeleton-bar short"></div>
        <div class="skeleton-bar short"></div>
      </div>
      <div v-for="i in 5" :key="i" class="skeleton-row">
        <div class="skeleton-cell title">
          <div class="skeleton-bar"></div>
          <div class="skeleton-bar short"></div>
        </div>
        <div class="skeleton-cell"><div class="skeleton-bar"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar badge"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar short"></div></div>
        <div class="skeleton-cell actions">
          <div class="skeleton-circle"></div>
          <div class="skeleton-circle"></div>
        </div>
      </div>
    </div>
    <div v-else-if="tracks.length === 0" class="empty">
      <div class="empty-icon">🎵</div>
      <p>No tracks match your filters.</p>
      <button class="btn btn-outline" @click="emit('reset')">Reset filters</button>
    </div>
    <table v-else class="music-table">
      <thead>
        <tr>
          <th>Title</th>
          <th>Artist</th>
          <th>Album</th>
          <th>Genre</th>
          <th>Duration</th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr 
          v-for="(track, index) in tracks" 
          :key="track.id" 
          :class="['music-row', { 
            'playing': isCurrentTrack(track), 
            'selected': selectedIndex === index 
          }]"
          @click="handleRowClick(index)"
          @dblclick="handleRowDoubleClick(track)"
          tabindex="0"
        >
          <td>
            <div class="title-cell">
              <div v-if="isCurrentTrack(track)" class="now-playing-indicator">
                <span :class="['bar', { animating: isPlaying }]"></span>
                <span :class="['bar', { animating: isPlaying }]"></span>
                <span :class="['bar', { animating: isPlaying }]"></span>
              </div>
              <div>
                <div class="title">{{ track.title }}</div>
                <div class="timestamp">Added {{ formatDate(track.created_at) }}</div>
              </div>
            </div>
          </td>
          <td>{{ track.artist || '—' }}</td>
          <td>{{ track.album || '—' }}</td>
          <td>
            <span v-if="track.genre" class="genre-badge">{{ track.genre }}</span>
            <span 
              v-else-if="track.guessed_genre" 
              class="genre-badge muted clickable"
              @click.stop="emit('track:confirm-genre', track)"
              title="Click to confirm genre"
            >
              {{ track.guessed_genre }}*
            </span>
            <span v-else class="genre-badge empty">Untagged</span>
          </td>
          <td class="duration">
            <Icon name="clock" :size="12" />
            {{ formatDuration(track.duration) }}
          </td>
          <td class="actions">
            <button 
              class="btn-icon play-btn" 
              @click.stop="emit('track:play', track)" 
              :title="isCurrentTrack(track) && isPlaying ? 'Now Playing' : 'Play'"
            >
              <Icon :name="isCurrentTrack(track) && isPlaying ? 'pause' : 'play'" :size="16" />
            </button>
            
            <!-- Edit actions only when logged in -->
            <template v-if="canEdit">
              <div class="playlist-menu">
                <button
                  class="btn-icon"
                  @click.stop="emit('playlist:toggle', track.id)"
                  title="Add to playlist"
                >
                  <Icon name="plus" :size="16" />
                </button>
                <div v-if="playlistMenuOpen === track.id" class="menu">
                  <button
                    v-for="playlist in playlists"
                    :key="playlist.id"
                    @click.stop="emit('playlist:add', track.id, playlist.id)"
                  >
                    {{ playlist.name }}
                  </button>
                  <p v-if="!playlists.length">Create a playlist first</p>
                </div>
              </div>
              <button class="btn-icon" @click.stop="emit('track:edit', track)" title="Edit metadata"><Icon name="edit" :size="16" /></button>
              <button class="btn-icon danger" @click.stop="emit('track:delete', track)" title="Delete"><Icon name="trash" :size="16" /></button>
            </template>
          </td>
        </tr>
      </tbody>
    </table>
    
    <!-- Keyboard hints -->
    <div v-if="tracks.length > 0" class="keyboard-hints">
      <span><kbd>↑</kbd><kbd>↓</kbd> Navigate</span>
      <span><kbd>Enter</kbd> Play selected</span>
      <span><kbd>Space</kbd> Play/Pause</span>
      <span>Double-click to play</span>
    </div>
  </div>
</template>

<style scoped>
.music-container {
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
  background: var(--surface-color);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

/* Loading Skeleton */
.loading-skeleton {
  padding: 0;
}

.skeleton-header {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 100px 80px 120px;
  gap: 20px;
  padding: 16px 24px;
  background: var(--background-elevated);
  border-bottom: 2px solid var(--border-color);
}

.skeleton-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 100px 80px 120px;
  gap: 20px;
  padding: 18px 24px;
  border-bottom: 1px solid var(--border-color);
}

.skeleton-cell {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-cell.title {
  gap: 6px;
}

.skeleton-cell.actions {
  flex-direction: row;
  gap: 8px;
}

.skeleton-bar {
  height: 14px;
  background: linear-gradient(90deg, var(--border-color) 25%, var(--background-elevated) 50%, var(--border-color) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
  width: 80%;
}

.skeleton-bar.short {
  width: 50%;
  height: 12px;
}

.skeleton-bar.badge {
  width: 70px;
  height: 24px;
  border-radius: 999px;
}

.skeleton-circle {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: linear-gradient(90deg, var(--border-color) 25%, var(--background-elevated) 50%, var(--border-color) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.empty {
  padding: 80px 40px;
  text-align: center;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.empty-icon {
  font-size: 48px;
  opacity: 0.5;
}

.empty p {
  margin: 0;
  font-size: 16px;
}

.music-table {
  width: 100%;
  border-collapse: collapse;
  background: var(--surface-color);
}

.music-table th {
  text-align: left;
  padding: 16px 20px;
  background: var(--background-elevated);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-tertiary);
  font-weight: 700;
  border-bottom: 2px solid var(--border-color);
  position: sticky;
  top: 0;
  z-index: 5;
}

.music-table th:first-child {
  padding-left: 24px;
}

.music-row {
  transition: all 0.2s ease;
  cursor: pointer;
}

.music-row:hover {
  background: linear-gradient(90deg, var(--primary-glow), transparent);
}

.music-row.selected {
  background: var(--primary-glow);
  outline: 2px solid var(--primary-color);
  outline-offset: -2px;
}

.music-row.playing {
  background: linear-gradient(90deg, rgba(34, 197, 94, 0.15), transparent);
}

.music-row.playing:hover {
  background: linear-gradient(90deg, rgba(34, 197, 94, 0.2), transparent);
}

.music-row td {
  padding: 18px 20px;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-color);
  vertical-align: middle;
}

.music-row td:first-child {
  padding-left: 24px;
}

.title-cell {
  display: flex;
  align-items: center;
  gap: 12px;
}

.now-playing-indicator {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 16px;
  width: 16px;
}

.now-playing-indicator .bar {
  width: 3px;
  background: #22c55e;
  border-radius: 1px;
  height: 4px;
}

.now-playing-indicator .bar:nth-child(1) { height: 8px; }
.now-playing-indicator .bar:nth-child(2) { height: 12px; }
.now-playing-indicator .bar:nth-child(3) { height: 6px; }

.now-playing-indicator .bar.animating {
  animation: equalizer 0.5s ease infinite alternate;
}

.now-playing-indicator .bar.animating:nth-child(1) { animation-delay: 0s; }
.now-playing-indicator .bar.animating:nth-child(2) { animation-delay: 0.1s; }
.now-playing-indicator .bar.animating:nth-child(3) { animation-delay: 0.2s; }

@keyframes equalizer {
  0% { height: 4px; }
  100% { height: 14px; }
}

.title {
  font-weight: 600;
  color: var(--text-color);
  font-size: 15px;
  line-height: 1.3;
}

.music-row.playing .title {
  color: #22c55e;
}

.timestamp {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.duration {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.genre-badge {
  display: inline-flex;
  align-items: center;
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  background: var(--primary-glow);
  color: var(--primary-light);
  border: 1px solid var(--primary-color);
  transition: all 0.2s ease;
}

.genre-badge:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 8px var(--primary-glow);
}

.genre-badge.muted {
  opacity: 0.8;
  background: rgba(148, 163, 184, 0.1);
  color: var(--text-secondary);
  border-color: rgba(148, 163, 184, 0.3);
  font-style: italic;
}

.genre-badge.muted.clickable {
  cursor: pointer;
  opacity: 1;
  background: rgba(251, 191, 36, 0.15);
  color: var(--text-color);
  border-color: rgba(251, 191, 36, 0.5);
  font-style: italic;
  transition: all 0.2s ease;
}

.genre-badge.muted.clickable:hover {
  opacity: 1;
  background: rgba(251, 191, 36, 0.25);
  border-color: rgba(251, 191, 36, 0.8);
  transform: scale(1.08);
}

.genre-badge.empty {
  background: var(--background-elevated);
  color: var(--text-tertiary);
  border-color: var(--border-color);
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
}

.btn-icon {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 16px;
  color: var(--text-color);
}

.btn-icon:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  transform: scale(1.08);
  box-shadow: 0 4px 12px var(--primary-glow);
}

.btn-icon.play-btn {
  background: rgba(34, 197, 94, 0.1);
  border-color: rgba(34, 197, 94, 0.3);
  color: #22c55e;
}

.btn-icon.play-btn:hover {
  background: rgba(34, 197, 94, 0.2);
  border-color: #22c55e;
  box-shadow: 0 4px 12px rgba(34, 197, 94, 0.3);
}

.music-row.playing .btn-icon.play-btn {
  background: #22c55e;
  border-color: #22c55e;
  color: white;
}

.btn-icon.danger {
  background: rgba(239, 68, 68, 0.08);
  border-color: rgba(239, 68, 68, 0.2);
}

.btn-icon.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.5);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.2);
}

.playlist-menu {
  position: relative;
}

.playlist-menu .menu {
  position: absolute;
  top: 44px;
  right: 0;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 14px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 220px;
  z-index: 20;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}

.playlist-menu .menu button {
  background: transparent;
  border: none;
  color: var(--text-color);
  text-align: left;
  padding: 10px 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  font-weight: 500;
}

.playlist-menu .menu button:hover {
  background: var(--primary-glow);
  color: var(--primary-light);
}

.playlist-menu .menu p {
  color: var(--text-tertiary);
  font-size: 13px;
  padding: 12px 14px;
  margin: 0;
  text-align: center;
  font-style: italic;
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-color);
  padding: 12px 24px;
  border-radius: 10px;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s ease;
}

.btn-outline:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px var(--primary-glow);
}

/* Keyboard hints */
.keyboard-hints {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 24px;
  padding: 12px 20px;
  background: var(--background-elevated);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--text-tertiary);
}

.keyboard-hints span {
  display: flex;
  align-items: center;
  gap: 6px;
}

.keyboard-hints kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 22px;
  padding: 0 6px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: inherit;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
  box-shadow: 0 1px 2px rgba(0, 0, 0, 0.1);
}

/* Responsive table */
@media (max-width: 900px) {
  .music-table th,
  .music-table td {
    padding: 12px 14px;
  }
  
  .music-table th:first-child,
  .music-row td:first-child {
    padding-left: 16px;
  }
  
  .btn-icon {
    width: 34px;
    height: 34px;
  }
  
  .keyboard-hints {
    display: none;
  }
  
  .skeleton-header,
  .skeleton-row {
    grid-template-columns: 2fr 1fr 80px 80px;
  }
  
  .skeleton-header > :nth-child(3),
  .skeleton-header > :nth-child(4),
  .skeleton-row > :nth-child(3),
  .skeleton-row > :nth-child(4) {
    display: none;
  }
}
</style>
