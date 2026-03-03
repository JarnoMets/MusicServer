<script setup lang="ts">
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import type { MusicFile, PlaylistSummary } from '../../types'
import { formatDuration, formatDate } from '../../utils/musicFormatters'
import Icon from '../../shared/components/Icons.vue'
import { usePlayer } from '../../composables/usePlayer'

interface Props {
  tracks: MusicFile[]
  playlists: PlaylistSummary[]
  loading: boolean
  playlistMenuOpen: string | null
  canEdit?: boolean
  sort?: 'title' | 'artist' | 'album' | 'genre' | 'created_at' | 'updated_at' | 'release_date' | 'duration' | 'bpm'
  order?: 'asc' | 'desc'
}

const props = withDefaults(defineProps<Props>(), {
  canEdit: false,
  sort: 'title',
  order: 'asc'
})

const emit = defineEmits<{
  'track:play': [track: MusicFile]
  'track:edit': [track: MusicFile]
  'track:delete': [track: MusicFile]
  'track:details': [track: MusicFile]
  'track:confirm-genre': [track: MusicFile]
  'track:contextmenu': [track: MusicFile, event: MouseEvent]
  'tracks:contextmenu': [tracks: MusicFile[], event: MouseEvent]
  'playlist:toggle': [trackId: string]
  'playlist:add': [trackId: string, playlistId: string]
  'update:sort': [value: 'title' | 'artist' | 'album' | 'genre' | 'created_at' | 'updated_at' | 'release_date' | 'duration' | 'bpm']
  'update:order': [value: 'asc' | 'desc']
  reset: []
}>()

const player = usePlayer()
const isCurrentTrack = (track: MusicFile) =>
  player.state.currentSource?.type === 'local' && player.state.currentSource.id === track.id
const isPlaying = computed(() => player.state.isPlaying)

// Selection state
const selectedIds = ref<Set<string>>(new Set())
const lastSelectedIndex = ref<number | null>(null)

const isSelected = (track: MusicFile) => selectedIds.value.has(track.id)

// Clear selection when tracks change (keep only valid ids)
watch(
  () => props.tracks,
  () => {
    const validIds = new Set(props.tracks.map((t) => t.id))
    const newSelection = new Set<string>()
    selectedIds.value.forEach((id) => {
      if (validIds.has(id)) newSelection.add(id)
    })
    selectedIds.value = newSelection
  },
  { deep: false },
)

// Local sorting state (used only for UI controls)
const sortKey = ref(props.sort)
const sortOrder = ref(props.order)

watch(
  () => props.sort,
  (newVal) => {
    if (newVal) sortKey.value = newVal
  },
)
watch(
  () => props.order,
  (newVal) => {
    if (newVal) sortOrder.value = newVal
  },
)

// Display tracks: parent usually provides pre-sorted subset. Use that directly.
const displayTracks = computed(() => props.tracks)

const handleRowClick = (index: number, event: MouseEvent) => {
  const track = displayTracks.value[index]

  if (event.shiftKey && lastSelectedIndex.value !== null) {
    const start = Math.min(lastSelectedIndex.value, index)
    const end = Math.max(lastSelectedIndex.value, index)
    for (let i = start; i <= end; i++) selectedIds.value.add(displayTracks.value[i].id)
  } else if (event.ctrlKey || event.metaKey) {
    if (selectedIds.value.has(track.id)) selectedIds.value.delete(track.id)
    else {
      selectedIds.value.add(track.id)
      lastSelectedIndex.value = index
    }
  } else {
    selectedIds.value.clear()
    selectedIds.value.add(track.id)
    lastSelectedIndex.value = index
  }
}

const handleRowDoubleClick = (track: MusicFile) => {
  handleTogglePlay(track)
}

const handleTitleClick = (track: MusicFile, event: MouseEvent) => {
  event.stopPropagation()
  handleTogglePlay(track)
}

const handleTogglePlay = (track: MusicFile) => {
  if (isCurrentTrack(track)) {
    player.setPlayingStatus(!player.state.isPlaying)
  } else {
    emit('track:play', track)
  }
}

const handleContextMenu = (track: MusicFile, event: MouseEvent) => {
  event.preventDefault()
  if (!selectedIds.value.has(track.id)) {
    selectedIds.value.clear()
    selectedIds.value.add(track.id)
    const idx = displayTracks.value.findIndex((t) => t.id === track.id)
    if (idx !== -1) lastSelectedIndex.value = idx
  }

  if (selectedIds.value.size > 1) {
    const selectedTracksList = props.tracks.filter((t) => selectedIds.value.has(t.id))
    emit('tracks:contextmenu', selectedTracksList, event)
  } else {
    emit('track:contextmenu', track, event)
  }
}

const handleSort = (
  column: 'title' | 'artist' | 'album' | 'genre' | 'created_at' | 'updated_at' | 'release_date' | 'duration' | 'bpm',
) => {
  if (sortKey.value === column) sortOrder.value = sortOrder.value === 'asc' ? 'desc' : 'asc'
  else {
    sortKey.value = column
    sortOrder.value = 'asc'
  }
  emit('update:sort', sortKey.value)
  emit('update:order', sortOrder.value)
}

const onGlobalClick = (e: MouseEvent) => {
  const target = e.target as HTMLElement
  if (!target.closest('.music-table')) {
    selectedIds.value.clear()
    lastSelectedIndex.value = null
  }
}

const onKeyDown = (e: KeyboardEvent) => {
  if (e.code === 'Space' && !['INPUT', 'TEXTAREA'].includes(document.activeElement?.tagName || '')) {
    e.preventDefault()
    if (selectedIds.value.size === 1) {
      const id = Array.from(selectedIds.value)[0]
      const track = props.tracks.find(t => t.id === id)
      if (track) handleTogglePlay(track)
    } else if (player.state.currentSource) {
      player.setPlayingStatus(!player.state.isPlaying)
    }
  } else if (e.code === 'Enter' && selectedIds.value.size === 1) {
    const id = Array.from(selectedIds.value)[0]
    const track = props.tracks.find(t => t.id === id)
    if (track) handleTogglePlay(track)
  }
}

onMounted(() => {
  window.addEventListener('click', onGlobalClick)
  window.addEventListener('keydown', onKeyDown)
})
onUnmounted(() => {
  window.removeEventListener('click', onGlobalClick)
  window.removeEventListener('keydown', onKeyDown)
})
</script>

<template>
  <div class="music-container" :class="{ 'is-loading': loading && tracks.length > 0 }">
    <div v-if="loading && tracks.length > 0" class="loading-progress"></div>
    <div v-if="loading && tracks.length === 0" class="loading-skeleton">
      <div class="skeleton-header">
        <div class="skeleton-bar"></div>
        <div class="skeleton-bar"></div>
        <div class="skeleton-bar"></div>
        <div class="skeleton-bar short"></div>
        <div class="skeleton-bar short"></div>
        <div class="skeleton-bar short"></div>
        <div class="skeleton-bar short"></div>
        <div class="skeleton-bar short"></div>
      </div>
      <div v-for="i in 5" :key="i" class="skeleton-row">
        <div class="skeleton-cell title">
          <div class="skeleton-bar"></div>
        </div>
        <div class="skeleton-cell"><div class="skeleton-bar"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar short"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar badge"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar date"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar short"></div></div>
        <div class="skeleton-cell"><div class="skeleton-bar short"></div></div>
        <div class="skeleton-cell actions">
          <div class="skeleton-circle"></div>
          <div class="skeleton-circle"></div>
        </div>
      </div>
    </div>
    <div v-else-if="tracks.length === 0" class="empty">
      <div class="empty-icon"><Icon name="music" :size="48" /></div>
      <p>No tracks match your filters.</p>
      <button class="btn btn-outline" @click="emit('reset')">Reset filters</button>
    </div>
    <table v-else class="music-table">
      <thead>
        <tr>
          <th class="sortable" @click="handleSort('title')">
            <div class="header-content">
              <span>Title</span>
              <Icon v-if="sortKey === 'title'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th class="sortable" @click="handleSort('artist')">
            <div class="header-content">
              <span>Artist</span>
              <Icon v-if="sortKey === 'artist'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th class="sortable" @click="handleSort('album')">
            <div class="header-content">
              <span>Album</span>
              <Icon v-if="sortKey === 'album'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th class="sortable" @click="handleSort('release_date')">
            <div class="header-content">
              <span>Year</span>
              <Icon v-if="sortKey === 'release_date'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th class="sortable" @click="handleSort('genre')">
            <div class="header-content">
              <span>Genre</span>
              <Icon v-if="sortKey === 'genre'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th class="sortable" @click="handleSort('created_at')">
            <div class="header-content">
              <span>Added</span>
              <Icon v-if="sortKey === 'created_at'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th class="sortable" @click="handleSort('duration')">
            <div class="header-content">
              <span>Duration</span>
              <Icon v-if="sortKey === 'duration'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th class="sortable" @click="handleSort('bpm')">
            <div class="header-content">
              <span>BPM</span>
              <Icon v-if="sortKey === 'bpm'" :name="sortOrder === 'asc' ? 'chevron-up' : 'chevron-down'" :size="12" class="sort-icon" />
            </div>
          </th>
          <th></th>
        </tr>
      </thead>
      <tbody>
        <tr 
          v-for="(track, index) in displayTracks" 
          :key="track.id" 
          :class="['music-row', { 
            'playing': isCurrentTrack(track), 
            'selected': isSelected(track) 
          }]"
          @click="handleRowClick(index, $event)"
          @dblclick="handleRowDoubleClick(track)"
          @contextmenu="handleContextMenu(track, $event)"
          tabindex="0"
        >
          <td :data-artist="track.artist || 'Unknown Artist'">
            <div class="title-cell">
              <div v-if="isCurrentTrack(track)" class="now-playing-indicator">
                <span :class="['bar', { animating: isPlaying }]"></span>
                <span :class="['bar', { animating: isPlaying }]"></span>
                <span :class="['bar', { animating: isPlaying }]"></span>
              </div>
              <div>
                <button class="title-link" @click="handleTitleClick(track, $event)">{{ track.title }}</button>
                <div class="mobile-artist">{{ track.artist || 'Unknown Artist' }}</div>
              </div>
            </div>
          </td>
          <td>{{ track.artist || '&mdash;' }}</td>
          <td>{{ track.album || '&mdash;' }}</td>
          <td>{{ track.release_date ? new Date(track.release_date).getFullYear() : '&mdash;' }}</td>
          <td>
            <span v-if="track.genre" class="genre-badge">{{ track.genre }}</span>
            <span 
              v-else-if="track.guessed_genre" 
              class="genre-badge muted clickable"
              @click.stop="emit('track:confirm-genre', track)"
              title="Click to confirm genre"
            >
              {{ track.guessed_genre }}*
            </span>
            <span v-else class="genre-badge empty">Untagged</span>
          </td>
          <td class="date-cell">{{ formatDate(track.created_at) }}</td>
          <td>
            <div class="duration-cell">
              <Icon name="clock" :size="12" />
              {{ formatDuration(track.duration) }}
            </div>
          </td>
          <td>
            <span v-if="track.bpm" class="bpm-text">{{ Math.round(track.bpm) }}</span>
            <span v-else class="bpm-text muted">&mdash;</span>
          </td>
          <td class="actions">
            <button 
              class="btn-icon play-btn" 
              @click.stop="handleTogglePlay(track)" 
              :title="isCurrentTrack(track) && isPlaying ? 'Pause' : 'Play'"
            >
              <Icon :name="isCurrentTrack(track) && isPlaying ? 'pause' : 'play'" :size="16" />
            </button>
            <!-- Edit actions only when logged in -->
            <template v-if="canEdit">
              <div class="playlist-menu">
                <button
                  class="btn-icon"
                  @click.stop="emit('playlist:toggle', track.id)"
                  title="Manage playlists"
                >
                  <Icon name="plus-circle" :size="16" />
                </button>
                <div v-if="playlistMenuOpen === track.id" class="menu">
                  <div class="menu-header">Add to playlists</div>
                  <button
                    v-for="playlist in playlists"
                    :key="playlist.id"
                    @click.stop="emit('playlist:add', track.id, playlist.id)"
                  >
                    {{ playlist.name }}
                  </button>
                  <p v-if="!playlists.length">Create a playlist first</p>
                </div>
              </div>
              <button class="btn-icon" @click.stop="emit('track:edit', track)" title="Edit metadata"><Icon name="edit" :size="16" /></button>
              <button class="btn-icon danger" @click.stop="emit('track:delete', track)" title="Delete"><Icon name="trash" :size="16" /></button>
            </template>
          </td>
        </tr>
      </tbody>
    </table>
    
    <!-- Keyboard hints -->
    <div v-if="tracks.length > 0" class="keyboard-hints">
      <span><kbd>&uarr;</kbd><kbd>&darr;</kbd> Navigate</span>
      <span><kbd>Enter</kbd> Play selected</span>
      <span><kbd>Space</kbd> Play/Pause</span>
      <span>Double-click to play</span>
    </div>
  </div>
</template>

<style scoped>
.music-container {
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
  background: var(--surface-color);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
  position: relative;
  min-height: 400px; /* Prevent layout collapse during refreshes */
}

.music-container.is-loading .music-table {
  opacity: 0.6;
  pointer-events: none;
  filter: grayscale(0.5);
}

.loading-progress {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 3px;
  background: var(--primary-color);
  z-index: 100;
  animation: loading-bar 2s infinite ease-in-out;
  transform-origin: 0% 50%;
}

@keyframes loading-bar {
  0% { transform: scaleX(0); left: 0; }
  50% { transform: scaleX(0.5); left: 25%; }
  100% { transform: scaleX(0); left: 100%; }
}

/* Loading Skeleton */
.loading-skeleton {
  padding: 0;
  min-height: 400px;
}

.skeleton-header {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 80px 100px 100px 100px 60px 80px 120px;
  gap: 20px;
  padding: 16px 24px;
  background: var(--background-elevated);
  border-bottom: 2px solid var(--border-color);
}

.skeleton-row {
  display: grid;
  grid-template-columns: 2fr 1fr 1fr 80px 100px 100px 100px 60px 80px 120px;
  gap: 20px;
  padding: 18px 24px;
  border-bottom: 1px solid var(--border-color);
}

.skeleton-cell {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.skeleton-cell.title {
  gap: 6px;
}

.skeleton-cell.actions {
  flex-direction: row;
  gap: 8px;
}

.skeleton-bar {
  height: 14px;
  background: linear-gradient(90deg, var(--border-color) 25%, var(--background-elevated) 50%, var(--border-color) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
  border-radius: 4px;
  width: 80%;
}

.skeleton-bar.short {
  width: 50%;
  height: 12px;
}

.skeleton-bar.badge {
  width: 70px;
  height: 24px;
  border-radius: 999px;
}

.skeleton-bar.date {
  width: 80px;
}

.skeleton-circle {
  width: 38px;
  height: 38px;
  border-radius: 10px;
  background: linear-gradient(90deg, var(--border-color) 25%, var(--background-elevated) 50%, var(--border-color) 75%);
  background-size: 200% 100%;
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { background-position: 200% 0; }
  100% { background-position: -200% 0; }
}

.empty {
  padding: 80px 40px;
  text-align: center;
  color: var(--text-secondary);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 20px;
}

.empty-icon {
  font-size: 48px;
  opacity: 0.5;
}

.empty p {
  margin: 0;
  font-size: 16px;
}

.music-table {
  width: 100%;
  border-collapse: collapse;
  background: var(--surface-color);
}

.music-table th {
  text-align: left;
  padding: 16px 20px;
  background: var(--background-elevated);
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-tertiary);
  font-weight: 700;
  border-bottom: 2px solid var(--border-color);
  position: sticky;
  top: 0;
  z-index: 5;
}

.music-table th.sortable {
  cursor: pointer;
  user-select: none;
  transition: background 0.2s, color 0.2s;
}

.music-table th.sortable:hover {
  background: var(--surface-hover);
  color: var(--text-color);
}

.header-content {
  display: flex;
  align-items: center;
  gap: 6px;
}

.sort-icon {
  color: var(--primary-color);
  animation: fadeInIcon 0.15s ease-out;
}

@keyframes fadeInIcon {
  from { opacity: 0; transform: translateY(-3px); }
  to { opacity: 1; transform: translateY(0); }
}

.music-table th:first-child {
  padding-left: 24px;
}

.music-row {
  transition: all 0.2s ease;
  cursor: pointer;
}

.music-row:hover {
  background: linear-gradient(90deg, var(--primary-glow), transparent);
}

.music-row.selected {
  background: var(--primary-glow);
  outline: 2px solid var(--primary-color);
  outline-offset: -2px;
}

.music-row.playing {
  background: linear-gradient(90deg, rgba(34, 197, 94, 0.15), transparent);
}

.music-row.playing:hover {
  background: linear-gradient(90deg, rgba(34, 197, 94, 0.2), transparent);
}

.music-row td {
  padding: 18px 20px;
  border-bottom: 1px solid var(--border-color);
  color: var(--text-color);
  vertical-align: middle;
}

.music-row td:first-child {
  padding-left: 24px;
}

.title-cell {
  display: flex;
  align-items: center;
  gap: 12px;
}

.now-playing-indicator {
  display: flex;
  align-items: flex-end;
  gap: 2px;
  height: 16px;
  width: 16px;
}

.now-playing-indicator .bar {
  width: 3px;
  background: #22c55e;
  border-radius: 1px;
  height: 4px;
}

.now-playing-indicator .bar:nth-child(1) { height: 8px; }
.now-playing-indicator .bar:nth-child(2) { height: 12px; }
.now-playing-indicator .bar:nth-child(3) { height: 6px; }

.now-playing-indicator .bar.animating {
  animation: equalizer 0.5s ease infinite alternate;
}

.now-playing-indicator .bar.animating:nth-child(1) { animation-delay: 0s; }
.now-playing-indicator .bar.animating:nth-child(2) { animation-delay: 0.1s; }
.now-playing-indicator .bar.animating:nth-child(3) { animation-delay: 0.2s; }

@keyframes equalizer {
  0% { height: 4px; }
  100% { height: 14px; }
}

.title {
  font-weight: 600;
  color: var(--text-color);
  font-size: 15px;
  line-height: 1.3;
}

.title-link {
  font-weight: 600;
  color: var(--text-color);
  font-size: 15px;
  line-height: 1.3;
  background: none;
  border: none;
  padding: 0;
  cursor: pointer;
  text-align: left;
  transition: color 0.2s;
}

.title-link:hover {
  color: var(--primary-light);
  text-decoration: underline;
  text-underline-offset: 3px;
}

.mobile-artist {
  display: none;
}

.music-row.playing .title-link,
.music-row.playing .title {
  color: #22c55e;
}

.timestamp {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-top: 4px;
}

.duration-cell {
  display: flex;
  align-items: center;
  gap: 6px;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.genre-badge {
  display: inline-flex;
  align-items: center;
  padding: 6px 14px;
  border-radius: 999px;
  font-size: 12px;
  font-weight: 600;
  background: var(--primary-glow);
  color: var(--primary-light);
  border: 1px solid var(--primary-color);
  transition: all 0.2s ease;
}

.genre-badge:hover {
  transform: scale(1.05);
  box-shadow: 0 2px 8px var(--primary-glow);
}

.genre-badge.muted {
  opacity: 0.8;
  background: rgba(148, 163, 184, 0.1);
  color: var(--text-secondary);
  border-color: rgba(148, 163, 184, 0.3);
  font-style: italic;
}

.genre-badge.muted.clickable {
  cursor: pointer;
  opacity: 1;
  background: rgba(251, 191, 36, 0.15);
  color: var(--text-color);
  border-color: rgba(251, 191, 36, 0.5);
  font-style: italic;
  transition: all 0.2s ease;
}

.genre-badge.muted.clickable:hover {
  opacity: 1;
  background: rgba(251, 191, 36, 0.25);
  border-color: rgba(251, 191, 36, 0.8);
  transform: scale(1.08);
}

.genre-badge.empty {
  background: var(--background-elevated);
  color: var(--text-tertiary);
  border-color: var(--border-color);
}

.bpm-text {
  font-weight: 600;
  color: var(--text-secondary);
  font-variant-numeric: tabular-nums;
}

.bpm-text.muted {
  color: var(--text-tertiary);
  font-weight: 400;
}

.loading-indicator {
  font-style: italic;
  color: var(--text-tertiary);
  font-size: 12px;
  opacity: 0.6;
}

.actions {
  display: flex;
  align-items: center;
  gap: 8px;
  position: relative;
}

.btn-icon {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 16px;
  color: var(--text-color);
}

.btn-icon:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  transform: scale(1.08);
  box-shadow: 0 4px 12px var(--primary-glow);
}

.btn-icon.play-btn {
  background: rgba(34, 197, 94, 0.1);
  border-color: rgba(34, 197, 94, 0.3);
  color: #22c55e;
}

.btn-icon.play-btn:hover {
  background: rgba(34, 197, 94, 0.2);
  border-color: #22c55e;
  box-shadow: 0 4px 12px rgba(34, 197, 94, 0.3);
}

.music-row.playing .btn-icon.play-btn {
  background: #22c55e;
  border-color: #22c55e;
  color: white;
}

.btn-icon.danger {
  background: rgba(239, 68, 68, 0.08);
  border-color: rgba(239, 68, 68, 0.2);
}

.btn-icon.danger:hover {
  background: rgba(239, 68, 68, 0.15);
  border-color: rgba(239, 68, 68, 0.5);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.2);
}

.playlist-menu {
  position: relative;
}

.playlist-menu .menu {
  position: absolute;
  top: 44px;
  right: 0;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 14px;
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
  min-width: 220px;
  z-index: 20;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  animation: fadeIn 0.2s ease;
}

.playlist-menu .menu .menu-header {
  padding: 8px 14px;
  font-size: 11px;
  text-transform: uppercase;
  font-weight: 700;
  color: var(--text-tertiary);
  letter-spacing: 0.05em;
  border-bottom: 1px solid var(--border-color);
  margin-bottom: 4px;
}

@keyframes fadeIn {
  from { opacity: 0; transform: translateY(-8px); }
  to { opacity: 1; transform: translateY(0); }
}

.playlist-menu .menu button {
  background: transparent;
  border: none;
  color: var(--text-color);
  text-align: left;
  padding: 10px 14px;
  border-radius: 10px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  font-weight: 500;
}

.playlist-menu .menu button:hover {
  background: var(--primary-glow);
  color: var(--primary-light);
}

.playlist-menu .menu p {
  color: var(--text-tertiary);
  font-size: 13px;
  padding: 12px 14px;
  margin: 0;
  text-align: center;
  font-style: italic;
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-color);
  padding: 12px 24px;
  border-radius: 10px;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s ease;
}

.btn-outline:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px var(--primary-glow);
}

/* Keyboard hints */
.keyboard-hints {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 24px;
  padding: 12px 20px;
  background: var(--background-elevated);
  border-top: 1px solid var(--border-color);
  font-size: 12px;
  color: var(--text-tertiary);
}

.keyboard-hints span {
  display: flex;
  align-items: center;
  gap: 6px;
}

.keyboard-hints kbd {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  min-width: 22px;
  height: 22px;
  padding: 0 6px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-family: inherit;
  font-size: 11px;
  font-weight: 600;
  color: var(--text-secondary);
}

/* Responsive table */
@media (max-width: 1200px) {
  .music-table th:nth-child(6), /* Date Added */
  .music-row td:nth-child(6) {
    display: none;
  }
}

@media (max-width: 1024px) {
  .music-table th:nth-child(3), /* Album */
  .music-row td:nth-child(3),
  .music-table th:nth-child(4), /* Year */
  .music-row td:nth-child(4),
  .music-table th:nth-child(7), /* BPM */
  .music-row td:nth-child(7) {
    display: none;
  }

  .skeleton-header,
  .skeleton-row {
    grid-template-columns: 2fr 1fr 100px 80px 120px;
  }
}

@media (max-width: 768px) {
  .music-table thead {
    display: none; /* Hide headers on tablet/mobile */
  }

  .music-row {
    display: flex;
    flex-direction: column;
    padding: 12px;
    border-bottom: 1px solid var(--border-color);
    position: relative;
  }

  .music-row td {
    display: block;
    padding: 0 !important;
    border: none;
    width: 100%;
  }

  .music-row td:not(:first-child):not(.actions) {
    display: none; /* Hide everything except Title/Artist and Actions */
  }

  .music-row td:first-child {
    padding-right: 140px !important; /* Make room for actions */
  }

  .title-cell {
    gap: 10px;
  }

  .title-link {
    font-size: 16px;
  }

  .mobile-artist {
    display: block;
    font-size: 13px;
    color: var(--text-secondary);
    margin-top: 2px;
  }

  .actions {
    position: absolute;
    top: 50%;
    right: 12px;
    transform: translateY(-50%);
    background: transparent;
    padding: 0 !important;
  }

  .btn-icon {
    width: 42px; /* Bigger touch targets */
    height: 42px;
  }

  .skeleton-header {
    display: none;
  }

  .skeleton-row {
    display: block;
    padding: 16px;
  }

  .skeleton-cell:not(.title):not(.actions) {
    display: none;
  }

  .skeleton-cell.actions {
    position: absolute;
    right: 16px;
    top: 16px;
  }
}

@media (max-width: 480px) {
  .music-row td:first-child {
    padding-right: 100px !important;
  }
  
  .btn-icon:not(.play-btn):not(.danger) {
    display: none; /* Hide edit/playlist on very small screens, use long press or separate detail view? */
    /* For now let's keep it simple and just show play/delete if space is tight */
  }
}
</style>
