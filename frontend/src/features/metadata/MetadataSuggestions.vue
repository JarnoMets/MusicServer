<template>
  <div class="metadata-suggestions-view">
    <div class="header-section">
      <div>
        <h1>Metadata Suggestions</h1>
        <p class="description">Review and apply metadata found by the background scanner.</p>
      </div>
      <div class="actions">
        <button 
          @click="performRemoveAll" 
          class="btn btn-danger"
          :disabled="isLoading || suggestions.length === 0"
        >
          <Icon name="trash" :size="16" />
          Reject All
        </button>
        <button 
          @click="() => performFetch()" 
          class="btn btn-secondary"
          :disabled="isLoading"
        >
          <Icon name="refresh" :size="16" :class="{ 'animate-spin': isLoading }" />
          Refresh
        </button>
      </div>
    </div>

    <div v-if="isLoading" class="loading-state">
      <Icon name="loader" class="animate-spin text-primary" :size="32" />
      <p>Loading suggestions...</p>
    </div>

    <div v-else-if="suggestions.length === 0" class="empty-state">
      <div class="empty-icon"><Icon name="check-circle" :size="48" /></div>
      <h3>All caught up!</h3>
      <p>No pending metadata suggestions found.</p>
    </div>

    <div v-else class="suggestions-container">
       <div class="suggestions-grid">
          <div v-for="suggestion in suggestions" :key="suggestion.music_file_id" class="suggestion-card">
            <div class="card-header">
              <span class="confidence-indicator" :class="getConfidenceClass(suggestion.confidence)">
                {{ Math.round(suggestion.confidence * 100) }}% match
              </span>
            </div>
            <div class="card-body">
              <!-- Current track info -->
              <div class="track-section">
                <span class="section-label">Current Track</span>
                <div class="track-title">{{ truncate(suggestion.track?.title || 'Unknown', 50) }}</div>
                <div class="track-artist">{{ suggestion.track?.artist || 'Unknown Artist' }}</div>
                <div class="track-meta-row">
                  <span class="meta-tag" v-if="suggestion.track?.album">{{ suggestion.track.album }}</span>
                  <span class="meta-tag" v-if="suggestion.track?.genre_name">{{ suggestion.track.genre_name }}</span>
                  <span class="meta-tag" v-if="suggestion.track?.release_date">{{ suggestion.track.release_date }}</span>
                </div>
              </div>

              <div class="divider">
                <Icon name="arrow-right" :size="16" />
              </div>

              <!-- Suggested changes -->
              <div class="suggestion-section">
                <span class="section-label">Suggested Changes</span>
                <div class="suggested-fields">
                  <div class="field" :class="{ highlight: suggestion.album && suggestion.album !== suggestion.track?.album }" v-if="suggestion.album">
                    <Icon name="disc" :size="14" /> Album: {{ suggestion.album }}
                  </div>
                  <div class="field" :class="{ highlight: suggestion.genre && suggestion.genre !== suggestion.track?.genre_name }" v-if="suggestion.genre">
                    <Icon name="tag" :size="14" /> Genre: {{ suggestion.genre }}
                  </div>
                  <div class="field" :class="{ highlight: suggestion.release_date && suggestion.release_date !== suggestion.track?.release_date }" v-if="suggestion.release_date">
                    <Icon name="calendar" :size="14" /> Release: {{ suggestion.release_date }}
                  </div>
                  <div v-if="!suggestion.album && !suggestion.genre && !suggestion.release_date" class="field text-tertiary">
                    No metadata changes suggested
                  </div>
                </div>
              </div>
            </div>
            <div class="card-actions">
              <button class="btn-reject" :disabled="isProcessing === suggestion.music_file_id" @click="rejectSuggestion(suggestion)">
                <Icon name="x" :size="16" /> Reject
              </button>
              <button class="btn-accept" :disabled="isProcessing === suggestion.music_file_id" @click="applySuggestion(suggestion)">
                <Icon name="check" :size="16" /> Apply
              </button>
            </div>
          </div>
       </div>

       <!-- Pagination -->
       <div class="pagination-bar" v-if="pagination.total > pagination.limit">
          <button class="btn btn-secondary" :disabled="pagination.offset === 0" @click="prevPage">
            <Icon name="arrow-left" :size="16" /> Previous
          </button>
          <span class="pagination-info">
            {{ pagination.offset + 1 }} - {{ Math.min(pagination.offset + pagination.limit, pagination.total) }} of {{ pagination.total }}
          </span>
          <button class="btn btn-secondary" :disabled="pagination.offset + pagination.limit >= pagination.total" @click="nextPage">
            Next <Icon name="arrow-right" :size="16" />
          </button>
       </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { musicAPI } from '@/api/music'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import Icon from '@/shared/components/Icons.vue'
import type { MetadataSuggestion } from '@/types/metadata'
import type { MusicFile } from '@/types'

const { success, error } = useToast()
const { confirm } = useConfirm()
const suggestions = ref<(MetadataSuggestion & { track?: MusicFile })[]>([])
const isLoading = ref(true)
const isProcessing = ref<string | null>(null)
const pagination = ref({ offset: 0, limit: 24, total: 0 })

const truncate = (str: string, length: number) => {
  if (!str) return ''
  return str.length > length ? str.substring(0, length) + '...' : str
}

const performFetch = async (reset = false) => {
    if (reset) pagination.value.offset = 0
    isLoading.value = true
    await fetchSuggestions()
    isLoading.value = false
}

const nextPage = () => {
    if (pagination.value.offset + pagination.value.limit < pagination.value.total) {
        pagination.value.offset += pagination.value.limit
        performFetch()
    }
}

const prevPage = () => {
    if (pagination.value.offset > 0) {
        pagination.value.offset = Math.max(0, pagination.value.offset - pagination.value.limit)
        performFetch()
    }
}

const performRemoveAll = async () => {
  const confirmed = await confirm({
    title: 'Reject All Suggestions?',
    message: 'Are you sure you want to reject ALL pending metadata suggestions? This cannot be undone.',
    confirmText: 'Reject All',
    variant: 'danger'
  })

  if (!confirmed) return
  
  isLoading.value = true
  try {
    await musicAPI.deleteAllMetadataSuggestions()
    suggestions.value = []
    pagination.value.total = 0
    success('Cleared', 'All suggestions rejected')
  } catch(err) {
    console.error(err)
    error('Error', 'Failed to reject suggestions')
  } finally {
    isLoading.value = false
  }
}

const fetchSuggestions = async () => {
  try {
    const { data } = await musicAPI.getMetadataSuggestions({
        limit: pagination.value.limit,
        offset: pagination.value.offset
    })
    
    pagination.value.total = data.total
    const incomingSuggestions = data.suggestions || []
    
    // Fetch track details for each suggestion
    const promises = incomingSuggestions.map(async (sug: MetadataSuggestion) => {
        try {
            const trackRes = await musicAPI.getMusicFile(sug.music_file_id)
            return { ...sug, track: trackRes.data }
        } catch (e) {
            console.warn('Track not found for suggestion', sug.music_file_id)
            return null
        }
    })
    
    const results = await Promise.all(promises)
    // Convert and filter out suggestions where the only suggested change is a release year
    // that's identical to the existing track year (these are noisy and not helpful).
    const rawSuggestions = results.filter((s: any) => s !== null && s.track)

    const extractYear = (d: string | null | undefined) => {
      if (!d) return null
      // If it's an ISO-ish string with time, parse it
      try {
        if (d.includes('T') || d.includes('-')) {
          const parsed = new Date(d)
          if (!isNaN(parsed.getTime())) return parsed.getFullYear()
        }
      } catch (e) {
        // fallthrough
      }
      // Try to extract a 4-digit year
      const m = (d || '').match(/(\d{4})/)
      return m ? Number(m[1]) : null
    }
    // Helper: determine if suggestion proposes any meaningful change
    const shouldIncludeSuggestion = (s: any) => {
      if (!s) return false
      // If suggestion proposes nothing, skip it
      const hasAnyField = !!(s.album || s.genre || s.release_date)
      if (!hasAnyField) return false

      const track = s.track
      // If no track data, include (can't compare)
      if (!track) return true

      // Album change
      if (s.album && (!track.album || s.album !== track.album)) return true

      // Genre change
      if (s.genre && s.genre !== track.genre_name) return true

      // Release date change: compare normalized ISO or year
      if (s.release_date) {
        if (!track.release_date) return true
        const normalize = (d: string | null | undefined) => {
          if (!d) return ''
          try {
            const dt = new Date(d)
            if (!isNaN(dt.getTime())) return dt.toISOString()
          } catch (e) {}
          return d || ''
        }
        const sNorm = normalize(s.release_date)
        const tNorm = normalize(track.release_date)
        if (sNorm && sNorm !== tNorm) return true
        // As fallback, compare years
        const ys = extractYear(s.release_date)
        const yt = extractYear(track.release_date)
        if (ys && yt && ys !== yt) return true
      }

      // Nothing different
      return false
    }

    // Filter suggestions to only include those with actual changes
    const filtered = rawSuggestions.filter((s: any) => shouldIncludeSuggestion(s))

    // For suggestions we keep: if the suggested release_date is only the same year
    // as the existing track date, remove the release_date field so UI won't show it.
    filtered.forEach((s: any) => {
      const sugDate = s.release_date
      const trackDate = s.track?.release_date
      if (sugDate && trackDate) {
        const ys = extractYear(sugDate)
        const yt = extractYear(trackDate)
        if (ys && yt && ys === yt) {
          delete s.release_date
        }
      }
    })

    suggestions.value = filtered
    
  } catch (err) {
    console.error('Failed to fetch suggestions:', err)
    error('Error', 'Failed to load metadata suggestions')
  }
}

const getConfidenceClass = (conf: number) => {
  if (conf >= 0.9) return 'high'
  if (conf >= 0.7) return 'medium'
  return 'low'
}

const applySuggestion = async (suggestion: MetadataSuggestion & { track?: MusicFile }) => {
  if (!suggestion.track) return
  isProcessing.value = suggestion.music_file_id
  
  try {
    let releaseDate = suggestion.release_date || suggestion.track.release_date || null
    
    if (releaseDate && !releaseDate.includes('T')) {
         const d = new Date(releaseDate)
         if (!isNaN(d.getTime())) {
             releaseDate = d.toISOString()
         } else {
             releaseDate = null
         }
    }

    await musicAPI.updateMusicFile(suggestion.music_file_id, {
      title: suggestion.track.title,
      artist: suggestion.track.artist || '',
      album: suggestion.album || suggestion.track.album || '',
      release_date: releaseDate || undefined,
      metadata_analyzed: true
    })
    
    await musicAPI.deleteMetadataSuggestion(suggestion.music_file_id)
    suggestions.value = suggestions.value.filter(s => s.music_file_id !== suggestion.music_file_id)
    
    success('Applied', `Updated metadata for "${suggestion.track.title}"`)
  } catch (err) {
    console.error(err)
    error('Failed', 'Could not apply suggestion')
  } finally {
    isProcessing.value = null
  }
}

const rejectSuggestion = async (suggestion: MetadataSuggestion & { track?: MusicFile }) => {
  isProcessing.value = suggestion.music_file_id
  try {
    await musicAPI.updateMusicFile(suggestion.music_file_id, {
        metadata_analyzed: true
    })

    await musicAPI.deleteMetadataSuggestion(suggestion.music_file_id)
    suggestions.value = suggestions.value.filter(s => s.music_file_id !== suggestion.music_file_id)
    success('Rejected', 'Suggestion removed')
  } catch (err) {
    console.error(err)
    error('Failed', 'Could not reject suggestion')
  } finally {
    isProcessing.value = null
  }
}

onMounted(() => {
  performFetch()
})
</script>

<style scoped>
.metadata-suggestions-view {
  max-width: 1400px;
  margin: 0 auto;
  padding: 32px;
  color: var(--text-color);
}

.header-section {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 32px;
}

.header-section h1 {
  font-size: 2rem;
  font-weight: 800;
  margin-bottom: 8px;
  background: linear-gradient(to right, #fff, #ccc);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
}

.description {
  color: var(--text-secondary);
  font-size: 1rem;
}

.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  font-weight: 600;
  border-radius: 8px;
  border: none;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 0.9rem;
}

.btn-secondary {
  background: var(--surface-color);
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--surface-hover);
  border-color: var(--text-secondary);
}

.btn-danger {
  background: rgba(244, 67, 54, 0.1);
  color: #f44336;
  border: 1px solid rgba(244, 67, 54, 0.2);
}

.btn-danger:hover:not(:disabled) {
  background: rgba(244, 67, 54, 0.2);
  border-color: #f44336;
}

.loading-state, .empty-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 64px;
  text-align: center;
  background: var(--surface-color);
  border-radius: 16px;
  border: 1px dashed var(--border-color);
}

.empty-icon {
  color: var(--success-color);
  margin-bottom: 16px;
}

.empty-state h3 {
  font-size: 1.25rem;
  font-weight: 700;
  margin-bottom: 8px;
}

.empty-state p {
  color: var(--text-secondary);
}

/* Grid Layout */
.suggestions-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(360px, 1fr));
  gap: 24px;
}

.suggestion-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  transition: all 0.2s ease;
  display: flex;
  flex-direction: column;
  position: relative;
}

.suggestion-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0,0,0,0.2);
  border-color: var(--primary-color-dim);
}

.card-header {
  padding: 12px 16px;
  background: rgba(255, 255, 255, 0.02);
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
}

.confidence-indicator {
  font-size: 0.75rem;
  font-weight: 700;
  padding: 4px 8px;
  border-radius: 4px;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.confidence-indicator.high {
  background: rgba(30, 215, 96, 0.15);
  color: var(--success-color);
}

.confidence-indicator.medium {
  background: rgba(255, 193, 7, 0.15);
  color: #fbbf24;
}

.confidence-indicator.low {
  background: rgba(244, 67, 54, 0.15);
  color: #f87171;
}

.card-body {
  padding: 20px;
  flex: 1;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.track-section {
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
  margin-bottom: 4px;
  display: block;
}

.track-title {
  font-weight: 700;
  font-size: 1.1rem;
  margin-bottom: 2px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-artist {
  color: var(--text-secondary);
  font-size: 0.95rem;
  margin-bottom: 8px;
}

.track-meta-row {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.meta-tag {
  font-size: 0.75rem;
  padding: 2px 6px;
  background: rgba(255,255,255,0.05);
  border-radius: 4px;
  color: var(--text-tertiary);
}

.divider {
  display: flex;
  justify-content: center;
  align-items: center;
  opacity: 0.5;
}

.suggestion-section {
    background: rgba(var(--primary-color-rgb), 0.05);
    border-radius: 8px;
    padding: 12px;
    border: 1px dashed var(--border-color);
}

.suggested-fields {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.field {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 0.9rem;
  padding: 6px 10px;
  border-radius: 6px;
}

.field.highlight {
  background: rgba(30, 215, 96, 0.1);
  color: var(--primary-color);
  font-weight: 500;
}

.card-actions {
  display: flex;
  border-top: 1px solid var(--border-color);
}

.btn-reject, .btn-accept {
  flex: 1;
  padding: 16px;
  border: none;
  background: transparent;
  cursor: pointer;
  font-weight: 600;
  font-size: 0.9rem;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  transition: all 0.2s;
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

.btn:disabled, .btn-reject:disabled, .btn-accept:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.text-tertiary { color: var(--text-tertiary); }
.text-primary { color: var(--primary-color); }

.pagination-bar {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 24px;
  margin-top: 48px;
  padding: 24px;
  background: var(--surface-color);
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.pagination-info {
  font-weight: 600;
  color: var(--text-secondary);
  font-size: 0.95rem;
}
</style>
