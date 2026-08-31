<template>
  <div class="genres-tab">
    <!-- Genre Detail View -->
    <div v-if="selectedGenre" class="genre-detail">
      <header class="detail-header">
        <button class="btn-back" @click="closeGenreDetail">
          <Icon name="arrow-left" :size="20" />
          <span>Back to Genres</span>
        </button>
        
        <div class="detail-title-section">
          <div class="genre-icon large">
            <Icon name="tag" :size="48" />
          </div>
          <div class="genre-info-large">
            <div class="genre-name-row">
              <h2>{{ selectedGenre.name }}</h2>
              <div v-if="isLoggedIn" class="admin-inline-actions">
                <button 
                  class="btn-icon-small"
                  @click="handleEditGenre(selectedGenre)"
                  title="Edit genre"
                >
                  <Icon name="edit" :size="16" />
                </button>
                <button 
                  class="btn-icon-small danger"
                  @click="handleDeleteGenre(selectedGenre)"
                  title="Delete genre"
                >
                  <Icon name="trash" :size="16" />
                </button>
              </div>
            </div>
            <div class="detail-meta">
              <span class="stat-badge">
                <Icon name="disc" :size="14" />
                {{ selectedGenre.track_count || genreTracks.length }} tracks
              </span>
              <span v-if="selectedGenre.description" class="genre-desc">
                {{ selectedGenre.description }}
              </span>
            </div>
          </div>
        </div>
        <div class="detail-actions">
          <button class="btn btn-primary" @click="playAllTracks" :disabled="!genreTracks.length">
            <Icon name="play" :size="16" />
            <span>Play All</span>
          </button>
        </div>
      </header>

      <!-- Genre Tracks -->
      <div v-if="loadingTracks && genreTracks.length === 0" class="loading-tracks-container">
        <div class="loading-spinner"></div>
        <span>Loading tracks...</span>
      </div>
      
      <div v-else-if="!genreTracks.length" class="empty-genre">
        <Icon name="disc" :size="48" />
        <h3>No tracks found</h3>
        <p>No tracks are currently assigned to this genre</p>
      </div>

      <div v-else class="tracks-list-container" :class="{ 'is-loading': loadingTracks }">
        <div v-if="loadingTracks" class="loading-progress"></div>
        <div class="tracks-list-header">
          <span class="col-num">#</span>
          <span class="col-title">Title</span>
          <span class="col-artist">Artist</span>
          <span class="col-album">Album</span>
          <span class="col-meta">Meta</span>
          <span class="col-duration">
            <Icon name="clock" :size="14" />
          </span>
        </div>
        <div class="tracks-list">
          <div
            v-for="(track, index) in genreTracks"
            :key="track.id"
            :class="['track-item', { 'is-playing': isTrackPlaying(track) }]"
            @click="playTrack(track)"
          >
            <div class="track-number">
              <span v-if="isTrackPlaying(track)" class="playing-indicator">
                <span class="bar"></span>
                <span class="bar"></span>
                <span class="bar"></span>
              </span>
              <span v-else>{{ index + 1 }}</span>
            </div>
            <div class="track-info">
              <span class="track-title">{{ track.title }}</span>
            </div>
            <div class="track-artist-cell">
              <span class="artist-link">{{ track.artist || 'Unknown Artist' }}</span>
            </div>
            <div class="track-album-cell">{{ track.album || '&mdash;' }}</div>
            <div class="track-meta">
              <span v-if="track.bpm" class="bpm-tag">{{ track.bpm }}</span>
              <span v-if="track.initial_key" class="key-tag">{{ track.initial_key }}</span>
            </div>
            <div class="track-duration">{{ formatDuration(track.duration) }}</div>
          </div>
        </div>
      </div>
    </div>

    <!-- Genres List View -->
    <template v-else>
      <div class="header">
        <div class="header-content">
          <h2>Genres</h2>
          <p class="subtitle">Browse your music library by genre</p>
        </div>
        <div class="header-actions">
          <div class="search-box">
            <Icon name="search" :size="16" />
            <input 
              v-model="searchQuery" 
              type="text" 
              placeholder="Search genres..."
              @input="debouncedSearch"
            />
            <button v-if="searchQuery" class="clear-search" @click="clearSearch" title="Clear search">
              <Icon name="x" :size="14" />
            </button>
          </div>
          <div class="sort-controls">
            <select v-model="sortBy" class="sort-select">
              <option value="tracks-desc">Most Tracks</option>
              <option value="tracks-asc">Least Tracks</option>
              <option value="name-asc">Name (A-Z)</option>
              <option value="name-desc">Name (Z-A)</option>
            </select>
          </div>
          <div v-if="isLoggedIn" class="admin-actions">
            <button class="btn btn-secondary btn-small" @click="router.push('/admin/genres')">
              <Icon name="link" :size="14" />
              <span>Map Genres</span>
            </button>
            <button class="btn btn-primary btn-small" @click="handleCreateGenre">
              <Icon name="plus" :size="14" />
              <span>New Genre</span>
            </button>
          </div>
        </div>
      </div>
      <div class="results-summary">
        <span class="summary-chip">
          <Icon name="tag" :size="14" />
          {{ genres.length }} total genres
        </span>
        <span class="summary-chip">
          <Icon name="search" :size="14" />
          {{ filteredGenres.length }} shown
        </span>
        <span class="summary-chip">
          <Icon name="disc" :size="14" />
          {{ visibleTracksCount }} tracks in view
        </span>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="loading-grid">
        <div v-for="i in 12" :key="i" class="skeleton-card">
          <div class="skeleton-icon"></div>
          <div class="skeleton-info">
            <div class="skeleton skeleton-title"></div>
            <div class="skeleton skeleton-count"></div>
          </div>
        </div>
      </div>

      <!-- Genre Grid -->
      <div v-else-if="filteredGenres.length" class="genre-grid-container" :class="{ 'is-loading': loadingSilent }">
        <div v-if="loadingSilent" class="loading-progress"></div>
        <div class="genre-grid">
          <div
            v-for="(genre, index) in filteredGenres"
            :key="genre.name"
            class="genre-card"
            @click="selectGenre(genre)"
            @contextmenu.prevent="openContextMenu(genre, $event)"
            @keydown.enter.prevent="selectGenre(genre)"
            @keydown.space.prevent="selectGenre(genre)"
            role="button"
            tabindex="0"
          >
            <span class="genre-rank">#{{ index + 1 }}</span>
            <div class="genre-card-icon">
              <Icon name="tag" :size="20" />
            </div>
            <div class="genre-card-content">
              <h3>{{ genre.name }}</h3>
              <span class="track-count">{{ genre.track_count || 0 }} tracks</span>
            </div>
            <div class="genre-card-actions">
              <button 
                v-if="isLoggedIn" 
                class="btn-icon-tiny" 
                @click.stop="openContextMenu(genre, $event)"
              >
                <Icon name="more-vertical" :size="14" />
              </button>
              <Icon name="chevron-right" :size="16" class="view-icon" />
            </div>
          </div>
        </div>
      </div>

      <!-- Empty State -->
      <div v-else class="empty-state">
        <div class="empty-icon">
          <Icon name="tag" :size="48" />
        </div>
        <h3>No genres found</h3>
        <p v-if="searchQuery">No genres match "{{ searchQuery }}"</p>
        <p v-else>Upload some music to see your genres here</p>
        <button v-if="searchQuery" class="btn btn-secondary mt-4" @click="clearSearch">
          Clear search
        </button>
      </div>
    </template>

    <GenreContextMenu 
      ref="contextMenu" 
      :is-admin="isLoggedIn"
      @create="handleCreateGenre"
      @edit="handleEditGenre"
      @delete="handleDeleteGenre"
      @open="selectGenre"
      @play-all="playGenre"
      @queue:add-genre="addGenreToQueue"
    />

    <GenreEditModal 
      :is-open="editModalOpen"
      :genre="selectedGenreForEdit"
      @close="editModalOpen = false"
      @saved="fetchGenres"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { useRouter } from 'vue-router'
import { musicAPI } from '../../api/music'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import { useAuth } from '../../composables/useAuth'
import { useConfirm } from '../../composables/useConfirm'
import { useDjStore } from '../../stores/djStore'
import type { Genre, MusicFile } from '../../types'
import { formatDuration } from '../../utils/musicFormatters'
import Icon from '../../shared/components/Icons.vue'
import GenreContextMenu from './GenreContextMenu.vue'
import GenreEditModal from './GenreEditModal.vue'

const genres = ref<Genre[]>([])
const genreTracks = ref<MusicFile[]>([])
const loading = ref(false)
const loadingSilent = ref(false)
const loadingTracks = ref(false)
const selectedGenre = ref<Genre | null>(null)
const searchQuery = ref('')
const sortBy = ref<'tracks-desc' | 'tracks-asc' | 'name-asc' | 'name-desc'>('tracks-desc')

const router = useRouter()
const { playLocalTrack, state: playerState } = usePlayer()
const { error, success } = useToast()
const { isLoggedIn } = useAuth()
const { confirm } = useConfirm()
const djStore = useDjStore()

const contextMenu = ref<InstanceType<typeof GenreContextMenu> | null>(null)
const editModalOpen = ref(false)
const selectedGenreForEdit = ref<Genre | null>(null)

// Filter genres based on search
const filteredGenres = computed(() => {
  let result = genres.value
  
  // Apply search filter
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(genre => 
      genre.name.toLowerCase().includes(query)
    )
  }
  
  // Apply sorting
  const sorted = [...result]
  switch (sortBy.value) {
    case 'tracks-desc':
      return sorted.sort((a, b) => (b.track_count || 0) - (a.track_count || 0))
    case 'tracks-asc':
      return sorted.sort((a, b) => (a.track_count || 0) - (b.track_count || 0))
    case 'name-asc':
      return sorted.sort((a, b) => a.name.localeCompare(b.name))
    case 'name-desc':
      return sorted.sort((a, b) => b.name.localeCompare(a.name))
    default:
      return sorted
  }
})

const visibleTracksCount = computed(() => {
  return filteredGenres.value.reduce((sum, genre) => sum + (genre.track_count || 0), 0)
})

const isTrackPlaying = (track: MusicFile) => {
  return playerState.currentSource?.type === 'local' &&
         playerState.currentSource?.id === track.id &&
         playerState.isPlaying
}

const debouncedSearch = () => {
  // Filter is reactive, no action needed
}

const clearSearch = () => {
  searchQuery.value = ''
}

const fetchGenres = async () => {
  const isSilent = genres.value.length > 0
  if (!isSilent) {
    loading.value = true
  }
  loadingSilent.value = true

  try {
    const response = await musicAPI.listGenres()
    genres.value = response.data
    
    // Update selected genre if it was open
    if (selectedGenre.value) {
      const updated = genres.value.find(g => g.id === selectedGenre.value?.id || g.name === selectedGenre.value?.name)
      if (updated) {
        selectedGenre.value = updated
      }
    }
  } catch (err: any) {
    console.error('Error fetching genres:', err)
    error('Failed to load genres', err?.response?.data?.error || err?.message)
  } finally {
    loading.value = false
    loadingSilent.value = false
  }
}

const openContextMenu = (genre: Genre, event: MouseEvent) => {
  if (contextMenu.value) {
    contextMenu.value.open(genre, event)
  }
}

const handleCreateGenre = () => {
  selectedGenreForEdit.value = null
  editModalOpen.value = true
}

const handleEditGenre = (genre: Genre) => {
  selectedGenreForEdit.value = genre
  editModalOpen.value = true
}

const handleDeleteGenre = async (genre: Genre) => {
  const ok = await confirm({
    title: 'Delete Genre',
    message: `Are you sure you want to delete the genre "${genre.name}"? This will also delete any aliases. This does NOT delete the music files, but they will no longer be associated with this canonical genre.`,
    confirmText: 'Delete',
    variant: 'danger'
  })

  if (ok && genre.id) {
    try {
      await musicAPI.deleteGenre(genre.id)
      success('Success', `Genre "${genre.name}" deleted`)
      await fetchGenres()
      if (selectedGenre.value?.id === genre.id || selectedGenre.value?.name === genre.name) {
        selectedGenre.value = null
      }
    } catch (err: any) {
      error('Error', err.response?.data?.error || 'Failed to delete genre')
    }
  }
}

const selectGenre = async (genre: Genre) => {
  selectedGenre.value = genre
  loadingTracks.value = true
  
  try {
    const response = await musicAPI.getMusicFiles({ genre: genre.name, limit: 100 })
    genreTracks.value = response.data
  } catch (err: any) {
    console.error('Error fetching tracks:', err)
    error('Failed to load tracks', err?.response?.data?.error || err?.message)
    genreTracks.value = []
  } finally {
    loadingTracks.value = false
  }
}

const closeGenreDetail = () => {
  selectedGenre.value = null
  genreTracks.value = []
}

const playTrack = (track: MusicFile) => {
  playLocalTrack({
    id: track.id,
    title: track.title,
    artist: track.artist,
    bpm: track.bpm,
    initial_key: track.initial_key,
    duration: track.duration,
  })
}

const playAllTracks = () => {
  if (genreTracks.value.length > 0) {
    playTrack(genreTracks.value[0])
  }
}

const playGenre = async (genre: Genre) => {
  // Open the genre and then play all once tracks are loaded
  await selectGenre(genre)
  if (genreTracks.value.length) playAllTracks()
}

const addGenreToQueue = async (genre: Genre) => {
  try {
    // Fetch all tracks for this genre
    const res = await musicAPI.getMusicFiles({ genre: genre.name, limit: 1000, sort: 'title', order: 'asc' })
    const tracks = res.data as MusicFile[]
    if (tracks.length > 0) {
      djStore.addTracksToQueue(tracks)
      success('Added to DJ Queue', `${tracks.length} tracks from "${genre.name}" added to the DJ queue`)
    } else {
      error('No Tracks', `No tracks found for genre "${genre.name}"`)
    }
  } catch (e) {
    error('Failed', 'Could not fetch tracks for this genre')
  }
}

onMounted(() => {
  fetchGenres()
})
</script>

<style scoped>
.genres-tab {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 16px;
  flex-wrap: wrap;
}

.header-content h2 {
  margin: 0 0 4px 0;
  font-size: 1.5rem;
  font-weight: 700;
}

.subtitle {
  margin: 0;
  color: var(--text-secondary);
  font-size: 0.9rem;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  align-items: center;
}

.results-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.summary-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  font-size: 0.8rem;
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
  background: var(--surface-muted);
  border-radius: 9999px;
  padding: 6px 10px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  min-width: 250px;
  transition: all 0.2s ease;
}

.search-box:focus-within {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--accent-muted, rgba(139, 92, 246, 0.15));
}

.search-box input {
  flex: 1;
  border: none;
  background: none;
  color: var(--text-color);
  font-size: 14px;
  outline: none;
}

.sort-select {
  padding: 10px 14px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-color);
  font-weight: 500;
  font-size: 14px;
  cursor: pointer;
}

.sort-controls {
  min-width: 170px;
}

.clear-search {
  width: 24px;
  height: 24px;
  border-radius: 9999px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  color: var(--text-tertiary);
}

.clear-search:hover {
  background: var(--surface-muted);
  color: var(--text-color);
}

.admin-actions {
  display: flex;
  gap: 8px;
}

/* Genres Grid */
.genre-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.genre-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.2s ease;
  cursor: pointer;
  position: relative;
}

.genre-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  background: var(--surface-hover);
}

.genre-card:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
  border-color: var(--primary-color);
}

.genre-rank {
  position: absolute;
  top: 10px;
  right: 12px;
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

.genre-card-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--primary-glow, rgba(139, 92, 246, 0.15));
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color);
  flex-shrink: 0;
}

.genre-card-content {
  flex: 1;
  min-width: 0;
}

.genre-card-content h3 {
  margin: 0 0 2px 0;
  font-size: 1rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-count {
  font-size: 0.85rem;
  color: var(--text-tertiary);
}

.genre-card-actions {
  display: flex;
  align-items: center;
  gap: 4px;
}

.view-icon {
  color: var(--text-tertiary);
  opacity: 0;
  transition: all 0.2s ease;
}

.genre-card:hover .view-icon {
  opacity: 1;
  transform: translateX(4px);
}

/* Detail View */
.genre-detail {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.detail-header {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.btn-back {
  display: flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-weight: 500;
  cursor: pointer;
  padding: 4px 0;
  width: fit-content;
}

.btn-back:hover {
  color: var(--text-color);
}

.btn-back:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
  border-radius: 8px;
}

.detail-title-section {
  display: flex;
  align-items: center;
  gap: 24px;
}

.genre-icon.large {
  width: 96px;
  height: 96px;
  border-radius: 24px;
  background: var(--primary-glow);
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color);
}

.genre-info-large {
  flex: 1;
}

.genre-name-row {
  display: flex;
  align-items: center;
  gap: 16px;
}

.genre-name-row h2 {
  margin: 0;
  font-size: 2rem;
  font-weight: 700;
}

.admin-inline-actions {
  display: flex;
  gap: 8px;
}

.detail-meta {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-top: 8px;
}

.stat-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-weight: 500;
  font-size: 0.95rem;
}

.genre-desc {
  color: var(--text-tertiary);
  font-size: 0.95rem;
  max-width: 600px;
}

/* Tracks Table */
.tracks-list-container {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
  position: relative;
}

.tracks-list-header {
  display: grid;
  grid-template-columns: 48px 2fr 1.5fr 1.5fr 100px 70px;
  gap: 16px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-tertiary);
  font-size: 0.8rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  background: var(--surface-muted);
}

.track-item {
  display: grid;
  grid-template-columns: 48px 2fr 1.5fr 1.5fr 100px 70px;
  gap: 16px;
  align-items: center;
  padding: 10px 20px;
  border-bottom: 1px solid var(--border-color);
  cursor: pointer;
  transition: all 0.2s ease;
}

.track-item:last-child {
  border-bottom: none;
}

.track-item:hover {
  background: var(--surface-hover);
}

.track-number {
  text-align: center;
  color: var(--text-tertiary);
}

.track-title {
  font-weight: 600;
  color: var(--text-color);
}

.artist-link {
  color: var(--text-secondary);
}

.track-album-cell {
  color: var(--text-tertiary);
  font-size: 0.9rem;
}

.track-meta {
  display: flex;
  gap: 6px;
}

.bpm-tag, .key-tag {
  font-size: 0.7rem;
  padding: 2px 6px;
  background: var(--surface-muted);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-tertiary);
  font-weight: 600;
}

.track-duration {
  text-align: right;
  color: var(--text-tertiary);
  font-family: monospace;
}

/* Utils */
.btn-icon-small, .btn-icon-tiny {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s;
}

.btn-icon-small { padding: 6px; }
.btn-icon-tiny { padding: 4px; }

.btn-icon-small:hover {
  background: var(--surface-muted);
  color: var(--primary-color);
}

.btn-icon-small.danger:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
}

.loading-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.empty-state {
  border: 1px dashed var(--border-color);
  border-radius: 16px;
  padding: 40px 24px;
  text-align: center;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.empty-state h3 {
  margin: 0;
}

.empty-state p {
  margin: 0;
}

.skeleton-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
}

.skeleton-icon {
  width: 44px;
  height: 44px;
  border-radius: 12px;
  background: var(--surface-muted);
}

.skeleton-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton {
  background: var(--surface-muted);
  border-radius: 4px;
}

.skeleton-title { height: 16px; width: 60%; }
.skeleton-count { height: 12px; width: 30%; }

.loading-progress {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 2px;
  background: var(--primary-color);
  animation: loading-bar 1.5s infinite ease-in-out;
  transform-origin: 0% 50%;
}

@keyframes loading-bar {
  0% { transform: scaleX(0); left: 0; }
  50% { transform: scaleX(0.5); left: 25%; }
  100% { transform: scaleX(0); left: 100%; }
}

/* Animations */
.playing-indicator {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 2px;
  height: 14px;
}

.playing-indicator .bar {
  width: 2px;
  background: var(--primary-color);
  border-radius: 1px;
  animation: bounce 0.5s ease infinite alternate;
}

.playing-indicator .bar:nth-child(1) { height: 40%; animation-delay: 0s; }
.playing-indicator .bar:nth-child(2) { height: 100%; animation-delay: 0.15s; }
.playing-indicator .bar:nth-child(3) { height: 60%; animation-delay: 0.3s; }

@keyframes bounce {
  0% { height: 30%; }
  100% { height: 100%; }
}

/* Responsive */
@media (max-width: 1024px) {
  .tracks-list-header, .track-item {
    grid-template-columns: 48px 2fr 1.5fr 1fr 60px;
  }
  .track-album-cell, .col-album { display: none; }
}

@media (max-width: 768px) {
  .header {
    flex-direction: column;
    align-items: stretch;
  }

  .header-actions {
    flex-direction: column;
  }

  .results-summary {
    gap: 6px;
  }

  .summary-chip {
    font-size: 0.75rem;
  }

  .search-box {
    min-width: unset;
    width: 100%;
  }

  .sort-controls {
    min-width: unset;
    width: 100%;
  }

  .admin-actions {
    width: 100%;
  }

  .admin-actions .btn {
    flex: 1;
  }

  .genre-grid {
    grid-template-columns: 1fr;
  }

  .detail-title-section {
    flex-direction: column;
    text-align: center;
    gap: 16px;
  }
  
  .genre-icon.large { margin: 0 auto; }
  
  .genre-name-row { justify-content: center; }
  
  .detail-meta { flex-direction: column; gap: 8px; }
  
  .tracks-list-header { display: none; }
  
  .track-item {
    grid-template-columns: 40px 1fr 60px;
    padding: 12px 16px;
  }
  
  .track-artist-cell, .track-meta, .col-meta, .col-artist { display: none; }
}
</style>
