<script setup lang="ts">
import { useMusicData } from '../../composables/useMusicData'
import { useTrackEdit } from '../../composables/useTrackEdit'
import { useTrackActions } from '../../composables/useTrackActions'
import { useAuth } from '../../composables/useAuth'
import { useConfirm } from '../../composables/useConfirm'
import { useRouter } from 'vue-router'
import MusicFiltersPanel from './MusicFiltersPanel.vue'
import MusicTable from './MusicTable.vue'
import EditTrackDrawer from './EditTrackDrawer.vue'
import ConfirmGenreModal from './ConfirmGenreModal.vue'
import TrackDetailsModal from './TrackDetailsModal.vue'
import TrackContextMenu from './TrackContextMenu.vue'
import BulkActionsContextMenu from './BulkActionsContextMenu.vue'
import { musicAPI } from '../../api/music'
import type { MusicFile } from '../../types'
import { ref, computed } from 'vue'
import { useMusicStore } from '../../stores/musicStore'
import { useDjStore } from '../../stores/djStore'
import { useToast } from '../../composables/useToast'

// Props for permission-based UI
interface Props {
  canEdit?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  canEdit: false
})

const router = useRouter()
const { confirm } = useConfirm()

// Use auth for fallback if canEdit not passed
const { isLoggedIn } = useAuth()
const canManage = () => props.canEdit || isLoggedIn.value

// Data management
const {
  musicFiles,
  playlists,
  genres,
  canonicalGenres,
  loading,
  hasMore,
  playlistMenuOpen,
  filters,
  pagination,
  stats,
  fetchMusic,
  resetFilters,
  nextPage,
  prevPage,
} = useMusicData()

const store = useMusicStore()
const djStore = useDjStore()
const { success: toastSuccess } = useToast()

// Edit functionality
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
  fetchMusic()
})

// Track actions
const { playTrack, deleteTrack, addTrackToPlaylist, removeTrackFromPlaylist, setTrackGenre } = useTrackActions(() => {
  fetchMusic()
})

// Confirm genre modal state
const confirmGenreModal = ref({
  isOpen: false,
  track: null as MusicFile | null,
})

// Track details modal state
const detailsModal = ref({
  isOpen: false,
  track: null as MusicFile | null,
})

// Context menu refs
const contextMenuRef = ref<InstanceType<typeof TrackContextMenu> | null>(null)
const bulkMenuRef = ref<InstanceType<typeof BulkActionsContextMenu> | null>(null)

// Formatted stats
const formattedTotalDuration = computed(() => {
  const ms = stats.value.total_duration_ms
  if (!ms) return '0:00'
  const totalSeconds = Math.floor(ms / 1000)
  const hours = Math.floor(totalSeconds / 3600)
  const mins = Math.floor((totalSeconds % 3600) / 60)
  if (hours > 0) {
    return `${hours}h ${mins}m`
  }
  return `${mins}m`
})

const activeFilterCount = computed(() => {
  return [
    filters.search.trim(),
    filters.genre,
    filters.sort !== 'title',
    filters.order !== 'asc',
  ].filter(Boolean).length
})

const filterSummary = computed(() => {
  if (!activeFilterCount.value) return 'All tracks'
  const parts: string[] = []
  if (filters.search.trim()) parts.push(`Search: "${filters.search.trim()}"`)
  if (filters.genre) parts.push(filters.genre === 'unconfirmed' ? 'Unconfirmed' : `Genre: ${filters.genre}`)
  if (filters.sort !== 'title') parts.push(`Sort: ${filters.sort}`)
  if (filters.order !== 'asc') parts.push(`Order: ${filters.order}`)
  return parts.join(' • ')
})

// Track event handlers
const handlePlayTrack = (track: MusicFile) => {
  playTrack(track)
}

const handleEditTrack = (track: MusicFile) => {
  if (canManage()) {
    openEdit(track)
  }
}

const handleCutTrack = (track: MusicFile) => {
  if (canManage()) {
    router.push(`/edit/${track.id}`)
  }
}

const handleDeleteTrack = (track: MusicFile) => {
  if (canManage()) {
    deleteTrack(track)
  }
}

const handleTogglePlaylistMenu = (trackId: string) => {
  if (canManage()) {
    playlistMenuOpen.value = playlistMenuOpen.value === trackId ? null : trackId
  }
}

const handleAddToPlaylist = async (trackId: string, playlistId: string) => {
  if (canManage()) {
    await addTrackToPlaylist(trackId, playlistId)
    playlistMenuOpen.value = null
  }
}

const handleConfirmGenre = (track: MusicFile) => {
  if (canManage()) {
    confirmGenreModal.value = {
      isOpen: true,
      track,
    }
  }
}

const handleGenreConfirmed = () => {
  confirmGenreModal.value.isOpen = false
  confirmGenreModal.value.track = null
  fetchMusic()
}

// Track details handlers
const handleTrackDetails = (track: MusicFile) => {
  detailsModal.value = { isOpen: true, track }
}

const handleDetailsClose = () => {
  detailsModal.value = { isOpen: false, track: null }
}

const handleNavigateArtist = (artist: string) => {
  detailsModal.value.isOpen = false
  filters.search = artist
  filters.genre = ''
  pagination.page = 1
  fetchMusic()
}

const handleNavigateGenre = (genre: string) => {
  detailsModal.value.isOpen = false
  filters.search = ''
  filters.genre = genre
  pagination.page = 1
  fetchMusic()
}

const handleNavigateAlbum = (album: string) => {
  detailsModal.value.isOpen = false
  filters.search = album
  filters.genre = ''
  pagination.page = 1
  fetchMusic()
}

// Context menu handlers
const handleContextMenu = (track: MusicFile, event: MouseEvent) => {
  bulkMenuRef.value?.close()
  releaseDateSuggestion.value = null
  contextMenuRef.value?.open(track, event)
}

const handleBulkContextMenu = (tracks: MusicFile[], event: MouseEvent) => {
  contextMenuRef.value?.close()
  bulkMenuRef.value?.open(tracks, event)
}

const handleLookupDate = (track: MusicFile) => {
  lookupReleaseDate(track.title, track.artist || undefined)
}

const handleApplyDate = async (track: MusicFile, date: string) => {
  const suggestion = releaseDateSuggestion.value
  if (suggestion && suggestion.date === date) {
    await quickSetDate(track.id, date, suggestion.album, suggestion.genre)
  } else {
    await quickSetDate(track.id, date)
  }
  releaseDateSuggestion.value = null
}

const handleBulkPlaylistAdd = async (trackIds: string[], playlistId: string) => {
  if (canManage()) {
    try {
      await Promise.all(trackIds.map(id => musicAPI.addPlaylistTrack(playlistId, { music_file_id: id })))
      fetchMusic()
    } catch (e) {
      console.error('Bulk playlist add failed', e)
    }
  }
}

const handleBulkSetGenre = async (tracks: MusicFile[], genre: string) => {
  if (canManage()) {
    try {
      await musicAPI.bulkUpdateMusic({
        ids: tracks.map(t => t.id),
        genre
      })
      fetchMusic()
    } catch (e) {
      console.error('Bulk genre update failed', e)
    }
  }
}

const handleBulkDelete = async (tracks: MusicFile[]) => {
  if (canManage()) {
    const isConfirmed = await confirm({
      title: 'Delete Tracks',
      message: `Are you sure you want to delete ${tracks.length} tracks? This cannot be undone.`,
      confirmText: 'Delete',
      variant: 'danger'
    })
    
    if (isConfirmed) {
      for (const track of tracks) {
        try {
          await musicAPI.deleteMusicFile(track.id)
        } catch (e) {
          console.error('Failed to delete track', track.id, e)
        }
      }
      fetchMusic()
    }
  }
}

// DJ Queue handlers
const handleAddToQueue = (track: MusicFile) => {
  djStore.addToQueue(track)
  toastSuccess('Added to DJ Queue', `"${track.title}" added to the DJ queue`)
}

const handleBulkAddToQueue = (tracks: MusicFile[]) => {
  djStore.addTracksToQueue(tracks)
  toastSuccess('Added to DJ Queue', `${tracks.length} tracks added to the DJ queue`)
}

// Edit state handlers
const handleEditFormUpdate = {
  title: (val: string) => {
    editState.form.title = val
  },
  artist: (val: string) => {
    editState.form.artist = val
  },
  album: (val: string) => {
    editState.form.album = val
  },
  genre: (val: string) => {
    editState.form.genre = val
  },
  release_date: (val: string) => {
    editState.form.release_date = val
  },
  bpm: (val: number | null) => {
    editState.form.bpm = val
  },
  initial_key: (val: string) => {
    editState.form.initial_key = val
  },
  beat_grid_offset: (val: number | null) => {
    editState.form.beat_grid_offset = val
  },
}
</script>

<template>
  <div class="music-tab">
    <!-- Full database loading indicator -->
    <div v-if="store.loadingAll" class="sync-indicator">
      <div class="sync-spinner"></div>
      <span>Synchronizing music library...</span>
    </div>

    <!-- Filters -->
    <MusicFiltersPanel
      :filters="filters"
      :genres="genres"
      :loading="loading"
      :page-size="pagination.limit"
      @update:search="filters.search = $event"
      @update:genre="filters.genre = $event"
      @update:sort="filters.sort = $event"
      @update:order="filters.order = $event"
      @update:pageSize="pagination.limit = $event"
      @reset="resetFilters"
    />

    <section class="library-summary" aria-label="Library summary">
      <div class="summary-chip primary">
        <Icon name="disc" :size="14" />
        {{ stats.total_count }} tracks
      </div>
      <div class="summary-chip">
        <Icon name="clock" :size="14" />
        {{ formattedTotalDuration }}
      </div>
      <div class="summary-chip">
        <Icon name="filter" :size="14" />
        {{ activeFilterCount }} filters active
      </div>
      <div class="summary-chip summary-text">
        {{ filterSummary }}
      </div>
    </section>

    <!-- Music Table -->
    <MusicTable
      :tracks="musicFiles"
      :playlists="playlists"
      :loading="loading"
      :playlist-menu-open="playlistMenuOpen"
      :can-edit="canManage()"
      :sort="filters.sort"
      :order="filters.order"
      @update:sort="filters.sort = $event"
      @update:order="filters.order = $event"
      @track:play="handlePlayTrack"
      @track:edit="handleEditTrack"
      @track:delete="handleDeleteTrack"
      @track:details="handleTrackDetails"
      @track:confirm-genre="handleConfirmGenre"
      @track:contextmenu="handleContextMenu"
      @tracks:contextmenu="handleBulkContextMenu"
      @playlist:toggle="handleTogglePlaylistMenu"
      @playlist:add="handleAddToPlaylist"
      @reset="resetFilters"
    />

    <!-- Footer with pagination and stats -->
    <footer v-if="musicFiles.length || stats.total_count > 0" class="footer">
      <div class="stats">
        <span>{{ stats.total_count }} tracks</span>
        <span>{{ formattedTotalDuration }}</span>
      </div>
      <div class="pagination">
        <button class="btn btn-outline" :disabled="pagination.page === 1 || loading" @click="prevPage">
          Previous
        </button>
        <span>Page {{ pagination.page }}</span>
        <button class="btn btn-outline" :disabled="!hasMore || loading" @click="nextPage">
          Next
        </button>
      </div>
    </footer>

    <!-- Edit Drawer (only when logged in) -->
    <EditTrackDrawer
      v-if="canManage()"
      :edit-state="editState"
      :saving="editState.saving"
      :genres="canonicalGenres"
      :release-date-suggestion="releaseDateSuggestion"
      :looking-up-release-date="lookingUpReleaseDate"
      @update:title="handleEditFormUpdate.title"
      @update:artist="handleEditFormUpdate.artist"
      @update:album="handleEditFormUpdate.album"
      @update:genre="handleEditFormUpdate.genre"
      @update:release_date="handleEditFormUpdate.release_date"
      @update:bpm="handleEditFormUpdate.bpm"
      @update:initial_key="handleEditFormUpdate.initial_key"
      @update:beat_grid_offset="handleEditFormUpdate.beat_grid_offset"
      @apply-suggestion="applySuggestedReleaseDate"
      @save="saveEdit"
      @close="closeEdit"
    />

    <!-- Track Details Modal -->
    <TrackDetailsModal
      :is-open="detailsModal.isOpen"
      :track="detailsModal.track || undefined"
      @close="handleDetailsClose"
      @navigate:artist="handleNavigateArtist"
      @navigate:genre="handleNavigateGenre"
      @navigate:album="handleNavigateAlbum"
      @edit="handleEditTrack"
      @playlist:add="handleAddToPlaylist"
      @playlist:remove="(trackId, playlistId) => removeTrackFromPlaylist(trackId, playlistId)"
    />

    <!-- Context Menu -->
    <TrackContextMenu
      ref="contextMenuRef"
      :is-admin="canManage()"
      :playlists="playlists"
      :genres="canonicalGenres"
      :date-suggestion="releaseDateSuggestion"
      :looking-up-date="lookingUpReleaseDate"
      @play="handlePlayTrack"
      @details="handleTrackDetails"
      @edit="handleEditTrack"
      @cut="handleCutTrack"
      @delete="handleDeleteTrack"
      @confirm-genre="handleConfirmGenre"
      @playlist:add="handleAddToPlaylist"
      @set-genre="(track, genre) => setTrackGenre(track, genre)"
      @lookup-date="handleLookupDate"
      @apply-date="handleApplyDate"
      @queue:add="handleAddToQueue"
    />

    <!-- Bulk Context Menu -->
    <BulkActionsContextMenu
      ref="bulkMenuRef"
      :is-admin="canManage()"
      :playlists="playlists"
      :genres="canonicalGenres"
      @bulk:playlist:add="handleBulkPlaylistAdd"
      @bulk:set-genre="handleBulkSetGenre"
      @bulk:delete="handleBulkDelete"
      @queue:add-bulk="handleBulkAddToQueue"
    />

    <!-- Confirm Genre Modal (only when logged in) -->
    <ConfirmGenreModal
      v-if="canManage()"
      :is-open="confirmGenreModal.isOpen"
      :track="confirmGenreModal.track || undefined"
      @close="confirmGenreModal.isOpen = false"
      @confirm="handleGenreConfirmed"
    />
  </div>
</template>

<style scoped>
/* Layout */
.music-tab {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.library-summary {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  align-items: center;
}

.summary-chip {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 8px 12px;
  border-radius: 9999px;
  border: 1px solid var(--border-color);
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
  font-size: 0.8rem;
  font-weight: 500;
}

.summary-chip.primary {
  background: rgba(30, 215, 96, 0.12);
  border-color: rgba(30, 215, 96, 0.25);
  color: var(--text-color);
}

.summary-text {
  flex: 1 1 320px;
}

.sync-indicator {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 12px;
  background: var(--primary-glow);
  border: 1px solid var(--primary-color);
  border-radius: 12px;
  color: var(--primary-light);
  font-size: 14px;
  font-weight: 500;
  animation: fadeInDown 0.3s ease-out;
}

.sync-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--primary-light);
  border-top-color: transparent;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

@keyframes fadeInDown {
  from { opacity: 0; transform: translateY(-10px); }
  to { opacity: 1; transform: translateY(0); }
}

.footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  padding: 20px 28px;
  border: 1px solid var(--border-color);
  border-radius: 16px;
  background: var(--surface-color);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
}

.stats {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: var(--text-secondary);
}

.stats span {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: var(--background-elevated);
  border-radius: 999px;
  font-weight: 500;
}



.pagination {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 14px;
  color: var(--text-secondary);
}

.pagination span {
  font-weight: 600;
  color: var(--text-color);
  min-width: 70px;
  text-align: center;
}

.pagination .btn-outline {
  padding: 10px 16px;
}



@media (max-width: 900px) {
  .library-summary {
    gap: 6px;
  }

  .summary-chip {
    font-size: 0.75rem;
  }

  .header {
    flex-direction: column;
    padding: 24px;
  }

  .header-actions {
    width: 100%;
  }

  .header-actions .btn {
    flex: 1;
    justify-content: center;
  }

  .footer {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
    padding: 20px;
  }

  .stats {
    justify-content: center;
  }

  .pagination {
    justify-content: space-between;
  }
}
</style>
