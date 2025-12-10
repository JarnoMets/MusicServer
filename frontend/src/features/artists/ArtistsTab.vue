<template>
  <div class="artists-tab">
    <!-- Artist Detail View -->
    <div v-if="selectedArtist" class="artist-detail">
      <header class="detail-header">
        <button class="btn-back" @click="closeArtistDetail">
          <Icon name="arrow-left" :size="20" />
          <span>Back to Artists</span>
        </button>
        <div class="detail-title-section">
          <div class="artist-avatar large">
            <span>{{ getInitials(selectedArtist.name) }}</span>
          </div>
          <div>
            <div class="artist-name-row">
              <h2>{{ selectedArtist.name }}</h2>
              <button 
                v-if="canEdit" 
                class="btn-icon-small"
                @click="openRenameEditor"
                title="Rename artist"
              >
                <Icon name="edit" :size="14" />
              </button>
            </div>
            <div class="detail-meta">
              <span 
                v-if="selectedArtist.genre" 
                class="genre-tag"
                :class="{ editable: canEdit }"
                @click="canEdit && openGenreEditor()"
              >
                <Icon name="music" :size="14" />
                {{ selectedArtist.genre }}
                <Icon v-if="canEdit" name="edit" :size="12" class="edit-icon" />
              </span>
              <button 
                v-else-if="canEdit" 
                class="btn btn-small btn-secondary"
                @click="openGenreEditor()"
              >
                <Icon name="plus" :size="14" />
                Set Genre
              </button>
              <span v-else class="no-genre">No genre set</span>
              <span class="separator">•</span>
              <span>{{ artistTracks.length }} {{ artistTracks.length === 1 ? 'track' : 'tracks' }}</span>
            </div>
          </div>
        </div>
      </header>

      <!-- Artist Tracks -->
      <div v-if="loadingDetail" class="loading-tracks">
        <div class="loading-spinner"></div>
        <span>Loading tracks...</span>
      </div>
      <div v-else-if="!artistTracks.length" class="empty-artist">
        <Icon name="disc" :size="48" />
        <h3>No tracks found</h3>
        <p>This artist has no tracks in your library</p>
      </div>
      <div v-else class="artist-tracks">
        <!-- Group tracks by category -->
        <section v-if="originalTracks.length" class="track-section">
          <h4 class="section-title">
            <Icon name="mic" :size="16" />
            Original Tracks ({{ originalTracks.length }})
          </h4>
          <div class="tracks-list">
            <div 
              v-for="(track, index) in originalTracks" 
              :key="track.id" 
              class="track-row"
            >
              <span class="track-number">{{ index + 1 }}</span>
              <button class="track-play" @click="playTrack(track)">
                <Icon name="play" :size="16" />
              </button>
              <div class="track-info">
                <span class="track-title">{{ track.title }}</span>
                <span class="track-album">{{ track.album || 'Unknown Album' }}</span>
              </div>
              <span class="track-genre">{{ track.genre || track.guessed_genre || '—' }}</span>
              <span class="track-duration">{{ formatDuration(track.duration) }}</span>
            </div>
          </div>
        </section>

        <section v-if="featuredTracks.length" class="track-section">
          <h4 class="section-title">
            <Icon name="users" :size="16" />
            Featured On ({{ featuredTracks.length }})
          </h4>
          <div class="tracks-list">
            <div 
              v-for="(track, index) in featuredTracks" 
              :key="track.id" 
              class="track-row featured"
            >
              <span class="track-number">{{ index + 1 }}</span>
              <button class="track-play" @click="playTrack(track)">
                <Icon name="play" :size="16" />
              </button>
              <div class="track-info">
                <span class="track-title">{{ track.title }}</span>
                <span class="track-artist">{{ track.artist || 'Unknown Artist' }}</span>
              </div>
              <span class="track-genre">{{ track.genre || track.guessed_genre || '—' }}</span>
              <span class="track-duration">{{ formatDuration(track.duration) }}</span>
            </div>
          </div>
        </section>

        <section v-if="remixTracks.length" class="track-section">
          <h4 class="section-title">
            <Icon name="refresh-cw" :size="16" />
            Remixes ({{ remixTracks.length }})
          </h4>
          <div class="tracks-list">
            <div 
              v-for="(track, index) in remixTracks" 
              :key="track.id" 
              class="track-row remix"
            >
              <span class="track-number">{{ index + 1 }}</span>
              <button class="track-play" @click="playTrack(track)">
                <Icon name="play" :size="16" />
              </button>
              <div class="track-info">
                <span class="track-title">{{ track.title }}</span>
                <span class="track-artist">{{ track.artist || 'Unknown Artist' }}</span>
              </div>
              <span class="track-genre">{{ track.genre || track.guessed_genre || '—' }}</span>
              <span class="track-duration">{{ formatDuration(track.duration) }}</span>
            </div>
          </div>
        </section>
      </div>
    </div>

    <!-- Artists List View -->
    <template v-else>
      <div class="header">
        <div class="header-content">
          <h2>Artists</h2>
          <p class="subtitle">Browse your music library by artist</p>
        </div>
        <div class="header-actions">
          <button 
            v-if="canEdit" 
            class="btn btn-secondary btn-small"
            @click="reprocessAllArtists"
            :disabled="isReprocessing"
            title="Re-scan all tracks to extract additional artists from titles"
          >
            <Icon name="refresh-cw" :size="14" :class="{ spinning: isReprocessing }" />
            {{ isReprocessing ? 'Processing...' : 'Reprocess Artists' }}
          </button>
          <div class="search-box">
            <Icon name="search" :size="16" />
            <input 
              v-model="searchQuery" 
              type="text" 
              placeholder="Search artists..."
              @input="debouncedSearch"
            />
            <button v-if="searchQuery" class="clear-search" @click="clearSearch">
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
        </div>
      </div>

      <!-- Loading State -->
      <div v-if="loading" class="loading-grid">
        <div v-for="i in 8" :key="i" class="skeleton-card">
          <div class="skeleton-avatar"></div>
          <div class="skeleton-info">
            <div class="skeleton skeleton-name"></div>
            <div class="skeleton skeleton-meta"></div>
          </div>
        </div>
      </div>

      <!-- Artists Grid -->
      <div v-else-if="filteredArtists.length" class="artists-grid">
        <article 
          v-for="artist in filteredArtists" 
          :key="artist.name" 
          class="artist-card"
          @click="openArtistDetail(artist)"
        >
          <div class="artist-avatar">
            <span>{{ getInitials(artist.name) }}</span>
          </div>
          <div class="artist-info">
            <h3>{{ artist.name }}</h3>
            <div class="artist-meta">
              <span v-if="artist.genre" class="genre-badge">{{ artist.genre }}</span>
              <span v-else class="no-genre-badge">No genre</span>
              <span class="track-count">{{ artist.song_count }} {{ artist.song_count === 1 ? 'track' : 'tracks' }}</span>
            </div>
          </div>
          <Icon name="chevron-right" :size="18" class="view-icon" />
        </article>
      </div>

      <!-- Empty State -->
      <div v-else-if="searchQuery" class="empty-state">
        <Icon name="search" :size="48" />
        <h3>No artists found</h3>
        <p>No artists match "{{ searchQuery }}"</p>
        <button class="btn btn-secondary" @click="clearSearch">
          Clear search
        </button>
      </div>
      <div v-else class="empty-state">
        <Icon name="users" :size="48" />
        <h3>No artists yet</h3>
        <p>Upload some music to see your artists here</p>
      </div>
    </template>

    <!-- Genre Editor Modal -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showGenreEditor" class="modal-overlay" @click="closeGenreEditor">
          <div class="modal" @click.stop>
            <div class="modal-header">
              <h3>Set Artist Genre</h3>
              <button class="modal-close" @click="closeGenreEditor">
                <Icon name="x" :size="20" />
              </button>
            </div>
            <div class="modal-body">
              <p class="modal-description">
                Set a genre for <strong>{{ selectedArtist?.name }}</strong>. This will be applied to all their future tracks.
              </p>
              <div class="form-group">
                <label for="artist-genre">Genre</label>
                <div class="genre-input-wrapper">
                  <input 
                    id="artist-genre"
                    v-model="genreInput"
                    type="text" 
                    placeholder="e.g., Electronic, Hip Hop, Rock"
                    list="genre-suggestions"
                    @keydown.enter="saveArtistGenre"
                  />
                  <datalist id="genre-suggestions">
                    <option v-for="genre in availableGenres" :key="genre" :value="genre" />
                  </datalist>
                </div>
              </div>
            </div>
            <div class="modal-footer">
              <button class="btn btn-secondary" @click="closeGenreEditor">Cancel</button>
              <button 
                class="btn btn-primary" 
                :disabled="!genreInput.trim() || savingGenre"
                @click="saveArtistGenre"
              >
                {{ savingGenre ? 'Saving...' : 'Save Genre' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Rename Artist Modal -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showRenameEditor" class="modal-overlay" @click="closeRenameEditor">
          <div class="modal" @click.stop>
            <div class="modal-header">
              <h3>Rename Artist</h3>
              <button class="modal-close" @click="closeRenameEditor">
                <Icon name="x" :size="20" />
              </button>
            </div>
            <div class="modal-body">
              <p class="modal-description">
                Rename <strong>{{ selectedArtist?.name }}</strong>. All tracks by this artist will be updated.
                <br><br>
                <em class="merge-note">Tip: To merge with another artist, enter their exact name.</em>
              </p>
              <div class="form-group">
                <label for="artist-name">New Name</label>
                <div class="genre-input-wrapper">
                  <input 
                    id="artist-name"
                    v-model="renameInput"
                    type="text" 
                    placeholder="Enter new artist name"
                    list="artist-suggestions"
                    @keydown.enter="saveArtistRename"
                  />
                  <datalist id="artist-suggestions">
                    <option 
                      v-for="artist in artists.filter(a => a.name !== selectedArtist?.name)" 
                      :key="artist.name" 
                      :value="artist.name" 
                    />
                  </datalist>
                </div>
              </div>
            </div>
            <div class="modal-footer">
              <button class="btn btn-secondary" @click="closeRenameEditor">Cancel</button>
              <button 
                class="btn btn-primary" 
                :disabled="!renameInput.trim() || renameInput.trim() === selectedArtist?.name || savingRename"
                @click="saveArtistRename"
              >
                {{ savingRename ? 'Saving...' : 'Rename' }}
              </button>
            </div>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { musicAPI } from '../../api/music'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import { useAuth } from '../../composables/useAuth'
import { formatDuration } from '../../utils/musicFormatters'
import Icon from '../../shared/components/Icons.vue'

interface Props {
  canEdit?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  canEdit: false
})

// Use auth for fallback
const { isLoggedIn } = useAuth()
const canEdit = computed(() => props.canEdit || isLoggedIn.value)

interface ArtistSummary {
  name: string
  genre: string | null
  song_count: number
}

interface MusicFile {
  id: string
  title: string
  artist?: string | null
  album?: string | null
  genre?: string | null
  guessed_genre?: string | null
  duration?: number | null
  file_path: string
}

const artists = ref<ArtistSummary[]>([])
const selectedArtist = ref<ArtistSummary | null>(null)
const artistTracks = ref<MusicFile[]>([])
const loading = ref(false)
const loadingDetail = ref(false)
const searchQuery = ref('')
const sortBy = ref<'tracks-desc' | 'tracks-asc' | 'name-asc' | 'name-desc'>('tracks-desc')
const showGenreEditor = ref(false)
const genreInput = ref('')
const savingGenre = ref(false)
const availableGenres = ref<string[]>([])
const showRenameEditor = ref(false)
const renameInput = ref('')
const savingRename = ref(false)
const isReprocessing = ref(false)

const { playLocalTrack } = usePlayer()
const { success, error } = useToast()

// Debounce search
let searchTimeout: ReturnType<typeof setTimeout> | null = null
const debouncedSearch = () => {
  if (searchTimeout) clearTimeout(searchTimeout)
  searchTimeout = setTimeout(() => {
    // Filter is reactive, no action needed
  }, 200)
}

const clearSearch = () => {
  searchQuery.value = ''
}

// Filter artists based on search
const filteredArtists = computed(() => {
  if (!searchQuery.value.trim()) return sortedArtists.value
  const query = searchQuery.value.toLowerCase()
  return sortedArtists.value.filter(artist => 
    artist.name.toLowerCase().includes(query) ||
    (artist.genre && artist.genre.toLowerCase().includes(query))
  )
})

// Sort artists based on selected sort option
const sortedArtists = computed(() => {
  const list = [...artists.value]
  
  switch (sortBy.value) {
    case 'tracks-desc':
      return list.sort((a, b) => b.song_count - a.song_count)
    case 'tracks-asc':
      return list.sort((a, b) => a.song_count - b.song_count)
    case 'name-asc':
      return list.sort((a, b) => a.name.localeCompare(b.name))
    case 'name-desc':
      return list.sort((a, b) => b.name.localeCompare(a.name))
    default:
      return list
  }
})

// Categorize tracks
const originalTracks = computed(() => {
  if (!selectedArtist.value) return []
  const artistLower = selectedArtist.value.name.toLowerCase()
  return artistTracks.value.filter(track => {
    const trackArtist = (track.artist || '').toLowerCase()
    const titleLower = (track.title || '').toLowerCase()
    // Original track: artist matches exactly and title doesn't indicate remix/feature
    return trackArtist === artistLower && 
           !titleLower.includes('remix') &&
           !titleLower.includes('feat.') &&
           !titleLower.includes('ft.')
  })
})

const featuredTracks = computed(() => {
  if (!selectedArtist.value) return []
  const artistLower = selectedArtist.value.name.toLowerCase()
  return artistTracks.value.filter(track => {
    const trackArtist = (track.artist || '').toLowerCase()
    const titleLower = (track.title || '').toLowerCase()
    // Featured: different artist but title mentions this artist (feat/ft)
    return trackArtist !== artistLower && 
           (titleLower.includes(`feat. ${artistLower}`) || 
            titleLower.includes(`ft. ${artistLower}`) ||
            titleLower.includes(`featuring ${artistLower}`) ||
            titleLower.includes(`(${artistLower})`) ||
            titleLower.includes(`& ${artistLower}`) ||
            titleLower.includes(`x ${artistLower}`))
  })
})

const remixTracks = computed(() => {
  if (!selectedArtist.value) return []
  const artistLower = selectedArtist.value.name.toLowerCase()
  return artistTracks.value.filter(track => {
    const trackArtist = (track.artist || '').toLowerCase()
    const titleLower = (track.title || '').toLowerCase()
    // Remix: title contains "[artist] remix" or "remix" and artist matches
    if (titleLower.includes(`${artistLower} remix`)) return true
    if (trackArtist === artistLower && titleLower.includes('remix')) return true
    return false
  })
})

const getInitials = (name: string) => {
  return name
    .split(' ')
    .map(word => word[0])
    .join('')
    .substring(0, 2)
    .toUpperCase()
}

const fetchArtists = async () => {
  try {
    loading.value = true
    const response = await musicAPI.getArtists()
    artists.value = response.data
  } catch (err: any) {
    console.error('Error fetching artists:', err)
    error('Failed to load artists', err?.response?.data?.error || err?.message)
  } finally {
    loading.value = false
  }
}

const fetchGenres = async () => {
  try {
    const response = await musicAPI.listGenres()
    availableGenres.value = response.data.map((g: { name: string }) => g.name)
  } catch (err) {
    console.error('Error fetching genres:', err)
  }
}

const openArtistDetail = async (artist: ArtistSummary) => {
  selectedArtist.value = artist
  loadingDetail.value = true
  try {
    const response = await musicAPI.getArtistMusic(artist.name)
    artistTracks.value = response.data
  } catch (err: any) {
    console.error('Error fetching artist tracks:', err)
    error('Failed to load tracks', err?.response?.data?.error || err?.message)
  } finally {
    loadingDetail.value = false
  }
}

const closeArtistDetail = () => {
  selectedArtist.value = null
  artistTracks.value = []
  // Refresh list in case genres were changed
  fetchArtists()
}

const playTrack = (track: MusicFile) => {
  playLocalTrack({
    id: track.id,
    title: track.title,
    artist: track.artist,
  })
}

const openGenreEditor = () => {
  genreInput.value = selectedArtist.value?.genre || ''
  showGenreEditor.value = true
}

const closeGenreEditor = () => {
  showGenreEditor.value = false
  genreInput.value = ''
}

const saveArtistGenre = async () => {
  if (!selectedArtist.value || !genreInput.value.trim()) return
  
  savingGenre.value = true
  try {
    await musicAPI.setArtistGenre(selectedArtist.value.name, genreInput.value.trim())
    
    // Update local state
    selectedArtist.value.genre = genreInput.value.trim()
    
    // Update in the list too
    const idx = artists.value.findIndex(a => a.name === selectedArtist.value?.name)
    if (idx !== -1) {
      artists.value[idx].genre = genreInput.value.trim()
    }
    
    success('Genre updated', `Genre set to "${genreInput.value.trim()}" for ${selectedArtist.value.name}`)
    closeGenreEditor()
  } catch (err: any) {
    console.error('Error saving genre:', err)
    error('Failed to save genre', err?.response?.data?.error || err?.message)
  } finally {
    savingGenre.value = false
  }
}

const openRenameEditor = () => {
  renameInput.value = selectedArtist.value?.name || ''
  showRenameEditor.value = true
}

const closeRenameEditor = () => {
  showRenameEditor.value = false
  renameInput.value = ''
}

const saveArtistRename = async () => {
  if (!selectedArtist.value || !renameInput.value.trim()) return
  if (renameInput.value.trim() === selectedArtist.value.name) return
  
  savingRename.value = true
  const oldName = selectedArtist.value.name
  const newName = renameInput.value.trim()
  
  try {
    const response = await musicAPI.renameArtist(oldName, newName)
    const result = response.data
    
    if (result.was_merge) {
      success('Artists merged', `Merged "${oldName}" into "${newName}" (${result.tracks_updated} tracks updated)`)
    } else {
      success('Artist renamed', `Renamed "${oldName}" to "${newName}" (${result.tracks_updated} tracks updated)`)
    }
    
    closeRenameEditor()
    // Close detail and refresh list
    selectedArtist.value = null
    await fetchArtists()
  } catch (err: any) {
    console.error('Error renaming artist:', err)
    error('Failed to rename artist', err?.response?.data?.error || err?.message)
  } finally {
    savingRename.value = false
  }
}

// Reprocess all artists from track metadata
const reprocessAllArtists = async () => {
  isReprocessing.value = true
  try {
    const response = await musicAPI.reprocessArtists()
    const result = response.data
    success(
      'Artists reprocessed', 
      result.message || `Added ${result.artists_added || 0} new artists`
    )
    // Refresh the artist list
    await fetchArtists()
  } catch (err: any) {
    console.error('Error reprocessing artists:', err)
    error('Failed to reprocess artists', err?.response?.data?.error || err?.message)
  } finally {
    isReprocessing.value = false
  }
}

onMounted(() => {
  fetchArtists()
  fetchGenres()
})
</script>

<style scoped>
.artists-tab {
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

.search-box input::placeholder {
  color: var(--text-tertiary);
}

.clear-search {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 2px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  transition: all 0.2s ease;
}

.clear-search:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.sort-controls {
  display: flex;
  align-items: center;
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
  transition: all 0.2s ease;
}

.sort-select:hover {
  border-color: var(--border-hover);
  background: var(--surface-hover);
}

.sort-select:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--accent-muted, rgba(139, 92, 246, 0.15));
}

/* Artist Detail View */
.artist-detail {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.btn-back {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  background: none;
  border: none;
  color: var(--text-secondary);
  font-size: 14px;
  font-weight: 500;
  cursor: pointer;
  padding: 8px 0;
  transition: all 0.2s ease;
}

.btn-back:hover {
  color: var(--primary-color);
}

.detail-header {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.detail-title-section {
  display: flex;
  align-items: flex-start;
  gap: 20px;
}

.artist-avatar {
  width: 48px;
  height: 48px;
  background: linear-gradient(135deg, var(--primary-color), var(--accent-color));
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  font-weight: 700;
  font-size: 16px;
  flex-shrink: 0;
}

.artist-avatar.large {
  width: 100px;
  height: 100px;
  font-size: 32px;
  border-radius: 50%;
}

.artist-name-row {
  display: flex;
  align-items: center;
  gap: 10px;
}

.artist-name-row h2 {
  margin: 0;
  font-size: 1.75rem;
  font-weight: 700;
}

.btn-icon-small {
  width: 28px;
  height: 28px;
  border-radius: 6px;
  border: 1px solid var(--border-color);
  background: var(--surface-color);
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  opacity: 0.7;
}

.btn-icon-small:hover {
  opacity: 1;
  background: var(--surface-hover);
  border-color: var(--primary-color);
  color: var(--primary-color);
}

.detail-title-section h2 {
  margin: 0 0 8px 0;
  font-size: 1.75rem;
  font-weight: 700;
}

.detail-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  color: var(--text-tertiary);
  font-size: 14px;
  margin-top: 8px;
}

.separator {
  color: var(--text-tertiary);
}

.genre-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--accent-muted, rgba(139, 92, 246, 0.15));
  color: var(--primary-color);
  border-radius: 20px;
  font-weight: 500;
  font-size: 13px;
}

.genre-tag.editable {
  cursor: pointer;
  transition: all 0.2s ease;
}

.genre-tag.editable:hover {
  background: var(--accent-muted, rgba(139, 92, 246, 0.25));
}

.genre-tag .edit-icon {
  opacity: 0;
  margin-left: 4px;
  transition: opacity 0.2s ease;
}

.genre-tag.editable:hover .edit-icon {
  opacity: 1;
}

.no-genre {
  color: var(--text-tertiary);
  font-style: italic;
}

/* Loading tracks */
.loading-tracks {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-secondary);
}

.loading-spinner {
  width: 24px;
  height: 24px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Empty artist */
.empty-artist {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-tertiary);
}

.empty-artist h3 {
  margin: 16px 0 8px;
  color: var(--text-color);
}

.empty-artist p {
  margin: 0;
  color: var(--text-secondary);
}

/* Track sections */
.track-section {
  margin-bottom: 32px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 10px;
  margin: 0 0 16px 0;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.tracks-list {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
  background: var(--surface-color);
}

.track-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 14px 20px;
  border-bottom: 1px solid var(--border-color);
  transition: all 0.2s ease;
}

.track-row:last-child {
  border-bottom: none;
}

.track-row:hover {
  background: var(--surface-hover);
}

.track-row.featured {
  border-left: 3px solid var(--info-color, #3b82f6);
}

.track-row.remix {
  border-left: 3px solid var(--warning-color, #f59e0b);
}

.track-number {
  width: 28px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 14px;
  font-weight: 500;
}

.track-play {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: none;
  background: var(--primary-color);
  color: white;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  flex-shrink: 0;
}

.track-play:hover {
  transform: scale(1.1);
  box-shadow: 0 4px 12px var(--primary-glow, rgba(139, 92, 246, 0.3));
}

.track-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.track-title {
  font-weight: 500;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-artist,
.track-album {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-genre {
  color: var(--text-tertiary);
  font-size: 13px;
  min-width: 100px;
  text-align: center;
}

.track-duration {
  color: var(--text-tertiary);
  font-size: 13px;
  font-family: monospace;
  min-width: 50px;
  text-align: right;
}

/* Buttons */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border: none;
  border-radius: 8px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-small {
  padding: 6px 12px;
  font-size: 13px;
}

.btn-primary {
  background: linear-gradient(135deg, var(--primary-color), var(--accent-color));
  color: white;
  box-shadow: 0 4px 15px var(--accent-muted, rgba(139, 92, 246, 0.2));
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px var(--accent-muted, rgba(139, 92, 246, 0.3));
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--surface-color);
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

/* Loading */
.loading-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.skeleton-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
}

.skeleton-avatar {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  background: linear-gradient(
    90deg,
    var(--surface-muted) 25%,
    var(--surface-hover) 50%,
    var(--surface-muted) 75%
  );
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

.skeleton-info {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
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
  border-radius: 4px;
}

.skeleton-name {
  height: 20px;
  width: 70%;
}

.skeleton-meta {
  height: 16px;
  width: 50%;
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

/* Artists Grid */
.artists-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 16px;
}

.artist-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.artist-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.artist-info {
  flex: 1;
  min-width: 0;
}

.artist-info h3 {
  margin: 0 0 6px 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.artist-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 13px;
}

.genre-badge {
  padding: 3px 10px;
  background: var(--accent-muted, rgba(139, 92, 246, 0.15));
  color: var(--primary-color);
  border-radius: 12px;
  font-weight: 500;
  font-size: 12px;
}

.no-genre-badge {
  padding: 3px 10px;
  background: var(--surface-muted);
  color: var(--text-tertiary);
  border-radius: 12px;
  font-size: 12px;
}

.track-count {
  color: var(--text-tertiary);
}

.view-icon {
  color: var(--text-tertiary);
  transition: all 0.2s ease;
}

.artist-card:hover .view-icon {
  color: var(--primary-color);
  transform: translateX(4px);
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal {
  background: var(--background-elevated, var(--surface-color));
  border: 1px solid var(--border-color);
  border-radius: 20px;
  width: 100%;
  max-width: 440px;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
  background: var(--surface-muted);
}

.modal-header h3 {
  margin: 0;
  font-size: 1.125rem;
}

.modal-close {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.modal-close:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.modal-body {
  padding: 24px;
}

.modal-description {
  margin: 0 0 20px 0;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.5;
}

.merge-note {
  color: var(--text-tertiary);
  font-size: 13px;
}

.modal-footer {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  background: var(--surface-muted);
}

.form-group {
  margin-bottom: 0;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  font-size: 13px;
  color: var(--text-secondary);
}

.genre-input-wrapper input {
  width: 100%;
  padding: 12px 14px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-color);
  font-family: inherit;
  font-size: 14px;
  transition: all 0.2s ease;
}

.genre-input-wrapper input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--accent-muted, rgba(139, 92, 246, 0.15));
}

/* Modal animation */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal,
.modal-leave-to .modal {
  transform: scale(0.95) translateY(20px);
}

/* Empty State */
.empty-state {
  text-align: center;
  padding: 60px 20px;
  border: 2px dashed var(--border-color);
  border-radius: 20px;
  background: var(--surface-muted);
}

.empty-state svg {
  color: var(--text-tertiary);
  margin-bottom: 16px;
}

.empty-state h3 {
  margin: 0 0 8px 0;
  font-size: 1.25rem;
  color: var(--text-color);
}

.empty-state p {
  margin: 0 0 24px 0;
  color: var(--text-secondary);
}

/* Spinning animation for refresh icon */
@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.spinning {
  animation: spin 1s linear infinite;
}

/* Responsive */
@media (max-width: 640px) {
  .header {
    flex-direction: column;
    align-items: stretch;
  }

  .header-actions {
    flex-direction: column;
    gap: 10px;
  }

  .search-box {
    min-width: unset;
    width: 100%;
  }

  .sort-controls {
    width: 100%;
  }

  .sort-select {
    width: 100%;
  }

  .artists-grid {
    grid-template-columns: 1fr;
  }

  .detail-title-section {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .detail-meta {
    flex-wrap: wrap;
    justify-content: center;
  }

  .track-row {
    flex-wrap: wrap;
    gap: 12px;
  }

  .track-number {
    display: none;
  }

  .track-genre {
    display: none;
  }
}
</style>
