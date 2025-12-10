<template>
  <div class="playlists-tab">
    <!-- Playlist Detail View -->
    <div v-if="selectedPlaylist" class="playlist-detail">
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
              <span v-if="selectedPlaylist.items?.length">•</span>
              <span v-if="selectedPlaylist.items?.length">{{ calculatePlaylistDuration(selectedPlaylist.items) }}</span>
            </div>
          </div>
        </div>
        <div v-if="canEdit" class="detail-actions">
          <button class="btn btn-secondary" @click="editPlaylist(selectedPlaylist)">
            <Icon name="edit" :size="16" />
            Edit
          </button>
          <button class="btn btn-danger" @click="confirmDeletePlaylist(selectedPlaylist.id, selectedPlaylist.name)">
            <Icon name="trash" :size="16" />
            Delete
          </button>
        </div>
      </header>

      <!-- Playlist Tracks -->
      <div v-if="loadingDetail" class="loading-tracks">
        <div class="loading-spinner"></div>
        <span>Loading tracks...</span>
      </div>
      <div v-else-if="!selectedPlaylist.items?.length" class="empty-playlist">
        <Icon name="disc" :size="48" />
        <h3>No tracks yet</h3>
        <p>Add tracks to this playlist from the Library tab</p>
      </div>
      <div v-else class="playlist-tracks">
        <div 
          v-for="(track, index) in selectedPlaylist.items" 
          :key="track.id" 
          class="track-row"
        >
          <span class="track-number">{{ index + 1 }}</span>
          <button class="track-play" @click="playTrack(track)">
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
            @click="removeTrack(track.id)"
            title="Remove from playlist"
          >
            <Icon name="x" :size="16" />
          </button>
        </div>
      </div>
    </div>

    <!-- Playlists List View -->
    <template v-else>
      <div class="header">
        <div class="header-content">
          <h2>Playlists</h2>
          <p class="subtitle">Organize your music into custom playlists</p>
        </div>
        <button v-if="canEdit" @click="showCreateForm = true" class="btn btn-primary">
          <Icon name="plus" :size="16" /> New Playlist
        </button>
      </div>

      <!-- Create/Edit Form Modal -->
      <Teleport to="body">
        <Transition name="modal">
          <div v-if="showCreateForm || editingPlaylist" class="modal-overlay" @click="closeForm">
            <div class="modal" @click.stop>
              <div class="modal-header">
                <h3>{{ editingPlaylist ? 'Edit Playlist' : 'Create New Playlist' }}</h3>
                <button class="modal-close" @click="closeForm">
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

      <!-- Loading State -->
      <div v-if="loading" class="loading-grid">
        <div v-for="i in 4" :key="i" class="skeleton-card">
          <div class="skeleton-card-header">
            <div class="skeleton skeleton-title"></div>
            <div class="skeleton skeleton-btn"></div>
          </div>
          <div class="skeleton skeleton-desc"></div>
          <div class="skeleton skeleton-footer"></div>
        </div>
      </div>

      <!-- Playlists Grid -->
      <div v-else-if="playlists.length" class="playlists-grid">
        <article 
          v-for="playlist in playlists" 
          :key="playlist.id" 
          class="playlist-card"
          @click="openPlaylistDetail(playlist.id)"
        >
          <div class="playlist-header">
            <div class="playlist-icon">
              <Icon name="list" :size="24" />
            </div>
            <div class="playlist-title-group">
              <h3>{{ playlist.name }}</h3>
              <span class="playlist-date">Created {{ formatDate(playlist.created_at) }}</span>
            </div>
            <div v-if="canEdit" class="playlist-actions" @click.stop>
              <button class="btn-icon" @click="editPlaylist(playlist)" title="Edit">
                <Icon name="edit" :size="16" />
              </button>
              <button class="btn-icon danger" @click="confirmDeletePlaylist(playlist.id, playlist.name)" title="Delete">
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

      <!-- Empty State -->
      <div v-else class="empty-state">
        <div class="empty-icon">
          <Icon name="list" :size="64" />
        </div>
        <h3>No playlists yet</h3>
        <p v-if="canEdit">Create your first playlist to start organizing your music</p>
        <p v-else>Login to create and manage playlists</p>
        <button v-if="canEdit" @click="showCreateForm = true" class="btn btn-primary">
          <Icon name="plus" :size="16" /> Create Your First Playlist
        </button>
      </div>
    </template>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { musicAPI } from '../../api/music'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import { useConfirm } from '../../composables/useConfirm'
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

interface MusicFile {
  id: string
  title: string
  artist?: string | null
  album?: string | null
  duration?: number | null
  file_path: string
}

interface Playlist {
  id: string
  name: string
  description?: string
  track_count?: number
  created_at: string
  updated_at: string
}

interface PlaylistWithItems extends Playlist {
  items?: MusicFile[]
}

const playlists = ref<Playlist[]>([])
const selectedPlaylist = ref<PlaylistWithItems | null>(null)
const loading = ref(false)
const loadingDetail = ref(false)
const saving = ref(false)
const showCreateForm = ref(false)
const editingPlaylist = ref<Playlist | null>(null)
const formData = ref({
  name: '',
  description: '',
})

const { playLocalTrack } = usePlayer()
const { success, error } = useToast()
const { confirm } = useConfirm()

const fetchPlaylists = async () => {
  try {
    loading.value = true
    const response = await musicAPI.getPlaylists()
    playlists.value = response.data
  } catch (err: any) {
    console.error('Error fetching playlists:', err)
    error('Failed to load playlists', err?.response?.data?.error || err?.message)
  } finally {
    loading.value = false
  }
}

const openPlaylistDetail = async (playlistId: string) => {
  loadingDetail.value = true
  try {
    const response = await musicAPI.getPlaylist(playlistId)
    selectedPlaylist.value = response.data
  } catch (err: any) {
    console.error('Error fetching playlist:', err)
    error('Failed to load playlist', err?.response?.data?.error || err?.message)
  } finally {
    loadingDetail.value = false
  }
}

const closePlaylistDetail = () => {
  selectedPlaylist.value = null
  // Refresh list in case changes were made
  fetchPlaylists()
}

const playTrack = (track: MusicFile) => {
  playLocalTrack({
    id: track.id,
    title: track.title,
    artist: track.artist,
  })
}

const removeTrack = async (trackId: string) => {
  if (!selectedPlaylist.value) return
  
  try {
    await musicAPI.removePlaylistTrack(selectedPlaylist.value.id, trackId)
    // Update local state
    if (selectedPlaylist.value.items) {
      selectedPlaylist.value.items = selectedPlaylist.value.items.filter(t => t.id !== trackId)
    }
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

const resetForm = () => {
  formData.value = { name: '', description: '' }
}

const closeForm = () => {
  showCreateForm.value = false
  editingPlaylist.value = null
  resetForm()
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
      
      // Update selected playlist if viewing it
      if (selectedPlaylist.value && selectedPlaylist.value.id === editingPlaylist.value.id) {
        selectedPlaylist.value.name = formData.value.name
        selectedPlaylist.value.description = formData.value.description
      }
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
    
    // If we were viewing this playlist, go back to list
    if (selectedPlaylist.value && selectedPlaylist.value.id === id) {
      selectedPlaylist.value = null
    }
    
    await fetchPlaylists()
  } catch (err: any) {
    console.error('Error deleting playlist:', err)
    error('Failed to delete playlist', err?.response?.data?.error || err?.message)
  }
}

const formatDate = (dateString: string) => {
  const date = new Date(dateString)
  const now = new Date()
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24))
  
  if (diffDays === 0) return 'today'
  if (diffDays === 1) return 'yesterday'
  if (diffDays < 7) return `${diffDays} days ago`
  return date.toLocaleDateString()
}

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
}

.skeleton-card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
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

.skeleton-title {
  height: 24px;
  width: 60%;
}

.skeleton-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
}

.skeleton-desc {
  height: 40px;
  width: 100%;
}

.skeleton-footer {
  height: 16px;
  width: 30%;
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
}

.playlist-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
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
  margin: 0 0 24px 0;
  color: var(--text-secondary);
}

/* Responsive */
@media (max-width: 640px) {
  .header {
    flex-direction: column;
    align-items: stretch;
  }

  .playlists-grid {
    grid-template-columns: 1fr;
  }

  .playlist-card:hover .btn-icon {
    opacity: 1;
  }

  .detail-title-section {
    flex-direction: column;
    align-items: center;
    text-align: center;
  }

  .detail-actions {
    justify-content: center;
  }

  .track-row {
    flex-wrap: wrap;
    gap: 12px;
  }

  .track-number {
    display: none;
  }

  .track-info {
    order: 1;
    width: calc(100% - 52px);
  }

  .track-duration {
    order: 2;
  }
}
</style>
