<script setup lang="ts">
import { useMusicData } from '../../composables/useMusicData'
import { useTrackEdit } from '../../composables/useTrackEdit'
import { useTrackActions } from '../../composables/useTrackActions'
import { useAuth } from '../../composables/useAuth'
import { calculateTotalDuration } from '../../utils/musicFormatters'
import MusicFiltersPanel from './MusicFiltersPanel.vue'
import MusicTable from './MusicTable.vue'
import EditTrackDrawer from './EditTrackDrawer.vue'
import type { MusicFile } from '../../types/MusicTab'

// Props for permission-based UI
interface Props {
  canEdit?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  canEdit: false
})

// Use auth for fallback if canEdit not passed
const { isLoggedIn } = useAuth()
const canManage = () => props.canEdit || isLoggedIn.value

// Data management
const {
  musicFiles,
  playlists,
  genres,
  loading,
  hasMore,
  playlistMenuOpen,
  filters,
  pagination,
  fetchMusic,
  refreshAll,
  resetFilters,
  nextPage,
  prevPage,
} = useMusicData()

// Edit functionality
const { editState, openEdit, closeEdit, saveEdit } = useTrackEdit(() => {
  fetchMusic()
})

// Track actions
const { playTrack, deleteTrack, addTrackToPlaylist } = useTrackActions(() => {
  fetchMusic()
})

// Track event handlers
const handlePlayTrack = (track: MusicFile) => {
  playTrack(track)
}

const handleEditTrack = (track: MusicFile) => {
  if (canManage()) {
    openEdit(track)
  }
}

const handleDeleteTrack = (track: MusicFile) => {
  if (canManage()) {
    deleteTrack(track)
  }
}

const handleTogglePlaylistMenu = (trackId: string) => {
  if (canManage()) {
    playlistMenuOpen.value = playlistMenuOpen.value === trackId ? null : trackId
  }
}

const handleAddToPlaylist = async (trackId: string, playlistId: string) => {
  if (canManage()) {
    await addTrackToPlaylist(trackId, playlistId)
    playlistMenuOpen.value = null
  }
}

// Edit state handlers
const handleEditFormUpdate = {
  title: (val: string) => {
    editState.form.title = val
  },
  artist: (val: string) => {
    editState.form.artist = val
  },
  album: (val: string) => {
    editState.form.album = val
  },
  genre: (val: string) => {
    editState.form.genre = val
  },
  duration: (val: number | undefined) => {
    editState.form.duration = val
  },
}
</script>

<template>
  <div class="music-tab">
    <!-- Header -->
    <header class="header">
      <div>
        <h2>Library</h2>
        <p class="subtitle">Search, filter, and play everything in your catalog.</p>
      </div>
      <div class="header-actions">
        <router-link v-if="canManage()" to="/upload" class="btn btn-secondary">Upload Music</router-link>
        <button class="btn btn-outline" @click="refreshAll" :disabled="loading">Refresh</button>
      </div>
    </header>

    <!-- Filters -->
    <MusicFiltersPanel
      :filters="filters"
      :genres="genres"
      :loading="loading"
      :page-size="pagination.limit"
      @update:search="filters.search = $event"
      @update:genre="filters.genre = $event"
      @update:sort="filters.sort = $event"
      @update:order="filters.order = $event"
      @update:pageSize="pagination.limit = $event"
      @reset="resetFilters"
    />

    <!-- Music Table -->
    <MusicTable
      :tracks="musicFiles"
      :playlists="playlists"
      :loading="loading"
      :playlist-menu-open="playlistMenuOpen"
      :can-edit="canManage()"
      @track:play="handlePlayTrack"
      @track:edit="handleEditTrack"
      @track:delete="handleDeleteTrack"
      @playlist:toggle="handleTogglePlaylistMenu"
      @playlist:add="handleAddToPlaylist"
      @reset="resetFilters"
    />

    <!-- Footer with pagination and stats -->
    <footer v-if="musicFiles.length" class="footer">
      <div class="stats">
        <span>{{ musicFiles.length }} tracks</span>
        <span>{{ calculateTotalDuration(musicFiles) }}</span>
      </div>
      <div class="pagination">
        <button class="btn btn-outline" :disabled="pagination.page === 1 || loading" @click="prevPage">
          Previous
        </button>
        <span>Page {{ pagination.page }}</span>
        <button class="btn btn-outline" :disabled="!hasMore || loading" @click="nextPage">
          Next
        </button>
      </div>
    </footer>

    <!-- Edit Drawer (only when logged in) -->
    <EditTrackDrawer
      v-if="canManage()"
      :edit-state="editState"
      :saving="editState.saving"
      @update:title="handleEditFormUpdate.title"
      @update:artist="handleEditFormUpdate.artist"
      @update:album="handleEditFormUpdate.album"
      @update:genre="handleEditFormUpdate.genre"
      @update:duration="handleEditFormUpdate.duration"
      @save="saveEdit"
      @close="closeEdit"
    />
  </div>
</template>

<style scoped>
/* Layout */
.music-tab {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 20px;
  padding: 28px;
  background: linear-gradient(145deg, var(--primary-glow), var(--surface-color));
  border: 1px solid var(--border-color);
  border-radius: 20px;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.header h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: var(--text-color);
}

.subtitle {
  margin: 6px 0 0 0;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.5;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-shrink: 0;
}

.btn {
  border: none;
  border-radius: 12px;
  padding: 12px 20px;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-secondary {
  background: var(--surface-color);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  text-decoration: none;
  display: inline-flex;
  align-items: center;
  gap: 8px;
}



.btn-secondary:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
  transform: translateY(-2px);
  box-shadow: 0 4px 12px var(--primary-glow);
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
}



.btn-outline:hover:not(:disabled) {
  background: var(--background-elevated);
  border-color: var(--text-tertiary);
  color: var(--text-color);
}

.btn-outline:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.footer {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 20px;
  padding: 20px 28px;
  border: 1px solid var(--border-color);
  border-radius: 16px;
  background: var(--surface-color);
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.1);
}

.stats {
  display: flex;
  gap: 20px;
  font-size: 14px;
  color: var(--text-secondary);
}

.stats span {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 14px;
  background: var(--background-elevated);
  border-radius: 999px;
  font-weight: 500;
}



.pagination {
  display: flex;
  align-items: center;
  gap: 14px;
  font-size: 14px;
  color: var(--text-secondary);
}

.pagination span {
  font-weight: 600;
  color: var(--text-color);
  min-width: 70px;
  text-align: center;
}

.pagination .btn-outline {
  padding: 10px 16px;
}



@media (max-width: 900px) {
  .header {
    flex-direction: column;
    padding: 24px;
  }

  .header-actions {
    width: 100%;
  }

  .header-actions .btn {
    flex: 1;
    justify-content: center;
  }

  .footer {
    flex-direction: column;
    align-items: stretch;
    gap: 16px;
    padding: 20px;
  }

  .stats {
    justify-content: center;
  }

  .pagination {
    justify-content: space-between;
  }
}
</style>
