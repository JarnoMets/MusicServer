<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import Icon from '../../shared/components/Icons.vue'
import type { MusicFile, PlaylistSummary } from '../../types'

interface Props {
  isAdmin: boolean
  playlists: PlaylistSummary[]
  genres: { id: string; name: string }[]
}

defineProps<Props>()

const emit = defineEmits<{
  'bulk:playlist:add': [trackIds: string[], playlistId: string]
  'bulk:set-genre': [tracks: MusicFile[], genre: string]
  'bulk:delete': [tracks: MusicFile[]]
  'queue:add-bulk': [tracks: MusicFile[]]
  close: []
}>()

const visible = ref(false)
const position = ref({ x: 0, y: 0 })
const currentTracks = ref<MusicFile[]>([])
const showPlaylistSub = ref(false)
const showGenreSub = ref(false)
const menuRef = ref<HTMLElement | null>(null)

const open = (tracks: MusicFile[], event: MouseEvent) => {
  event.preventDefault()
  currentTracks.value = tracks
  showPlaylistSub.value = false
  showGenreSub.value = false
  
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
  currentTracks.value = []
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
      v-if="visible && currentTracks.length"
      ref="menuRef"
      class="context-menu bulk-menu"
      :style="{ left: position.x + 'px', top: position.y + 'px' }"
    >
      <div class="menu-header">
        <Icon name="check-square" :size="14" />
        <span>{{ currentTracks.length }} tracks selected</span>
      </div>

      <div class="menu-separator"></div>

      <button class="menu-item queue-item" @click="emit('queue:add-bulk', currentTracks); close()">
        <Icon name="plus-circle" :size="15" />
        <span>Add {{ currentTracks.length }} to DJ Queue</span>
      </button>

      <div class="menu-separator"></div>

      <!-- Bulk Playlist submenu -->
      <div class="menu-item-group">
        <button class="menu-item" @click.stop="togglePlaylistSub">
          <Icon name="plus-circle" :size="15" />
          <span>Add to Playlist</span>
          <Icon name="chevron-right" :size="12" class="submenu-arrow" />
        </button>
        <div v-if="showPlaylistSub" class="submenu">
          <button
            v-for="pl in playlists"
            :key="pl.id"
            class="menu-item"
            @click="emit('bulk:playlist:add', currentTracks.map(t => t.id), pl.id); close()"
          >
            {{ pl.name }}
          </button>
          <p v-if="!playlists.length" class="menu-empty">No playlists</p>
        </div>
      </div>

      <template v-if="isAdmin">
        <!-- Bulk Genre submenu -->
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
              @click="emit('bulk:set-genre', currentTracks, g.id); close()"
            >
              {{ g.name }}
            </button>
          </div>
        </div>

        <div class="menu-separator"></div>

        <button class="menu-item danger" @click="emit('bulk:delete', currentTracks); close()">
          <Icon name="trash" :size="15" />
          <span>Delete All ({{ currentTracks.length }})</span>
        </button>
      </template>
    </div>
  </Teleport>
</template>

<style scoped>
.bulk-context-menu {
  position: fixed;
  z-index: 2500;
  min-width: 200px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 6px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  animation: context-fade 0.15s ease-out;
}
.bulk-menu {
  border-top: 3px solid var(--primary-color);
}

.menu-header {
  padding: 10px 14px;
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  color: var(--text-tertiary);
  display: flex;
  align-items: center;
  gap: 8px;
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
