<template>
  <div class="track-browser">
    <div class="browser-header">
      <h3 class="browser-title">
        <Icon name="folder" :size="16" />
        Browse
      </h3>
      <div class="browser-source-tabs">
        <button
          v-for="tab in sourceTabs"
          :key="tab.value"
          class="source-tab"
          :class="{ active: source === tab.value }"
          @click="source = tab.value"
        >
          <Icon :name="tab.icon" :size="14" />
          {{ tab.label }}
        </button>
      </div>
    </div>

    <!-- Source Selection -->
    <div class="browser-filters">
      <div class="search-box">
        <Icon name="search" :size="14" />
        <input
          type="text"
          v-model="searchQuery"
          placeholder="Search tracks..."
          class="search-input"
        />
      </div>

      <select v-if="source === 'playlist'" v-model="selectedPlaylistId" class="browser-select">
        <option value="">Select playlist...</option>
        <option v-for="pl in playlists" :key="pl.id" :value="pl.id">{{ pl.name }}</option>
      </select>

      <select v-if="source === 'genre'" v-model="selectedGenre" class="browser-select">
        <option value="">All genres...</option>
        <option v-for="g in genres" :key="g" :value="g">{{ g }}</option>
      </select>

      <button 
        v-if="tracks.length > 0"
        class="btn-add-autoplay" 
        :class="{ success: showSuccessFeedback }"
        @click="addAllToAutoplay"
        :title="`Add ${tracks.length} tracks to DJ queue`"
      >
        <Icon :name="showSuccessFeedback ? 'check' : 'plus-circle'" :size="12" />
        {{ showSuccessFeedback ? 'Added to Queue' : 'Add all to Queue' }}
      </button>
    </div>

    <!-- Track List -->
    <div class="browser-track-list" ref="trackListRef">
      <div v-if="loading" class="browser-loading">
        <Icon name="loader" :size="18" class="animate-spin" />
        Loading...
      </div>

      <div v-else-if="tracks.length === 0" class="browser-empty">
        No tracks found
      </div>

      <div
        v-for="track in tracks"
        :key="track.id"
        class="browser-track"
        :class="{ 'is-loaded': isTrackLoaded(track.id) }"
        draggable="true"
        @dragstart="handleDragStart(track, $event)"
        @dblclick="handleTrackDoubleClick(track)"
      >
        <div class="track-main">
          <div class="track-name">{{ track.title }}</div>
          <div class="track-meta">
            {{ track.artist || 'Unknown' }}
            <span v-if="track.genre" class="track-genre-badge">{{ track.genre }}</span>
          </div>
        </div>
        <div class="track-stats">
          <span v-if="track.bpm" class="track-bpm">{{ Math.round(track.bpm) }}</span>
          <span v-if="track.initial_key && track.initial_key !== 'NONE'" class="track-key">{{ track.initial_key }}</span>
          <span class="track-duration">{{ formatTrackDuration(track.duration) }}</span>
        </div>
        <div class="track-load-btns">
          <button
            class="btn-add-queue"
            @click="addTrackToQueue(track)"
            title="Add to DJ Queue"
          >
            <Icon name="plus" :size="10" />
          </button>
          <template v-if="targetDeckId">
            <button
              class="btn-load-deck"
              :class="[`deck-${targetDeckId}`]"
              @click="loadToDeck(targetDeckId, track)"
              :title="`Load to Deck ${targetDeckId}`"
            >
              Load
            </button>
          </template>
          <template v-else>
            <button
              v-for="d in ([1, 2, 3, 4] as const)"
              :key="d"
              class="btn-load-deck"
              :class="[`deck-${d}`]"
              @click="loadToDeck(d, track)"
              :title="`Load to Deck ${d}`"
            >
              {{ d }}
            </button>
          </template>
        </div>
      </div>

      <!-- Load More -->
      <button v-if="hasMore" class="btn-load-more" @click="loadMore">
        Load more...
      </button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, watch, onMounted, computed } from 'vue'
import { musicAPI } from '../../api/music'
import { useDjStore } from '../../stores/djStore'
import { useDjAudioEngine } from '../../composables/useDjAudioEngine'
import { formatDuration } from '../../utils/musicFormatters'
import type { MusicFile } from '../../types/index'
import type { DeckId, BrowserSource } from '../../types/dj'
import Icon from '../../shared/components/Icons.vue'

const props = defineProps<{
  targetDeckId?: DeckId
}>()

const emit = defineEmits<{
  loaded: [track: MusicFile]
}>()

const store = useDjStore()
const engine = useDjAudioEngine()

const source = computed({
  get: () => store.browserState.source,
  set: (val) => store.browserState.source = val
})
const searchQuery = computed({
  get: () => store.browserState.searchQuery,
  set: (val) => store.browserState.searchQuery = val
})
const selectedPlaylistId = computed({
  get: () => store.browserState.playlistId || '',
  set: (val) => store.browserState.playlistId = val || null
})
const selectedGenre = computed({
  get: () => store.browserState.genreFilter,
  set: (val) => store.browserState.genreFilter = val
})

const loading = ref(false)
const tracks = ref<MusicFile[]>([])
const playlists = ref<{ id: string; name: string }[]>([])
const genres = ref<string[]>([])
const hasMore = ref(false)
const page = ref(1)
const limit = 50
const trackListRef = ref<HTMLElement | null>(null)

const sourceTabs = [
  { value: 'all' as BrowserSource, label: 'All', icon: 'list' },
  { value: 'playlist' as BrowserSource, label: 'Playlists', icon: 'music' },
  { value: 'genre' as BrowserSource, label: 'Genres', icon: 'tag' },
]

// ─── Data Loading ────────────────────────────────────────────────

const fetchTracks = async (reset = true) => {
  if (reset) {
    page.value = 1
    tracks.value = []
  }

  loading.value = true
  try {
    if (source.value === 'playlist' && selectedPlaylistId.value) {
      const res = await musicAPI.getPlaylist(selectedPlaylistId.value)
      tracks.value = res.data.items || []
      hasMore.value = false
    } else {
      const params: Record<string, any> = {
        limit,
        offset: (page.value - 1) * limit,
        sort: 'title',
        order: 'asc',
      }
      if (searchQuery.value) params.search = searchQuery.value
      if (source.value === 'genre' && selectedGenre.value) params.genre = selectedGenre.value

      const res = await musicAPI.getMusicFiles(params)
      if (reset) {
        tracks.value = res.data
      } else {
        tracks.value = [...tracks.value, ...res.data]
      }
      hasMore.value = res.data.length === limit
    }
  } catch (e) {
    console.warn('Failed to fetch tracks:', e)
  } finally {
    loading.value = false
  }
}

const loadMore = () => {
  page.value++
  fetchTracks(false)
}

const fetchPlaylists = async () => {
  try {
    const res = await musicAPI.getPlaylists()
    playlists.value = res.data
  } catch { /* */ }
}

const fetchGenres = async () => {
  try {
    const res = await musicAPI.listGenres()
    genres.value = res.data.map((g: { name: string }) => g.name)
  } catch { /* */ }
}

// ─── Debounced Search ────────────────────────────────────────────

let searchTimeout: number | undefined
watch(searchQuery, () => {
  clearTimeout(searchTimeout)
  searchTimeout = window.setTimeout(() => fetchTracks(), 350)
})

watch(source, () => fetchTracks())
watch(selectedPlaylistId, () => { if (source.value === 'playlist') fetchTracks() })
watch(selectedGenre, () => { if (source.value === 'genre') fetchTracks() })

// ─── Actions ─────────────────────────────────────────────────────

const loadToDeck = (deckId: DeckId, track: MusicFile) => {
  engine.loadTrackToDeck(deckId, track)
  emit('loaded', track)
}

const handleTrackDoubleClick = (track: MusicFile) => {
  if (props.targetDeckId) {
    loadToDeck(props.targetDeckId, track)
    return
  }
  // Load to first empty deck, or deck 1
  for (const d of store.decks) {
    if (d.playState === 'empty') {
      engine.loadTrackToDeck(d.id, track)
      emit('loaded', track)
      return
    }
  }
  engine.loadTrackToDeck(1, track)
  emit('loaded', track)
}

const handleDragStart = (track: MusicFile, e: DragEvent) => {
  if (e.dataTransfer) {
    e.dataTransfer.setData('application/json', JSON.stringify(track))
    e.dataTransfer.effectAllowed = 'copy'
  }
}

const isTrackLoaded = (trackId: string): boolean => {
  return store.decks.some(d => d.track?.id === trackId)
}

const formatTrackDuration = (ms?: number | null): string => {
  if (!ms) return '--:--'
  return formatDuration(ms)
}

const addAllToAutoplay = () => {
  if (tracks.value.length === 0) return
  
  store.addTracksToQueue(tracks.value)
  
  // Provide feedback
  showSuccessFeedback.value = true
  setTimeout(() => showSuccessFeedback.value = false, 2000)
}

const addTrackToQueue = (track: MusicFile) => {
  store.addToQueue(track)
}

const showSuccessFeedback = ref(false)

// ─── Init ────────────────────────────────────────────────────────

onMounted(() => {
  fetchTracks()
  fetchPlaylists()
  fetchGenres()
})
</script>

<style scoped>
.track-browser {
  background: var(--surface-color, #1a1a2e);
  border: 1px solid var(--border-color, #2a2a3e);
  border-radius: 12px;
  display: flex;
  flex-direction: column;
  overflow: hidden;
  height: 100%;
}

.browser-header {
  padding: 10px 12px;
  border-bottom: 1px solid rgba(255,255,255,0.06);
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.browser-title {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 13px;
  font-weight: 700;
  color: var(--text-color, #fff);
  margin: 0;
}

.browser-source-tabs {
  display: flex;
  gap: 4px;
}

.source-tab {
  flex: 1;
  padding: 5px 8px;
  border-radius: 6px;
  border: 1px solid rgba(255,255,255,0.06);
  background: rgba(255,255,255,0.02);
  color: var(--text-secondary, #aaa);
  font-size: 11px;
  font-weight: 600;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 4px;
  transition: all 0.15s;
}

.source-tab:hover {
  background: rgba(255,255,255,0.06);
}

.source-tab.active {
  background: var(--primary-color, #4f46e5);
  border-color: var(--primary-color, #4f46e5);
  color: white;
}

.browser-filters {
  padding: 8px 12px;
  display: flex;
  flex-direction: column;
  gap: 6px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
}

.search-box {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 10px;
  background: rgba(255,255,255,0.05);
  border-radius: 6px;
  border: 1px solid rgba(255,255,255,0.06);
}

.search-box .icon {
  color: var(--text-tertiary, #666);
  flex-shrink: 0;
}

.search-input {
  flex: 1;
  background: transparent;
  border: none;
  color: var(--text-color, #fff);
  font-size: 12px;
  outline: none;
  min-width: 0;
}

.search-input::placeholder {
  color: var(--text-tertiary, #555);
}

.browser-select {
  padding: 6px 8px;
  background: rgba(255,255,255,0.05);
  border: 1px solid rgba(255,255,255,0.08);
  border-radius: 6px;
  color: var(--text-color, #fff);
  font-size: 12px;
  outline: none;
  cursor: pointer;
}

.browser-select option {
  background: #1a1a2e;
  color: white;
}

.btn-add-autoplay {
  background: rgba(34, 197, 94, 0.15);
  border: 1px solid rgba(34, 197, 94, 0.3);
  color: #22c55e;
  padding: 6px 10px;
  border-radius: 6px;
  font-size: 11px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 6px;
  transition: all 0.2s;
  margin-top: 4px;
}

.btn-add-autoplay:hover {
  background: rgba(34, 197, 94, 0.25);
  transform: translateY(-1px);
}

.btn-add-autoplay.success {
  background: #22c55e;
  color: white;
  border-color: #22c55e;
}

/* Track List */
.browser-track-list {
  flex: 1;
  overflow-y: auto;
  padding: 4px;
}

.browser-loading,
.browser-empty {
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 24px;
  color: var(--text-tertiary, #666);
  font-size: 12px;
}

.browser-track {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 8px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.12s;
  user-select: none;
}

.browser-track:hover {
  background: rgba(255,255,255,0.05);
}

.browser-track.is-loaded {
  background: rgba(79, 70, 229, 0.1);
  border-left: 2px solid var(--primary-color, #4f46e5);
}

.track-main {
  flex: 1;
  min-width: 0;
}

.track-name {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-color, #fff);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.track-meta {
  font-size: 10px;
  color: var(--text-secondary, #aaa);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  display: flex;
  align-items: center;
  gap: 4px;
}

.track-genre-badge {
  padding: 1px 4px;
  background: rgba(255,255,255,0.06);
  border-radius: 3px;
  font-size: 9px;
  color: var(--text-tertiary, #888);
}

.track-stats {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-shrink: 0;
}

.track-bpm {
  font-size: 10px;
  font-weight: 700;
  color: var(--primary-color, #4f46e5);
  font-family: 'Courier New', monospace;
}

.track-key {
  font-size: 10px;
  font-weight: 600;
  color: #22c55e;
}

.track-duration {
  font-size: 10px;
  color: var(--text-tertiary, #666);
  font-family: 'Courier New', monospace;
}

.track-load-btns {
  display: flex;
  gap: 2px;
  flex-shrink: 0;
}

.btn-add-queue {
  min-width: 20px;
  height: 20px;
  padding: 0 4px;
  border-radius: 3px;
  border: 1px solid rgba(245, 158, 11, 0.2);
  background: rgba(245, 158, 11, 0.08);
  color: #f59e0b;
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s;
}

.btn-add-queue:hover {
  background: rgba(245, 158, 11, 0.25);
  border-color: rgba(245, 158, 11, 0.4);
  transform: scale(1.1);
}

.btn-load-deck {
  /* Use auto width if there is a targetDeckId (to accommodate 'Load') */
  min-width: 20px;
  height: 20px;
  padding: 0 4px;
  border-radius: 3px;
  border: 1px solid rgba(255,255,255,0.08);
  background: rgba(255,255,255,0.03);
  color: var(--text-tertiary, #555);
  font-size: 10px;
  font-weight: 700;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.12s;
}

.btn-load-deck:hover {
  transform: scale(1.1);
}

.btn-load-deck.deck-1:hover { background: rgba(79, 70, 229, 0.3); border-color: #4f46e5; color: #818cf8; }
.btn-load-deck.deck-2:hover { background: rgba(239, 68, 68, 0.3); border-color: #ef4444; color: #fca5a5; }
.btn-load-deck.deck-3:hover { background: rgba(34, 197, 94, 0.3); border-color: #22c55e; color: #86efac; }
.btn-load-deck.deck-4:hover { background: rgba(245, 158, 11, 0.3); border-color: #f59e0b; color: #fcd34d; }

.btn-load-more {
  width: 100%;
  padding: 8px;
  border-radius: 6px;
  border: 1px solid rgba(255,255,255,0.06);
  background: rgba(255,255,255,0.03);
  color: var(--text-secondary, #aaa);
  font-size: 11px;
  cursor: pointer;
  margin-top: 4px;
  transition: all 0.15s;
}

.btn-load-more:hover {
  background: rgba(255,255,255,0.08);
}

/* Scrollbar */
.browser-track-list::-webkit-scrollbar {
  width: 4px;
}

.browser-track-list::-webkit-scrollbar-track {
  background: transparent;
}

.browser-track-list::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.1);
  border-radius: 2px;
}
</style>
