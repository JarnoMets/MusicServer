<template>
  <div class="downloader-tab">
    <div class="header">
      <h2>YouTube Downloader</h2>
    </div>

    <!-- Tab Navigation -->
    <div class="tabs-nav">
      <button 
        class="tab-btn" 
        :class="{ active: activeTab === 'status' }" 
        @click="activeTab = 'status'"
      >
        <div class="tab-indicator"></div>
        <Icon name="activity" :size="18" />
        <span>Status</span>
      </button>
      <button 
        class="tab-btn" 
        :class="{ active: activeTab === 'download' }" 
        @click="activeTab = 'download'"
      >
        <div class="tab-indicator"></div>
        <Icon name="download" :size="18" />
        <span>Manual Download</span>
      </button>
      <button 
        class="tab-btn" 
        :class="{ active: activeTab === 'playlists' }" 
        @click="activeTab = 'playlists'"
      >
        <div class="tab-indicator"></div>
        <Icon name="list" :size="18" />
        <span>Playlists</span>
      </button>
      <button 
        class="tab-btn" 
        :class="{ active: activeTab === 'config' }" 
        @click="activeTab = 'config'"
      >
        <div class="tab-indicator"></div>
        <Icon name="settings" :size="18" />
        <span>Maintenance</span>
      </button>
      <button 
        class="tab-btn" 
        :class="{ active: activeTab === 'history' }" 
        @click="activeTab = 'history'"
      >
        <div class="tab-indicator"></div>
        <Icon name="history" :size="18" />
        <span>History</span>
      </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content">
      <!-- Status Tab -->
      <div v-if="activeTab === 'status'" class="tab-pane fade-in">
        <div class="status-layout">
          <div class="main-status-area">
            <!-- Active Process Card -->
            <div class="glass-card status-hero" :class="{ 'execution-active': autoDownloadStatus?.is_running || isDownloading }">
              <div class="hero-top">
                <div class="hero-info">
                  <h3>System Engine</h3>
                  <p v-if="autoDownloadStatus?.is_running">Auto-downloader is currently processing playlists.</p>
                  <p v-else-if="isDownloading">Manual download session is active.</p>
                  <p v-else>All systems stand-by. Ready for new tasks.</p>
                </div>
                <div class="engine-state">
                  <div class="pulse-ring" v-if="autoDownloadStatus?.is_running || isDownloading"></div>
                  <div class="state-label" :class="{ active: autoDownloadStatus?.is_running || isDownloading }">
                    {{ (autoDownloadStatus?.is_running || isDownloading) ? 'Active' : 'Idle' }}
                  </div>
                </div>
              </div>

              <div class="hero-actions">
                <button 
                  v-if="!autoDownloadStatus?.is_running"
                  class="btn btn-primary" 
                  @click="triggerAutoDownload"
                  :disabled="!autoDownloadConfig?.enabled"
                >
                  <Icon name="play" :size="20" />
                  Run Auto-sync Now
                </button>
                <button 
                  v-else
                  class="btn btn-danger" 
                  @click="stopAutoDownload"
                >
                  <Icon name="square" :size="20" />
                  Abort Operation
                </button>

                <button 
                  class="btn btn-secondary-blur" 
                  @click="processDownloads"
                  :disabled="isProcessingDownloads"
                >
                  <Icon name="refresh-cw" :size="18" :class="{ spinning: isProcessingDownloads }" />
                  Rescan Local Storage
                </button>
              </div>

              <!-- Animated progress if active -->
              <div v-if="autoDownloadStatus?.is_running" class="execution-details">
                <div class="exec-playlist">
                   <Icon name="youtube" :size="20" color="#ff0000" />
                   <span class="p-name">{{ autoDownloadStatus.current_playlist || 'Negotiating with YouTube...' }}</span>
                </div>
                
                <div class="exec-stats-grid">
                  <div class="exec-stat">
                    <span class="val">{{ autoDownloadStatus.downloads_completed }}</span>
                    <span class="lbl">Succeeded</span>
                  </div>
                  <div class="exec-stat">
                    <span class="val">{{ autoDownloadStatus.downloads_in_progress }}</span>
                    <span class="lbl">Concurrent</span>
                  </div>
                  <div class="exec-stat">
                    <span class="val">{{ autoDownloadStatus.downloads_skipped }}</span>
                    <span class="lbl">Redundant</span>
                  </div>
                </div>
              </div>

              <div v-if="isDownloading" class="manual-execution-details">
                <div class="divider"></div>
                <div class="manual-info">
                   <div class="info-text">
                      <span class="title">Manual Task: {{ currentProgress.currentItem?.title || 'Unknown' }}</span>
                      <span class="subtext">{{ currentProgress.message }}</span>
                   </div>
                   <div class="progress-info">
                      <span class="percent">{{ currentProgress.itemProgress }}%</span>
                      <span class="counter">{{ currentProgress.itemIndex + 1 }} / {{ currentProgress.totalItems }}</span>
                   </div>
                </div>
                <div class="modern-progress-bar">
                   <div class="progress-fill" :style="{ width: currentProgress.itemProgress + '%' }"></div>
                </div>
              </div>
            </div>

            <!-- Stats Overview -->
            <div class="quick-stats-row">
              <div class="mini-card-stat">
                <div class="icon-wrap"><Icon name="database" :size="24" /></div>
                <div class="data">
                   <div class="num">{{ downloadStats?.uniqueVideos || 0 }}</div>
                   <div class="label">Cached Index</div>
                </div>
              </div>
              <div class="mini-card-stat">
                <div class="icon-wrap color-blue"><Icon name="download-cloud" :size="24" /></div>
                <div class="data">
                   <div class="num">{{ downloadStats?.totalDownloads || 0 }}</div>
                   <div class="label">Total Transfers</div>
                </div>
              </div>
              <div class="mini-card-stat">
                <div class="icon-wrap color-green"><Icon name="hard-drive" :size="24" /></div>
                <div class="data">
                   <div class="num">{{ formatBytes(downloadStats?.totalSize || 0) }}</div>
                   <div class="label">Data Footprint</div>
                </div>
              </div>
            </div>
          </div>

          <div class="status-sidebar-info">
             <div class="panel-card config-overview">
                <h4>Engine Config</h4>
                <div class="config-mini-list">
                   <div class="c-item">
                      <span class="c-label">Automation</span>
                      <span class="c-value" :class="autoDownloadConfig?.enabled ? 'text-success' : 'text-danger'">
                        {{ autoDownloadConfig?.enabled ? 'Enabled' : 'Paused' }}
                      </span>
                   </div>
                   <div class="c-item">
                      <span class="c-label">Scan Delay</span>
                      <span class="c-value">{{ autoDownloadConfig?.check_interval_minutes }} min</span>
                   </div>
                   <div class="c-item">
                      <span class="c-label">Worker Limit</span>
                      <span class="c-value">{{ autoDownloadConfig?.max_concurrent_downloads }} threads</span>
                   </div>
                </div>
                <button class="btn-text-primary" @click="activeTab = 'config'">Modify Settings</button>
             </div>

             <div class="panel-card recent-activity">
                <h4>Recent Syncs</h4>
                <div class="history-mini">
                   <div v-for="video in downloadedVideos.slice(0, 5)" :key="video.videoId" class="mini-v-item">
                      <div class="v-title">{{ video.title }}</div>
                      <div class="v-date">{{ formatDate(video.downloadedAt) }}</div>
                   </div>
                </div>
                <button class="btn-text-primary" @click="activeTab = 'history'">Full History</button>
             </div>
          </div>
        </div>
      </div>

      <!-- Manual Download Tab -->
      <div v-if="activeTab === 'download'" class="tab-pane fade-in">
        <div class="download-container-focused">
          <div class="download-card">
            <div class="card-header">
              <h3>Initiate Manual Transfer</h3>
              <p>Download a specific video or entire playlist directly into your library.</p>
            </div>
            
            <form @submit.prevent="startDownload" class="premium-form">
              <div class="input-glow-group">
                <label>YouTube Link or Playlist URL</label>
                <div class="glow-input-wrapper">
                  <Icon name="link" :size="20" class="field-icon" />
                  <input
                    v-model="downloadUrl"
                    type="url"
                    required
                    placeholder="https://www.youtube.com/watch?v=..."
                    :disabled="isDownloading"
                  />
                </div>
              </div>

              <div class="options-grid">
                <div class="option-col">
                  <label>Audio Fidelity</label>
                  <select v-model="options.audio_quality" :disabled="isDownloading">
                    <option value="best">Lossless / Best Available</option>
                    <option value="192">High Fidelity (192kbps)</option>
                    <option value="128">Standard (128kbps)</option>
                  </select>
                </div>
                <div class="option-col">
                  <label>Parallel Workers</label>
                  <input
                    v-model.number="options.max_concurrent"
                    type="number"
                    min="1"
                    max="10"
                    :disabled="isDownloading"
                  />
                </div>
                <div class="option-col">
                  <label>Maximum Files</label>
                  <input
                    v-model.number="options.limit"
                    type="number"
                    min="1"
                    :disabled="isDownloading"
                    placeholder="Unlimited"
                  />
                </div>
              </div>

              <div class="form-footer">
                 <button type="submit" class="btn btn-primary btn-xl" :disabled="isDownloading || !downloadUrl">
                   <div v-if="isDownloading" class="btn-loading">
                      <Icon name="refresh-cw" :size="22" class="spinning" />
                      <span>Transferring...</span>
                   </div>
                   <div v-else class="btn-content">
                      <Icon name="download" :size="22" />
                      <span>Start Transfer</span>
                   </div>
                 </button>
                 
                 <button v-if="isDownloading" type="button" @click="cancelDownload" class="btn btn-outline-danger">
                   Cancel Current Session
                 </button>
              </div>
            </form>
          </div>

          <!-- Progress Visualization Overlay -->
          <Transition name="slide-up">
            <div v-if="isDownloading" class="progress-panel-modern">
              <div class="p-header">
                 <h4>Transfer Protocol Active</h4>
                 <div class="tag-live">Live</div>
              </div>
              
              <div class="p-body">
                 <div class="current-track-info">
                    <div class="marquee-wrapper">
                       <span class="t-title">{{ currentProgress.currentItem?.title || 'Resolving URL...' }}</span>
                    </div>
                    <span class="t-status">{{ currentProgress.message }}</span>
                 </div>
                 
                 <div class="visual-progress">
                    <div class="radial-wrap">
                      <svg viewBox="0 0 36 36" class="circular-chart-alt">
                        <path class="circle-bg" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" />
                        <path class="circle-active" :stroke-dasharray="currentProgress.itemProgress + ', 100'" d="M18 2.0845 a 15.9155 15.9155 0 0 1 0 31.831 a 15.9155 15.9155 0 0 1 0 -31.831" />
                      </svg>
                      <span class="radial-val">{{ currentProgress.itemProgress }}%</span>
                    </div>
                    
                    <div class="progress-counters">
                       <div class="counter-item">
                          <span class="c-num">{{ currentProgress.itemIndex + 1 }}</span>
                          <span class="c-lbl">Current</span>
                       </div>
                       <div class="counter-sep">/</div>
                       <div class="counter-item">
                          <span class="c-num">{{ currentProgress.totalItems }}</span>
                          <span class="c-lbl">Total</span>
                       </div>
                    </div>
                 </div>
              </div>
            </div>
          </Transition>
        </div>
      </div>

      <!-- Playlists Tab -->
      <div v-if="activeTab === 'playlists'" class="tab-pane fade-in">
        <div class="playlists-view-container">
          <div class="view-header">
            <div class="header-text">
              <h3>Managed Playlists</h3>
              <p>Sync these automated collections with your local library.</p>
            </div>
            <button class="btn btn-primary" @click="showAddPlaylistModal = true">
              <Icon name="plus" :size="18" />
              Register New
            </button>
          </div>
          
          <div v-if="loadingPlaylists" class="centered-loading">
            <div class="loader-wave">
               <span></span><span></span><span></span><span></span>
            </div>
            <p>Scanning registry...</p>
          </div>
          
          <div v-else-if="savedPlaylists.length === 0" class="empty-placeholder-v2">
            <div class="icon-circle"><Icon name="list" :size="40" /></div>
            <h3>No Playlists Registered</h3>
            <p>Start by adding a YouTube playlist to automatically track new releases.</p>
            <button class="btn btn-outline" @click="showAddPlaylistModal = true">Register First Playlist</button>
          </div>
          
          <div v-else class="playlist-grid-modern">
            <div v-for="playlist in savedPlaylists" :key="playlist.id" class="playlist-card-v2">
              <div class="card-bg-glow"></div>
              <div class="card-inner">
                 <div class="p-top-row">
                    <div class="p-type-icon"><Icon name="youtube" :size="20" /></div>
                    <div class="p-actions-overlay">
                       <button class="p-icon-btn sync" @click="syncPlaylist(playlist)" title="Sync manually">
                          <Icon name="refresh-cw" :size="16" />
                       </button>
                       <button class="p-icon-btn delete" @click="deletePlaylist(playlist)" title="Remove">
                          <Icon name="trash" :size="16" />
                       </button>
                    </div>
                 </div>
                 
                 <div class="p-content">
                    <div class="p-title-wrap">
                       <span class="title">{{ playlist.name }}</span>
                       <div v-if="playlist.autoDownload" class="badge-status active">Auto</div>
                    </div>
                    <div class="p-link">{{ playlist.url }}</div>
                    <div v-if="playlist.description" class="p-memo">{{ playlist.description }}</div>
                 </div>
                 
                 <div class="p-footer">
                    <div class="last-sync">
                       <Icon name="clock" :size="12" />
                       <span>{{ playlist.lastSyncedAt ? 'Last sync: ' + formatDate(playlist.lastSyncedAt) : 'Never synced' }}</span>
                    </div>
                    <button 
                      class="toggle-action" 
                      :class="{ enabled: playlist.autoDownload }"
                      @click="toggleAutoDownload(playlist)"
                    >
                       {{ playlist.autoDownload ? 'Disable Auto' : 'Enable Auto' }}
                    </button>
                 </div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Maintenance / Config Tab -->
      <div v-if="activeTab === 'config'" class="tab-pane fade-in">
        <div class="maintenance-view">
          <div class="settings-card-complex">
            <div class="settings-header">
               <h3>Engine Parameters</h3>
               <p>Core settings for the background automation services.</p>
            </div>
            
            <div class="settings-list-v2">
              <!-- Master Toggle -->
              <div class="setting-item-v2">
                <div class="s-info">
                   <span class="s-title">Automation Engine</span>
                   <span class="s-desc">Master switch for the background scanning process.</span>
                </div>
                <div class="s-control">
                   <label class="premium-toggle">
                     <input 
                       type="checkbox" 
                       :checked="autoDownloadConfig?.enabled"
                       @change="updateAutoDownloadEnabled"
                     />
                     <span class="t-slider"></span>
                   </label>
                </div>
              </div>

              <!-- Interval -->
              <div class="setting-item-v2">
                <div class="s-info">
                   <span class="s-title">Check Frequency</span>
                   <span class="s-desc">Interval between repository scans (minutes).</span>
                </div>
                <div class="s-control">
                   <div class="input-with-unit">
                      <input 
                        type="number" 
                        :value="autoDownloadConfig?.check_interval_minutes"
                        @change="updateCheckInterval"
                        min="5"
                        max="1440"
                      />
                      <span class="u-tag">MIN</span>
                   </div>
                </div>
              </div>

              <!-- Concurrency -->
              <div class="setting-item-v2">
                <div class="s-info">
                   <span class="s-title">Parallel Scaling</span>
                   <span class="s-desc">Maximum concurrent transfer workers allowed.</span>
                </div>
                <div class="s-control">
                   <div class="input-with-unit">
                      <input 
                        type="number" 
                        :value="autoDownloadConfig?.max_concurrent_downloads"
                        @change="updateMaxConcurrent"
                        min="1"
                        max="5"
                      />
                      <span class="u-tag">WORKERS</span>
                   </div>
                </div>
              </div>
            </div>
          </div>
          
          <div class="troubleshooting-card">
             <h4>System Maintenance</h4>
             <p>Use these tools if the engine is unresponsive or local cache is out of sync.</p>
             <div class="actions-grid">
                <button class="btn btn-outline" @click="processDownloads">
                   <Icon name="refresh-cw" :size="16" />
                   Rebuild Index
                </button>
                <button class="btn btn-outline-danger" @click="stopAutoDownload">
                   <Icon name="alert-circle" :size="16" />
                   Emergency Stop
                </button>
             </div>
          </div>
        </div>
      </div>

      <!-- History Tab -->
      <div v-if="activeTab === 'history'" class="tab-pane fade-in">
        <div class="history-container-modern">
          <div class="history-header-bar">
             <h3>Download History</h3>
             <div class="history-search">
                <!-- Potential for a search input here -->
             </div>
          </div>
          
          <div v-if="loadingHistory" class="loading-state">
            <Icon name="refresh-cw" :size="32" class="spinning" />
            <p>Loading history...</p>
          </div>
          <div v-else-if="downloadedVideos.length === 0" class="empty-state">
            <p>No downloads recorded yet.</p>
          </div>
          <div v-else class="history-table-wrapper">
            <table class="history-table-new">
              <thead>
                <tr>
                  <th>Video Title</th>
                  <th>ID</th>
                  <th>Date</th>
                  <th class="text-right">Action</th>
                </tr>
              </thead>
              <tbody>
                <tr v-for="video in downloadedVideos" :key="video.videoId">
                  <td class="video-title-cell">{{ video.title }}</td>
                  <td><code class="v-id">{{ video.videoId }}</code></td>
                  <td>{{ formatDate(video.downloadedAt) }}</td>
                  <td class="text-right">
                    <button
                      @click="removeFromHistory(video.videoId)"
                      class="btn-icon-danger"
                      title="Forget this video"
                    >
                      <Icon name="trash" :size="16" />
                    </button>
                  </td>
                </tr>
              </tbody>
            </table>
          </div>
        </div>
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
import { ref, onMounted, onUnmounted, watch } from 'vue'
import { musicAPI } from '../../api/music'
import { useToast } from '../../composables/useToast'
import { useConfirm } from '../../composables/useConfirm'
import { useDownloadStore } from '../../stores/downloadStore'
import { formatBytes, formatDate } from '../../utils/musicFormatters'
import Icon from '../../shared/components/Icons.vue'

const { success, error: showError } = useToast()
const { confirm } = useConfirm()
const downloadStore = useDownloadStore()

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

// Computed-like refs for backward compatibility in template
const isDownloading = ref(false)
const currentProgress = ref({
  itemIndex: 0,
  totalItems: 0,
  itemProgress: 0,
  currentItem: { title: '', size: 0 },
  message: ''
})

// Sync local refs with store
watch(() => downloadStore.currentProgress, (newProgress) => {
  if (newProgress) {
    isDownloading.value = downloadStore.isDownloading
    currentProgress.value = {
      itemIndex: newProgress.completed_files || 0,
      totalItems: newProgress.total_files || 0,
      itemProgress: Math.round(newProgress.progress || 0),
      currentItem: { 
        title: newProgress.current_file || '', 
        size: 0 
      },
      message: newProgress.status
    }
  } else {
    isDownloading.value = false
  }
}, { immediate: true, deep: true })

// Watch for completion to refresh history
watch(() => downloadStore.lastCompletedAt, () => {
  fetchDownloadedVideos()
  fetchDownloadStats()
  downloadUrl.value = ''
})

const downloadedVideos = ref<DownloadedVideo[]>([])
const downloadStats = ref<DownloadStats | null>(null)
const loadingHistory = ref(false)

// Saved playlists state
const savedPlaylists = ref<YoutubePlaylist[]>([])
const loadingPlaylists = ref(false)
const showAddPlaylistModal = ref(false)
const newPlaylist = ref({ name: '', url: '', description: '', autoDownload: false })

// Auto-download state
const autoDownloadConfig = ref<AutoDownloadConfig | null>(null)
const autoDownloadStatus = ref<AutoDownloadStatus | null>(null)
let statusPollInterval: number | null = null

const isProcessingDownloads = ref(false)

const activeTab = ref('status')

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
    if (response.data?.sessionId) {
      downloadStore.setSession(response.data.sessionId)
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

const processDownloads = async () => {
  try {
    isProcessingDownloads.value = true
    const response = await musicAPI.syncMusicFolder('/music/downloads')
    success('Processing Complete', `Found and added ${response.data.inserted} new tracks from downloads folder.`)
  } catch (error: any) {
    console.error('Error processing downloads:', error)
    showError('Process Failed', error.response?.data?.error || 'Failed to process downloads folder')
  } finally {
    isProcessingDownloads.value = false
  }
}

// === Manual Download Functions ===
const startDownload = async () => {
  if (!downloadUrl.value.trim()) {
    showError('Missing URL', 'Please enter a YouTube URL')
    return
  }

  try {
    const response = await musicAPI.startYoutubeDownload({
      url: downloadUrl.value,
      output_dir: '/music/downloads',
      limit: options.value.limit,
      max_concurrent: options.value.max_concurrent,
      audio_quality: options.value.audio_quality,
    })

    const sessionId = response.data.session_id || response.data.sessionId
    downloadStore.setSession(sessionId)
  } catch (error) {
    console.error('Error starting download:', error)
    showError('Download Failed', 'Failed to start download')
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

  downloadStore.cancelDownload()
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
  if (statusPollInterval) {
    clearInterval(statusPollInterval)
  }
})
</script>

<style scoped>
.downloader-tab {
  display: flex;
  flex-direction: column;
  gap: 24px;
  animation: fadeIn 0.5s ease-out;
  color: var(--text-color);
}

.header h2 {
  margin: 0;
  font-size: 32px;
  font-weight: 800;
  letter-spacing: -0.03em;
  background: linear-gradient(to right, var(--text-color), var(--text-tertiary));
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

/* Tabs Navigation Modern */
.tabs-nav {
  display: flex;
  gap: 2px;
  background: var(--background-elevated);
  padding: 4px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  width: max-content;
}

.tab-btn {
  position: relative;
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 20px;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  background: transparent;
  transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  border: none;
  cursor: pointer;
  overflow: hidden;
}

.tab-btn:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.tab-btn.active {
  color: var(--primary-light);
  background: var(--primary-glow);
}

.tab-indicator {
  position: absolute;
  bottom: 0;
  left: 50%;
  transform: translateX(-50%);
  width: 0;
  height: 2px;
  background: var(--primary-color);
  transition: width 0.3s ease;
}

.tab-btn.active .tab-indicator {
  width: 100%;
}

/* Status Layout */
.status-layout {
  display: grid;
  grid-template-columns: 1fr 340px;
  gap: 24px;
}

.main-status-area {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.glass-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 24px;
  padding: 32px;
  box-shadow: var(--shadow-lg);
  position: relative;
  overflow: hidden;
}

.status-hero {
  border: 1px solid var(--border-color);
  transition: border-color 0.5s ease;
}

.status-hero.execution-active {
  border-color: var(--primary-color);
}

.hero-top {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 32px;
}

.hero-info h3 {
  font-size: 24px;
  margin: 0 0 8px 0;
  font-weight: 700;
}

.hero-info p {
  color: var(--text-tertiary);
  margin: 0;
  font-size: 15px;
}

.engine-state {
  display: flex;
  align-items: center;
  gap: 12px;
  position: relative;
}

.state-label {
  padding: 6px 14px;
  border-radius: 20px;
  font-size: 12px;
  font-weight: 800;
  text-transform: uppercase;
  background: var(--background-elevated);
  color: var(--text-tertiary);
  border: 1px solid var(--border-color);
}

.state-label.active {
  background: var(--primary-glow);
  color: var(--primary-light);
  border-color: var(--primary-color);
}

.pulse-ring {
  width: 12px;
  height: 12px;
  background: var(--primary-color);
  border-radius: 50%;
  box-shadow: 0 0 0 0 rgba(139, 92, 246, 0.7);
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(139, 92, 246, 0.7); }
  70% { transform: scale(1); box-shadow: 0 0 0 10px rgba(139, 92, 246, 0); }
  100% { transform: scale(0.95); box-shadow: 0 0 0 0 rgba(139, 92, 246, 0); }
}

.hero-actions {
  display: flex;
  gap: 16px;
  margin-bottom: 0;
}

.execution-details, .manual-execution-details {
  margin-top: 32px;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
  animation: slideDown 0.4s ease-out;
}

.exec-playlist {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 20px;
}

.exec-playlist .p-name {
  font-size: 18px;
  font-weight: 600;
}

.exec-stats-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 16px;
}

.exec-stat {
  background: var(--background-elevated);
  padding: 16px;
  border-radius: 16px;
  display: flex;
  flex-direction: column;
  align-items: center;
}

.exec-stat .val {
  font-size: 24px;
  font-weight: 800;
  color: var(--primary-light);
}

.exec-stat .lbl {
  font-size: 11px;
  text-transform: uppercase;
  color: var(--text-tertiary);
  margin-top: 4px;
}

/* Manual progress bar modern */
.manual-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 12px;
}

.info-text .title {
  display: block;
  font-weight: 700;
  font-size: 16px;
}

.info-text .subtext {
  font-size: 13px;
  color: var(--text-tertiary);
}

.progress-info {
  text-align: right;
}

.progress-info .percent {
  display: block;
  font-size: 20px;
  font-weight: 800;
  color: var(--accent-color);
}

.progress-info .counter {
  font-size: 12px;
  color: var(--text-tertiary);
}

.modern-progress-bar {
  height: 8px;
  background: var(--background-elevated);
  border-radius: 4px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--accent-color));
  box-shadow: 0 0 12px var(--primary-glow);
  transition: width 0.4s cubic-bezier(0.1, 0.7, 1, 0.1);
}

/* Quick Stats Row */
.quick-stats-row {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
}

.mini-card-stat {
  background: var(--surface-color);
  padding: 20px;
  border-radius: 20px;
  border: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  gap: 16px;
  transition: transform 0.2s;
}

.mini-card-stat:hover {
  transform: translateY(-4px);
  border-color: var(--primary-color);
}

.icon-wrap {
  width: 52px;
  height: 52px;
  border-radius: 14px;
  background: var(--primary-glow);
  color: var(--primary-color);
  display: flex;
  align-items: center;
  justify-content: center;
}

.icon-wrap.color-blue { background: rgba(59, 130, 246, 0.1); color: #3b82f6; }
.icon-wrap.color-green { background: rgba(16, 185, 129, 0.1); color: #10b981; }

.mini-card-stat .num {
  font-size: 22px;
  font-weight: 800;
}

.mini-card-stat .label {
  font-size: 12px;
  color: var(--text-tertiary);
}

/* Sidebar Panels */
.status-sidebar-info {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.panel-card {
  background: var(--surface-color);
  padding: 24px;
  border-radius: 20px;
  border: 1px solid var(--border-color);
}

.panel-card h4 {
  margin: 0 0 16px 0;
  font-size: 16px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
}

.config-mini-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 16px;
}

.c-item {
  display: flex;
  justify-content: space-between;
  font-size: 14px;
}

.history-mini {
  display: flex;
  flex-direction: column;
  gap: 10px;
  margin-bottom: 16px;
}

.mini-v-item {
  font-size: 13px;
}

.mini-v-item .v-title {
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.mini-v-item .v-date {
  font-size: 11px;
  color: var(--text-tertiary);
}

/* Manual Download Tab Focused */
.download-container-focused {
  max-width: 800px;
  margin: 0 auto;
}

.download-card {
  background: var(--surface-color);
  border-radius: 28px;
  padding: 40px;
  border: 1px solid var(--border-color);
  box-shadow: var(--shadow-xl);
}

.card-header h3 {
  font-size: 28px;
  margin: 0 0 10px 0;
}

.card-header p {
  color: var(--text-tertiary);
  margin-bottom: 40px;
}

.input-glow-group label {
  display: block;
  font-weight: 600;
  margin-bottom: 12px;
}

.glow-input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.glow-input-wrapper .field-icon {
  position: absolute;
  left: 18px;
  color: var(--text-tertiary);
}

.glow-input-wrapper input {
  width: 100%;
  padding: 18px 24px 18px 52px;
  background: var(--background-elevated);
  border: 2px solid var(--border-color);
  border-radius: 16px;
  font-size: 16px;
  transition: all 0.3s;
}

.glow-input-wrapper input:focus {
  border-color: var(--primary-color);
  box-shadow: 0 0 20px var(--primary-glow);
  background: var(--surface-color);
}

.options-grid {
  display: grid;
  grid-template-columns: repeat(3, 1fr);
  gap: 20px;
  margin: 32px 0;
}

.option-col label {
  display: block;
  font-size: 13px;
  font-weight: 700;
  margin-bottom: 8px;
  color: var(--text-tertiary);
  text-transform: uppercase;
}

.option-col select, .option-col input {
  width: 100%;
  padding: 12px 16px;
  border-radius: 10px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
}

.btn-xl {
  padding: 20px 40px;
  font-size: 18px;
  font-weight: 800;
  border-radius: 18px;
  width: 100%;
  display: flex;
  justify-content: center;
}

.btn-loading {
  display: flex;
  align-items: center;
  gap: 12px;
}

.btn-content {
  display: flex;
  align-items: center;
  gap: 12px;
}

/* Progress Panel Modern */
.progress-panel-modern {
  margin-top: 24px;
  background: var(--surface-color);
  border-radius: 20px;
  padding: 24px;
  border: 1px solid var(--primary-color);
  box-shadow: 0 8px 32px var(--primary-glow);
}

.p-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 20px;
}

.tag-live {
  background: #ef4444;
  color: white;
  padding: 2px 8px;
  font-size: 10px;
  font-weight: 900;
  border-radius: 4px;
  text-transform: uppercase;
  animation: blink 1s infinite;
}

@keyframes blink { 50% { opacity: 0.5; } }

.current-track-info {
  margin-bottom: 24px;
}

.t-title {
  font-size: 18px;
  font-weight: 700;
  display: block;
}

.visual-progress {
  display: flex;
  align-items: center;
  justify-content: space-around;
}

.radial-wrap {
  width: 100px;
  height: 100px;
  position: relative;
}

.circular-chart-alt {
  width: 100%;
  height: 100%;
}

.circle-active {
  fill: none;
  stroke: var(--primary-color);
  stroke-width: 3;
  stroke-linecap: round;
  transition: stroke-dasharray 0.5s ease;
}

.radial-val {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  font-size: 20px;
  font-weight: 800;
}

.progress-counters {
  display: flex;
  gap: 20px;
  align-items: center;
}

.counter-item {
  text-align: center;
}

.counter-item .c-num { font-size: 32px; font-weight: 800; display: block; }
.counter-item .c-lbl { font-size: 11px; color: var(--text-tertiary); text-transform: uppercase; }

/* Playlist Grid Modern */
.playlist-grid-modern {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(320px, 1fr));
  gap: 24px;
}

.playlist-card-v2 {
  position: relative;
  background: var(--surface-color);
  border-radius: 24px;
  overflow: hidden;
  border: 1px solid var(--border-color);
  transition: all 0.4s cubic-bezier(0.4, 0, 0.2, 1);
}

.playlist-card-v2:hover {
  transform: translateY(-8px);
  border-color: var(--primary-color);
  box-shadow: 0 20px 40px rgba(0,0,0,0.3);
}

.card-bg-glow {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  height: 120px;
  background: linear-gradient(135deg, var(--primary-glow) 0%, transparent 100%);
  opacity: 0.5;
  pointer-events: none;
}

.card-inner {
  position: relative;
  padding: 24px;
  z-index: 1;
  height: 100%;
  display: flex;
  flex-direction: column;
}

.p-top-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 20px;
}

.p-type-icon {
  width: 40px;
  height: 40px;
  border-radius: 12px;
  background: rgba(255,0,0,0.1);
  color: #ff0000;
  display: flex;
  align-items: center;
  justify-content: center;
}

.p-actions-overlay {
  display: flex;
  gap: 8px;
}

.p-icon-btn {
  width: 32px;
  height: 32px;
  border-radius: 8px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s;
}

.p-icon-btn:hover { color: var(--text-color); border-color: var(--primary-color); }
.p-icon-btn.delete:hover { color: #ef4444; border-color: #ef4444; }

.p-content {
  flex: 1;
}

.p-title-wrap .title {
  font-size: 20px;
  font-weight: 700;
  display: block;
}

.badge-status.active {
  background: var(--primary-color);
  color: white;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 10px;
  font-weight: 800;
  text-transform: uppercase;
  display: inline-block;
  margin-top: 4px;
}

.p-link {
  font-size: 11px;
  color: var(--text-tertiary);
  margin: 12px 0;
  word-break: break-all;
  font-family: monospace;
}

.p-memo {
  font-size: 14px;
  opacity: 0.8;
}

.p-footer {
  margin-top: 24px;
  padding-top: 20px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.last-sync {
  font-size: 12px;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  gap: 6px;
}

.toggle-action {
  font-size: 12px;
  font-weight: 700;
  padding: 8px 16px;
  border-radius: 10px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
}

.toggle-action.enabled {
  background: var(--primary-glow);
  color: var(--primary-light);
  border-color: var(--primary-color);
}

/* Maintenance Page */
.maintenance-view {
  max-width: 900px;
  margin: 0 auto;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.settings-card-complex {
  background: var(--surface-color);
  border-radius: 24px;
  padding: 32px;
  border: 1px solid var(--border-color);
}

.setting-item-v2 {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px 0;
  border-bottom: 1px solid var(--border-color);
}

.setting-item-v2:last-child { border: none; }

.s-title { font-size: 17px; font-weight: 700; display: block; }
.s-desc { font-size: 13px; color: var(--text-tertiary); }

.premium-toggle {
  position: relative;
  width: 60px;
  height: 32px;
}

.premium-toggle input { opacity: 0; width: 0; height: 0; }

.t-slider {
  position: absolute;
  cursor: pointer;
  top: 0; left: 0; right: 0; bottom: 0;
  background-color: var(--background-elevated);
  transition: .4s;
  border-radius: 34px;
  border: 1px solid var(--border-color);
}

.t-slider:before {
  position: absolute;
  content: "";
  height: 24px; width: 24px;
  left: 4px; bottom: 3px;
  background-color: white;
  transition: .4s;
  border-radius: 50%;
  box-shadow: 0 2px 4px rgba(0,0,0,0.2);
}

input:checked + .t-slider { background-color: var(--primary-color); border-color: var(--primary-color); }
input:checked + .t-slider:before { transform: translateX(26px); }

.input-with-unit {
  display: flex;
  align-items: center;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  overflow: hidden;
}

.input-with-unit input {
  padding: 10px 14px;
  border: none;
  background: transparent;
  width: 80px;
  text-align: right;
  font-weight: 700;
}

.u-tag {
  background: var(--surface-muted);
  padding: 10px 12px;
  font-size: 11px;
  font-weight: 900;
  color: var(--text-tertiary);
  border-left: 1px solid var(--border-color);
}

.troubleshooting-card {
  background: rgba(239, 68, 68, 0.05);
  border: 1px solid rgba(239, 68, 68, 0.2);
  border-radius: 20px;
  padding: 24px;
}

.troubleshooting-card h4 { color: #ef4444; margin: 0 0 8px 0; }
.actions-grid { display: flex; gap: 16px; margin-top: 20px; }

/* Transitions */
.slide-up-enter-active, .slide-up-leave-active { transition: all 0.4s ease; }
.slide-up-enter-from, .slide-up-leave-to { opacity: 0; transform: translateY(20px); }

@keyframes fadeIn { from { opacity: 0; transform: translateY(10px); } to { opacity: 1; transform: translateY(0); } }
@keyframes slideDown { from { opacity: 0; height: 0; transform: translateY(-10px); } to { opacity: 1; height: auto; transform: translateY(0); } }

@media (max-width: 1000px) {
  .status-layout { grid-template-columns: 1fr; }
  .quick-stats-row { grid-template-columns: 1fr 1fr; }
}

@media (max-width: 768px) {
  .header h2 {
    font-size: 24px;
  }

  .tabs-nav {
    width: 100%;
    overflow-x: auto;
    white-space: nowrap;
    display: flex;
    padding: 6px;
    -ms-overflow-style: none;
    scrollbar-width: none;
  }

  .tabs-nav::-webkit-scrollbar {
    display: none;
  }

  .tab-btn {
    padding: 8px 16px;
    flex-shrink: 0;
  }

  .glass-card {
    padding: 20px;
  }

  .hero-top {
    flex-direction: column;
    gap: 16px;
  }

  .hero-actions {
    flex-direction: column;
    width: 100%;
  }

  .hero-actions .btn {
    width: 100%;
  }

  .exec-stats-grid {
    grid-template-columns: 1fr;
  }

  .quick-stats-row {
    grid-template-columns: 1fr;
  }

  .download-card {
    padding: 24px 20px;
  }

  .card-header h3 {
    font-size: 22px;
  }

  .visual-progress {
    flex-direction: column;
    gap: 20px;
  }

  .playlist-grid-modern {
    grid-template-columns: 1fr;
  }

  .setting-item-v2 {
    flex-direction: column;
    align-items: flex-start;
    gap: 16px;
  }

  .s-control {
    width: 100%;
    display: flex;
    justify-content: flex-end;
  }

  .actions-grid {
    flex-direction: column;
  }

  .history-table-wrapper {
    overflow-x: auto;
  }

  .history-table-new th:nth-child(2),
  .history-table-new td:nth-child(2) {
    display: none;
  }
}

@media (max-width: 480px) {
  .tab-btn span {
    display: none;
  }
  
  .tab-btn {
    padding: 10px;
  }
}
</style>
