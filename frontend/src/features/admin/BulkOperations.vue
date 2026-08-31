<template>
  <div class="bulk-operations">
    <div class="tabs-container">
      <div class="tabs">
        <button
          :class="['tab-button', { active: activeTab === 'enrichment' }]"
          @click="activeTab = 'enrichment'"
        >
          Metadata Enrichment
        </button>
        <button
          :class="['tab-button', { active: activeTab === 'rename' }]"
          @click="activeTab = 'rename'"
        >
          Bulk Rename
        </button>
        <button
          :class="['tab-button', { active: activeTab === 'playlist' }]"
          @click="activeTab = 'playlist'"
        >
          Bulk Add to Playlist
        </button>
        <button
          :class="['tab-button', { active: activeTab === 'maintenance' }]"
          @click="activeTab = 'maintenance'"
        >
          Maintenance
        </button>
      </div>
    </div>

    <!-- Enrichment Tab -->
    <div v-if="activeTab === 'enrichment'" class="tab-content">
      <div class="header-section mb-6">
        <h3>Metadata Enrichment</h3>
        <p class="info-text">
          Identify and automatically fill in missing metadata (Release Date, Album, Genre) for tracks.
          Scanning helps identify tracks with incorrect titles or artists that cannot be found on Discogs/MusicBrainz.
        </p>
      </div>

      <!-- Metadata Suggestions Section -->
      <transition-group name="list" tag="div" v-if="suggestions.length > 0" class="suggestions-container mb-8">
        <div class="flex items-center justify-between mb-4">
          <h4 class="text-xl font-bold flex items-center gap-2">
            <Icon name="sparkles" :size="20" class="text-primary" />
            Metadata Suggestions <span class="badge-count">{{ suggestions.length }}</span>
          </h4>
          <button 
            v-if="!isApplyingAll && hasSuccessfulScans" 
            @click="applyAllSuggestions" 
            class="btn btn-success"
          >
            Apply All Suggestions
          </button>
        </div>
        
        <div v-if="suggestionsLoading" class="loading" key="loading">
          Loading suggestions...
        </div>
        <div v-else class="suggestions-grid" key="grid">
          <div v-for="suggestion in suggestions" :key="suggestion.music_file_id" class="suggestion-card">
            <div class="card-header">
              <div class="confidence-indicator" :class="getConfidenceClass(suggestion.confidence)">
                {{ Math.round(suggestion.confidence * 100) }}% Match
              </div>
            </div>
            
            <div class="card-body">
              <!-- Current Track -->
              <div class="track-section">
                <span class="section-label">Current</span>
                <div class="track-details">
                  <span class="track-title" :title="suggestion.track?.title">{{ truncate(suggestion.track?.title || '', 30) }}</span>
                  <span class="track-artist" :title="suggestion.track?.artist ?? undefined">{{ truncate(suggestion.track?.artist || 'Unknown', 30) }}</span>
                </div>
              </div>

              <div class="divider">
                <Icon name="arrow-right" :size="16" class="text-tertiary" />
              </div>

              <!-- Suggested -->
              <div class="suggestion-section">
                <span class="section-label">Suggested</span>
                <div class="suggested-fields">
                  <div v-if="suggestion.release_date" class="field highlight">
                    <Icon name="calendar" :size="12" /> {{ suggestion.release_date }}
                  </div>
                  <div v-if="suggestion.album" class="field highlight" :title="suggestion.album">
                    <Icon name="disc" :size="12" /> {{ truncate(suggestion.album, 25) }}
                  </div>
                  <div v-if="suggestion.genre" class="field highlight">
                    <Icon name="tag" :size="12" /> {{ suggestion.genre }}
                  </div>
                </div>
              </div>
            </div>

            <div class="card-actions">
              <button 
                @click="rejectSuggestion(suggestion)" 
                class="btn-reject" 
                title="Reject"
                :disabled="!!suggestionProcessing"
              >
                <Icon name="close" :size="16" />
              </button>
              <button 
                @click="applySuggestion(suggestion)" 
                class="btn-accept" 
                title="Accept"
                :disabled="!!suggestionProcessing"
              >
                <Icon v-if="suggestionProcessing === suggestion.music_file_id" name="loader" class="animate-spin" :size="16" />
                <Icon v-else name="check" :size="16" />
              </button>
            </div>
          </div>
        </div>
      </transition-group>

      <div class="missing-metadata-section">
        <h4 class="text-xl font-bold mb-4">Tracks with Missing Metadata</h4>
        <div class="enrichment-controls glass-panel">
          <div class="control-group">
            <button 
                @click="fetchMissingMetadataTracks" 
                class="btn btn-secondary" 
                :disabled="enrichmentLoading || isScanningAll"
            >
                <Icon name="refresh" :size="16" /> Refresh List
            </button>
            <span class="stats-text" v-if="enrichmentTracks.length > 0">
                {{ enrichmentTracks.length }} tracks found
            </span>
          </div>

          <div class="control-group right">
            <button 
                v-if="enrichmentTracks.some(t => scanResults[t.id]?.data)"
                @click="applyAllSuccessful" 
                class="btn btn-success" 
                :disabled="isApplyingAll || isScanningAll"
                style="margin-right: 8px;"
            >
                <Icon v-if="isApplyingAll" name="loader" class="animate-spin" />
                {{ isApplyingAll ? 'Applying...' : 'Apply All Scans' }}
            </button>
            <button 
                v-if="enrichmentTracks.length > 0" 
                @click="scanAllTracks" 
                class="btn btn-primary" 
                :disabled="isScanningAll"
            >
                <Icon v-if="isScanningAll" name="loader" class="animate-spin" />
                {{ isScanningAll ? `Scanning ${currentScanIndex + 1}/${enrichmentTracks.length}...` : 'Scan All' }}
            </button>
          </div>
        </div>

        <div v-if="enrichmentLoading && enrichmentTracks.length === 0" class="loading">
            Loading tracks...
        </div>

        <div v-else-if="enrichmentTracks.length === 0" class="empty-state">
            <div class="empty-icon"><Icon name="check-circle" :size="48" /></div>
            <p>No tracks found with missing metadata!</p>
        </div>

        <div v-else class="enrichment-list-container">
            <table class="enrichment-table">
            <thead>
                <tr>
                <th>Track Details</th>
                <th>Missing Fields</th>
                <th>Scan Status</th>
                <th>Actions</th>
                </tr>
            </thead>
            <tbody>
                <tr v-for="track in enrichmentTracks" :key="track.id" :class="{ 'has-error': scanResults[track.id]?.error }">
                <td>
                    <div class="track-info">
                    <span class="track-title">{{ track.title }}</span>
                    <span class="track-artist">{{ track.artist || 'Unknown Artist' }}</span>
                    </div>
                </td>
                <td>
                    <div class="missing-badges">
                    <span v-if="!track.release_date" class="badge missing">Date</span>
                    <span v-if="!track.album" class="badge missing">Album</span>
                    <span v-if="!track.genre_id" class="badge missing">Genre</span>
                    </div>
                </td>
                <td>
                    <div v-if="scanResults[track.id]" class="scan-result">
                    <template v-if="scanResults[track.id].loading">
                        <span class="status-scanning"><Icon name="loader" class="animate-spin" :size="14" /> Scanning...</span>
                    </template>
                    <template v-else-if="scanResults[track.id].error">
                        <span class="status-error" :title="scanResults[track.id].error || ''">No match found</span>
                    </template>
                    <template v-else-if="scanResults[track.id].data">
                        <div class="suggested-data-mini">
                        <div v-if="scanResults[track.id].data.release_date" class="suggestion-pill">
                            <Icon name="calendar" :size="12" /> {{ scanResults[track.id].data.release_date }}
                        </div>
                        <div v-if="scanResults[track.id].data.album" class="suggestion-pill" :title="scanResults[track.id].data.album">
                            <Icon name="disc" :size="12" /> {{ truncate(scanResults[track.id].data.album, 20) }}
                        </div>
                        <div v-if="scanResults[track.id].data.genre" class="suggestion-pill">
                            <Icon name="tag" :size="12" /> {{ scanResults[track.id].data.genre }}
                        </div>
                        </div>
                    </template>
                    </div>
                    <div v-else class="text-muted text-xs">Waiting to scan</div>
                </td>
                <td>
                    <div class="actions">
                    <button 
                        v-if="!scanResults[track.id]?.data && !scanResults[track.id]?.loading"
                        @click="scanTrack(track)"
                        class="btn-icon"
                        title="Scan metadata"
                    >
                        <Icon name="search" :size="18" />
                    </button>
                    <button 
                        v-if="scanResults[track.id]?.data"
                        @click="applyScanResult(track)"
                        class="btn-icon success"
                        title="Apply suggestions"
                    >
                        <Icon name="check" :size="18" />
                    </button>
                    </div>
                </td>
                </tr>
            </tbody>
            </table>
        </div>
      </div>
    </div>

    <!-- Bulk Rename Tab -->
    <div v-if="activeTab === 'rename'" class="tab-content">
      <h3>Bulk Rename by Regex</h3>
      <p class="info-text">
        Use regular expressions to rename titles, artists, or albums in bulk.
        Example: To remove "24. " from the start of titles, use pattern <code>^\d+\.\s*</code> with empty replacement.
      </p>

      <form @submit.prevent="handleBulkRename" class="form">
        <div class="form-group">
          <label for="rename-field">Field to Rename:</label>
          <select v-model="renameForm.field" id="rename-field" required>
            <option value="title">Title</option>
            <option value="artist">Artist</option>
            <option value="album">Album</option>
          </select>
        </div>

        <div class="form-group">
          <label for="rename-pattern">Regex Pattern:</label>
          <input
            v-model="renameForm.pattern"
            type="text"
            id="rename-pattern"
            placeholder="e.g., ^\d+\.\s*"
            required
          />
          <small class="help-text">
            Enter a regular expression pattern to match text you want to replace.
          </small>
        </div>

        <div class="form-group">
          <label for="rename-replacement">Replacement:</label>
          <input
            v-model="renameForm.replacement"
            type="text"
            id="rename-replacement"
            placeholder="e.g., (leave empty to remove matched text)"
          />
          <small class="help-text">
            Enter the replacement text. Use $1, $2, etc. for capture groups.
          </small>
        </div>

        <button type="submit" :disabled="renameLoading" class="btn btn-primary">
          {{ renameLoading ? 'Processing...' : 'Execute Bulk Rename' }}
        </button>
      </form>

      <div v-if="renameResult" class="result">
        <h4>Rename Results</h4>
        <p class="success-text flex items-center gap-2"><Icon name="check" :size="16" /> Updated {{ renameResult.updated_count }} file(s)</p>
        
        <div v-if="renameResult.updated_files.length > 0" class="updated-items">
          <h5>Updated Items:</h5>
          <div class="items-list">
            <div v-for="file in renameResult.updated_files.slice(0, 10)" :key="file.id" class="item">
              <div v-if="renameForm.field === 'title'" class="item-info">
                <strong>{{ file.title }}</strong>
              </div>
              <div v-else-if="renameForm.field === 'artist'" class="item-info">
                <strong>{{ file.artist || '(no artist)' }}</strong> - {{ file.title }}
              </div>
              <div v-else class="item-info">
                <strong>{{ file.album || '(no album)' }}</strong> - {{ file.title }}
              </div>
            </div>
          </div>
          <small v-if="renameResult.updated_files.length > 10" class="text-muted">
            ... and {{ renameResult.updated_files.length - 10 }} more
          </small>
        </div>
      </div>

      <div v-if="renameError" class="error">
        <strong>Error:</strong> {{ renameError }}
      </div>
    </div>

    <!-- Bulk Add to Playlist Tab -->
    <div v-if="activeTab === 'playlist'" class="tab-content">
      <h3>Bulk Add to Playlist by Regex</h3>
      <p class="info-text">
        Use regular expressions to match music files and add them to a playlist in bulk.
        Example: To add all titles starting with a number, use pattern <code>^\d+\.</code>
      </p>

      <form @submit.prevent="handleBulkAddToPlaylist" class="form">
        <div class="form-group">
          <label for="playlist-select">Target Playlist:</label>
          <select v-model="playlistForm.playlist_id" id="playlist-select" required>
            <option value="">Select a playlist...</option>
            <option v-for="playlist in playlists" :key="playlist.id" :value="playlist.id">
              {{ playlist.name }} ({{ playlist.track_count }} tracks)
            </option>
          </select>
        </div>

        <div class="form-group">
          <label for="match-field">Match Field:</label>
          <select v-model="playlistForm.field" id="match-field" required>
            <option value="title">Title</option>
            <option value="artist">Artist</option>
            <option value="album">Album</option>
          </select>
        </div>

        <div class="form-group">
          <label for="match-pattern">Regex Pattern:</label>
          <input
            v-model="playlistForm.pattern"
            type="text"
            id="match-pattern"
            placeholder="e.g., ^\d+\."
            required
          />
          <small class="help-text">
            Enter a regular expression pattern to match files you want to add.
          </small>
        </div>

        <button type="submit" :disabled="playlistLoading" class="btn btn-primary">
          {{ playlistLoading ? 'Processing...' : 'Add to Playlist' }}
        </button>
      </form>

      <div v-if="playlistResult" class="result">
        <h4>Add to Playlist Results</h4>
        <p class="success-text flex items-center gap-2"><Icon name="check" :size="16" /> Added {{ playlistResult.added_count }} track(s)</p>
        <p class="info-text">Total tracks in playlist: {{ playlistResult.total_playlist_count }}</p>
      </div>

      <div v-if="playlistError" class="error">
        <strong>Error:</strong> {{ playlistError }}
      </div>
    </div>

    <!-- Maintenance Tab -->
    <div v-if="activeTab === 'maintenance'" class="tab-content">
      <h3>Global Metadata Maintenance</h3>
      <p class="info-text">
        Perform global operations on your entire library. Use with caution as these changes are bulk updates.
      </p>

      <div class="maintenance-actions glass-panel p-6 flex flex-col gap-6">
        <div class="maintenance-group">
          <h4 class="text-lg font-bold mb-2">Clear BPM, Key & Beat Maps</h4>
          <p class="text-secondary text-sm mb-4">
            If BPM detection, key analysis, or beat grids aren't working correctly, you can clear all stored BPM, Key, and Beat Map metadata for all tracks. This will force re-analysis if automatic analysis is enabled.
          </p>
          <div class="flex gap-4">
            <button 
              @click="handleClearAllBpmAndKey" 
              class="btn btn-secondary border-red-500 text-red-500 hover:bg-red-500 hover:text-white"
              :disabled="maintenanceLoading"
            >
              <Icon v-if="maintenanceLoading" name="loader" class="animate-spin" />
              Clear all BPM, Key & Beat Maps
            </button>
          </div>
        </div>
      </div>

      <div v-if="maintenanceResult" class="result mt-6">
        <h4>Operation Result</h4>
        <p class="success-text flex items-center gap-2">
          <Icon name="check" :size="16" /> Updated {{ maintenanceResult.updated_count }} track(s)
        </p>
      </div>

      <div v-if="maintenanceError" class="error mt-6">
        <strong>Error:</strong> {{ maintenanceError }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'
import { musicAPI } from '@/api/music'
import type { PlaylistSummary, MusicFile } from '@/types'
import Icon from '@/shared/components/Icons.vue'
import type { MetadataSuggestion } from '@/types/metadata'
import { truncate } from '@/utils/musicFormatters'
import { useConfirm } from '@/composables/useConfirm'

const activeTab = ref<'rename' | 'playlist' | 'enrichment' | 'maintenance'>('enrichment')

// Metadata Enrichment
const enrichmentTracks = ref<MusicFile[]>([])
const enrichmentLoading = ref(false)
const scanResults = ref<Record<string, { loading: boolean; error: string | null; data: any | null }>>({})
const isScanningAll = ref(false)
const currentScanIndex = ref(0)
const isApplyingAll = ref(false)

const hasSuccessfulScans = computed(() => {
  return Object.values(scanResults.value).some(r => r.data !== null)
})

const fetchMissingMetadataTracks = async () => {
  enrichmentLoading.value = true
  try {
    const response = await musicAPI.getMusicFiles({ missing_metadata: true, sort: 'artist', order: 'asc' })
    enrichmentTracks.value = response.data
    // Reset scan results
    scanResults.value = {}
  } catch (error) {
    console.error('Failed to fetch enrichment tracks:', error)
  } finally {
    enrichmentLoading.value = false
  }
}

const scanTrack = async (track: MusicFile) => {
  scanResults.value[track.id] = { loading: true, error: null, data: null }
  try {
    const response = await musicAPI.lookupReleaseDate(track.title, track.artist || undefined)
    const result = response.data
    if (result.release_date || result.album || result.genre) {
      scanResults.value[track.id] = { loading: false, error: null, data: result }
    } else {
      scanResults.value[track.id] = { loading: false, error: result.error || 'No release found', data: null }
    }
  } catch (error: any) {
    console.error('Lookup failed for track:', track.title, error)
    scanResults.value[track.id] = { loading: false, error: 'Lookup failed', data: null }
  }
}

const scanAllTracks = async () => {
  if (isScanningAll.value) return
  isScanningAll.value = true
  
  const tracksToScan = [...enrichmentTracks.value]
  
  for (let i = 0; i < tracksToScan.length; i++) {
    const track = tracksToScan[i]
    currentScanIndex.value = i
    
    // Skip if already has data or is loading
    if (scanResults.value[track.id]?.data || scanResults.value[track.id]?.loading) continue
    
    await scanTrack(track)
    // Delay increased to 1.5s to respect Discogs rate limits (60/min)
    await new Promise(resolve => setTimeout(resolve, 1500))
  }
  
  isScanningAll.value = false
}

const applyScanResult = async (track: MusicFile) => {
  const result = scanResults.value[track.id]?.data
  if (!result) return

  try {
    let releaseDate = result.release_date || undefined
    
    // Convert partial dates to ISO if needed, as backend expects DateTime<Utc>
    // Discogs/MusicBrainz might return "2023" or "2023-01" or empty string
    if (releaseDate && !releaseDate.includes('T')) {
        const d = new Date(releaseDate)
        if (!isNaN(d.getTime())) {
            releaseDate = d.toISOString()
        } else {
            releaseDate = undefined // Invalid date or empty, skip
        }
    } else if (releaseDate === '') {
        releaseDate = undefined
    }

    await musicAPI.updateMusicFile(track.id, {
      release_date: releaseDate,
      album: result.album || undefined,
      metadata_analyzed: true
    })
    
    // Remove from enrichment tracks list on success
    enrichmentTracks.value = enrichmentTracks.value.filter(t => t.id !== track.id)
    delete scanResults.value[track.id]
  } catch (error) {
    console.error('Failed to apply metadata enrichment:', error)
  }
}



// Bulk Rename
const renameForm = ref({
  field: 'title' as 'title' | 'artist' | 'album',
  pattern: '',
  replacement: ''
})
const renameLoading = ref(false)
const renameError = ref('')
const renameResult = ref<any>(null)

// Bulk Add to Playlist
const playlistForm = ref({
  playlist_id: '',
  field: 'title' as 'title' | 'artist' | 'album',
  pattern: ''
})
const playlistLoading = ref(false)
const playlistError = ref('')
const playlistResult = ref<any>(null)
const playlists = ref<PlaylistSummary[]>([])

// Maintenance
const maintenanceLoading = ref(false)
const maintenanceError = ref('')
const maintenanceResult = ref<{ updated_count: number } | null>(null)
const { confirm } = useConfirm()

const handleClearAllBpmAndKey = async () => {
  const isConfirmed = await confirm({
    title: 'Clear all BPM/Key & Beat Maps',
    message: 'Are you sure you want to clear ALL BPM, Key, and Beat Map metadata for every track in your library? This cannot be undone (without a backup).',
    confirmText: 'Clear All',
    cancelText: 'Cancel',
    variant: 'danger',
  })

  if (!isConfirmed) {
    return
  }

  maintenanceLoading.value = true
  maintenanceError.value = ''
  maintenanceResult.value = null

  try {
    // 1. Fetch all track IDs
    const response = await musicAPI.getMusicFiles({ limit: 10000 })
    const trackIds = response.data.map((t: MusicFile) => t.id)

    if (trackIds.length === 0) {
      throw new Error('No tracks found to update.')
    }

    // 2. Perform bulk update
    const result = await musicAPI.bulkUpdateMusic({
      ids: trackIds,
      clear_bpm: true,
      clear_key: true,
      clear_beat_map: true
    })

    maintenanceResult.value = { updated_count: result.data.updated_count }
  } catch (error: any) {
    maintenanceError.value = error.response?.data?.error || error.message || 'Failed to clear BPM/Key'
  } finally {
    maintenanceLoading.value = false
  }
}

// Metadata Suggestions
const suggestions = ref<(MetadataSuggestion & { track: MusicFile })[]>([])
const suggestionsLoading = ref(false)
const suggestionProcessing = ref<string | null>(null)

const fetchSuggestions = async () => {
  suggestionsLoading.value = true
  try {
    const { data } = await musicAPI.getMetadataSuggestions()
    const suggestionList = data.suggestions || []
    
    // Fetch track details for each suggestion
    // Parallelize requests for better performance
    const promises = suggestionList.map(async (sug: MetadataSuggestion) => {
      try {
        const trackRes = await musicAPI.getMusicFile(sug.music_file_id)
        // Skip suggestions that don't contain any meaningful changes
        const track = trackRes.data
        if (!shouldIncludeSuggestion(sug, track)) {
          return null
        }
        return { ...sug, track }
      } catch (e) {
        console.warn('Track not found for suggestion', sug.music_file_id)
        return null
      }
    })
    
    const results = await Promise.all(promises)
    suggestions.value = results.filter((s: (MetadataSuggestion & { track?: MusicFile }) | null) => s !== null && s.track) as (MetadataSuggestion & { track: MusicFile })[]
    
  } catch (err) {
    console.error('Failed to fetch suggestions:', err)
  } finally {
    suggestionsLoading.value = false
  }
}

// Helper: decide whether a suggestion contains meaningful changes compared to the current track
function shouldIncludeSuggestion(sug: MetadataSuggestion, track: MusicFile | undefined | null) {
  if (!sug) return false
  // If the suggestion doesn't propose any of these fields, it's not useful
  const hasAnyField = !!(sug.album || sug.genre || sug.release_date)
  if (!hasAnyField) return false

  // If we don't have track data, include the suggestion so it can be reviewed
  if (!track) return true

  // Album change?
  if (sug.album && (!track.album || sug.album !== track.album)) return true

  // Genre change?
  if (sug.genre && sug.genre !== track.genre_name) return true

  // Release date change? Do a simple string compare; if either is missing treat as a change
  if (sug.release_date) {
    if (!track.release_date) return true
    // Normalize to date-only if possible for a fairer comparison
    const normalize = (d: string | undefined | null) => {
      if (!d) return ''
      try {
        const dt = new Date(d)
        if (!isNaN(dt.getTime())) return dt.toISOString()
      } catch (e) {}
      return d || ''
    }
    const sNorm = normalize(sug.release_date)
    const tNorm = normalize(track.release_date)
    if (sNorm && sNorm !== tNorm) return true
  }

  // If none of the above indicate a change, skip the suggestion
  return false
}

const applySuggestion = async (suggestion: MetadataSuggestion & { track: MusicFile }) => {
  if (!suggestion.track) return
  suggestionProcessing.value = suggestion.music_file_id
  
  try {
    // 1. Update the track
    let releaseDate = suggestion.release_date || suggestion.track.release_date || null
    
    // Convert partial dates to ISO if needed, as backend expects DateTime<Utc>
    if (releaseDate && !releaseDate.includes('T')) {
        const d = new Date(releaseDate)
        if (!isNaN(d.getTime())) {
            releaseDate = d.toISOString()
        } else {
            releaseDate = null // Invalid date, don't send anything
        }
    }
    
    await musicAPI.updateMusicFile(suggestion.music_file_id, {
      title: suggestion.track.title, // keep existing title
      artist: suggestion.track.artist || '', // keep existing artist
      album: suggestion.album || suggestion.track.album || '',
      release_date: releaseDate || undefined,
      metadata_analyzed: true
    })
    
    // 2. Delete the suggestion (it's applied)
    await musicAPI.deleteMetadataSuggestion(suggestion.music_file_id)
    
    // 3. Remove from UI
    suggestions.value = suggestions.value.filter(s => s.music_file_id !== suggestion.music_file_id)
    // Also remove from enrichment list if present
    enrichmentTracks.value = enrichmentTracks.value.filter(t => t.id !== suggestion.music_file_id)

  } catch (err) {
    console.error('Failed to apply suggestion:', err)
  } finally {
    suggestionProcessing.value = null
  }
}

const rejectSuggestion = async (suggestion: MetadataSuggestion & { track: MusicFile }) => {
  suggestionProcessing.value = suggestion.music_file_id
  try {
    // Mark as analyzed so we don't re-scan immediately
    await musicAPI.updateMusicFile(suggestion.music_file_id, {
        metadata_analyzed: true
    })

    await musicAPI.deleteMetadataSuggestion(suggestion.music_file_id)
    suggestions.value = suggestions.value.filter(s => s.music_file_id !== suggestion.music_file_id)
    // Also remove from enrichment list if present (since analyzed=true now)
    enrichmentTracks.value = enrichmentTracks.value.filter(t => t.id !== suggestion.music_file_id)
  } catch (err) {
    console.error('Failed to reject suggestion:', err)
  } finally {
    suggestionProcessing.value = null
  }
}

const getConfidenceClass = (conf: number) => {
  if (conf >= 0.9) return 'high'
  if (conf >= 0.7) return 'medium'
  return 'low'
}

const applyAllSuggestions = async () => {
    if (isApplyingAll.value) return
    isApplyingAll.value = true
    
    // Process sequentially
    for (const sug of suggestions.value) {
        await applySuggestion(sug)
    }
    
    isApplyingAll.value = false
}

const applyAllSuccessful = async () => {
  if (isApplyingAll.value) return
  isApplyingAll.value = true
  
  const tracksToApply = enrichmentTracks.value.filter(t => scanResults.value[t.id]?.data)
  
  // Apply sequentially to avoid race conditions or DB overhead
  for (const track of tracksToApply) {
    await applyScanResult(track)
  }
  
  isApplyingAll.value = false
}

// SSE Connection
let eventSource: EventSource | null = null

const setupSSE = () => {
    eventSource = new EventSource('/api/updates/stream')
    
    eventSource.onmessage = async (event) => {
        try {
            const data = JSON.parse(event.data)
            
            if (data.type === 'metadata_suggestion_found') {
                const suggestion = data.payload
                
                // Fetch track details for the new suggestion
                try {
                    const trackRes = await musicAPI.getMusicFile(suggestion.music_file_id)
                    const track = trackRes.data
                    // Skip if suggestion contains no meaningful changes
                    if (!shouldIncludeSuggestion(suggestion, track)) {
                      // still mark analyzed removal from enrichment list if present
                      enrichmentTracks.value = enrichmentTracks.value.filter(t => t.id !== suggestion.music_file_id)
                      return
                    }
                    const fullSuggestion = { ...suggestion, track }

                    // Add to list (avoid duplicates)
                    const exists = suggestions.value.some(s => s.music_file_id === fullSuggestion.music_file_id)
                    if (!exists) {
                        suggestions.value.unshift(fullSuggestion)
                      } else {
                        // Update existing
                         suggestions.value = suggestions.value.map(s => 
                            s.music_file_id === fullSuggestion.music_file_id ? fullSuggestion : s
                        )
                      }

                      // Remove from "Missing Metadata" list if it's there
                      enrichmentTracks.value = enrichmentTracks.value.filter(t => t.id !== suggestion.music_file_id)
                      
                } catch (e) {
                    console.error('Failed to load track for new suggestion', e)
                }
            }
        } catch (e) {
            console.error('SSE Error:', e)
        }
    }
    
    eventSource.onerror = (e) => {
        console.error('SSE connection error, closing', e)
        eventSource?.close()
    }
}

onMounted(async () => {
  fetchSuggestions() // Fetch suggestions on load
  fetchMissingMetadataTracks()
  setupSSE() // Start listening for updates
  try {
    const response = await musicAPI.getPlaylists()
    playlists.value = response.data
  } catch (error) {
    console.error('Failed to load playlists:', error)
  }
})

onUnmounted(() => {
    if (eventSource) {
        eventSource.close()
    }
})

const handleBulkRename = async () => {
  renameError.value = ''
  renameResult.value = null
  renameLoading.value = true

  try {
    if (!renameForm.value.pattern.trim()) {
      throw new Error('Regex pattern is required')
    }

    const response = await musicAPI.bulkRenameByRegex({
      field: renameForm.value.field,
      pattern: renameForm.value.pattern,
      replacement: renameForm.value.replacement
    })

    renameResult.value = response.data
  } catch (error: any) {
    renameError.value = error.response?.data?.error || error.message || 'Failed to execute bulk rename'
  } finally {
    renameLoading.value = false
  }
}

const handleBulkAddToPlaylist = async () => {
  playlistError.value = ''
  playlistResult.value = null
  playlistLoading.value = true

  try {
    if (!playlistForm.value.playlist_id) {
      throw new Error('Please select a playlist')
    }

    if (!playlistForm.value.pattern.trim()) {
      throw new Error('Regex pattern is required')
    }

    const response = await musicAPI.bulkAddToPlaylistByRegex({
      playlist_id: playlistForm.value.playlist_id,
      field: playlistForm.value.field,
      pattern: playlistForm.value.pattern
    })

    playlistResult.value = response.data
  } catch (error: any) {
    playlistError.value = error.response?.data?.error || error.message || 'Failed to add tracks to playlist'
  } finally {
    playlistLoading.value = false
  }
}
</script>

<style scoped>
.bulk-operations {
  max-width: 1200px;
  margin: 0 auto;
  padding: 24px;
  color: var(--text-color);
}

.tabs-container {
  margin-bottom: 24px;
}

.tabs {
  display: flex;
  gap: 8px;
  background: var(--surface-color);
  padding: 4px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.tab-button {
  flex: 1;
  padding: 10px 16px;
  font-weight: 600;
  font-size: 14px;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tab-button:hover:not(.active) {
  color: var(--text-color);
  background: rgba(255, 255, 255, 0.05);
}

.tab-button.active {
  color: var(--primary-color);
  background: var(--surface-elevated, var(--surface-hover));
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.tab-content {
  padding: 24px 0;
  background: transparent;
  border: none;
  box-shadow: none;
}

.tab-content h3 {
  font-size: 1.25rem;
  font-weight: 700;
  margin-bottom: 8px;
}

.info-text {
  margin-bottom: 20px;
  color: var(--text-secondary);
  font-size: 0.9rem;
  line-height: 1.6;
}

.info-text code {
  background: var(--surface-color);
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.85em;
  color: var(--primary-color);
}

.enrichment-controls {
  display: flex;
  gap: 12px;
  margin-bottom: 24px;
  flex-wrap: wrap;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  font-size: 14px;
  font-weight: 600;
  border: none;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-primary {
  background: var(--primary-color);
  color: #000;
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px var(--primary-glow);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-secondary {
  background: var(--surface-color);
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--surface-hover);
  transform: translateY(-1px);
}

.btn-success {
  background: linear-gradient(135deg, var(--success-color), #059669);
  color: #000;
}

.btn-success:hover:not(:disabled) {
  box-shadow: 0 8px 20px rgba(6, 214, 160, 0.3);
  transform: translateY(-2px);
}

.btn-icon {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 18px;
  padding: 6px;
  border-radius: 8px;
  transition: background 0.2s;
}

.btn-icon:hover {
  background: var(--surface-hover);
}

.btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.loading {
  text-align: center;
  padding: 40px 20px;
  font-size: 15px;
  color: var(--text-secondary);
}

.empty-state {
  text-align: center;
  padding: 48px 20px;
  font-size: 15px;
  color: var(--success-color);
  background: var(--surface-color);
  border-radius: 16px;
  border: 1px dashed var(--border-color);
}

.enrichment-list-container {
  margin-top: 20px;
}

.list-stats {
  margin-bottom: 12px;
  font-weight: 600;
  font-size: 0.9rem;
  color: var(--text-secondary);
}

.enrichment-table {
  width: 100%;
  border-collapse: collapse;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
}

.enrichment-table th,
.enrichment-table td {
  padding: 14px 16px;
  text-align: left;
  border-bottom: 1px solid var(--border-color);
}

.enrichment-table th {
  background: rgba(255, 255, 255, 0.03);
  font-weight: 700;
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-tertiary);
}

.enrichment-table tr:last-child td {
  border-bottom: none;
}

.enrichment-table tr:hover {
  background: var(--surface-hover);
}

.enrichment-table tr.has-error {
  background: rgba(248, 113, 113, 0.05);
}

.track-info {
  display: flex;
  flex-direction: column;
  gap: 2px;
}

.track-title {
  font-weight: 600;
  color: var(--text-color);
}

.track-artist {
  font-size: 13px;
  color: var(--text-secondary);
}

.missing-badges {
  display: flex;
  gap: 6px;
  flex-wrap: wrap;
}

.badge {
  padding: 4px 10px;
  font-size: 11px;
  font-weight: 600;
  border-radius: 999px;
  background: rgba(30, 215, 96, 0.15);
  color: var(--primary-color);
}

.scan-result {
  display: flex;
  flex-direction: column;
}

.status-scanning {
  color: var(--primary-color);
  font-size: 13px;
  font-weight: 500;
}

.status-error {
  color: var(--text-tertiary);
  font-size: 13px;
  font-style: italic;
}

.suggested-data {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.suggestion {
  font-size: 13px;
  color: var(--text-color);
}

.confidence {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 2px;
}

.actions {
  display: flex;
  gap: 6px;
}

/* Form styles */
.form {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 24px;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
}

.form-group input,
.form-group select {
  width: 100%;
  background: var(--background-base);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 12px 14px;
  color: var(--text-color);
  font-size: 14px;
  transition: all 0.2s ease;
}

.form-group input:focus,
.form-group select:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.form-group select {
  cursor: pointer;
}

.help-text {
  display: block;
  margin-top: 6px;
  font-size: 12px;
  color: var(--text-tertiary);
}

/* Results section */
.result {
  margin-top: 24px;
  padding: 20px;
  border: 1px solid rgba(6, 214, 160, 0.3);
  border-radius: 16px;
  background: rgba(6, 214, 160, 0.05);
}

.result h4 {
  margin-bottom: 8px;
  font-size: 1rem;
}

.success-text {
  font-weight: 600;
  color: var(--success-color);
}

.updated-items {
  margin-top: 16px;
}

.updated-items h5 {
  margin-bottom: 10px;
  font-size: 0.85rem;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.item {
  padding: 12px 16px;
  border: 1px solid var(--border-color);
  border-radius: 10px;
  background: var(--surface-color);
}

.item-info {
  font-size: 14px;
  color: var(--text-color);
}

.error {
  margin-top: 24px;
  padding: 16px 20px;
  border: 1px solid rgba(248, 113, 113, 0.3);
  border-radius: 16px;
  background: rgba(248, 113, 113, 0.05);
  color: var(--error-color);
}

.text-muted {
  color: var(--text-tertiary);
  font-size: 13px;
}

.processing-indicator {
  font-size: 14px;
  color: var(--primary-color);
  font-weight: 500;
}

/* Suggestions section */
.suggestions-container {
  margin-top: 24px;
  background: var(--surface-color);
  padding: 24px;
  border-radius: 16px;
  border: 1px solid var(--border-color);
}

.suggestions-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.suggestion-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px;
  border: 1px solid var(--border-color);
  border-radius: 12px;
  background: var(--background-base);
  gap: 16px;
}

.suggestion-content {
  display: flex;
  align-items: center;
  flex: 1;
  gap: 24px;
}

.track-info-col,
.suggested-info-col {
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.arrow-col {
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--text-tertiary);
}

.label {
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-tertiary);
  margin-bottom: 4px;
  letter-spacing: 0.05em;
}

.current-meta {
  color: var(--text-tertiary);
}

.suggested-fields {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.field {
  font-size: 14px;
  color: var(--text-color);
  padding: 2px 6px;
  border-radius: 4px;
}

.field.highlight {
  background: rgba(30, 215, 96, 0.1);
  color: var(--primary-color);
  border: 1px solid rgba(30, 215, 96, 0.2);
}

.field-label {
  color: var(--text-tertiary);
  font-size: 0.85em;
  margin-right: 4px;
}

.confidence-badge {
  display: inline-block;
  padding: 2px 6px;
  border-radius: 4px;
  font-size: 0.75em;
  font-weight: 700;
  margin-left: auto;
}

.confidence-badge.high {
  background: rgba(30, 215, 96, 0.2);
  color: var(--success-color);
}

.confidence-badge.medium {
  background: rgba(255, 193, 7, 0.2);
  color: #ffc107;
}

.confidence-badge.low {
  background: rgba(244, 67, 54, 0.2);
  color: #f44336;
}

.actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.btn-icon.danger {
  color: #f44336;
}

.btn-icon.danger:hover {
  background: rgba(244, 67, 54, 0.1);
}

/* New Styles for Refactored UI */

.header-section h3 {
  font-size: 1.5rem;
  color: var(--text-color);
  margin-bottom: 8px;
}

.suggestions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(350px, 1fr));
  gap: 16px;
}

.suggestion-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  transition: transform 0.2s, box-shadow 0.2s;
  display: flex;
  flex-direction: column;
}

.suggestion-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0,0,0,0.2);
  border-color: var(--primary-color-dim);
}

.card-header {
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
}

.card-body {
  padding: 16px;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.track-section, .suggestion-section {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.section-label {
  font-size: 0.7rem;
  text-transform: uppercase;
  color: var(--text-tertiary);
  font-weight: 700;
  letter-spacing: 0.05em;
}

.divider {
  display: flex;
  justify-content: center;
  align-items: center;
  height: 20px;
}

.card-actions {
  display: flex;
  border-top: 1px solid var(--border-color);
}

.btn-reject, .btn-accept {
  flex: 1;
  padding: 12px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: background 0.2s;
}

.btn-reject {
  color: var(--text-secondary);
  border-right: 1px solid var(--border-color);
}

.btn-reject:hover {
  background: rgba(244, 67, 54, 0.1);
  color: #f44336;
}

.btn-accept {
  color: var(--primary-color);
}

.btn-accept:hover {
  background: rgba(30, 215, 96, 0.1);
  color: var(--success-color);
}

.glass-panel {
  background: rgba(255, 255, 255, 0.03);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 12px 16px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  backdrop-filter: blur(10px);
}

.control-group {
    display: flex;
    align-items: center;
    gap: 12px;
}

.stats-text {
    font-size: 0.9rem;
    color: var(--text-secondary);
}

.suggestion-pill {
    display: inline-flex;
    align-items: center;
    gap: 4px;
    padding: 2px 8px;
    background: rgba(30, 215, 96, 0.08); /* slight green tint */
    border-radius: 4px;
    font-size: 0.75rem;
    color: var(--text-secondary);
    border: 1px solid rgba(30, 215, 96, 0.15);
}

.badge.missing {
    background: rgba(255, 193, 7, 0.15);
    color: #ffc107;
}

.empty-icon {
    color: var(--success-color);
    margin-bottom: 16px;
    opacity: 0.5;
}

/* Transitions */
.list-move,
.list-enter-active,
.list-leave-active {
  transition: all 0.5s ease;
}

.list-enter-from,
.list-leave-to {
  opacity: 0;
  transform: translateY(30px);
}

.list-leave-active {
  position: absolute;
}
</style>
