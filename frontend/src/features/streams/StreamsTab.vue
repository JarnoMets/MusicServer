<template>
  <div class="streams-tab">
    <div class="header">
      <div class="header-content">
        <h2>Internet Streams</h2>
        <p class="subtitle">Add your favorite radio streams and play them directly from the player.</p>
      </div>
      <div class="header-actions">
        <div class="search-box">
          <Icon name="search" :size="16" />
          <input 
            v-model="searchQuery" 
            type="text" 
            placeholder="Search streams..."
            @input="debouncedSearch"
          />
          <button v-if="searchQuery" class="clear-search" @click="clearSearch">
            <Icon name="x" :size="14" />
          </button>
        </div>
        <div class="sort-controls">
          <select v-model="sortBy" class="sort-select">
            <option value="name-asc">Name (A-Z)</option>
            <option value="name-desc">Name (Z-A)</option>
            <option value="created-desc">Newest</option>
            <option value="created-asc">Oldest</option>
          </select>
        </div>
        <button v-if="canEdit" class="btn btn-primary" @click="showAddForm = true">
          <Icon name="plus" :size="16" /> Add Stream
        </button>
      </div>
    </div>

    <!-- Add/Edit Modal -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showAddForm || editingStream" class="modal-overlay" @click="closeForm">
          <div class="modal-content" @click.stop>
            <div class="modal-header">
              <h3>{{ editingStream ? 'Edit Stream' : 'Add New Stream' }}</h3>
              <button class="modal-close" @click="closeForm">
                <Icon name="x" :size="20" />
              </button>
            </div>
            <form class="stream-form" @submit.prevent="submitForm">
              <div class="form-group">
                <label for="stream-name">Stream Name</label>
                <input 
                  id="stream-name"
                  v-model="formData.name" 
                  type="text" 
                  placeholder="e.g., Rinse FM"
                  required 
                />
              </div>
              <div class="form-group">
                <label for="stream-url">Stream URL</label>
                <input
                  id="stream-url"
                  v-model="formData.url"
                  type="url"
                  placeholder="https://stream.example.com/audio"
                  required
                />
              </div>
              <div class="form-row">
                <div class="form-group">
                  <label for="stream-genre">Genre</label>
                  <input 
                    id="stream-genre"
                    v-model="formData.genre" 
                    type="text" 
                    placeholder="e.g., Jazz, Electronic" 
                  />
                </div>
                <div class="form-group">
                  <label for="stream-desc">Description</label>
                  <input 
                    id="stream-desc"
                    v-model="formData.description" 
                    type="text" 
                    placeholder="Optional description" 
                  />
                </div>
              </div>
              <div class="form-actions">
                <button type="button" class="btn btn-secondary" @click="closeForm">Cancel</button>
                <button type="submit" class="btn btn-primary" :disabled="saving">
                  {{ saving ? 'Saving...' : (editingStream ? 'Update' : 'Add Stream') }}
                </button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>

    <!-- Loading State -->
    <div v-if="loading" class="loading-state">
      <div class="skeleton-list">
        <div v-for="i in 3" :key="i" class="skeleton-item">
          <div class="skeleton skeleton-play"></div>
          <div class="skeleton-content">
            <div class="skeleton skeleton-title"></div>
            <div class="skeleton skeleton-subtitle"></div>
          </div>
          <div class="skeleton skeleton-actions"></div>
        </div>
      </div>
    </div>

    <!-- Streams Grid -->
    <div v-else-if="filteredStreams.length" class="streams-grid">
      <article 
        v-for="stream in filteredStreams" 
        :key="stream.id" 
        class="stream-card"
        :class="{ 'is-playing': isStreamPlaying(stream) }"
        @click="openStreamDetail(stream)"
        @contextmenu.prevent="openContextMenu(stream, $event)"
      >
        <button class="stream-play" @click.stop="playStream(stream)" :title="isStreamPlaying(stream) ? 'Now Playing' : 'Play'">
          <span v-if="isStreamPlaying(stream)" class="play-icon playing">
            <span class="bar"></span>
            <span class="bar"></span>
            <span class="bar"></span>
          </span>
          <Icon v-else name="play" :size="18" />
        </button>
        
        <div class="stream-info">
          <h3 class="stream-name">{{ stream.name }}</h3>
          <p v-if="stream.description" class="stream-description">{{ stream.description }}</p>
          <div class="stream-meta">
            <span v-if="stream.genre" class="stream-genre">{{ stream.genre }}</span>
            <span class="stream-url" :title="stream.url">{{ truncateUrl(stream.url) }}</span>
          </div>
        </div>

        <div v-if="canEdit" class="stream-actions">
          <button class="btn-icon" @click="editStream(stream)" title="Edit">
            <Icon name="edit" :size="16" />
          </button>
          <button class="btn-icon danger" @click="confirmDelete(stream)" title="Delete">
            <Icon name="trash" :size="16" />
          </button>
        </div>
      </article>
    </div>

    <!-- Empty State -->
    <div v-else class="empty-state">
      <div class="empty-icon">
        <Icon name="radio" :size="64" />
      </div>
      <h3>No streams yet</h3>
      <p v-if="canEdit">Add your first internet radio stream to start listening</p>
      <p v-else>Login to add and manage radio streams</p>
      <button v-if="canEdit" class="btn btn-primary" @click="showAddForm = true">
        <Icon name="plus" :size="16" /> Add Your First Stream
      </button>
    </div>
    
    <!-- Stream Context Menu -->
    <StreamContextMenu
      ref="streamContextMenu"
      :is-admin="canEdit"
      @play="playStream"
      @details="openStreamDetail"
      @edit="editStream"
      @delete="confirmDelete"
      @copy-url="copyStreamUrl"
    />
  </div>

    <!-- Stream Detail View -->
    <div v-if="selectedStream" class="stream-detail">
      <header class="detail-header">
        <button class="btn-back" @click="closeStreamDetail">
          <Icon name="arrow-left" :size="20" />
          <span>Back to Streams</span>
        </button>
        <div class="detail-title-section">
          <div class="playlist-icon large">
            <Icon name="radio" :size="32" />
          </div>
          <div>
            <h2>{{ selectedStream.name }}</h2>
            <p v-if="selectedStream.description" class="detail-description">{{ selectedStream.description }}</p>
            <div class="detail-meta">
              <span v-if="selectedStream.genre">{{ selectedStream.genre }}</span>
              <span v-if="selectedStream.url">&bull;</span>
              <span v-if="selectedStream.url" class="stream-url" :title="selectedStream.url">{{ truncateUrl(selectedStream.url) }}</span>
            </div>
          </div>
        </div>
        <div class="detail-actions">
          <button class="btn btn-primary" @click="playStream(selectedStream)">
            <Icon name="play" :size="16" /> Play
          </button>
          <button v-if="canEdit" class="btn btn-secondary" @click="editStream(selectedStream)">
            <Icon name="edit" :size="16" /> Edit
          </button>
          <button v-if="canEdit" class="btn btn-danger" @click="confirmDelete(selectedStream)">
            <Icon name="trash" :size="16" /> Delete
          </button>
          <button class="btn btn-secondary" @click="copyStreamUrl(selectedStream)">
            <Icon name="copy" :size="16" /> Copy URL
          </button>
        </div>
      </header>
    </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { musicAPI } from '../../api/music'
import { usePlayer } from '../../composables/usePlayer'
import { useToast } from '../../composables/useToast'
import { useConfirm } from '../../composables/useConfirm'
import { useAuth } from '../../composables/useAuth'
import Icon from '../../shared/components/Icons.vue'
import StreamContextMenu from './StreamContextMenu.vue'

interface Props {
  canEdit?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  canEdit: false
})

// Use auth for fallback
const { isLoggedIn } = useAuth()
const canEdit = computed(() => props.canEdit || isLoggedIn.value)

interface InternetStream {
  id: string
  name: string
  url: string
  genre?: string
  description?: string
  created_at?: string
}

const streams = ref<InternetStream[]>([])
const loading = ref(false)
const saving = ref(false)
const showAddForm = ref(false)
const editingStream = ref<InternetStream | null>(null)
const selectedStream = ref<InternetStream | null>(null)
const searchQuery = ref('')
const sortBy = ref<'name-asc' | 'name-desc' | 'created-desc' | 'created-asc'>('name-asc')

const formData = ref({
  name: '',
  url: '',
  genre: '',
  description: '',
})

const { playInternetStream, state: playerState } = usePlayer()
const { success, error } = useToast()
const { confirm } = useConfirm()
const streamContextMenu = ref<InstanceType<any> | null>(null)

const debouncedSearch = () => {
  // Filter is reactive, no action needed
}

const clearSearch = () => {
  searchQuery.value = ''
}

// Filter and sort streams
const filteredStreams = computed(() => {
  let result = streams.value
  
  // Apply search filter
  if (searchQuery.value.trim()) {
    const query = searchQuery.value.toLowerCase()
    result = result.filter(stream => 
      stream.name.toLowerCase().includes(query) ||
      (stream.description && stream.description.toLowerCase().includes(query)) ||
      (stream.genre && stream.genre.toLowerCase().includes(query))
    )
  }
  
  // Apply sorting
  const sorted = [...result]
  switch (sortBy.value) {
    case 'name-asc':
      return sorted.sort((a, b) => a.name.localeCompare(b.name))
    case 'name-desc':
      return sorted.sort((a, b) => b.name.localeCompare(a.name))
    case 'created-desc':
      return sorted.sort((a, b) => new Date(b.created_at || 0).getTime() - new Date(a.created_at || 0).getTime())
    case 'created-asc':
      return sorted.sort((a, b) => new Date(a.created_at || 0).getTime() - new Date(b.created_at || 0).getTime())
    default:
      return sorted
  }
})

const isStreamPlaying = (stream: InternetStream) => {
  return playerState.currentSource?.type === 'stream' && 
         playerState.currentSource?.url === stream.url &&
         playerState.isPlaying
}

const fetchStreams = async () => {
  try {
    loading.value = true
    const response = await musicAPI.listStreams()
    streams.value = response.data
  } catch (err: any) {
    console.error('Error fetching streams:', err)
    error('Failed to load streams', err?.response?.data?.error || err?.message)
  } finally {
    loading.value = false
  }
}

const resetForm = () => {
  formData.value = { name: '', url: '', genre: '', description: '' }
}

const closeForm = () => {
  showAddForm.value = false
  editingStream.value = null
  resetForm()
}

const submitForm = async () => {
  if (!formData.value.name || !formData.value.url) return

  saving.value = true
  try {
    if (editingStream.value) {
      await musicAPI.updateStream(editingStream.value.id, {
        name: formData.value.name,
        url: formData.value.url,
        genre: formData.value.genre || undefined,
        description: formData.value.description || undefined,
      })
      success('Stream updated', `"${formData.value.name}" has been updated`)
    } else {
      await musicAPI.createStream({
        name: formData.value.name,
        url: formData.value.url,
        genre: formData.value.genre || undefined,
        description: formData.value.description || undefined,
      })
      success('Stream added', `"${formData.value.name}" has been added to your streams`)
    }
    closeForm()
    await fetchStreams()
  } catch (err: any) {
    console.error('Error saving stream:', err)
    error('Failed to save stream', err?.response?.data?.error || err?.message)
  } finally {
    saving.value = false
  }
}

const editStream = (stream: InternetStream) => {
  editingStream.value = stream
  formData.value = {
    name: stream.name,
    url: stream.url,
    genre: stream.genre || '',
    description: stream.description || '',
  }
}

const confirmDelete = async (stream: InternetStream) => {
  const confirmed = await confirm({
    title: 'Delete Stream',
    message: `Delete "${stream.name}"? This cannot be undone.`,
    confirmText: 'Delete',
    cancelText: 'Cancel',
    variant: 'danger'
  })
  if (!confirmed) return

  try {
    await musicAPI.deleteStream(stream.id)
    success('Stream deleted', `"${stream.name}" has been removed`)
    await fetchStreams()
  } catch (err: any) {
    console.error('Error deleting stream:', err)
    error('Failed to delete stream', err?.response?.data?.error || err?.message)
  }
}

const playStream = (stream: InternetStream) => {
  playInternetStream({ 
    title: stream.name, 
    url: stream.url, 
    genre: stream.genre 
  })
}

const openStreamDetail = async (stream: InternetStream) => {
  // simply set selected stream; if later we need to fetch more details we can
  selectedStream.value = stream
}

const closeStreamDetail = () => {
  selectedStream.value = null
  // refresh list in case changes were made
  fetchStreams()
}

const openContextMenu = (stream: InternetStream, event: MouseEvent) => {
  if (streamContextMenu.value) streamContextMenu.value.open(stream, event)
}

const copyStreamUrl = async (stream: InternetStream) => {
  try {
    if (navigator && navigator.clipboard && stream.url) {
      await navigator.clipboard.writeText(stream.url)
      success('Copied', 'Stream URL copied to clipboard')
    } else {
      // Fallback: create a temporary input
      const input = document.createElement('input')
      input.value = stream.url || ''
      document.body.appendChild(input)
      input.select()
      document.execCommand('copy')
      document.body.removeChild(input)
      success('Copied', 'Stream URL copied to clipboard')
    }
  } catch (err: any) {
    console.error('Failed to copy URL', err)
    error('Failed to copy URL')
  }
}

const truncateUrl = (url: string) => {
  try {
    const parsed = new URL(url)
    return parsed.hostname + (parsed.pathname !== '/' ? parsed.pathname.slice(0, 20) + '...' : '')
  } catch {
    return url.slice(0, 40) + '...'
  }
}

onMounted(() => {
  fetchStreams()
})
</script>

<style scoped>
.streams-tab {
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
  flex: 1;
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

/* Buttons */
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
  box-shadow: 0 4px 15px var(--accent-muted);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px var(--accent-muted);
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

.modal-content {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-xl);
  width: 100%;
  max-width: 500px;
  box-shadow: var(--shadow-xl);
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
  border-radius: var(--radius-sm);
  transition: all var(--transition-base);
}

.modal-close:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.stream-form {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.form-group input {
  padding: 12px 14px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-color);
  font-size: 14px;
  transition: all var(--transition-base);
}

.form-group input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--accent-muted);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}

.form-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 8px;
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

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: scale(0.95) translateY(20px);
}

/* Loading State */
.loading-state {
  padding: 20px 0;
}

.skeleton-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.skeleton-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  background: var(--surface-color);
  border-radius: var(--radius-lg);
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

.skeleton-play {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  flex-shrink: 0;
}

.skeleton-content {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-title {
  height: 20px;
  width: 60%;
  border-radius: var(--radius-sm);
}

.skeleton-subtitle {
  height: 14px;
  width: 40%;
  border-radius: var(--radius-sm);
}

.skeleton-actions {
  width: 80px;
  height: 36px;
  border-radius: var(--radius-md);
}

@keyframes shimmer {
  0% { background-position: -200% 0; }
  100% { background-position: 200% 0; }
}

/* Streams Grid */
.streams-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.stream-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px 20px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  transition: all var(--transition-base);
}

.stream-card:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
  transform: translateX(4px);
}

.stream-card.is-playing {
  border-color: var(--primary-color);
  background: linear-gradient(135deg, var(--accent-muted), var(--surface-color));
}

.stream-play {
  width: 48px;
  height: 48px;
  border-radius: 50%;
  border: none;
  background: linear-gradient(135deg, var(--primary-color), var(--accent-color));
  color: white;
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  transition: all var(--transition-base);
  box-shadow: 0 4px 15px var(--accent-muted);
}

.stream-play:hover {
  transform: scale(1.1);
  box-shadow: 0 6px 20px var(--accent-muted);
}

.play-icon.playing {
  display: flex;
  align-items: flex-end;
  gap: 3px;
  height: 16px;
}

.play-icon.playing .bar {
  width: 3px;
  background: white;
  border-radius: 2px;
  animation: soundBar 0.5s ease infinite alternate;
}

.play-icon.playing .bar:nth-child(1) { height: 60%; animation-delay: 0s; }
.play-icon.playing .bar:nth-child(2) { height: 100%; animation-delay: 0.2s; }
.play-icon.playing .bar:nth-child(3) { height: 40%; animation-delay: 0.4s; }

@keyframes soundBar {
  0% { height: 30%; }
  100% { height: 100%; }
}

.stream-info {
  flex: 1;
  min-width: 0;
}

.stream-name {
  margin: 0 0 4px 0;
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-color);
}

.stream-description {
  margin: 0 0 8px 0;
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.stream-meta {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.stream-genre {
  padding: 4px 10px;
  background: var(--accent-muted);
  color: var(--accent-color);
  border-radius: var(--radius-full);
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.stream-url {
  font-size: 12px;
  color: var(--text-tertiary);
  font-family: monospace;
}

.stream-actions {
  display: flex;
  gap: 8px;
}

.btn-icon {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-md);
  border: 1px solid var(--border-color);
  background: transparent;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 14px;
  transition: all var(--transition-base);
}

.btn-icon:hover {
  background: var(--surface-muted);
  border-color: var(--border-hover);
}

.btn-icon.danger:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
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
  margin: 0 0 24px 0;
  color: var(--text-secondary);
}

/* Responsive */
@media (max-width: 768px) {
  .header {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
  }

  .header-actions {
    flex-direction: column;
    align-items: stretch;
    gap: 12px;
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

  .form-row {
    grid-template-columns: 1fr;
  }

  .stream-card {
    flex-direction: column;
    align-items: center;
    text-align: center;
    padding: 24px 16px;
    gap: 16px;
  }

  .stream-play {
    width: 64px;
    height: 64px;
  }

  .stream-info {
    width: 100%;
  }

  .stream-meta {
    justify-content: center;
  }

  .stream-actions {
    width: 100%;
    justify-content: center;
    border-top: 1px solid var(--border-color);
    padding-top: 16px;
  }

  .btn-icon {
    flex: 1;
    height: 44px;
  }

  .modal-content {
    max-width: 100%;
    margin: 0;
    border-radius: 20px 20px 0 0;
    position: fixed;
    bottom: 0;
  }
}
</style>
