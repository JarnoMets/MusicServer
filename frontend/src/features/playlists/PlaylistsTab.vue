<template>
  <div class="playlists-tab">
    <!-- Playlist Detail View -->
    <Transition name="fade">
      <div v-if="selectedPlaylist" class="playlist-detail" key="detail">
        <header class="detail-header">
          <button class="btn-back" @click="closePlaylistDetail">
            <Icon name="arrow-left" :size="20" />
            <span>Back to Playlists</span>
          </button>
          <div class="detail-title-section">
            <div class="playlist-icon large">
              <Icon name="list" :size="32" />
            </div>
            <div>
              <h2>{{ selectedPlaylist.name }}</h2>
              <p v-if="selectedPlaylist.description" class="detail-description">{{ selectedPlaylist.description }}</p>
              <div class="detail-meta">
                <span>{{ selectedPlaylist.items?.length || 0 }} tracks</span>
                <span v-if="selectedPlaylist.items?.length">&bull;</span>
                <span v-if="selectedPlaylist.items?.length">{{ calculatePlaylistDuration(selectedPlaylist.items) }}</span>
              </div>
            </div>
          </div>
          <div v-if="canEdit" class="detail-actions">
            <button class="btn btn-queue" type="button" @click="handleAddAllToQueue" :disabled="loadingDetail || !selectedPlaylist?.items?.length">
              <Icon name="plus-circle" :size="16" />
              Add All to DJ Queue
            </button>
            <button class="btn btn-secondary" type="button" @click="exportPlaylist('zip')" :disabled="loadingDetail || !selectedPlaylist?.items?.length">
              <Icon name="download" :size="16" />
              Export ZIP
            </button>
            <button class="btn btn-secondary" type="button" @click="exportPlaylist('rekordbox')" :disabled="loadingDetail || !selectedPlaylist?.items?.length">
              <Icon name="download" :size="16" />
              Rekordbox
            </button>
            <button class="btn btn-secondary" type="button" @click="editPlaylist(selectedPlaylist)">
              <Icon name="edit" :size="16" />
              Edit
            </button>
            <button class="btn btn-danger" type="button" @click="confirmDeletePlaylist(selectedPlaylist.id, selectedPlaylist.name)">
              <Icon name="trash" :size="16" />
              Delete
            </button>
          </div>
        </header>

        <!-- Playlist Tracks -->
        <div v-if="loadingDetail && (!selectedPlaylist.items || selectedPlaylist.items.length === 0)" class="loading-tracks">
          <div class="loading-spinner"></div>
          <span>Loading tracks...</span>
        </div>
        <div v-else-if="!selectedPlaylist.items?.length" class="empty-playlist">
          <Icon name="disc" :size="48" />
          <h3>No tracks yet</h3>
          <p>Add tracks to this playlist from the Library tab</p>
        </div>
        <div v-else class="playlist-tracks-container" :class="{ 'is-loading': loadingDetail }">
          <div v-if="loadingDetail" class="loading-progress"></div>
          <div class="playlist-tracks">
            <draggable 
              v-model="playlistItems" 
              item-key="id"
              handle=".drag-handle"
              :options="draggableOptions"
              @end="handleReorder"
              :disabled="!canEdit || isReordering"
              class="draggable-list"
            >
              <template #item="{ element: track, index }">
                <div 
                  class="track-row"
                  @contextmenu.prevent="handleContextMenu(track, $event)"
                  role="button"
                  tabindex="0"
                  @keydown.enter.prevent="playTrack(track)"
                  @keydown.space.prevent="playTrack(track)"
                >
                  <div v-if="canEdit" class="drag-handle" title="Drag to reorder">
                    <Icon name="more-vertical" :size="16" />
                  </div>
                  <span class="track-number">{{ index + 1 }}</span>
                  <button class="track-play" type="button" @click="playTrack(track)">
                    <Icon name="play" :size="16" />
                  </button>
                  <div class="track-info">
                    <span class="track-title">{{ track.title }}</span>
                    <span class="track-artist">{{ track.artist || 'Unknown Artist' }}</span>
                  </div>
                  <span class="track-duration">{{ formatDuration(track.duration) }}</span>
                  <button 
                    v-if="canEdit"
                    class="btn-icon danger" 
                    type="button"
                    @click="removeTrack(track.id)"
                    title="Remove from playlist"
                  >
                    <Icon name="x" :size="16" />
                  </button>
                </div>
              </template>
            </draggable>
          </div>
        </div>
      </div>

      <!-- Playlists List View -->
      <div v-else class="playlists-list-container" key="list">
        <div class="header">
          <div class="header-content">
            <h2>Playlists</h2>
            <p class="subtitle">Organize your music into custom playlists</p>
          </div>
          <div class="header-actions">
            <div class="search-box">
              <Icon name="search" :size="16" />
              <input 
                v-model="searchQuery" 
                type="text" 
                placeholder="Search playlists..."
              />
              <button v-if="searchQuery" class="clear-search" type="button" @click="searchQuery = ''" title="Clear search">
                <Icon name="x" :size="14" />
              </button>
            </div>
            <div class="sort-controls">
              <select v-model="sortBy" class="sort-select">
                <option value="created-desc">Newest</option>
                <option value="created-asc">Oldest</option>
                <option value="tracks-desc">Most Tracks</option>
                <option value="tracks-asc">Least Tracks</option>
                <option value="name-asc">Name (A-Z)</option>
                <option value="name-desc">Name (Z-A)</option>
              </select>
            </div>
            <button v-if="canEdit" @click="showCreateForm = true" class="btn btn-primary" type="button">
              <Icon name="plus" :size="16" /> New Playlist
            </button>
          </div>
        </div>

        <div class="results-summary">
          <span class="summary-chip">
            <Icon name="list" :size="14" />
            {{ allPlaylists.length }} total playlists
          </span>
          <span class="summary-chip">
            <Icon name="search" :size="14" />
            {{ filteredPlaylists.length }} shown
          </span>
          <span class="summary-chip">
            <Icon name="disc" :size="14" />
            {{ visibleTracksCount }} tracks in view
          </span>
          <span class="summary-chip" v-if="editablePlaylistCount">
            <Icon name="edit" :size="14" />
            {{ editablePlaylistCount }} editable
          </span>
        </div>

        <div v-if="loading && !filteredPlaylists.length" class="loading-grid">
          <div v-for="i in 4" :key="i" class="skeleton-card" />
        </div>

        <div v-else-if="filteredPlaylists.length" class="playlists-grid">
          <article 
            v-for="(playlist, index) in filteredPlaylists" 
            :key="playlist.id" 
            class="playlist-card"
            @click="openPlaylistDetail(playlist.id)"
            @keydown.enter.prevent="openPlaylistDetail(playlist.id)"
            @keydown.space.prevent="openPlaylistDetail(playlist.id)"
            role="button"
            tabindex="0"
          >
            <span class="playlist-rank">#{{ index + 1 }}</span>
            <div class="playlist-header">
              <div class="playlist-icon">
                <Icon name="list" :size="24" />
              </div>
              <div class="playlist-title-group">
                <h3>{{ playlist.name }}</h3>
                <span class="playlist-date">Created {{ playlist.created_at ? formatDate(playlist.created_at) : 'Unknown' }}</span>
              </div>
              <div v-if="canEdit" class="playlist-actions" @click.stop>
                <button class="btn-icon" type="button" @click="editPlaylist(playlist)" title="Edit">
                  <Icon name="edit" :size="16" />
                </button>
                <button class="btn-icon danger" type="button" @click="confirmDeletePlaylist(playlist.id, playlist.name)" title="Delete">
                  <Icon name="trash" :size="16" />
                </button>
              </div>
            </div>
            <p v-if="playlist.description" class="playlist-description">{{ playlist.description }}</p>
            <div class="playlist-footer">
              <span class="track-count">
                {{ playlist.track_count || 0 }} tracks
              </span>
              <Icon name="chevron-right" :size="18" class="view-icon" />
            </div>
          </article>
        </div>

        <div v-else class="empty-state">
          <div class="empty-icon">
            <Icon name="list" :size="64" />
          </div>
          <h3>No playlists yet</h3>
          <p v-if="canEdit">Create your first playlist to start organizing your music</p>
          <p v-else>Login to create and manage playlists</p>
          <button v-if="canEdit" @click="showCreateForm = true" class="btn btn-primary" type="button">
            <Icon name="plus" :size="16" /> Create Your First Playlist
          </button>
        </div>
      </div>
    </Transition>

    <!-- Create/Edit Form Modal -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showCreateForm || editingPlaylist" class="modal-overlay" @click="closeForm">
          <div class="modal" @click.stop>
            <div class="modal-header">
              <h3>{{ editingPlaylist ? 'Edit Playlist' : 'Create New Playlist' }}</h3>
              <button class="modal-close" type="button" @click="closeForm">
                <Icon name="x" :size="20" />
              </button>
            </div>
            <form @submit.prevent="submitForm">
              <div class="form-group">
                <label for="playlist-name">Playlist Name</label>
                <input 
                  id="playlist-name"
                  v-model="formData.name" 
                  type="text" 
                  required 
                  placeholder="My Awesome Playlist" 
                />
              </div>
              <div class="form-group">
                <label for="playlist-desc">Description</label>
                <textarea 
                  id="playlist-desc"
                  v-model="formData.description" 
                  placeholder="Optional description for your playlist"
                  rows="3"
                ></textarea>
              </div>
              <div class="form-actions">
                <button type="button" @click="closeForm" class="btn btn-secondary">
                  Cancel
                </button>
                <button type="submit" class="btn btn-primary" :disabled="saving">
                  {{ saving ? 'Saving...' : (editingPlaylist ? 'Update' : 'Create') }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Context Menu -->
    <TrackContextMenu
      ref="contextMenuRef"
      :is-admin="canEdit"
      :playlists="allPlaylists"
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
import { ref, onMounted, computed, watch } from 'vue'
// @ts-ignore
import draggable from 'vuedraggable'
import { musicAPI } from '../../api/music'
import type { MusicFile } from '../../types'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import { useConfirm } from '../../composables/useConfirm'
import { useAuth } from '../../composables/useAuth'
import { useMusicData } from '../../composables/useMusicData'
import { useTrackEdit } from '../../composables/useTrackEdit'
import { useTrackActions } from '../../composables/useTrackActions'
import { useDjStore } from '../../stores/djStore'
import { useMusicStore } from '../../stores/musicStore'
import { formatDuration, formatRelativeDate as formatDate } from '../../utils/musicFormatters'
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
const { isLoggedIn, token: authToken } = useAuth()
const canEdit = computed(() => props.canEdit || isLoggedIn.value)

interface Playlist {
  id: string
  name: string
  description?: string
  track_count?: number
  created_at?: string
  updated_at?: string
}

interface PlaylistWithItems extends Playlist {
  items?: MusicFile[]
  created_at: string
  updated_at: string
}

const selectedPlaylist = ref<PlaylistWithItems | null>(null)
const loading = ref(false)
const loadingDetail = ref(false)
const saving = ref(false)
const showCreateForm = ref(false)
const editingPlaylist = ref<Playlist | null>(null)
const searchQuery = ref('')
const sortBy = ref<'created-desc' | 'created-asc' | 'tracks-desc' | 'tracks-asc' | 'name-asc' | 'name-desc'>('created-desc')
const formData = ref({
  name: '',
  description: '',
})

const playlistItems = ref<MusicFile[]>([])
const isReordering = ref(false)

const { playLocalTrack } = usePlayer()
const { success, error } = useToast()
const { confirm } = useConfirm()
const djStore = useDjStore()
const musicStore = useMusicStore()

// Integrated tools from MusicTab
const { playlists: allPlaylists, canonicalGenres, fetchMusic } = useMusicData()

const openPlaylistDetail = async (playlistId: string) => {
  loadingDetail.value = true
  try {
    const response = await musicAPI.getPlaylist(playlistId)
    selectedPlaylist.value = response.data
    playlistItems.value = response.data.items || []
  } catch (err: any) {
    console.error('Error fetching playlist:', err)
    error('Failed to load playlist', err?.response?.data?.error || err?.message)
  } finally {
    loadingDetail.value = false
  }
}

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
  if (selectedPlaylist.value) openPlaylistDetail(selectedPlaylist.value.id)
  fetchMusic()
})

const { deleteTrack, addTrackToPlaylist, setTrackGenre } = useTrackActions(() => {
  if (selectedPlaylist.value) openPlaylistDetail(selectedPlaylist.value.id)
  fetchMusic()
})

const handleAddToQueue = (track: MusicFile) => {
  djStore.addToQueue(track)
}

const handleAddAllToQueue = () => {
  if (!selectedPlaylist.value?.items) return
  djStore.addTracksToQueue(selectedPlaylist.value.items)
  success('Added all tracks', `Added ${selectedPlaylist.value.items.length} tracks to queue`)
}

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
  if (confirmGenreModal.value.track) {
    setTrackGenre(confirmGenreModal.value.track, confirmGenreModal.value.track.genre_id || '')
  }
  confirmGenreModal.value.isOpen = false
}

const filteredPlaylists = computed(() => {
  let result = [...musicStore.playlists]
  
  if (searchQuery.value) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(p => 
      p.name.toLowerCase().includes(q) || 
      (p.description && p.description.toLowerCase().includes(q))
    )
  }

  const sorted = [...result]
  switch (sortBy.value) {
    case 'created-desc':
      return sorted.sort((a, b) => {
        const dateA = a.created_at ? new Date(a.created_at).getTime() : 0
        const dateB = b.created_at ? new Date(b.created_at).getTime() : 0
        return dateB - dateA
      })
    case 'created-asc':
      return sorted.sort((a, b) => {
        const dateA = a.created_at ? new Date(a.created_at).getTime() : 0
        const dateB = b.created_at ? new Date(b.created_at).getTime() : 0
        return dateA - dateB
      })
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
  return filteredPlaylists.value.reduce((sum, playlist) => sum + (playlist.track_count || 0), 0)
})

const editablePlaylistCount = computed(() => {
  return canEdit.value ? filteredPlaylists.value.length : 0
})

const fetchPlaylists = async () => {
  try {
    loading.value = true
    await musicStore.refreshPlaylists()
  } catch (err: any) {
    console.error('Error fetching playlists:', err)
    error('Failed to load playlists', err?.response?.data?.error || err?.message)
  } finally {
    loading.value = false
  }
}

const handleReorder = async () => {
  if (!selectedPlaylist.value || isReordering.value) return
  
  isReordering.value = true
  try {
    const ids = playlistItems.value.map(t => t.id)
    await musicAPI.reorderPlaylistTracks(selectedPlaylist.value.id, ids)
    success('Order saved', 'Playlist order has been updated')
    // We update local items to keep it in sync until store update arrives
    selectedPlaylist.value.items = [...playlistItems.value]
  } catch (err: any) {
    console.error('Error reordering playlist:', err)
    error('Failed to save order', err?.response?.data?.error || err?.message)
    // Revert on error
    if (selectedPlaylist.value.items) {
      playlistItems.value = [...selectedPlaylist.value.items]
    }
  } finally {
    isReordering.value = false
  }
}

// Draggable / SortableJS options to allow auto-scrolling while dragging
const draggableOptions = {
  scroll: true,
  scrollSensitivity: 60,
  scrollSpeed: 10,
  bubbleScroll: true,
  fallbackOnBody: true,
}

const closePlaylistDetail = () => {
  selectedPlaylist.value = null
  fetchPlaylists()
}

const exportPlaylist = (format: 'zip' | 'rekordbox') => {
  if (!selectedPlaylist.value) return
  
  let url = format === 'zip' 
    ? musicAPI.exportPlaylistZip(selectedPlaylist.value.id)
    : musicAPI.exportPlaylistRekordbox(selectedPlaylist.value.id)
  
  // Append token for authentication since browser navigation won't include Bearer headers
  if (authToken.value) {
    const separator = url.includes('?') ? '&' : '?';
    url = `${url}${separator}token=${encodeURIComponent(authToken.value)}`;
  }
  
  const link = document.createElement('a')
  link.href = url
  link.setAttribute('download', '')
  document.body.appendChild(link)
  link.click()
  document.body.removeChild(link)
  
  success('Export started', `Your ${format === 'rekordbox' ? 'Rekordbox' : 'ZIP'} export should begin downloading shortly.`)
}

const playTrack = (track: MusicFile) => {
  playLocalTrack({
    id: track.id,
    title: track.title,
    artist: track.artist,
    duration: track.duration,
  })
}

const removeTrack = async (track_id: string) => {
  if (!selectedPlaylist.value) return
  
  try {
    await musicAPI.removePlaylistTrack(selectedPlaylist.value.id, track_id)
    // Update local state
    if (selectedPlaylist.value.items) {
      selectedPlaylist.value.items = selectedPlaylist.value.items.filter(t => t.id !== track_id)
    }
    playlistItems.value = playlistItems.value.filter(t => t.id !== track_id)
    success('Track removed', 'Track has been removed from the playlist')
  } catch (err: any) {
    console.error('Error removing track:', err)
    error('Failed to remove track', err?.response?.data?.error || err?.message)
  }
}

const calculatePlaylistDuration = (items: MusicFile[] | undefined) => {
  if (!items) return '0:00'
  const totalMs = items.reduce((sum, track) => sum + (track.duration || 0), 0)
  return formatDuration(totalMs)
}

const closeForm = () => {
  showCreateForm.value = false
  editingPlaylist.value = null
  formData.value = { name: '', description: '' }
}

const editPlaylist = (playlist: Playlist | PlaylistWithItems) => {
  editingPlaylist.value = playlist as Playlist
  formData.value = {
    name: playlist.name,
    description: playlist.description || '',
  }
}

const submitForm = async () => {
  if (!formData.value.name.trim()) return

  saving.value = true
  try {
    if (editingPlaylist.value) {
      await musicAPI.updatePlaylist(editingPlaylist.value.id, {
        name: formData.value.name,
        description: formData.value.description || undefined,
      })
      success('Playlist updated', `"${formData.value.name}" has been updated`)
    } else {
      await musicAPI.createPlaylist(formData.value.name, formData.value.description || undefined)
      success('Playlist created', `"${formData.value.name}" has been created`)
    }
    closeForm()
    await fetchPlaylists()
  } catch (err: any) {
    console.error('Error saving playlist:', err)
    error('Failed to save playlist', err?.response?.data?.error || err?.message)
  } finally {
    saving.value = false
  }
}

const confirmDeletePlaylist = async (id: string, name: string) => {
  const confirmed = await confirm({
    title: 'Delete Playlist',
    message: `Delete "${name}"? This will remove all tracks from the playlist.`,
    confirmText: 'Delete',
    cancelText: 'Cancel',
    variant: 'danger'
  })
  if (!confirmed) return

  try {
    await musicAPI.deletePlaylist(id)
    success('Playlist deleted', `"${name}" has been removed`)
    
    if (selectedPlaylist.value && selectedPlaylist.value.id === id) {
      selectedPlaylist.value = null
    }
    
    await fetchPlaylists()
  } catch (err: any) {
    console.error('Error deleting playlist:', err)
    error('Failed to delete playlist', err?.response?.data?.error || err?.message)
  }
}

// Watch for store updates to update details if open
watch(() => musicStore.playlists, (newPlaylists) => {
  if (selectedPlaylist.value) {
    const updated = newPlaylists.find(p => p.id === selectedPlaylist.value?.id)
    if (updated) {
      if (updated.track_count !== selectedPlaylist.value.track_count && !isReordering.value) {
         openPlaylistDetail(selectedPlaylist.value.id)
      }
      selectedPlaylist.value.name = updated.name
      selectedPlaylist.value.description = updated.description
      selectedPlaylist.value.track_count = updated.track_count
    }
  }
}, { deep: true })

onMounted(() => {
  fetchPlaylists()
})
</script>

<style scoped>
.playlists-tab {
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
  padding: 6px 10px;
  border-radius: 9999px;
  border: 1px solid var(--border-color);
  background: var(--surface-muted);
  color: var(--text-secondary);
  font-size: 0.8rem;
  font-weight: 500;
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

.clear-search:focus-visible,
.sort-select:focus-visible,
.btn:focus-visible,
.btn-icon:focus-visible,
.btn-back:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
}

.sort-controls {
  display: flex;
  align-items: center;
  min-width: 170px;
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

/* Playlist Detail View */
.playlist-detail {
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

.playlist-icon.large {
  width: 80px;
  height: 80px;
  background: var(--primary-glow, rgba(139, 92, 246, 0.15));
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color);
  flex-shrink: 0;
}

.detail-title-section h2 {
  margin: 0 0 8px 0;
  font-size: 1.75rem;
  font-weight: 700;
}

.detail-description {
  margin: 0 0 8px 0;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.5;
}

.detail-meta {
  display: flex;
  gap: 8px;
  color: var(--text-tertiary);
  font-size: 13px;
}

.detail-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

/* Loading tracks */
.loading-tracks {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 12px;
  padding: 60px 20px;
  color: var(--text-secondary);
  min-height: 200px;
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

/* Empty playlist */
.empty-playlist {
  text-align: center;
  padding: 60px 20px;
  color: var(--text-tertiary);
}

.empty-playlist h3 {
  margin: 16px 0 8px;
  color: var(--text-color);
}

.empty-playlist p {
  margin: 0;
  color: var(--text-secondary);
}

/* Playlist tracks */
.playlist-tracks {
  display: flex;
  flex-direction: column;
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
  background: var(--surface-color);
}

.playlist-tracks-container {
  position: relative;
  transition: opacity 0.3s ease;
  min-height: 200px;
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

.track-row:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: -2px;
  background: var(--surface-hover);
}

.drag-handle {
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: grab;
  color: var(--text-tertiary);
  padding: 4px;
  border-radius: 4px;
  transition: all 0.2s ease;
  opacity: 0;
}

.track-row:hover .drag-handle {
  opacity: 1;
}

.drag-handle:hover {
  background: var(--surface-muted);
  color: var(--text-secondary);
}

.drag-handle:active {
  cursor: grabbing;
}

.draggable-list .sortable-ghost {
  opacity: 0.3;
  background: var(--primary-glow);
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

.track-play:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
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

.track-artist {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-duration {
  color: var(--text-tertiary);
  font-size: 13px;
  font-family: monospace;
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

.btn-queue {
  background: rgba(245, 158, 11, 0.1);
  color: #f59e0b;
  border: 1px solid rgba(245, 158, 11, 0.3);
}

.btn-queue:hover:not(:disabled) {
  background: rgba(245, 158, 11, 0.2);
  border-color: rgba(245, 158, 11, 0.5);
  transform: translateY(-2px);
}

.btn-queue:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-danger {
  background: rgba(239, 68, 68, 0.1);
  color: #f87171;
  border: 1px solid rgba(239, 68, 68, 0.3);
}

.btn-danger:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.5);
}

.btn-icon {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  border: none;
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  opacity: 0;
  transition: all 0.2s ease;
  color: var(--text-secondary);
}

.playlist-card:hover .btn-icon,
.track-row:hover .btn-icon {
  opacity: 1;
}

.btn-icon:hover {
  background: var(--surface-muted);
}

.btn-icon:focus-visible {
  background: var(--surface-muted);
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
}

.btn-icon.danger {
  color: var(--text-secondary);
}

.btn-icon.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  color: #f87171;
}

/* Loading */
.loading-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.skeleton-card {
  padding: 24px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
  min-height: 180px;
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
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

/* Playlists Grid */
.playlists-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.playlist-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 20px;
  transition: all 0.2s ease;
  cursor: pointer;
  position: relative;
}

.playlist-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.playlist-card:focus-visible {
  border-color: var(--primary-color);
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
}

.playlist-rank {
  position: absolute;
  top: 10px;
  right: 12px;
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

.playlist-header {
  display: flex;
  align-items: flex-start;
  gap: 14px;
  margin-bottom: 12px;
}

.playlist-icon {
  width: 48px;
  height: 48px;
  background: var(--accent-muted, rgba(139, 92, 246, 0.15));
  border-radius: 10px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  color: var(--primary-color);
}

.playlist-title-group {
  flex: 1;
  min-width: 0;
}

.playlist-title-group h3 {
  margin: 0 0 4px 0;
  font-size: 1.1rem;
  font-weight: 600;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.playlist-date {
  font-size: 12px;
  color: var(--text-tertiary);
}

.playlist-actions {
  display: flex;
  gap: 4px;
}

.playlist-description {
  margin: 0 0 16px 0;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.5;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.playlist-footer {
  padding-top: 16px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.track-count {
  font-size: 13px;
  color: var(--text-tertiary);
  font-weight: 500;
}

.view-icon {
  color: var(--text-tertiary);
  transition: all 0.2s ease;
}

.playlist-card:hover .view-icon {
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
  max-width: 480px;
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

.modal form {
  padding: 24px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  font-size: 13px;
  color: var(--text-secondary);
}

.form-group input,
.form-group textarea {
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

.form-group input:focus,
.form-group textarea:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--accent-muted, rgba(139, 92, 246, 0.15));
}

.form-group textarea {
  resize: vertical;
  min-height: 80px;
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

/* Transitions */
.fade-enter-active, .fade-leave-active {
  transition: opacity 0.2s ease;
}
.fade-enter-from, .fade-leave-to {
  opacity: 0;
}

.modal-enter-active, .modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from, .modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal, .modal-leave-to .modal {
  transform: scale(0.95) translateY(20px);
}

@media (max-width: 900px) {
  .header {
    flex-direction: column;
    align-items: stretch;
  }

  .header-actions {
    width: 100%;
  }

  .search-box {
    min-width: unset;
    width: 100%;
  }

  .sort-controls {
    min-width: unset;
    width: 100%;
  }

  .header-actions .btn {
    width: 100%;
    justify-content: center;
  }

  .results-summary {
    gap: 6px;
  }

  .summary-chip {
    font-size: 0.75rem;
  }

  .playlists-grid {
    grid-template-columns: 1fr;
  }

  .detail-title-section {
    flex-direction: column;
  }

  .detail-actions {
    flex-direction: column;
  }

  .track-row {
    gap: 12px;
    padding: 12px 16px;
  }

  .track-number {
    display: none;
  }

  .track-duration {
    margin-left: auto;
  }
}
</style>
