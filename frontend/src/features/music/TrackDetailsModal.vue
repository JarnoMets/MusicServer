<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { MusicFile } from '../../types'
import { formatDuration, formatDate } from '../../utils/musicFormatters'
import Icon from '../../shared/components/Icons.vue'
import { musicAPI } from '../../api/music'

interface Props {
  isOpen: boolean
  track?: MusicFile
}

const props = withDefaults(defineProps<Props>(), {
  isOpen: false,
})

const emit = defineEmits<{
  close: []
  'navigate:artist': [artist: string]
  'navigate:genre': [genre: string]
  'navigate:album': [album: string]
  edit: [track: MusicFile]
  'playlist:add': [trackId: string, playlistId: string]
  'playlist:remove': [trackId: string, playlistId: string]
}>()

interface PlaylistInfo {
  id: string
  name: string
  track_count: number
}

const trackPlaylists = ref<PlaylistInfo[]>([])
const allPlaylists = ref<PlaylistInfo[]>([])
const loadingPlaylists = ref(false)
const showPlaylistPicker = ref(false)

const effectiveGenre = computed(() => {
  if (!props.track) return null
  return props.track.genre || props.track.guessed_genre || null
})

const isGuessedGenre = computed(() => {
  return props.track && !props.track.genre && props.track.guessed_genre
})

const releaseYear = computed(() => {
  if (!props.track?.release_date) return null
  return new Date(props.track.release_date).getFullYear()
})

watch(() => props.isOpen, async (open) => {
  if (open && props.track) {
    loadingPlaylists.value = true
    try {
      const [trackPl, allPl] = await Promise.all([
        musicAPI.getTrackPlaylists(props.track.id),
        musicAPI.getPlaylists(),
      ])
      trackPlaylists.value = trackPl.data
      allPlaylists.value = allPl.data
    } catch (err) {
      console.warn('Failed to load playlists', err)
    } finally {
      loadingPlaylists.value = false
    }
  } else {
    showPlaylistPicker.value = false
  }
})

const availablePlaylists = computed(() => {
  const inIds = new Set(trackPlaylists.value.map(p => p.id))
  return allPlaylists.value.filter(p => !inIds.has(p.id))
})

const handleAddToPlaylist = async (playlistId: string) => {
  if (!props.track) return
  emit('playlist:add', props.track.id, playlistId)
  // Optimistically update
  const added = allPlaylists.value.find(p => p.id === playlistId)
  if (added) trackPlaylists.value.push(added)
  showPlaylistPicker.value = false
}

const handleRemoveFromPlaylist = async (playlistId: string) => {
  if (!props.track) return
  emit('playlist:remove', props.track.id, playlistId)
  trackPlaylists.value = trackPlaylists.value.filter(p => p.id !== playlistId)
}

const isDetectingBpm = ref(false)

const handleDetectBpm = async () => {
  if (!props.track || isDetectingBpm.value) return
  
  isDetectingBpm.value = true
  try {
    const response = await musicAPI.detectBpm(props.track.id)
    if (response.data && response.data.bpm) {
      if (props.track) {
          props.track.bpm = response.data.bpm
      }
    }
  } catch (err) {
    console.error('Failed to detect BPM:', err)
  } finally {
    isDetectingBpm.value = false
  }
}
</script>

<template>
  <div v-if="isOpen && track" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content">
      <div class="modal-header">
        <h3>Track Details</h3>
        <button class="close-btn" @click="emit('close')">
          <Icon name="x" :size="20" />
        </button>
      </div>

      <div class="modal-body">
        <!-- Track title -->
        <div class="detail-hero">
          <div class="hero-icon">
            <Icon name="music" :size="32" />
          </div>
          <div class="hero-text">
            <h2 class="track-title">{{ track.title }}</h2>
            <p class="track-subtitle" v-if="track.artist">
              by
              <button class="link-btn" @click="emit('navigate:artist', track.artist!)">
                {{ track.artist }}
              </button>
            </p>
          </div>
        </div>

        <!-- Details grid -->
        <div class="details-grid">
          <div class="detail-item" v-if="track.album">
            <span class="detail-label">Album</span>
            <button class="detail-value link-btn" @click="emit('navigate:album', track.album!)">
              {{ track.album }}
            </button>
          </div>

          <div class="detail-item">
            <span class="detail-label">Genre</span>
            <span v-if="effectiveGenre" class="detail-value">
              <button
                :class="['genre-tag', { guessed: isGuessedGenre }]"
                @click="emit('navigate:genre', effectiveGenre!)"
              >
                {{ effectiveGenre }}
                <span v-if="isGuessedGenre" class="guessed-marker">*</span>
              </button>
            </span>
            <span v-else class="detail-value muted">Not set</span>
          </div>

          <div class="detail-item">
            <span class="detail-label">Duration</span>
            <span class="detail-value">
              <Icon name="clock" :size="14" />
              {{ formatDuration(track.duration) }}
            </span>
          </div>


          <div class="detail-item">
            <span class="detail-label">BPM</span>
            <span class="detail-value">
              <span v-if="track.bpm && track.bpm > 0">{{ track.bpm.toFixed(1) }} BPM</span>
              <span v-else class="muted">Unknown</span>
              
              <button 
                class="detect-btn" 
                @click="handleDetectBpm" 
                :disabled="isDetectingBpm"
                title="Detect BPM"
              >
                <Icon v-if="isDetectingBpm" name="loader" class="animate-spin" :size="12" />
                <Icon v-else name="activity" :size="12" />
                <span class="detect-text">Detect</span>
              </button>
            </span>
          </div>

          <div class="detail-item" v-if="track.initial_key && track.initial_key !== 'NONE'">
            <span class="detail-label">Key</span>
            <span class="detail-value">{{ track.initial_key }}</span>
          </div>

          <div class="detail-item" v-if="releaseYear">
            <span class="detail-label">Release Date</span>
            <span class="detail-value">{{ new Date(track.release_date!).toLocaleDateString() }}</span>
          </div>

          <div class="detail-item">
            <span class="detail-label">Added</span>
            <span class="detail-value">{{ formatDate(track.created_at) }}</span>
          </div>

          <div class="detail-item">
            <span class="detail-label">Last Updated</span>
            <span class="detail-value">{{ formatDate(track.updated_at) }}</span>
          </div>
        </div>

        <!-- Playlists section -->
        <div class="playlists-section">
          <div class="section-header">
            <h4>Playlists</h4>
            <button class="btn-sm" @click="showPlaylistPicker = !showPlaylistPicker">
              <Icon name="plus" :size="14" />
              Add
            </button>
          </div>

          <div v-if="loadingPlaylists" class="loading-playlists">Loading&hellip;</div>

          <div v-else-if="trackPlaylists.length" class="playlist-tags">
            <span v-for="pl in trackPlaylists" :key="pl.id" class="playlist-tag">
              <Icon name="list" :size="12" />
              {{ pl.name }}
              <button class="remove-tag" @click="handleRemoveFromPlaylist(pl.id)" title="Remove from playlist">
                <Icon name="x" :size="10" />
              </button>
            </span>
          </div>
          <p v-else class="muted-text">Not in any playlist</p>

          <!-- Playlist picker -->
          <div v-if="showPlaylistPicker && availablePlaylists.length" class="playlist-picker">
            <button
              v-for="pl in availablePlaylists"
              :key="pl.id"
              class="picker-item"
              @click="handleAddToPlaylist(pl.id)"
            >
              <Icon name="plus" :size="14" />
              {{ pl.name }}
            </button>
          </div>
          <p v-else-if="showPlaylistPicker && !availablePlaylists.length" class="muted-text">
            Already in all playlists
          </p>
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-outline" @click="emit('close')">Close</button>
        <button class="btn btn-primary" @click="emit('edit', track!)">
          <Icon name="edit" :size="16" />
          Edit Track
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from { opacity: 0; }
  to { opacity: 1; }
}

.modal-content {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4);
  max-width: 560px;
  width: 92%;
  max-height: 85vh;
  overflow-y: auto;
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from { transform: translateY(20px); opacity: 0; }
  to { transform: translateY(0); opacity: 1; }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--background-elevated);
  color: var(--text-color);
}

.modal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.detail-hero {
  display: flex;
  align-items: center;
  gap: 16px;
}

.hero-icon {
  width: 64px;
  height: 64px;
  background: linear-gradient(135deg, var(--primary-color), var(--accent-color));
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  flex-shrink: 0;
}

.hero-text {
  min-width: 0;
}

.track-title {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  line-height: 1.3;
  word-break: break-word;
}

.track-subtitle {
  margin: 4px 0 0;
  color: var(--text-secondary);
  font-size: 14px;
}

.link-btn {
  background: none;
  border: none;
  color: var(--primary-light);
  cursor: pointer;
  font-size: inherit;
  font-weight: 600;
  padding: 0;
  text-decoration: underline;
  text-decoration-style: dotted;
  text-underline-offset: 3px;
  transition: all 0.2s;
}

.link-btn:hover {
  color: var(--primary-color);
  text-decoration-style: solid;
}

.details-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.detail-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.detail-label {
  font-size: 11px;
  font-weight: 700;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.detail-value {
  font-size: 14px;
  color: var(--text-color);
  display: flex;
  align-items: center;
  gap: 6px;
}

.detail-value.muted {
  color: var(--text-tertiary);
  font-style: italic;
}

.genre-tag {
  display: inline-flex;
  align-items: center;
  gap: 2px;
  padding: 4px 12px;
  border-radius: 999px;
  font-size: 13px;
  font-weight: 600;
  background: var(--primary-glow);
  color: var(--primary-light);
  border: 1px solid var(--primary-color);
  cursor: pointer;
  transition: all 0.2s;
}

.genre-tag:hover {
  transform: scale(1.05);
}

.genre-tag.guessed {
  opacity: 0.8;
  border-style: dashed;
  font-style: italic;
}

.guessed-marker {
  font-size: 11px;
}

.playlists-section {
  border-top: 1px solid var(--border-color);
  padding-top: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.section-header h4 {
  margin: 0;
  font-size: 14px;
  font-weight: 700;
}

.btn-sm {
  display: flex;
  align-items: center;
  gap: 4px;
  padding: 6px 12px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-secondary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-sm:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
}

.playlist-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.playlist-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 13px;
  font-weight: 500;
  color: var(--text-color);
}

.remove-tag {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  transition: all 0.2s;
}

.remove-tag:hover {
  color: #ef4444;
  background: rgba(239, 68, 68, 0.1);
}

.playlist-picker {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-top: 8px;
  padding: 8px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  max-height: 200px;
  overflow-y: auto;
}

.picker-item {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 12px;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--text-color);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  transition: all 0.2s;
}

.picker-item:hover {
  background: var(--primary-glow);
  color: var(--primary-light);
}

.loading-playlists, .muted-text {
  font-size: 13px;
  color: var(--text-tertiary);
  font-style: italic;
}

.modal-footer {
  display: flex;
  gap: 12px;
  padding: 20px 24px;
  border-top: 1px solid var(--border-color);
  justify-content: flex-end;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 10px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-color);
}

.btn-outline:hover {
  background: var(--surface-hover);
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover {
  background: var(--primary-light);
  box-shadow: 0 4px 12px var(--primary-glow);
  transform: translateY(-1px);
}

.detect-btn {
  background: none;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px 6px;
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  font-weight: 600;
  margin-left: 8px;
  transition: all 0.2s;
}

.detect-btn:hover:not(:disabled) {
  background: var(--primary-glow);
  color: var(--primary-light);
  border-color: var(--primary-color);
}

.detect-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.animate-spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

@media (max-width: 600px) {
  .details-grid {
    grid-template-columns: 1fr;
  }

  .modal-footer {
    flex-direction: column-reverse;
  }

  .btn {
    width: 100%;
    justify-content: center;
  }
}
</style>
