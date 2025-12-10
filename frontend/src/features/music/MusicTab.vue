<script setup lang="ts">
import { useMusicData } from '../../composables/useMusicData'
import { useTrackEdit } from '../../composables/useTrackEdit'
import { useTrackActions } from '../../composables/useTrackActions'
import { useAuth } from '../../composables/useAuth'
import { calculateTotalDuration } from '../../utils/musicFormatters'
import MusicFiltersPanel from './MusicFiltersPanel.vue'
import MusicTable from './MusicTable.vue'
import EditTrackDrawer from './EditTrackDrawer.vue'
import ConfirmGenreModal from './ConfirmGenreModal.vue'
import type { MusicFile } from '../../types/MusicTab'
import { ref } from 'vue'

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

// Confirm genre modal state
const confirmGenreModal = ref({
  isOpen: false,
  track: null as MusicFile | null,
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

const handleConfirmGenre = (track: MusicFile) => {
  if (canManage()) {
    confirmGenreModal.value = {
      isOpen: true,
      track,
    }
  }
}

const handleGenreConfirmed = () => {
  confirmGenreModal.value.isOpen = false
  confirmGenreModal.value.track = null
  fetchMusic()
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
      @track:confirm-genre="handleConfirmGenre"
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

    <!-- Confirm Genre Modal (only when logged in) -->
    <ConfirmGenreModal
      v-if="canManage()"
      :is-open="confirmGenreModal.isOpen"
      :track="confirmGenreModal.track || undefined"
      @close="confirmGenreModal.isOpen = false"
      @confirm="handleGenreConfirmed"
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
