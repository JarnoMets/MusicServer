<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import type { EditState } from '../../types'
import Icon from '../../shared/components/Icons.vue'
import { musicAPI } from '../../api/music'

interface Props {
  editState: EditState
  saving: boolean
  genres: { id: string; name: string; description?: string }[]
  releaseDateSuggestion?: { date: string; album?: string; genre?: string; confidence: number } | null
  lookingUpReleaseDate?: boolean
}

const props = withDefaults(defineProps<Props>(), {
  releaseDateSuggestion: null,
  lookingUpReleaseDate: false,
})

const emit = defineEmits<{
  'update:title': [value: string]
  'update:artist': [value: string]
  'update:album': [value: string]
  'update:genre': [value: string]
  'update:genre-id': [value: string]
  'update:release_date': [value: string]
  'update:bpm': [value: number | null]
  'update:initial_key': [value: string]
  'update:beat_grid_offset': [value: number | null]
  'apply-suggestion': []
  save: []
  close: []
}>()

// Genre combobox state
const genreSearch = ref('')
const genreDropdownOpen = ref(false)
const newGenreName = ref('')
const showNewGenreInput = ref(false)

const filteredGenres = computed(() => {
  const search = genreSearch.value.toLowerCase()
  if (!search) return props.genres
  return props.genres.filter(g => g.name.toLowerCase().includes(search))
})

watch(() => props.editState.open, (open) => {
  if (open) {
    genreSearch.value = ''
    genreDropdownOpen.value = false
    showNewGenreInput.value = false
    newGenreName.value = ''
  }
})

const selectGenre = (genre: { id: string; name: string }) => {
  emit('update:genre', genre.name)
  emit('update:genre-id', genre.id)
  genreDropdownOpen.value = false
  genreSearch.value = ''
}

const handleGenreInput = (e: Event) => {
  const value = (e.target as HTMLInputElement).value
  genreSearch.value = value
  emit('update:genre', value)
  emit('update:genre-id', '')  // clear id when typing freeform
  genreDropdownOpen.value = true
}

const createAndSelectGenre = () => {
  const name = newGenreName.value.trim()
  if (name) {
    emit('update:genre', name)
    emit('update:genre-id', '')  // no id for a newly typed genre
    showNewGenreInput.value = false
    newGenreName.value = ''
    genreDropdownOpen.value = false
  }
}

// BPM detection
const detectingBpm = ref(false)

const detectBpm = async () => {
  if (!props.editState.trackId) return
  detectingBpm.value = true
  try {
    const { data } = (await musicAPI.detectBpm(props.editState.trackId)) as { data: { bpm: number; offset?: number } }
    if (data && data.bpm) {
      emit('update:bpm', data.bpm)
      if (data.offset !== undefined) {
        emit('update:beat_grid_offset', data.offset)
      }
    }
  } catch (e) {
    console.error('BPM detection failed', e)
  } finally {
    detectingBpm.value = false
  }
}
</script>

<template>
  <div v-if="editState.open" class="edit-drawer">
    <div class="drawer-content">
      <header>
        <h3>Edit Track</h3>
                <button class="btn-icon" @click="emit('close')"><Icon name="x" :size="16" /></button>
      </header>
      <form @submit.prevent="emit('save')" class="edit-form">
        <label>
          Title
          <input
            :value="editState.form.title"
            @input="emit('update:title', ($event.target as HTMLInputElement).value)"
            type="text"
            class="text-input"
            required
          />
        </label>
        <label>
          Artist
          <input
            :value="editState.form.artist"
            @input="emit('update:artist', ($event.target as HTMLInputElement).value)"
            type="text"
            class="text-input"
          />
        </label>
        <label>
          Album
          <input
            :value="editState.form.album"
            @input="emit('update:album', ($event.target as HTMLInputElement).value)"
            type="text"
            class="text-input"
          />
        </label>

        <div class="form-row">
          <label class="flex-1">
            BPM
            <div class="input-with-action">
              <input
                :value="editState.form.bpm"
                @input="emit('update:bpm', ($event.target as HTMLInputElement).value ? Number(($event.target as HTMLInputElement).value) : null)"
                type="number"
                step="0.01"
                class="text-input"
                placeholder="e.g. 124.0"
              />
              <button
                type="button"
                class="btn-icon btn-small"
                title="Detect BPM"
                @click="detectBpm"
                :disabled="detectingBpm"
              >
                <Icon :name="detectingBpm ? 'zap' : 'activity'" :size="14" :class="{ spin: detectingBpm }" />
              </button>
            </div>
          </label>
          <label class="flex-1">
            Key
            <input
              :value="editState.form.initial_key"
              @input="emit('update:initial_key', ($event.target as HTMLInputElement).value)"
              type="text"
              class="text-input"
              placeholder="e.g. 4A"
            />
          </label>
        </div>

        <label>
          Beat Grid Offset (seconds)
          <input
            :value="editState.form.beat_grid_offset"
            @input="emit('update:beat_grid_offset', ($event.target as HTMLInputElement).value ? Number(($event.target as HTMLInputElement).value) : null)"
            type="number"
            step="0.001"
            class="text-input"
            placeholder="e.g. 0.125"
          />
        </label>

        <!-- Genre combobox -->
        <label>
          Genre
          <div class="genre-combobox">
            <input
              :value="editState.form.genre"
              @input="handleGenreInput"
              @focus="genreDropdownOpen = true"
              type="text"
              class="text-input"
              placeholder="Search or type a genre&hellip;"
              autocomplete="off"
            />
            <Transition name="dropdown-fade">
              <div v-if="genreDropdownOpen && (filteredGenres.length > 0 || genreSearch)" class="genre-dropdown">
                <button
                  v-for="g in filteredGenres"
                  :key="g.id"
                  type="button"
                  class="genre-option"
                  :class="{ active: editState.form.genre === g.name }"
                  @mousedown.prevent="selectGenre(g)"
                >
                  {{ g.name }}
                  <span v-if="g.description" class="genre-desc">{{ g.description }}</span>
                </button>
                <div v-if="filteredGenres.length === 0 && genreSearch" class="genre-no-match">
                  No matching genre &mdash; type to use "{{ genreSearch }}"
                </div>
                <button
                  type="button"
                  class="genre-option genre-create-btn"
                  @mousedown.prevent="showNewGenreInput = !showNewGenreInput"
                >
                  <Icon name="plus-circle" :size="14" />
                  <span>Create new genre&hellip;</span>
                </button>
                <div v-if="showNewGenreInput" class="genre-create-row">
                  <input
                    v-model="newGenreName"
                    type="text"
                    class="text-input text-input-sm"
                    placeholder="New genre name"
                    @keydown.enter.prevent="createAndSelectGenre"
                  />
                  <button type="button" class="btn btn-xs btn-primary" @mousedown.prevent="createAndSelectGenre">Add</button>
                </div>
              </div>
            </Transition>
            <div v-if="genreDropdownOpen" class="genre-backdrop" @click="genreDropdownOpen = false"></div>
          </div>
        </label>

        <!-- Release Date -->
        <label>
          Release Date
          <input
            :value="editState.form.release_date"
            @input="emit('update:release_date', ($event.target as HTMLInputElement).value)"
            type="date"
            class="text-input"
          />
        </label>

        <!-- MusicBrainz suggestion banner -->
        <div v-if="lookingUpReleaseDate" class="suggestion-banner suggestion-loading">
          <Icon name="zap" :size="16" class="spin" />
          <span>Looking up metadata suggestion&hellip;</span>
        </div>
        <div v-else-if="releaseDateSuggestion" class="suggestion-banner">
          <div class="suggestion-info">
            <Icon name="calendar" :size="16" />
            <div>
              <div class="suggestion-label">Found Metadata</div>
              <div class="suggestion-value">
                {{ releaseDateSuggestion.date }}
                <span v-if="releaseDateSuggestion.album" class="suggestion-album">
                  from "{{ releaseDateSuggestion.album }}"
                </span>
                <span v-if="releaseDateSuggestion.genre" class="suggestion-album">
                  ({{ releaseDateSuggestion.genre }})
                </span>
              </div>
            </div>
          </div>
          <button type="button" class="btn btn-xs btn-accent" @click="emit('apply-suggestion')">
            Apply
          </button>
        </div>

        <div class="drawer-actions">
          <button type="button" class="btn btn-secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="saving">
            {{ saving ? 'Saving&hellip;' : 'Save changes' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.edit-drawer {
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  width: 380px;
  background: var(--surface-color);
  border-left: 1px solid var(--border-color);
  box-shadow: -12px 0 40px rgba(0, 0, 0, 0.5);
  z-index: 2100;
  overflow-y: auto;
  animation: slideInRight 0.3s ease;
}

@keyframes slideInRight {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.drawer-content {
  padding: 28px;
  display: flex;
  flex-direction: column;
  gap: 24px;
  min-height: 100%;
}

.drawer-content header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
}

.drawer-content h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--text-color);
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
  flex-shrink: 0;
  color: var(--text-secondary);
  font-size: 16px;
}

.btn-icon:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #ef4444;
  transform: scale(1.05);
}

.edit-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.edit-form label {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.text-input {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 12px 14px;
  color: var(--text-color);
  font-family: inherit;
  font-size: 15px;
  transition: all 0.2s ease;
}

.text-input:hover {
  border-color: var(--primary-color);
}

.text-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
  background: var(--surface-color);
}

.text-input::placeholder {
  color: var(--text-tertiary);
}

.drawer-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: auto;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
}

.btn {
  border: none;
  border-radius: 10px;
  padding: 12px 24px;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-primary {
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-dark) 100%);
  color: #fff;
  box-shadow: 0 4px 12px var(--primary-glow);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px var(--primary-glow);
}

.btn-primary:active:not(:disabled) {
  transform: translateY(0);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.btn-secondary {
  background: var(--background-elevated);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--surface-color);
  color: var(--text-color);
  border-color: var(--text-tertiary);
}

@media (max-width: 900px) {
  .edit-drawer {
    width: 100%;
    max-width: 100vw;
  }
}

/* Genre combobox */
.genre-combobox {
  position: relative;
}

.genre-backdrop {
  position: fixed;
  inset: 0;
  z-index: 9;
}

.genre-dropdown {
  position: absolute;
  top: calc(100% + 4px);
  left: 0;
  right: 0;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.3);
  max-height: 220px;
  overflow-y: auto;
  z-index: 10;
  padding: 4px;
}

.genre-option {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 12px;
  border: none;
  border-radius: 8px;
  background: none;
  color: var(--text-color);
  font-size: 14px;
  cursor: pointer;
  text-align: left;
  transition: background 0.15s;
}

.genre-option:hover {
  background: var(--background-elevated);
}

.genre-option.active {
  background: var(--primary-glow);
  color: var(--primary-light);
}

.genre-desc {
  font-size: 12px;
  color: var(--text-tertiary);
  margin-left: auto;
}

.genre-no-match {
  padding: 8px 12px;
  font-size: 13px;
  color: var(--text-tertiary);
}

.genre-create-btn {
  color: var(--primary-light);
  border-top: 1px solid var(--border-color);
  border-radius: 0 0 8px 8px;
  margin-top: 2px;
}

.genre-create-row {
  display: flex;
  gap: 6px;
  padding: 6px 8px 8px;
}

.text-input-sm {
  padding: 6px 10px;
  font-size: 13px;
}

.btn-xs {
  padding: 6px 12px;
  font-size: 12px;
  border-radius: 8px;
  white-space: nowrap;
}

.btn-accent {
  background: linear-gradient(135deg, #f59e0b, #d97706);
  color: #fff;
  border: none;
  cursor: pointer;
  font-weight: 600;
}

.btn-accent:hover {
  filter: brightness(1.1);
}

/* MusicBrainz suggestion */
.suggestion-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 12px;
  padding: 12px 14px;
  border-radius: 10px;
  background: rgba(245, 158, 11, 0.08);
  border: 1px solid rgba(245, 158, 11, 0.25);
}

.suggestion-loading {
  color: var(--text-secondary);
  font-size: 13px;
  gap: 8px;
  justify-content: flex-start;
}

.suggestion-info {
  display: flex;
  gap: 10px;
  align-items: flex-start;
  color: #f59e0b;
}

.suggestion-label {
  font-size: 11px;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  font-weight: 600;
}

.suggestion-value {
  font-size: 14px;
  color: var(--text-color);
  font-weight: 600;
}

.suggestion-album {
  font-weight: 400;
  color: var(--text-secondary);
  font-size: 13px;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}

.dropdown-fade-enter-active,
.dropdown-fade-leave-active {
  transition: opacity 0.15s, transform 0.15s;
}

.dropdown-fade-enter-from,
.dropdown-fade-leave-to {
  opacity: 0;
  transform: translateY(-4px);
}

.form-row {
  display: flex;
  gap: 16px;
}

.flex-1 {
  flex: 1;
}

.input-with-action {
  display: flex;
  align-items: center;
  gap: 8px;
}

.input-with-action input {
  flex: 1;
}

.btn-small {
  width: 44px;
  height: 44px;
}
</style>
