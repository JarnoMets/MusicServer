<template>
  <div class="downloader-tab">
    <div class="header">
      <h2>YouTube Downloader</h2>
    </div>

    <!-- Saved Playlists Section -->
    <div class="saved-playlists-section">
      <div class="form-card">
        <div class="section-header">
          <h3>Saved YouTube Playlists</h3>
          <button class="btn btn-small btn-primary" @click="showAddPlaylistModal = true">
            <Icon name="plus" :size="16" />
            Add Playlist
          </button>
        </div>
        
        <div v-if="loadingPlaylists" class="loading">Loading playlists...</div>
        <div v-else-if="savedPlaylists.length === 0" class="empty">
          <p>No saved playlists. Add a YouTube playlist to enable auto-download.</p>
        </div>
        <div v-else class="playlists-grid">
          <div v-for="playlist in savedPlaylists" :key="playlist.id" class="playlist-card">
            <div class="playlist-info">
              <div class="playlist-name">{{ playlist.name }}</div>
              <div class="playlist-url">{{ playlist.url }}</div>
              <div v-if="playlist.description" class="playlist-desc">{{ playlist.description }}</div>
              <div class="playlist-meta">
                <span v-if="playlist.lastSyncedAt" class="last-sync">
                  Last synced: {{ formatDate(playlist.lastSyncedAt) }}
                </span>
                <span v-else class="last-sync never">Never synced</span>
              </div>
            </div>
            <div class="playlist-actions">
              <label class="auto-download-toggle" :title="playlist.autoDownload ? 'Auto-download enabled' : 'Auto-download disabled'">
                <input 
                  type="checkbox" 
                  :checked="playlist.autoDownload"
                  @change="toggleAutoDownload(playlist)"
                />
                <span class="toggle-label">Auto</span>
              </label>
              <button class="btn-icon" @click="syncPlaylist(playlist)" title="Sync now">
                <Icon name="refresh-cw" :size="16" />
              </button>
              <button class="btn-icon btn-danger" @click="deletePlaylist(playlist)" title="Delete">
                <Icon name="trash" :size="16" />
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Auto-Download Status Section -->
    <div class="auto-download-section">
      <div class="form-card">
        <div class="section-header">
          <h3>Auto-Download</h3>
          <div class="auto-download-controls">
            <button 
              v-if="!autoDownloadStatus?.is_running"
              class="btn btn-small btn-primary" 
              @click="triggerAutoDownload"
              :disabled="!autoDownloadConfig?.enabled"
            >
              <Icon name="play" :size="16" />
              Run Now
            </button>
            <button 
              v-else
              class="btn btn-small btn-danger" 
              @click="stopAutoDownload"
            >
              <Icon name="x" :size="16" />
              Stop
            </button>
          </div>
        </div>
        
        <div class="auto-download-grid">
          <div class="config-item">
            <label class="toggle-switch">
              <input 
                type="checkbox" 
                :checked="autoDownloadConfig?.enabled"
                @change="updateAutoDownloadEnabled"
              />
              <span class="slider"></span>
            </label>
            <span class="config-label">Enable Auto-Download</span>
          </div>
          
          <div class="config-item">
            <label>Check Interval (minutes):</label>
            <input 
              type="number" 
              :value="autoDownloadConfig?.check_interval_minutes"
              @change="updateCheckInterval"
              min="5"
              max="1440"
            />
          </div>
          
          <div class="config-item">
            <label>Max Concurrent:</label>
            <input 
              type="number" 
              :value="autoDownloadConfig?.max_concurrent_downloads"
              @change="updateMaxConcurrent"
              min="1"
              max="5"
            />
          </div>
        </div>
        
        <div v-if="autoDownloadStatus?.is_running" class="auto-download-progress">
          <div class="progress-info">
            <Icon name="refresh-cw" :size="16" class="spinning" />
            <span>Downloading: {{ autoDownloadStatus.current_playlist || 'Starting...' }}</span>
          </div>
          <div class="progress-stats">
            <span>Completed: {{ autoDownloadStatus.downloads_completed }}</span>
            <span>Skipped: {{ autoDownloadStatus.downloads_skipped }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Download Form -->
    <div class="download-form-section">
      <div class="form-card">
        <h3>Manual Download</h3>
        <form @submit.prevent="startDownload" class="download-form">
          <div class="form-group">
            <label>YouTube URL:</label>
            <input
              v-model="downloadUrl"
              type="url"
              required
              placeholder="https://www.youtube.com/playlist?list=... or https://www.youtube.com/watch?v=..."
              :disabled="isDownloading"
            />
          </div>

          <div class="form-row">
            <div class="form-group">
              <label for="audio-quality">Audio Quality:</label>
              <select id="audio-quality" v-model="options.audio_quality" :disabled="isDownloading">
                <option value="best">Best</option>
                <option value="192">192 kbps</option>
                <option value="128">128 kbps</option>
              </select>
            </div>
            <div class="form-group">
              <label for="max-concurrent">Max Concurrent:</label>
              <input
                id="max-concurrent"
                v-model.number="options.max_concurrent"
                type="number"
                min="1"
                max="10"
                :disabled="isDownloading"
              />
            </div>
            <div class="form-group">
              <label for="limit">Limit (optional):</label>
              <input
                id="limit"
                v-model.number="options.limit"
                type="number"
                min="1"
                :disabled="isDownloading"
                placeholder="Leave empty for no limit"
              />
            </div>
          </div>

          <button type="submit" class="btn btn-primary" :disabled="isDownloading">
            {{ isDownloading ? 'Downloading...' : 'Start Download' }}
          </button>
        </form>

        <!-- Progress Section -->
        <div v-if="isDownloading" class="progress-section">
          <div class="progress-header">
            <h4>Download Progress</h4>
            <button @click="cancelDownload" class="btn btn-danger">Cancel</button>
          </div>

          <div class="progress-info">
            <p v-if="currentProgress.currentItem">
              Current: {{ currentProgress.currentItem.title }}
            </p>
            <p v-if="currentProgress.message" class="status-message">
              {{ currentProgress.message }}
            </p>
          </div>

          <div class="progress-bar-container">
            <div class="progress-bar">
              <div
                class="progress-fill"
                :style="{ width: currentProgress.itemProgress + '%' }"
              ></div>
            </div>
            <span class="progress-percent">{{ currentProgress.itemProgress }}%</span>
          </div>

          <div class="progress-stats">
            <span>
              Items: {{ currentProgress.itemIndex + 1 }} /
              {{ currentProgress.totalItems }}
            </span>
            <span v-if="currentProgress.currentItem?.size">
              Size: {{ formatBytes(currentProgress.currentItem.size) }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Statistics Section -->
    <div v-if="!isDownloading && downloadStats" class="stats-section">
      <div class="stats-card">
        <h3>Download Statistics</h3>
        <div class="stats-grid">
          <div class="stat-item">
            <span class="stat-label">Total Videos Downloaded</span>
            <span class="stat-value">{{ downloadStats.totalDownloads }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Unique Videos</span>
            <span class="stat-value">{{ downloadStats.uniqueVideos }}</span>
          </div>
          <div class="stat-item">
            <span class="stat-label">Total Size</span>
            <span class="stat-value">{{ formatBytes(downloadStats.totalSize) }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Download History -->
    <div class="history-section">
      <h3>Download History</h3>
      <div class="history-container">
        <div v-if="loadingHistory" class="loading">Loading history...</div>
        <div v-else-if="downloadedVideos.length === 0" class="empty">
          <p>No downloads yet.</p>
        </div>
        <table v-else class="history-table">
          <thead>
            <tr>
              <th>Title</th>
              <th>Video ID</th>
              <th>Downloaded</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="video in downloadedVideos" :key="video.videoId" class="history-row">
              <td class="title">{{ video.title }}</td>
              <td class="video-id">{{ video.videoId }}</td>
              <td class="date">{{ formatDate(video.downloadedAt) }}</td>
              <td class="actions">
                <button
                  @click="removeFromHistory(video.videoId)"
                  class="btn-delete"
                  title="Allow re-download"
                >
                  <Icon name="trash" :size="16" />
                </button>
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <!-- Add Playlist Modal -->
    <Teleport to="body">
      <Transition name="modal">
        <div v-if="showAddPlaylistModal" class="modal-overlay" @click="showAddPlaylistModal = false">
          <div class="modal" @click.stop>
            <div class="modal-header">
              <h3>Add YouTube Playlist</h3>
              <button class="modal-close" @click="showAddPlaylistModal = false">
                <Icon name="x" :size="20" />
              </button>
            </div>
            <form class="modal-body" @submit.prevent="addPlaylist">
              <div class="form-group">
                <label>Name:</label>
                <input v-model="newPlaylist.name" type="text" required placeholder="My Playlist" />
              </div>
              <div class="form-group">
                <label>YouTube URL:</label>
                <input v-model="newPlaylist.url" type="url" required placeholder="https://www.youtube.com/playlist?list=..." />
              </div>
              <div class="form-group">
                <label>Description (optional):</label>
                <input v-model="newPlaylist.description" type="text" placeholder="Optional description" />
              </div>
              <div class="form-group checkbox-group">
                <label>
                  <input type="checkbox" v-model="newPlaylist.autoDownload" />
                  <span>Enable auto-download</span>
                </label>
              </div>
              <div class="modal-actions">
                <button type="button" class="btn btn-secondary" @click="showAddPlaylistModal = false">Cancel</button>
                <button type="submit" class="btn btn-primary">Add Playlist</button>
              </div>
            </form>
          </div>
        </div>
      </Transition>
    </Teleport>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { musicAPI } from '../../api/music'
import { useToast } from '../../composables/useToast'
import { useConfirm } from '../../composables/useConfirm'
import Icon from '../../shared/components/Icons.vue'

const { success, error: showError } = useToast()
const { confirm } = useConfirm()

interface DownloadProgress {
  itemIndex: number
  totalItems: number
  itemProgress: number
  currentItem?: {
    title: string
    size?: number
  }
  message?: string
}

interface DownloadedVideo {
  videoId: string
  title: string
  downloadedAt: string
}

interface DownloadStats {
  totalDownloads: number
  uniqueVideos: number
  totalSize: number
}

interface YoutubePlaylist {
  id: string
  name: string
  url: string
  description?: string
  autoDownload: boolean
  lastSyncedAt?: string
}

interface AutoDownloadConfig {
  enabled: boolean
  check_interval_minutes: number
  max_concurrent_downloads: number
  delay_between_downloads_seconds: number
  allowed_start_hour?: number
  allowed_end_hour?: number
}

interface AutoDownloadStatus {
  is_running: boolean
  current_playlist?: string
  downloads_completed: number
  downloads_skipped: number
  downloads_in_progress: number
}

const downloadUrl = ref('')
const isDownloading = ref(false)
const currentProgress = ref<DownloadProgress>({
  itemIndex: 0,
  totalItems: 0,
  itemProgress: 0,
})
const downloadedVideos = ref<DownloadedVideo[]>([])
const downloadStats = ref<DownloadStats | null>(null)
const loadingHistory = ref(false)
const eventSource = ref<EventSource | null>(null)

// Saved playlists state
const savedPlaylists = ref<YoutubePlaylist[]>([])
const loadingPlaylists = ref(false)
const showAddPlaylistModal = ref(false)
const newPlaylist = ref({ name: '', url: '', description: '', autoDownload: false })

// Auto-download state
const autoDownloadConfig = ref<AutoDownloadConfig | null>(null)
const autoDownloadStatus = ref<AutoDownloadStatus | null>(null)
let statusPollInterval: number | null = null

const options = ref({
  audio_quality: 'best',
  max_concurrent: 3,
  limit: undefined as number | undefined,
})

// === Saved Playlists Functions ===
const fetchSavedPlaylists = async () => {
  try {
    loadingPlaylists.value = true
    const response = await musicAPI.listYoutubePlaylists()
    savedPlaylists.value = response.data.map((p: any) => ({
      id: p.id,
      name: p.name,
      url: p.url,
      description: p.description,
      autoDownload: p.autoDownload,
      lastSyncedAt: p.lastSyncedAt,
    }))
  } catch (error) {
    console.error('Error fetching playlists:', error)
  } finally {
    loadingPlaylists.value = false
  }
}

const addPlaylist = async () => {
  if (!newPlaylist.value.name.trim() || !newPlaylist.value.url.trim()) {
    showError('Missing Fields', 'Please enter a name and URL')
    return
  }
  
  try {
    await musicAPI.createYoutubePlaylist({
      name: newPlaylist.value.name,
      url: newPlaylist.value.url,
      description: newPlaylist.value.description || undefined,
      autoDownload: newPlaylist.value.autoDownload,
    })
    success('Playlist Added', 'YouTube playlist saved successfully')
    showAddPlaylistModal.value = false
    newPlaylist.value = { name: '', url: '', description: '', autoDownload: false }
    await fetchSavedPlaylists()
  } catch (error: any) {
    console.error('Error adding playlist:', error)
    if (error.response?.status === 409) {
      showError('Duplicate', 'This playlist URL already exists')
    } else {
      showError('Error', 'Failed to add playlist')
    }
  }
}

const toggleAutoDownload = async (playlist: YoutubePlaylist) => {
  try {
    await musicAPI.updateYoutubePlaylist(playlist.id, {
      autoDownload: !playlist.autoDownload,
    })
    playlist.autoDownload = !playlist.autoDownload
    success('Updated', `Auto-download ${playlist.autoDownload ? 'enabled' : 'disabled'}`)
  } catch (error) {
    console.error('Error toggling auto-download:', error)
    showError('Error', 'Failed to update playlist')
  }
}

const syncPlaylist = async (playlist: YoutubePlaylist) => {
  try {
    const response = await musicAPI.syncYoutubePlaylist(playlist.id)
    success('Sync Started', `Syncing "${playlist.name}"`)
    // Optionally subscribe to progress
    if (response.data?.sessionId) {
      subscribeToProgress(response.data.sessionId)
      isDownloading.value = true
    }
  } catch (error) {
    console.error('Error syncing playlist:', error)
    showError('Error', 'Failed to start sync')
  }
}

const deletePlaylist = async (playlist: YoutubePlaylist) => {
  const confirmed = await confirm({
    title: 'Delete Playlist',
    message: `Delete "${playlist.name}" from saved playlists? This will not delete downloaded files.`,
    confirmText: 'Delete',
    cancelText: 'Cancel',
    variant: 'danger'
  })
  if (!confirmed) return
  
  try {
    await musicAPI.deleteYoutubePlaylist(playlist.id)
    success('Deleted', 'Playlist removed')
    await fetchSavedPlaylists()
  } catch (error) {
    console.error('Error deleting playlist:', error)
    showError('Error', 'Failed to delete playlist')
  }
}

// === Auto-Download Functions ===
const fetchAutoDownloadConfig = async () => {
  try {
    const response = await musicAPI.getAutoDownloadConfig()
    autoDownloadConfig.value = response.data
  } catch (error) {
    console.error('Error fetching auto-download config:', error)
  }
}

const fetchAutoDownloadStatus = async () => {
  try {
    const response = await musicAPI.getAutoDownloadStatus()
    autoDownloadStatus.value = response.data
  } catch (error) {
    console.error('Error fetching auto-download status:', error)
  }
}

const updateAutoDownloadEnabled = async (event: Event) => {
  const target = event.target as HTMLInputElement
  try {
    await musicAPI.updateAutoDownloadConfig({ enabled: target.checked })
    await fetchAutoDownloadConfig()
    success('Updated', `Auto-download ${target.checked ? 'enabled' : 'disabled'}`)
  } catch (error) {
    console.error('Error updating config:', error)
    showError('Error', 'Failed to update configuration')
  }
}

const updateCheckInterval = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const value = parseInt(target.value, 10)
  if (value < 5) return
  try {
    await musicAPI.updateAutoDownloadConfig({ check_interval_minutes: value })
    await fetchAutoDownloadConfig()
  } catch (error) {
    console.error('Error updating config:', error)
  }
}

const updateMaxConcurrent = async (event: Event) => {
  const target = event.target as HTMLInputElement
  const value = parseInt(target.value, 10)
  if (value < 1 || value > 5) return
  try {
    await musicAPI.updateAutoDownloadConfig({ max_concurrent_downloads: value })
    await fetchAutoDownloadConfig()
  } catch (error) {
    console.error('Error updating config:', error)
  }
}

const triggerAutoDownload = async () => {
  try {
    await musicAPI.triggerAutoDownload()
    success('Started', 'Auto-download triggered')
    await fetchAutoDownloadStatus()
  } catch (error: any) {
    console.error('Error triggering auto-download:', error)
    showError('Error', error.response?.data?.error || 'Failed to trigger auto-download')
  }
}

const stopAutoDownload = async () => {
  try {
    await musicAPI.stopAutoDownload()
    success('Stopped', 'Stop signal sent')
    await fetchAutoDownloadStatus()
  } catch (error) {
    console.error('Error stopping auto-download:', error)
    showError('Error', 'Failed to stop auto-download')
  }
}

// === Manual Download Functions ===
const startDownload = async () => {
  if (!downloadUrl.value.trim()) {
    showError('Missing URL', 'Please enter a YouTube URL')
    return
  }

  try {
    isDownloading.value = true
    currentProgress.value = {
      itemIndex: 0,
      totalItems: 0,
      itemProgress: 0,
    }

    const response = await musicAPI.startYoutubeDownload({
      url: downloadUrl.value,
      output_dir: '/music/downloads',
      limit: options.value.limit,
      max_concurrent: options.value.max_concurrent,
      audio_quality: options.value.audio_quality,
    })

    const sessionId = response.data.sessionId
    subscribeToProgress(sessionId)
  } catch (error) {
    console.error('Error starting download:', error)
    showError('Download Failed', 'Failed to start download')
    isDownloading.value = false
  }
}

const subscribeToProgress = (sessionId: string) => {
  try {
    const stream = musicAPI.getYoutubeProgressStream(sessionId)

    stream.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)

        if (data.type === 'progress') {
          currentProgress.value = {
            itemIndex: data.itemIndex,
            totalItems: data.totalItems,
            itemProgress: data.progress,
            currentItem: data.currentItem,
            message: data.message,
          }
        } else if (data.type === 'complete') {
          isDownloading.value = false
          closeEventSource()
          downloadUrl.value = ''
          success('Download Complete', 'All files have been downloaded successfully')
          fetchDownloadedVideos()
          fetchDownloadStats()
        } else if (data.type === 'error') {
          isDownloading.value = false
          closeEventSource()
          showError('Download Error', data.message || 'An error occurred during download')
        }
      } catch (e) {
        console.error('Error parsing progress data:', e)
      }
    }

    stream.onerror = () => {
      isDownloading.value = false
      closeEventSource()
      console.error('SSE connection error')
    }

    eventSource.value = stream
  } catch (error) {
    console.error('Error subscribing to progress:', error)
    isDownloading.value = false
  }
}

const cancelDownload = async () => {
  const confirmed = await confirm({
    title: 'Cancel Download',
    message: 'Are you sure you want to cancel the current download?',
    confirmText: 'Cancel Download',
    cancelText: 'Keep Downloading',
    variant: 'warning'
  })
  if (!confirmed) return

  try {
    const sessionId = currentProgress.value.currentItem?.title
    if (sessionId) {
      await musicAPI.cancelYoutubeDownload(sessionId)
    }
    closeEventSource()
    isDownloading.value = false
  } catch (error) {
    console.error('Error canceling download:', error)
  }
}

const closeEventSource = () => {
  if (eventSource.value) {
    eventSource.value.close()
    eventSource.value = null
  }
}

const fetchDownloadedVideos = async () => {
  try {
    loadingHistory.value = true
    const response = await musicAPI.getDownloadedVideos()
    downloadedVideos.value = response.data.map((video: any) => ({
      videoId: video.video_id,
      title: video.title,
      downloadedAt: video.downloaded_at,
    }))
  } catch (error) {
    console.error('Error fetching download history:', error)
  } finally {
    loadingHistory.value = false
  }
}

const fetchDownloadStats = async () => {
  try {
    const response = await musicAPI.getDownloadStats()
    downloadStats.value = {
      totalDownloads: response.data.totalDownloads,
      uniqueVideos: response.data.uniqueVideos,
      totalSize: response.data.totalSize,
    }
  } catch (error) {
    console.error('Error fetching download stats:', error)
  }
}

const removeFromHistory = async (videoId: string) => {
  const confirmed = await confirm({
    title: 'Remove from History',
    message: 'Remove this video from download history? This will allow you to re-download it.',
    confirmText: 'Remove',
    cancelText: 'Cancel',
    variant: 'warning'
  })
  if (!confirmed) return

  try {
    await musicAPI.removeDownloadRecord(videoId)
    success('Removed', 'Video removed from history')
    await fetchDownloadedVideos()
    await fetchDownloadStats()
  } catch (error) {
    console.error('Error removing download record:', error)
    showError('Error', 'Failed to remove record')
  }
}

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}

const formatDate = (dateString: string) => {
  return new Date(dateString).toLocaleDateString()
}

onMounted(() => {
  fetchDownloadedVideos()
  fetchDownloadStats()
  fetchSavedPlaylists()
  fetchAutoDownloadConfig()
  fetchAutoDownloadStatus()
  // Poll status every 5 seconds when auto-download might be running
  statusPollInterval = window.setInterval(fetchAutoDownloadStatus, 5000)
})

onUnmounted(() => {
  closeEventSource()
  if (statusPollInterval) {
    clearInterval(statusPollInterval)
  }
})
</script>

<style scoped>
.downloader-tab {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.header h2 {
  margin: 0;
  font-size: 28px;
  font-weight: 700;
  color: var(--text-color);
  display: flex;
  align-items: center;
  gap: 12px;
}

/* Download Form Section */
.download-form-section {
  margin-bottom: 8px;
}

.form-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 28px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.form-card h3 {
  margin-top: 0;
  margin-bottom: 24px;
  color: var(--text-color);
  font-size: 18px;
  font-weight: 600;
}

.download-form {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.form-group {
  display: flex;
  flex-direction: column;
}

.form-group label {
  margin-bottom: 10px;
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 13px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.form-group input[type='url'],
.form-group input[type='text'],
.form-group input[type='number'] {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 14px 16px;
  color: var(--text-color);
  font-family: inherit;
  font-size: 15px;
  transition: all 0.2s ease;
}

.form-group input:hover {
  border-color: var(--primary-color);
}

.form-group input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.form-group input:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-group select {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 14px 16px;
  color: var(--text-color);
  font-family: inherit;
  font-size: 15px;
  transition: all 0.2s ease;
  cursor: pointer;
}

.form-group select:hover {
  border-color: var(--primary-color);
}

.form-group select:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.form-group select:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.form-row {
  display: flex;
  gap: 20px;
  flex-wrap: wrap;
}

.form-row .form-group {
  flex: 1;
  min-width: 180px;
}

.btn {
  padding: 14px 24px;
  border: none;
  border-radius: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 15px;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.btn-primary {
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-dark) 100%);
  color: #fff;
  box-shadow: 0 4px 16px var(--primary-glow);
  align-self: flex-start;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 24px var(--primary-glow);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.btn-danger {
  background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
  color: white;
  box-shadow: 0 4px 16px rgba(239, 68, 68, 0.3);
}

.btn-danger:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 24px rgba(239, 68, 68, 0.4);
}

/* Progress Section */
.progress-section {
  margin-top: 28px;
  padding-top: 28px;
  border-top: 1px solid var(--border-color);
}

.progress-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.progress-header h4 {
  margin: 0;
  color: var(--text-color);
  font-size: 16px;
  font-weight: 600;
}

.progress-info {
  margin-bottom: 20px;
  font-size: 14px;
  color: var(--text-color);
  background: var(--background-elevated);
  border-radius: 12px;
  padding: 16px;
}

.progress-info p {
  margin: 6px 0;
}

.status-message {
  color: var(--text-secondary);
  font-size: 13px;
  font-style: italic;
}

.progress-bar-container {
  display: flex;
  gap: 16px;
  align-items: center;
  margin-bottom: 20px;
}

.progress-bar {
  flex: 1;
  height: 10px;
  background: var(--background-elevated);
  border-radius: 999px;
  overflow: hidden;
  border: 1px solid var(--border-color);
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--accent-color));
  transition: width 0.3s ease;
  position: relative;
}

.progress-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent);
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

.progress-percent {
  min-width: 50px;
  text-align: right;
  font-size: 14px;
  font-weight: 600;
  color: var(--primary-light);
}

.progress-stats {
  display: flex;
  justify-content: space-between;
  font-size: 13px;
  color: var(--text-secondary);
}

/* Statistics Section */
.stats-section {
  margin-bottom: 8px;
}

.stats-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 24px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.stats-card h3 {
  margin: 0 0 24px 0;
  color: var(--text-color);
  font-size: 18px;
  font-weight: 600;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
}

.stat-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  padding: 20px;
  background: var(--background-elevated);
  border-radius: 12px;
  border: 1px solid var(--border-color);
  transition: all 0.2s ease;
}

.stat-item:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.stat-label {
  font-size: 12px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.1em;
  margin-bottom: 8px;
  font-weight: 600;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: var(--primary-light);
}

/* History Section */
.history-section h3 {
  margin: 0 0 20px 0;
  color: var(--text-color);
  font-size: 18px;
  font-weight: 600;
}

.history-container {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.loading,
.empty {
  text-align: center;
  padding: 48px;
  color: var(--text-secondary);
}

.loading::before {
  content: '';
  display: block;
  width: 32px;
  height: 32px;
  margin: 0 auto 16px;
  border: 3px solid var(--border-color);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.empty p {
  margin: 0;
}

.history-table {
  width: 100%;
  border-collapse: collapse;
}

.history-table thead {
  background: var(--background-elevated);
  border-bottom: 2px solid var(--border-color);
}

.history-table th {
  padding: 16px 20px;
  text-align: left;
  font-weight: 700;
  color: var(--text-tertiary);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
}

.history-row {
  border-bottom: 1px solid var(--border-color);
  transition: background 0.2s ease;
}

.history-row:hover {
  background: var(--primary-glow);
}

.history-row td {
  padding: 16px 20px;
  color: var(--text-color);
  font-size: 14px;
}

.title {
  font-weight: 600;
}

.video-id {
  color: var(--text-tertiary);
  font-size: 12px;
  font-family: 'JetBrains Mono', monospace;
  background: var(--background-elevated);
  padding: 4px 8px;
  border-radius: 6px;
}

.date {
  color: var(--text-secondary);
  font-size: 13px;
}

.actions {
  text-align: center;
}

.btn-delete {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  cursor: pointer;
  font-size: 16px;
  padding: 8px 12px;
  transition: all 0.2s ease;
}

.btn-delete:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  transform: scale(1.08);
}

/* Section Header */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 24px;
}

.section-header h3 {
  margin: 0;
}

/* Saved Playlists Section */
.playlists-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.playlist-card {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  transition: all 0.2s ease;
}

.playlist-card:hover {
  border-color: var(--primary-color);
}

.playlist-info {
  flex: 1;
  min-width: 0;
}

.playlist-name {
  font-weight: 600;
  color: var(--text-color);
  margin-bottom: 4px;
}

.playlist-url {
  font-size: 12px;
  color: var(--text-tertiary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  max-width: 400px;
}

.playlist-desc {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 4px;
}

.playlist-meta {
  margin-top: 8px;
  font-size: 12px;
}

.last-sync {
  color: var(--text-tertiary);
}

.last-sync.never {
  color: var(--warning-color);
}

.playlist-actions {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-left: 16px;
}

.auto-download-toggle {
  display: flex;
  align-items: center;
  gap: 6px;
  cursor: pointer;
  padding: 6px 12px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  font-size: 12px;
  transition: all 0.2s ease;
}

.auto-download-toggle:has(input:checked) {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
}

.auto-download-toggle input {
  accent-color: var(--primary-color);
}

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 36px;
  height: 36px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.btn-icon:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
}

.btn-icon.btn-danger:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #f87171;
}

.btn-small {
  padding: 8px 16px;
  font-size: 13px;
}

/* Auto-Download Section */
.auto-download-grid {
  display: flex;
  flex-wrap: wrap;
  gap: 24px;
  align-items: center;
  margin-bottom: 16px;
}

.config-item {
  display: flex;
  align-items: center;
  gap: 10px;
}

.config-item label {
  font-size: 13px;
  color: var(--text-secondary);
}

.config-item input[type="number"] {
  width: 80px;
  padding: 8px 12px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 8px;
  color: var(--text-color);
  font-size: 14px;
}

.config-label {
  font-weight: 500;
  color: var(--text-color);
}

/* Toggle Switch */
.toggle-switch {
  position: relative;
  display: inline-block;
  width: 48px;
  height: 26px;
}

.toggle-switch input {
  opacity: 0;
  width: 0;
  height: 0;
}

.slider {
  position: absolute;
  cursor: pointer;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: var(--background-elevated);
  border: 1px solid var(--border-color);
  transition: 0.3s;
  border-radius: 26px;
}

.slider:before {
  position: absolute;
  content: "";
  height: 20px;
  width: 20px;
  left: 2px;
  bottom: 2px;
  background-color: var(--text-tertiary);
  transition: 0.3s;
  border-radius: 50%;
}

.toggle-switch input:checked + .slider {
  background-color: var(--primary-color);
  border-color: var(--primary-color);
}

.toggle-switch input:checked + .slider:before {
  transform: translateX(22px);
  background-color: white;
}

.auto-download-progress {
  padding: 16px;
  background: var(--primary-glow);
  border: 1px solid var(--primary-color);
  border-radius: 12px;
  margin-top: 16px;
}

.auto-download-progress .progress-info {
  display: flex;
  align-items: center;
  gap: 10px;
  margin-bottom: 8px;
  font-weight: 500;
}

.auto-download-progress .progress-stats {
  display: flex;
  gap: 20px;
  font-size: 13px;
  color: var(--text-secondary);
}

.spinning {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
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
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 24px 0;
}

.modal-header h3 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 700;
}

.modal-close {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.modal-close:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.modal-body {
  padding: 24px;
}

.modal-body .form-group {
  margin-bottom: 16px;
}

.modal-body .form-group label {
  margin-bottom: 8px;
}

.modal-body .form-group input[type="text"],
.modal-body .form-group input[type="url"] {
  width: 100%;
  padding: 12px 16px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-color);
  font-size: 14px;
}

.modal-body .form-group input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.checkbox-group {
  flex-direction: row !important;
}

.checkbox-group label {
  display: flex;
  align-items: center;
  gap: 8px;
  cursor: pointer;
}

.checkbox-group input[type="checkbox"] {
  accent-color: var(--primary-color);
  width: 18px;
  height: 18px;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 24px;
}

.btn-secondary {
  background: var(--background-elevated);
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--surface-hover);
}

/* Modal Transitions */
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
  transform: scale(0.95) translateY(-20px);
}

@media (max-width: 768px) {
  .form-row {
    flex-direction: column;
  }

  .stats-grid {
    grid-template-columns: 1fr;
  }

  .history-table th,
  .history-table td {
    padding: 12px 16px;
  }

  .playlist-card {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .playlist-actions {
    margin-left: 0;
    width: 100%;
    justify-content: flex-start;
  }

  .auto-download-grid {
    flex-direction: column;
    align-items: flex-start;
  }
}
</style>
