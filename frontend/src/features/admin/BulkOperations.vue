<template>
  <div class="bulk-operations">
    <div class="tabs-container">
      <div class="tabs">
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
        <p class="success-text">✓ Updated {{ renameResult.updated_count }} file(s)</p>
        
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
        <p class="success-text">✓ Added {{ playlistResult.added_count }} track(s)</p>
        <p class="info-text">Total tracks in playlist: {{ playlistResult.total_playlist_count }}</p>
      </div>

      <div v-if="playlistError" class="error">
        <strong>Error:</strong> {{ playlistError }}
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { musicAPI } from '@/api/music'
import type { PlaylistSummary } from '@/types/index'

const activeTab = ref<'rename' | 'playlist'>('rename')

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

onMounted(async () => {
  try {
    const response = await musicAPI.getPlaylists()
    playlists.value = response.data
  } catch (error) {
    console.error('Failed to load playlists:', error)
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
  background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  border-radius: 12px;
  padding: 24px;
  color: #e0e0e0;
}

.tabs-container {
  margin-bottom: 24px;
}

.tabs {
  display: flex;
  gap: 12px;
  border-bottom: 2px solid rgba(255, 255, 255, 0.1);
}

.tab-button {
  padding: 12px 20px;
  background: none;
  border: none;
  color: #a0a0a0;
  cursor: pointer;
  font-size: 14px;
  font-weight: 500;
  transition: all 0.3s ease;
  border-bottom: 3px solid transparent;
  margin-bottom: -2px;
}

.tab-button:hover {
  color: #e0e0e0;
}

.tab-button.active {
  color: #4a9eff;
  border-bottom-color: #4a9eff;
}

.tab-content {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

h3 {
  margin-top: 0;
  margin-bottom: 12px;
  color: #4a9eff;
  font-size: 18px;
}

h4 {
  margin-top: 16px;
  margin-bottom: 12px;
  color: #4a9eff;
  font-size: 16px;
}

h5 {
  margin-top: 0;
  margin-bottom: 8px;
  color: #e0e0e0;
  font-size: 14px;
}

.info-text {
  color: #a0a0a0;
  margin: 12px 0;
  font-size: 14px;
  line-height: 1.5;
}

.info-text code {
  background: rgba(255, 255, 255, 0.1);
  padding: 2px 6px;
  border-radius: 4px;
  color: #4a9eff;
  font-family: 'Courier New', monospace;
}

.form {
  display: flex;
  flex-direction: column;
  gap: 16px;
  margin-bottom: 24px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

label {
  color: #e0e0e0;
  font-weight: 500;
  font-size: 14px;
}

input,
select {
  padding: 10px 12px;
  background: rgba(255, 255, 255, 0.05);
  border: 1px solid rgba(255, 255, 255, 0.15);
  border-radius: 6px;
  color: #e0e0e0;
  font-size: 14px;
  transition: all 0.3s ease;
  font-family: inherit;
}

input:focus,
select:focus {
  outline: none;
  background: rgba(255, 255, 255, 0.08);
  border-color: #4a9eff;
  box-shadow: 0 0 0 3px rgba(74, 158, 255, 0.1);
}

input::placeholder {
  color: #606080;
}

.help-text {
  color: #808080;
  font-size: 12px;
  margin-top: -4px;
}

.btn {
  padding: 12px 24px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.3s ease;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.btn-primary {
  background: linear-gradient(135deg, #4a9eff 0%, #357abd 100%);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: linear-gradient(135deg, #5aafff 0%, #3d87c8 100%);
  box-shadow: 0 8px 16px rgba(74, 158, 255, 0.3);
  transform: translateY(-2px);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.result {
  background: rgba(76, 175, 80, 0.1);
  border: 1px solid rgba(76, 175, 80, 0.3);
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 16px;
}

.success-text {
  color: #4caf50;
  font-weight: 600;
  margin: 0 0 12px 0;
}

.updated-items {
  margin-top: 12px;
}

.items-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-top: 8px;
  max-height: 300px;
  overflow-y: auto;
}

.item {
  background: rgba(255, 255, 255, 0.05);
  padding: 10px;
  border-radius: 4px;
  border-left: 3px solid #4caf50;
}

.item-info {
  color: #e0e0e0;
  font-size: 13px;
  line-height: 1.4;
}

.text-muted {
  color: #808080;
  font-size: 12px;
}

.error {
  background: rgba(244, 67, 54, 0.1);
  border: 1px solid rgba(244, 67, 54, 0.3);
  border-radius: 8px;
  padding: 16px;
  color: #ff6b6b;
  margin-bottom: 16px;
  font-size: 14px;
}

.error strong {
  color: #ff8787;
}
</style>
