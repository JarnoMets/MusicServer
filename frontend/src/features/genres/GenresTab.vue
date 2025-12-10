<template>
  <div class="genres-tab">
    <div class="header">
      <div class="header-content">
        <h2>Browse by Genre</h2>
        <p class="subtitle">Explore your music collection organized by genre</p>
      </div>
      <div class="header-stats" v-if="!loading && genres.length">
        <span class="stat">{{ genres.length }} genres</span>
        <span class="stat">{{ totalTracks }} tracks</span>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="loading-grid">
      <div v-for="i in 8" :key="i" class="skeleton-card">
        <div class="skeleton skeleton-icon"></div>
        <div class="skeleton skeleton-title"></div>
        <div class="skeleton skeleton-count"></div>
      </div>
    </div>

    <!-- Genre Grid -->
    <div v-else-if="genres.length" class="genre-grid">
      <button
        v-for="genre in genres"
        :key="genre.name"
        :class="['genre-card', { active: selectedGenre === genre.name }]"
        @click="selectGenre(genre.name)"
      >
        <div class="genre-icon">
          <Icon name="tag" :size="24" />
        </div>
        <div class="genre-info">
          <h3 class="genre-name">{{ genre.name }}</h3>
          <span class="genre-count">{{ genre.track_count || 0 }} tracks</span>
        </div>
        <div class="genre-indicator">
          <Icon v-if="selectedGenre === genre.name" name="check" :size="16" />
          <Icon v-else name="chevron-right" :size="16" />
        </div>
      </button>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <div class="empty-icon">
        <Icon name="tag" :size="64" />
      </div>
      <h3>No genres found</h3>
      <p>Upload some music and tag it with genres to see them here</p>
    </div>

    <!-- Selected Genre Tracks -->
    <Transition name="slide">
      <div v-if="selectedGenre" class="genre-tracks">
        <div class="tracks-header">
          <div class="tracks-header-left">
            <button class="btn-back" @click="selectedGenre = null">
              <Icon name="chevron-left" :size="16" /> Back
            </button>
            <h3>{{ selectedGenre }}</h3>
            <span class="track-count">{{ genreTracks.length }} tracks</span>
          </div>
          <div class="tracks-header-right">
            <button class="btn btn-primary" @click="playAllTracks" :disabled="!genreTracks.length">
              <Icon name="play" :size="16" /> Play All
            </button>
          </div>
        </div>

        <div v-if="loadingTracks" class="loading-tracks">
          <div v-for="i in 5" :key="i" class="skeleton-track">
            <div class="skeleton skeleton-play-btn"></div>
            <div class="skeleton-track-info">
              <div class="skeleton skeleton-track-title"></div>
              <div class="skeleton skeleton-track-artist"></div>
            </div>
            <div class="skeleton skeleton-duration"></div>
          </div>
        </div>

        <div v-else-if="genreTracks.length" class="tracks-list">
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
              <span class="track-artist">{{ track.artist || 'Unknown Artist' }}</span>
            </div>
            <div class="track-album">{{ track.album || '—' }}</div>
            <div class="track-duration">{{ formatDuration(track.duration) }}</div>
          </div>
        </div>

        <div v-else class="empty-tracks">
          No tracks found for this genre.
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { musicAPI } from '../../api/music'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import Icon from '../../shared/components/Icons.vue'

interface Genre {
  id: string
  name: string
  description?: string
  track_count?: number
}

interface Track {
  id: string
  title: string
  artist?: string
  album?: string
  duration?: number
  genre?: string
}

const genres = ref<Genre[]>([])
const genreTracks = ref<Track[]>([])
const loading = ref(false)
const loadingTracks = ref(false)
const selectedGenre = ref<string | null>(null)

const { playLocalTrack, state: playerState } = usePlayer()
const { error } = useToast()

const totalTracks = computed(() => {
  return genres.value.reduce((sum, g) => sum + (g.track_count || 0), 0)
})

const isTrackPlaying = (track: Track) => {
  return playerState.currentSource?.type === 'local' &&
         playerState.currentSource?.id === track.id &&
         playerState.isPlaying
}

const fetchGenres = async () => {
  loading.value = true
  try {
    const response = await musicAPI.listGenres()
    genres.value = response.data
  } catch (err: any) {
    console.error('Error fetching genres:', err)
    error('Failed to load genres', err?.response?.data?.error || err?.message)
  } finally {
    loading.value = false
  }
}

const selectGenre = async (genreName: string) => {
  if (selectedGenre.value === genreName) {
    selectedGenre.value = null
    return
  }
  
  selectedGenre.value = genreName
  loadingTracks.value = true
  
  try {
    const response = await musicAPI.getMusicFiles({ genre: genreName, limit: 100 })
    genreTracks.value = response.data
  } catch (err: any) {
    console.error('Error fetching tracks:', err)
    error('Failed to load tracks', err?.response?.data?.error || err?.message)
    genreTracks.value = []
  } finally {
    loadingTracks.value = false
  }
}

const playTrack = (track: Track) => {
  playLocalTrack({
    id: track.id,
    title: track.title,
    artist: track.artist,
  })
}

const playAllTracks = () => {
  if (genreTracks.value.length > 0) {
    playTrack(genreTracks.value[0])
  }
}

const formatDuration = (ms?: number) => {
  if (!ms) return '—'
  const totalSeconds = Math.floor(ms / 1000)
  const mins = Math.floor(totalSeconds / 60)
  const secs = totalSeconds % 60
  return `${mins}:${secs.toString().padStart(2, '0')}`
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

.header-stats {
  display: flex;
  gap: 16px;
}

.stat {
  padding: 6px 14px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-full);
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 500;
}

/* Loading */
.loading-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(200px, 1fr));
  gap: 16px;
}

.skeleton-card {
  padding: 24px;
  background: var(--surface-color);
  border-radius: var(--radius-lg);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
}

.skeleton {
  background: linear-gradient(
    90deg,
    var(--surface-muted) 25%,
    var(--surface-hover) 50%,
    var(--surface-muted) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: var(--radius-sm);
}

.skeleton-icon {
  width: 48px;
  height: 48px;
  border-radius: var(--radius-md);
}

.skeleton-title {
  width: 80%;
  height: 20px;
}

.skeleton-count {
  width: 50%;
  height: 14px;
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

/* Genre Grid */
.genre-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  gap: 16px;
}

.genre-card {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 18px 20px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: left;
}

.genre-card:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
  transform: translateY(-2px);
  box-shadow: var(--shadow-md);
}

.genre-card.active {
  border-color: var(--primary-color);
  background: linear-gradient(135deg, var(--accent-muted), var(--surface-color));
}

.genre-icon {
  width: 48px;
  height: 48px;
  background: var(--accent-muted);
  border-radius: var(--radius-md);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--primary-color);
}

.genre-info {
  flex: 1;
  min-width: 0;
}

.genre-name {
  margin: 0 0 4px 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.genre-count {
  font-size: 13px;
  color: var(--text-tertiary);
}

.genre-indicator {
  color: var(--text-tertiary);
  font-size: 14px;
  transition: all var(--transition-base);
}

.genre-card:hover .genre-indicator,
.genre-card.active .genre-indicator {
  color: var(--primary-color);
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  border: 2px dashed var(--border-color);
  border-radius: var(--radius-xl);
  background: var(--surface-muted);
}

.empty-icon {
  margin-bottom: 16px;
  color: var(--text-tertiary);
}

.empty-state h3 {
  margin: 0 0 8px 0;
  font-size: 1.25rem;
  color: var(--text-color);
}

.empty-state p {
  margin: 0;
  color: var(--text-secondary);
}

/* Selected Genre Tracks */
.genre-tracks {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-xl);
  overflow: hidden;
}

.tracks-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
  background: var(--surface-muted);
}

.tracks-header-left {
  display: flex;
  align-items: center;
  gap: 16px;
}

.tracks-header-left h3 {
  margin: 0;
  font-size: 1.125rem;
}

.track-count {
  color: var(--text-tertiary);
  font-size: 14px;
}

.btn-back {
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 14px;
  cursor: pointer;
  padding: 6px 12px;
  border-radius: var(--radius-md);
  transition: all var(--transition-base);
}

.btn-back:hover {
  background: var(--surface-hover);
  color: var(--text-color);
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border: none;
  border-radius: var(--radius-md);
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all var(--transition-base);
}

.btn-primary {
  background: linear-gradient(135deg, var(--primary-color), var(--accent-color));
  color: white;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-1px);
  box-shadow: 0 4px 15px var(--accent-muted);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

/* Tracks List */
.tracks-list {
  max-height: 500px;
  overflow-y: auto;
}

.track-item {
  display: grid;
  grid-template-columns: 48px 1fr 150px 60px;
  gap: 16px;
  align-items: center;
  padding: 12px 24px;
  cursor: pointer;
  transition: all var(--transition-base);
  border-bottom: 1px solid var(--border-color);
}

.track-item:last-child {
  border-bottom: none;
}

.track-item:hover {
  background: var(--surface-hover);
}

.track-item.is-playing {
  background: linear-gradient(90deg, var(--accent-muted), transparent);
}

.track-number {
  text-align: center;
  color: var(--text-tertiary);
  font-size: 14px;
  font-variant-numeric: tabular-nums;
}

.playing-indicator {
  display: flex;
  align-items: flex-end;
  justify-content: center;
  gap: 2px;
  height: 16px;
}

.playing-indicator .bar {
  width: 3px;
  background: var(--primary-color);
  border-radius: 2px;
  animation: soundBar 0.5s ease infinite alternate;
}

.playing-indicator .bar:nth-child(1) { height: 40%; animation-delay: 0s; }
.playing-indicator .bar:nth-child(2) { height: 100%; animation-delay: 0.15s; }
.playing-indicator .bar:nth-child(3) { height: 60%; animation-delay: 0.3s; }

@keyframes soundBar {
  0% { height: 20%; }
  100% { height: 100%; }
}

.track-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 0;
}

.track-title {
  font-weight: 600;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-artist {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-album {
  font-size: 13px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-duration {
  font-size: 13px;
  color: var(--text-tertiary);
  text-align: right;
  font-variant-numeric: tabular-nums;
}

/* Loading tracks */
.loading-tracks {
  padding: 8px 0;
}

.skeleton-track {
  display: grid;
  grid-template-columns: 48px 1fr 60px;
  gap: 16px;
  align-items: center;
  padding: 12px 24px;
}

.skeleton-play-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  margin: 0 auto;
}

.skeleton-track-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-track-title {
  height: 16px;
  width: 70%;
}

.skeleton-track-artist {
  height: 14px;
  width: 40%;
}

.skeleton-duration {
  height: 14px;
  width: 40px;
  margin-left: auto;
}

.empty-tracks {
  padding: 40px;
  text-align: center;
  color: var(--text-tertiary);
}

/* Slide animation */
.slide-enter-active,
.slide-leave-active {
  transition: all 0.3s ease;
}

.slide-enter-from {
  opacity: 0;
  transform: translateY(20px);
}

.slide-leave-to {
  opacity: 0;
  transform: translateY(-20px);
}

/* Responsive */
@media (max-width: 768px) {
  .track-item {
    grid-template-columns: 40px 1fr 50px;
  }

  .track-album {
    display: none;
  }
}

@media (max-width: 480px) {
  .genre-grid {
    grid-template-columns: 1fr;
  }

  .tracks-header {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
  }

  .tracks-header-left {
    flex-wrap: wrap;
  }
}
</style>
