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
              <span class="separator">&bull;</span>
              <span>{{ artistTracks.length }} {{ artistTracks.length === 1 ? 'track' : 'tracks' }}</span>
              <button 
                v-if="artistTracks.length > 0"
                class="btn btn-small btn-queue"
                @click="addArtistToQueue"
              >
                <Icon name="plus-circle" :size="14" />
                Add All to DJ Queue
              </button>
            </div>
          </div>
        </div>
      </header>

      <!-- Artist Tracks -->
      <div v-if="loadingDetail && artistTracks.length === 0" class="loading-tracks">
        <div class="loading-spinner"></div>
        <span>Loading tracks...</span>
      </div>
      <div v-else-if="!artistTracks.length" class="empty-artist">
        <Icon name="disc" :size="48" />
        <h3>No tracks found</h3>
        <p>This artist has no tracks in your library</p>
      </div>
      <div v-else class="artist-tracks-container" :class="{ 'is-loading': loadingDetail }">
        <div v-if="loadingDetail" class="loading-progress"></div>
        <div class="artist-tracks">
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
                @contextmenu.prevent="handleContextMenu(track, $event)"
              >
                <span class="track-number">{{ index + 1 }}</span>
                <button class="track-play" @click="playTrack(track)">
                  <Icon name="play" :size="16" />
                </button>
                <div class="track-info">
                  <span class="track-title">{{ track.title }}</span>
                  <span class="track-album">{{ track.album || 'Unknown Album' }}</span>
                </div>
                <span class="track-genre">{{ track.genre_name || '&mdash;' }}</span>
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
                @contextmenu.prevent="handleContextMenu(track, $event)"
              >
                <span class="track-number">{{ index + 1 }}</span>
                <button class="track-play" @click="playTrack(track)">
                  <Icon name="play" :size="16" />
                </button>
                <div class="track-info">
                  <span class="track-title">{{ track.title }}</span>
                  <span class="track-artist">{{ track.artist || 'Unknown Artist' }}</span>
                </div>
                <span class="track-genre">{{ track.genre_name || '&mdash;' }}</span>
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
                @contextmenu.prevent="handleContextMenu(track, $event)"
              >
                <span class="track-number">{{ index + 1 }}</span>
                <button class="track-play" @click="playTrack(track)">
                  <Icon name="play" :size="16" />
                </button>
                <div class="track-info">
                  <span class="track-title">{{ track.title }}</span>
                  <span class="track-artist">{{ track.artist || 'Unknown Artist' }}</span>
                </div>
                <span class="track-genre">{{ track.genre_name || '&mdash;' }}</span>
                <span class="track-duration">{{ formatDuration(track.duration) }}</span>
              </div>
            </div>
          </section>
        </div>
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
                      v-for="artist in store.artists.filter(a => a.name !== selectedArtist?.name)" 
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

    <!-- Context Menu -->
    <TrackContextMenu
      ref="contextMenuRef"
      :is-admin="canEdit"
      :playlists="playlists"
      :genres="canonicalGenres"
      :date-suggestion="releaseDateSuggestion"
      :looking-up-date="lookingUpReleaseDate"
      @play="playTrack"
      @details="handleTrackDetails"
      @edit="openEdit"
      @delete="deleteTrack"
      @confirm-genre="handleConfirmGenre"
      @playlist:add="(trackId, playlistId) => addTrackToPlaylist(trackId, playlistId)"
      @set-genre="(track, genre) => setTrackGenre(track, genre)"
      @lookup-date="handleLookupDate"
      @apply-date="handleApplyDate"
      @queue:add="handleAddToQueue"
    />

    <!-- Track Details Modal -->
    <TrackDetailsModal
      :is-open="detailsModal.isOpen"
      :track="detailsModal.track || undefined"
      @close="detailsModal.isOpen = false"
      @edit="openEdit"
      @playlist:add="addTrackToPlaylist"
    />

    <!-- Edit Drawer -->
    <EditTrackDrawer
      v-if="canEdit"
      :edit-state="editState"
      :saving="editState.saving"
      :genres="canonicalGenres"
      :release-date-suggestion="releaseDateSuggestion"
      :looking-up-release-date="lookingUpReleaseDate"
      @update:title="v => editState.form.title = v"
      @update:artist="v => editState.form.artist = v"
      @update:album="v => editState.form.album = v"
      @update:genre="v => editState.form.genre = v"
      @update:genre-id="v => editState.form.genre_id = v"
      @update:release_date="v => editState.form.release_date = v"
      @apply-suggestion="applySuggestedReleaseDate"
      @save="saveEdit"
      @close="closeEdit"
    />

    <!-- Confirm Genre Modal -->
    <ConfirmGenreModal
      v-if="canEdit"
      :is-open="confirmGenreModal.isOpen"
      :track="confirmGenreModal.track || undefined"
      @close="confirmGenreModal.isOpen = false"
      @confirm="handleGenreConfirmed"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { musicAPI } from '../../api/music'
import { useMusicStore } from '../../stores/musicStore'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import { useAuth } from '../../composables/useAuth'
import { useMusicData } from '../../composables/useMusicData'
import { useTrackEdit } from '../../composables/useTrackEdit'
import { useTrackActions } from '../../composables/useTrackActions'
import { useDjStore } from '../../stores/djStore'
import { formatDuration } from '../../utils/musicFormatters'
import type { ArtistSummary, MusicFile } from '../../types'
import Icon from '../../shared/components/Icons.vue'
import TrackContextMenu from '../music/TrackContextMenu.vue'
import TrackDetailsModal from '../music/TrackDetailsModal.vue'
import EditTrackDrawer from '../music/EditTrackDrawer.vue'
import ConfirmGenreModal from '../music/ConfirmGenreModal.vue'

interface Props {
  canEdit?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  canEdit: false
})

// Use auth for fallback
const { isLoggedIn } = useAuth()
const canEdit = computed(() => props.canEdit || isLoggedIn.value)

const store = useMusicStore()
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
const djStore = useDjStore()

// Integrated tools from MusicTab
const { playlists, canonicalGenres, fetchMusic } = useMusicData()
const {
  editState,
  releaseDateSuggestion,
  lookingUpReleaseDate,
  openEdit,
  closeEdit,
  saveEdit,
  applySuggestedReleaseDate,
  lookupReleaseDate,
  quickSetDate,
} = useTrackEdit(() => {
  if (selectedArtist.value) openArtistDetail(selectedArtist.value)
  fetchMusic()
})

const { deleteTrack, addTrackToPlaylist, setTrackGenre } = useTrackActions(() => {
  if (selectedArtist.value) openArtistDetail(selectedArtist.value)
  fetchMusic()
})

const contextMenuRef = ref<InstanceType<typeof TrackContextMenu> | null>(null)
const detailsModal = ref({
  isOpen: false,
  track: null as any | null,
})
const confirmGenreModal = ref({
  isOpen: false,
  track: null as any | null,
})

const handleContextMenu = (track: any, event: MouseEvent) => {
  releaseDateSuggestion.value = null
  contextMenuRef.value?.open(track, event)
}

const handleTrackDetails = (track: any) => {
  detailsModal.value = { isOpen: true, track }
}

const handleLookupDate = (track: any) => {
  lookupReleaseDate(track.title, track.artist || undefined)
}

const handleApplyDate = async (track: any, date: string) => {
  await quickSetDate(track.id, date)
  releaseDateSuggestion.value = null
}

const handleConfirmGenre = (track: any) => {
  confirmGenreModal.value = { isOpen: true, track }
}

const handleGenreConfirmed = () => {
  confirmGenreModal.value.isOpen = false
  if (selectedArtist.value) openArtistDetail(selectedArtist.value)
  fetchMusic()
}

// DJ Queue handler
const handleAddToQueue = (track: MusicFile) => {
  djStore.addToQueue(track)
  success('Added to DJ Queue', `"${track.title}" added to the DJ queue`)
}

const addArtistToQueue = () => {
  if (artistTracks.value.length === 0) return
  djStore.addTracksToQueue(artistTracks.value)
  success('Added to DJ Queue', `${artistTracks.value.length} tracks added to the DJ queue`)
}

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
  const list = [...store.artists]
  
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

// artist loading is handled by the Pinia store (store.refreshArtists)

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
  store.refreshArtists()
}

const playTrack = (track: MusicFile) => {
  playLocalTrack({
    id: track.id,
    title: track.title,
    artist: track.artist,
    duration: track.duration,
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

  // Look up the genre_id from the canonical genres list
  const matchedGenre = canonicalGenres.value.find(
    g => g.name.toLowerCase() === genreInput.value.trim().toLowerCase()
  )
  if (!matchedGenre) {
    error('Genre not found', `"${genreInput.value.trim()}" is not a canonical genre. Please select from the list.`)
    return
  }

  savingGenre.value = true
  try {
    await musicAPI.setArtistGenre(selectedArtist.value.name, matchedGenre.id)

    // Update local state
    selectedArtist.value.genre = matchedGenre.name

    // Update in the store's artists list too
    const idx = store.artists.findIndex(a => a.name === selectedArtist.value?.name)
    if (idx !== -1) {
      store.artists[idx].genre = matchedGenre.name
    }

    success('Genre updated', `Genre set to "${matchedGenre.name}" for ${selectedArtist.value.name}`)
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
    await store.refreshArtists()
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
    await store.refreshArtists()
  } catch (err: any) {
    console.error('Error reprocessing artists:', err)
    error('Failed to reprocess artists', err?.response?.data?.error || err?.message)
  } finally {
    isReprocessing.value = false
  }
}

onMounted(() => {
  // Initialize store (loads cached tracks and subscribes to updates) then refresh artists
  // store.init() is idempotent - safe to call from multiple tabs
  store.init().then(() => store.refreshArtists())
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
  flex-wrap: wrap;
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

/* Artists Grid */
.artists-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
  gap: 20px;
}

.artist-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 16px;
  transition: all 0.2s ease;
  cursor: pointer;
}

.artist-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  background: var(--surface-hover);
}

.artist-avatar {
  width: 56px;
  height: 56px;
  border-radius: 50%;
  background: var(--primary-glow, rgba(139, 92, 246, 0.15));
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 600;
  color: var(--primary-color);
  flex-shrink: 0;
  font-size: 1.2rem;
}

.artist-avatar.large {
  width: 100px;
  height: 100px;
  font-size: 2.5rem;
}

.artist-info {
  flex: 1;
  min-width: 0;
}

.artist-info h3 {
  margin: 0 0 4px 0;
  font-size: 1.1rem;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.artist-meta {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.85rem;
  color: var(--text-tertiary);
}

.genre-badge {
  background: var(--accent-muted, rgba(139, 92, 246, 0.15));
  color: var(--primary-color);
  padding: 1px 8px;
  border-radius: 100px;
  font-size: 0.75rem;
  font-weight: 500;
}

.view-icon {
  color: var(--text-tertiary);
  opacity: 0;
  transition: all 0.2s ease;
}

.artist-card:hover .view-icon {
  opacity: 1;
  transform: translateX(4px);
}

/* Detail View */
.artist-detail {
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

.detail-title-section {
  display: flex;
  align-items: center;
  gap: 24px;
}

.artist-name-row {
  display: flex;
  align-items: center;
  gap: 12px;
}

.artist-name-row h2 {
  margin: 0;
  font-size: 2rem;
  font-weight: 700;
}

.detail-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-top: 8px;
  color: var(--text-secondary);
}

.btn-queue {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.btn-queue:hover {
  background: rgba(245, 158, 11, 0.2);
  border-color: rgba(245, 158, 11, 0.5);
}

.genre-tag {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  background: var(--surface-muted);
  padding: 4px 12px;
  border-radius: 100px;
  font-size: 0.9rem;
  cursor: pointer;
}

.genre-tag.editable:hover {
  background: var(--surface-hover);
  color: var(--primary-color);
}

.track-section {
  margin-bottom: 32px;
}

.section-title {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 1.1rem;
  font-weight: 600;
  margin-bottom: 16px;
  color: var(--text-secondary);
}

.tracks-list {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
}

.track-row {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px 20px;
  border-bottom: 1px solid var(--border-color);
  transition: background 0.2s;
}

.track-row:last-child {
  border-bottom: none;
}

.track-row:hover {
  background: var(--surface-hover);
}

.track-play {
  width: 36px;
  height: 36px;
  border-radius: 50%;
  border: none;
  background: var(--primary-color);
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
}

.track-info {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.track-title {
  font-weight: 500;
}

.track-album, .track-artist {
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.track-genre {
  font-size: 0.85rem;
  color: var(--text-tertiary);
  width: 120px;
}

.track-duration {
  font-size: 0.85rem;
  color: var(--text-tertiary);
  font-family: monospace;
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
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  width: 100%;
  max-width: 480px;
  overflow: hidden;
}

.modal-header {
  padding: 20px 24px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-body {
  padding: 24px;
}

.modal-footer {
  padding: 16px 24px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  background: var(--surface-muted);
}

.form-group {
  margin-top: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.form-group input {
  width: 100%;
  padding: 10px 14px;
  border-radius: 8px;
  border: 1px solid var(--border-color);
  background: var(--background-page);
  color: var(--text-color);
}

/* Responsive */
@media (max-width: 768px) {
  .header {
    flex-direction: column;
    align-items: stretch;
  }

  .header-actions {
    flex-direction: column;
  }

  .search-box {
    min-width: unset;
    width: 100%;
  }

  .artists-grid {
    grid-template-columns: 1fr;
    gap: 12px;
  }

  .artist-card {
    padding: 12px;
  }

  .detail-title-section {
    flex-direction: column;
    text-align: center;
    gap: 16px;
  }

  .artist-name-row {
    justify-content: center;
  }

  .artist-name-row h2 {
    font-size: 1.5rem;
  }

  .detail-meta {
    flex-direction: column;
    gap: 8px;
  }

  .track-row {
    padding: 10px 12px;
    gap: 12px;
  }

  .track-number, .track-genre {
    display: none;
  }

  .modal {
    max-width: 100%;
    margin: 0;
    border-radius: 20px 20px 0 0;
    position: fixed;
    bottom: 0;
  }
}
</style>
