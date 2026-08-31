<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import Icon from '../../shared/components/Icons.vue'
import type { MusicFile, PlaylistSummary } from '../../types'

interface Props {
  isAdmin: boolean
  playlists: PlaylistSummary[]
  genres: { id: string; name: string }[]
  dateSuggestion?: { date: string; album?: string; genre?: string; confidence: number } | null
  lookingUpDate?: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  play: [track: MusicFile]
  details: [track: MusicFile]
  edit: [track: MusicFile]
  cut: [track: MusicFile]
  delete: [track: MusicFile]
  'playlist:add': [trackId: string, playlistId: string]
  'confirm-genre': [track: MusicFile]
  'set-genre': [track: MusicFile, genre: string]
  'apply-date': [track: MusicFile, date: string]
  'lookup-date': [track: MusicFile]
  'queue:add': [track: MusicFile]
  close: []
}>()

const visible = ref(false)
const position = ref({ x: 0, y: 0 })
const currentTrack = ref<MusicFile | null>(null)
const showPlaylistSub = ref(false)
const showGenreSub = ref(false)
const suggestion = ref<{ date: string; album?: string } | null>(null)
const lookingUp = ref(false)
const menuRef = ref<HTMLElement | null>(null)

const open = async (track: MusicFile, event: MouseEvent) => {
  event.preventDefault()
  currentTrack.value = track
  showPlaylistSub.value = false
  showGenreSub.value = false
  suggestion.value = null
  
  // Position the menu
  const x = event.clientX
  const y = event.clientY
  position.value = { x, y }
  visible.value = true

  // Adjust position after render to stay in viewport
  nextTick(() => {
    if (menuRef.value) {
      const rect = menuRef.value.getBoundingClientRect()
      const vw = window.innerWidth
      const vh = window.innerHeight
      if (rect.right > vw) {
        position.value.x = vw - rect.width - 8
      }
      if (rect.bottom > vh) {
        position.value.y = vh - rect.height - 8
      }
    }
  })

  // Lookup release date if missing
  if (track.id && !track.release_date) {
    lookingUp.value = true
    try {
      // Use the API directly or via props
      emit('lookup-date', track)
    } finally {
      lookingUp.value = false
    }
  }
}

const togglePlaylistSub = () => {
  showPlaylistSub.value = !showPlaylistSub.value
  showGenreSub.value = false
}

const toggleGenreSub = () => {
  showGenreSub.value = !showGenreSub.value
  showPlaylistSub.value = false
}

const close = () => {
  visible.value = false
  currentTrack.value = null
  showPlaylistSub.value = false
  showGenreSub.value = false
  emit('close')
}

const handleClickOutside = (e: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('scroll', close, true)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('scroll', close, true)
})

defineExpose({ open, close })
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible && currentTrack"
      ref="menuRef"
      class="context-menu"
      :style="{ left: position.x + 'px', top: position.y + 'px' }"
    >
      <button class="menu-item" @click="emit('play', currentTrack!); close()">
        <Icon name="play" :size="15" />
        <span>Play</span>
      </button>

      <button class="menu-item queue-item" @click="emit('queue:add', currentTrack!); close()">
        <Icon name="plus-circle" :size="15" />
        <span>Add to DJ Queue</span>
      </button>

      <button class="menu-item" @click="emit('details', currentTrack!); close()">
        <Icon name="info" :size="15" />
        <span>Track Details</span>
      </button>

      <div class="menu-separator"></div>

      <!-- Playlist submenu -->
      <div class="menu-item-group">
        <button class="menu-item" @click.stop="togglePlaylistSub">
          <Icon name="list" :size="15" />
          <span>Add to Playlist</span>
          <Icon name="chevron-right" :size="12" class="submenu-arrow" />
        </button>
        <div v-if="showPlaylistSub" class="submenu">
          <button
            v-for="pl in playlists"
            :key="pl.id"
            class="menu-item"
            @click="emit('playlist:add', currentTrack!.id, pl.id); close()"
          >
            {{ pl.name }}
          </button>
          <p v-if="!playlists.length" class="menu-empty">No playlists</p>
        </div>
      </div>

      <template v-if="isAdmin">
        <div class="menu-separator"></div>

        <!-- Genre submenu -->
        <div class="menu-item-group">
          <button class="menu-item" @click.stop="toggleGenreSub">
            <Icon name="tag" :size="15" />
            <span>Set Genre</span>
            <Icon name="chevron-right" :size="12" class="submenu-arrow" />
          </button>
          <div v-if="showGenreSub" class="submenu">
            <button
              v-for="g in genres"
              :key="g.id"
              class="menu-item"
              :class="{ active: currentTrack?.genre_id === g.id }"
              @click="emit('set-genre', currentTrack!, g.id); close()"
            >
              {{ g.name }}
            </button>
            <div class="menu-separator"></div>
            <button class="menu-item" @click="emit('edit', currentTrack!); close()">
              <Icon name="plus" :size="12" />
              <span>Other / New&hellip;</span>
            </button>
          </div>
        </div>

        <button 
          v-if="!currentTrack.release_date && !dateSuggestion && !lookingUpDate" 
          class="menu-item" 
          @click="emit('lookup-date', currentTrack!); $event.stopPropagation()"
        >
          <Icon name="calendar" :size="15" />
          <span>Lookup Release Date</span>
        </button>

        <div v-if="lookingUpDate" class="menu-item muted">
          <Icon name="zap" :size="15" class="spin" />
          <span>Looking up date&hellip;</span>
        </div>

        <button 
          v-if="dateSuggestion && !currentTrack.release_date" 
          class="menu-item suggestion" 
          @click="emit('apply-date', currentTrack!, dateSuggestion.date); close()"
        >
          <Icon name="check-circle" :size="15" />
          <div class="suggestion-text">
            <span>Set Date: {{ dateSuggestion.date }}</span>
            <small v-if="dateSuggestion.album">from {{ dateSuggestion.album }}</small>
            <small v-if="dateSuggestion.genre">({{ dateSuggestion.genre }})</small>
          </div>
        </button>

        <div class="menu-separator"></div>

        <button class="menu-item" @click="emit('edit', currentTrack!); close()">
          <Icon name="edit" :size="15" />
          <span>Edit Metadata</span>
        </button>

        <button class="menu-item" @click="emit('cut', currentTrack!); close()">
          <Icon name="scissors" :size="15" />
          <span>Cut Audio</span>
        </button>

        <button
          v-if="currentTrack.genre_source === 'auto'"
          class="menu-item"
          @click="emit('confirm-genre', currentTrack!); close()"
        >
          <Icon name="check-circle" :size="15" />
          <span>Confirm Genre ({{ currentTrack.genre_name }})</span>
        </button>

        <div class="menu-separator"></div>

        <button class="menu-item danger" @click="emit('delete', currentTrack!); close()">
          <Icon name="trash" :size="15" />
          <span>Delete Track</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 2500;
  min-width: 200px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 6px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.4);
  animation: menuIn 0.15s ease;
}

@keyframes menuIn {
  from {
    opacity: 0;
    transform: scale(0.95);
  }
  to {
    opacity: 1;
    transform: scale(1);
  }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 14px;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--text-color);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s;
}

.menu-item:hover {
  background: var(--primary-glow);
  color: var(--primary-light);
}

.menu-item.active {
  background: var(--primary-glow);
  color: var(--primary-light);
  font-weight: 700;
}

.menu-item.danger {
  color: #ef4444;
}

.menu-item.danger:hover {
  background: rgba(239, 68, 68, 0.1);
}

.menu-item.queue-item {
  color: #f59e0b;
}

.menu-item.queue-item:hover {
  background: rgba(245, 158, 11, 0.1);
  color: #fbbf24;
}

.menu-item.suggestion {
  background: rgba(245, 158, 11, 0.08);
  color: #f59e0b;
}

.menu-item.suggestion:hover {
  background: rgba(245, 158, 11, 0.15);
}

.suggestion-text {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.suggestion-text small {
  font-size: 10px;
  opacity: 0.7;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.submenu-arrow {
  margin-left: auto;
  opacity: 0.5;
}

.menu-separator {
  height: 1px;
  background: var(--border-color);
  margin: 4px 8px;
}

.menu-item-group {
  position: relative;
}

.submenu {
  position: absolute;
  left: 100%;
  top: 0;
  min-width: 180px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 6px;
  box-shadow: 0 8px 30px rgba(0, 0, 0, 0.3);
  animation: menuIn 0.15s ease;
  max-height: 300px;
  overflow-y: auto;
}

.menu-empty {
  padding: 12px;
  text-align: center;
  color: var(--text-tertiary);
  font-size: 12px;
  font-style: italic;
  margin: 0;
}
</style>
