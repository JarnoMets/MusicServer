<template>
  <div class="genre-manager">
    <header class="page-header">
      <div class="header-main">
        <h1>Genre Manager</h1>
        <div class="tabs">
          <button 
            v-for="tab in ['explore', 'unmapped', 'tools']" 
            :key="tab"
            :class="{ active: activeTab === tab }"
            @click="activeTab = tab"
          >
            {{ tab.charAt(0).toUpperCase() + tab.slice(1) }}
          </button>
        </div>
      </div>
      <div class="header-actions">
        <button class="btn btn-primary" @click="showAddGenreModal = true">
          <Icons name="plus" :size="18" />
          Add Genre
        </button>
      </div>
    </header>

    <!-- Explore Tab -->
    <div v-if="activeTab === 'explore'" class="tab-content">
      <div class="search-bar">
        <Icons name="search" :size="18" />
        <input v-model="searchQuery" placeholder="Search canonical genres..." />
      </div>

      <div v-if="loading" class="loading">
        <Icons name="loader" class="spin" />
        Loading genres...
      </div>
      <div v-else class="genre-grid">
        <div 
          v-for="genre in filteredGenres" 
          :key="genre.id" 
          class="genre-card"
          @contextmenu.prevent="onGenreContextMenu($event, genre)"
        >
          <div class="genre-card-header">
            <h3>{{ genre.name }}</h3>
            <span class="track-count">{{ genre.track_count }} tracks</span>
            <button class="more-btn" @click.stop="onGenreContextMenu($event, genre)">
              <Icons name="more-vertical" :size="16" />
            </button>
          </div>
          <div v-if="genre.description" class="genre-desc">
            {{ genre.description }}
          </div>
          <div class="aliases-list">
            <span v-for="alias in genre.aliases" :key="alias" class="alias-pill">
              {{ alias }}
            </span>
          </div>
        </div>
      </div>
    </div>

    <!-- Unmapped Tab -->
    <div v-if="activeTab === 'unmapped'" class="tab-content">
      <div class="unmapped-info">
        <div class="info-content">
          <h3>Unmapped Detected Tags</h3>
          <p>These tags were found in your music files but are not yet mapped to a canonical genre.</p>
        </div>
        <button class="btn btn-secondary" @click="startReprocess" :disabled="reprocessing">
          <Icons name="refresh-cw" :class="{ spin: reprocessing }" :size="18" />
          {{ reprocessing ? 'Processing...' : 'Reprocess Missing' }}
        </button>
      </div>
      
      <div v-if="reprocessing" class="progress-banner">
        <div class="progress-info">
          <span>Reprocessing: {{ progress.processed }} / {{ progress.total }}</span>
          <small v-if="progress.current">{{ progress.current }}</small>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: (progress.processed / (progress.total || 1) * 100) + '%' }"></div>
        </div>
      </div>

      <div v-if="loading" class="loading">Loading unmapped tags...</div>
      <div v-else-if="unmapped.length === 0" class="empty-state">
        No unmapped tags found. Everything is organized!
      </div>
      <div v-else class="unmapped-list">
        <div v-for="tag in unmapped" :key="tag" class="unmapped-item">
          <div class="item-header">
            <div class="tag-badge">{{ tag }}</div>
            <div class="item-actions">
              <button class="btn-sm" @click="preview(tag)">Preview</button>
              <button class="btn-sm" @click="openCreateMap(tag)">Create & Map</button>
            </div>
          </div>
          
          <div v-if="previewCounts[tag]" class="preview-mini">
            <small>Affects {{ previewCounts[tag].music_rows }} tracks</small>
          </div>

          <div class="suggestions-grid">
            <span class="label">Map to:</span>
            <div class="suggestion-pills">
              <button 
                v-for="(s, i) in suggestions[tag]" 
                :key="i"
                class="suggestion-pill"
                @click="mapTo(tag, s[0])"
              >
                {{ s[0] }}
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>

    <!-- Tools Tab -->
    <div v-if="activeTab === 'tools'" class="tab-content">
      <div class="tools-header">
        <h3>Management Tools</h3>
        <p class="tools-description">Utilities for maintaining your genre database and detection cache.</p>
      </div>
      <div class="tools-grid">
        <div class="tool-card">
          <div class="tool-icon-box tool-icon-primary">
            <Icons name="zap" :size="28" />
          </div>
          <div class="tool-info">
            <h4>Genre Detection Pass</h4>
            <p>Run a full background scan to auto-detect genres for all artists using the current rules and aliases.</p>
            <button class="btn btn-primary" @click="startReprocess" :disabled="reprocessing">
              <Icons v-if="reprocessing" name="loader" :size="16" class="spin" />
              {{ reprocessing ? 'Running...' : 'Run Detection Pass' }}
            </button>
          </div>
        </div>
        <div class="tool-card">
          <div class="tool-icon-box tool-icon-danger">
            <Icons name="trash" :size="28" />
          </div>
          <div class="tool-info">
            <h4>Clear Genre Cache</h4>
            <p>Reset all cached artist genre lookups. Unconfirmed genres will disappear until re-detected.</p>
            <button class="btn btn-danger" @click="clearCache">
              <Icons name="trash" :size="16" />
              Clear Cache
            </button>
          </div>
        </div>
      </div>

      <!-- Reprocess progress (shown in tools tab too) -->
      <div v-if="reprocessing" class="progress-banner tools-progress">
        <div class="progress-info">
          <span class="progress-label">Reprocessing: {{ progress.processed }} / {{ progress.total }}</span>
          <small v-if="progress.current" class="progress-current">{{ progress.current }}</small>
        </div>
        <div class="progress-bar">
          <div class="progress-fill" :style="{ width: (progress.processed / (progress.total || 1) * 100) + '%' }"></div>
        </div>
      </div>
    </div>

    <!-- Modals -->
    <Transition name="modal">
      <div v-if="showAddGenreModal || editingGenre" class="modal-overlay" @click.self="closeGenreModal">
        <div class="modal-card">
          <div class="modal-header">
            <h3>{{ editingGenre ? 'Edit Genre' : 'Add New Genre' }}</h3>
            <button class="close-btn" @click="closeGenreModal"><Icons name="x" /></button>
          </div>
          <div class="modal-body">
            <div class="form-group">
              <label>Name</label>
              <input v-model="genreForm.name" placeholder="e.g. Drum & Bass" ref="nameInput" />
            </div>
            <div class="form-group">
              <label>Description</label>
              <textarea v-model="genreForm.description" placeholder="Brief description of the genre..."></textarea>
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn btn-secondary" @click="closeGenreModal">Cancel</button>
            <button class="btn btn-primary" @click="saveGenre" :disabled="!genreForm.name">
              {{ editingGenre ? 'Save Changes' : 'Create Genre' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="modal">
      <div v-if="showMergeModal" class="modal-overlay" @click.self="showMergeModal = false">
        <div class="modal-card">
          <div class="modal-header">
            <h3>Merge Genre</h3>
            <button class="close-btn" @click="showMergeModal = false"><Icons name="x" /></button>
          </div>
          <div class="modal-body">
            <p>Merge <strong>{{ selectedGenre?.name }}</strong> into another genre.</p>
            <p class="warning-text">All tracks and aliases will be moved. This action cannot be undone.</p>
            <div class="form-group">
              <label>Target Genre</label>
              <select v-model="mergeTargetId">
                <option value="">Select target...</option>
                <option v-for="g in otherGenres" :key="g.id" :value="g.id">{{ g.name }}</option>
              </select>
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn btn-secondary" @click="showMergeModal = false">Cancel</button>
            <button class="btn btn-danger" @click="confirmMerge" :disabled="!mergeTargetId">Merge Genres</button>
          </div>
        </div>
      </div>
    </Transition>

    <Transition name="modal">
      <div v-if="showAliasModal" class="modal-overlay" @click.self="showAliasModal = false">
        <div class="modal-card">
          <div class="modal-header">
            <h3>Add Alias for {{ selectedGenre?.name }}</h3>
            <button class="close-btn" @click="showAliasModal = false"><Icons name="x" /></button>
          </div>
          <div class="modal-body">
            <div class="form-group">
              <label>Alias Name</label>
              <input v-model="newAliasName" placeholder="e.g. dnb" @keyup.enter="confirmAddAlias" />
              <small>This alias will be automatically mapped to {{ selectedGenre?.name }} during detection.</small>
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn btn-secondary" @click="showAliasModal = false">Cancel</button>
            <button class="btn btn-primary" @click="confirmAddAlias" :disabled="!newAliasName">Add Alias</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- Context Menu -->
    <ContextMenu v-model="contextMenu.visible" :x="contextMenu.x" :y="contextMenu.y">
      <ContextMenuItem @click="openEditGenre">
        <template #icon><Icons name="edit" :size="14" /></template>
        Edit Genre
      </ContextMenuItem>
      <ContextMenuItem @click="showAliasModal = true">
        <template #icon><Icons name="link" :size="14" /></template>
        Add Alias
      </ContextMenuItem>
      <ContextMenuItem @click="showMergeModal = true">
        <template #icon><Icons name="shuffle" :size="14" /></template>
        Merge Into...
      </ContextMenuItem>
      <div class="menu-divider"></div>
      <ContextMenuItem danger @click="confirmDeleteGenre">
        <template #icon><Icons name="trash" :size="14" /></template>
        Delete Genre
      </ContextMenuItem>
    </ContextMenu>

    <!-- Existing Backfill Modal -->
    <div v-if="modalVisible" class="modal-overlay">
      <div class="modal-card">
        <h3>Confirm Backfill</h3>
        <p>Alias: <strong>{{ modalData.alias }}</strong></p>
        <p>Canonical: <strong>{{ modalData.canonical }}</strong></p>
        <p v-if="modalData.music_rows">Will update approx. {{ modalData.music_rows }} tracks.</p>
        <div class="modal-footer">
          <button class="btn btn-primary" @click="startBackfillConfirmed" :disabled="backfillRunning">
            {{ backfillRunning ? 'Backfilling...' : 'Start Backfill' }}
          </button>
          <button class="btn btn-secondary" @click="closeModal" :disabled="backfillRunning">Cancel</button>
        </div>
        <div v-if="backfillRunning" class="progress-info">
          Processed {{ backfillProgress.processed }} / {{ backfillProgress.total }}
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed, reactive } from 'vue'
import { musicAPI } from '@/api/music'
import { useToast } from '../../composables/useToast'
import Icons from '../../shared/components/Icons.vue'
import ContextMenu from '../../shared/components/ContextMenu.vue'
import ContextMenuItem from '../../shared/components/ContextMenuItem.vue'
import { useConfirm } from '../../composables/useConfirm'

const { success, error: showError } = useToast()
const { confirm } = useConfirm()

const activeTab = ref('explore')
const loading = ref(false)
const searchQuery = ref('')
const genres = ref<any[]>([])
const unmapped = ref<string[]>([])
const suggestions = ref<Record<string, [string, number][]>>({})
const previewCounts = ref<Record<string, { music_rows: number; artist_rows: number }>>({})

// Modals state
const showAddGenreModal = ref(false)
const showMergeModal = ref(false)
const showAliasModal = ref(false)
const editingGenre = ref<any | null>(null)
const selectedGenre = ref<any | null>(null)
const mergeTargetId = ref('')
const newAliasName = ref('')
const genreForm = reactive({ name: '', description: '' })

// Context menu state
const contextMenu = reactive({ visible: false, x: 0, y: 0 })

// From original backfill logic
const modalVisible = ref(false)
const modalData = ref<any>({ alias: '', canonical: '', music_rows: 0, artist_rows: 0 })
const backfillRunning = ref(false)
const backfillProgress = ref({ processed: 0, total: 0 })
const reprocessing = ref(false)
const progress = ref({ processed: 0, total: 0, current: null as string | null, finished: false })

const filteredGenres = computed(() => {
  if (!searchQuery.value) return genres.value
  const q = searchQuery.value.toLowerCase()
  return genres.value.filter(g => 
    g.name.toLowerCase().includes(q) || 
    (g.aliases && g.aliases.some((a: string) => a.toLowerCase().includes(q)))
  )
})

const otherGenres = computed(() => {
  if (!selectedGenre.value) return genres.value
  return genres.value.filter(g => g.id !== selectedGenre.value.id)
})

async function loadGenres() {
  loading.value = true
  try {
    const res = await musicAPI.listCanonicalGenres()
    genres.value = res.data
  } catch (e) {
    showError('Error', 'Failed to load genres')
  } finally {
    loading.value = false
  }
}

async function loadUnmapped() {
  if (activeTab.value !== 'unmapped') return
  loading.value = true
  try {
    const res = await musicAPI.listUnmappedGenres()
    unmapped.value = res.data || []
    
    // Start loading suggestions in background
    loadSuggestionsBackground()
  } catch (e) {
    showError('Error', 'Failed to load unmapped tags')
  } finally {
    loading.value = false
  }
}

async function loadSuggestionsBackground() {
  for (const tag of unmapped.value) {
    if (!suggestions.value[tag]) {
      try {
        const sres = await musicAPI.suggestGenres(tag)
        suggestions.value[tag] = sres.data || []
      } catch (e) {
        console.error(`Failed to load suggestions for ${tag}`, e)
      }
    }
  }
}

function onGenreContextMenu(e: MouseEvent, genre: any) {
  selectedGenre.value = genre
  contextMenu.x = e.clientX
  contextMenu.y = e.clientY
  contextMenu.visible = true
}

function openEditGenre() {
  editingGenre.value = selectedGenre.value
  genreForm.name = selectedGenre.value.name
  genreForm.description = selectedGenre.value.description || ''
  showAddGenreModal.value = true
}

function closeGenreModal() {
  showAddGenreModal.value = false
  editingGenre.value = null
  genreForm.name = ''
  genreForm.description = ''
}

async function saveGenre() {
  try {
    if (editingGenre.value) {
      await musicAPI.updateGenre(editingGenre.value.id, genreForm.name, genreForm.description)
      success('Success', 'Genre updated')
    } else {
      await musicAPI.createGenre(genreForm.name, genreForm.description)
      success('Success', 'Genre created')
    }
    closeGenreModal()
    loadGenres()
  } catch (e) {
    showError('Error', 'Failed to save genre')
  }
}

async function confirmDeleteGenre() {
  if (!selectedGenre.value) return
    if (await confirm({
    title: 'Delete Genre',
    message: `Are you sure you want to delete "${selectedGenre.value.name}"? This will not delete the music files, but they will no longer be mapped to this genre.`,
    confirmText: 'Delete',
    variant: 'danger'
  })) {
    try {
      await musicAPI.deleteGenre(selectedGenre.value.id)
      success('Success', 'Genre deleted')
      loadGenres()
    } catch (e) {
      showError('Error', 'Failed to delete genre')
    }
  }
}

async function confirmMerge() {
  try {
    await musicAPI.mergeGenres(selectedGenre.value.id, mergeTargetId.value)
    success('Success', 'Genres merged successfully')
    showMergeModal.value = false
    loadGenres()
  } catch (e) {
    showError('Error', 'Failed to merge genres')
  }
}

async function confirmAddAlias() {
  if (!newAliasName.value || !selectedGenre.value) return
  try {
    await musicAPI.addGenreAlias(newAliasName.value, selectedGenre.value.id)
    success('Success', 'Alias added')
    newAliasName.value = ''
    showAliasModal.value = false
    loadGenres()
  } catch (e) {
    showError('Error', 'Failed to add alias')
  }
}

// Original mapping logic
async function mapTo(raw: string, canonicalName: string) {
  const genre = genres.value.find(g => g.name === canonicalName)
  if (genre) {
    await musicAPI.addGenreAliasBackfill(raw, genre.id)
    loadUnmapped()
    loadGenres()
  }
}

async function preview(raw: string) {
  const res = await musicAPI.previewBackfill(raw)
  previewCounts.value[raw] = res.data
}

async function openCreateMap(raw: string) {
  genreForm.name = raw
  editingGenre.value = null
  showAddGenreModal.value = true
  // We'll need specialized logic here or just let the user create then map
}

async function startReprocess() {
  reprocessing.value = true
  try {
    const res = await musicAPI.startReprocessMissing()
    const sessionId = res.data.session_id
    const es = musicAPI.getReprocessStream(sessionId)
    es.onmessage = (ev) => {
      const data = JSON.parse(ev.data)
      progress.value = data
      if (data.finished) {
        reprocessing.value = false
        es.close()
        loadUnmapped()
        loadGenres()
      }
    }
  } catch (e) {
    reprocessing.value = false
  }
}

async function clearCache() {
  if (await confirm({
    title: 'Clear Cache',
    message: 'This will reset all auto-detected genres. All unconfirmed genres will disappear until re-detected.',
    confirmText: 'Clear Cache',
    variant: 'danger'
  })) {
    await musicAPI.clearGenreCache()
    success('Success', 'Cache cleared')
    loadGenres()
    loadUnmapped()
  }
}

// Logic for backfill confirmation modal (from confirmCreateAndMap in original)
async function startBackfillConfirmed() {
  backfillRunning.value = true
  const alias = modalData.value.alias
  const genre_id = modalData.value['genre_id'] || ''
  const res = await musicAPI.startBackfill(alias, genre_id)
  const sessionId = res.data.session_id
  const es = musicAPI.getBackfillStream(sessionId)
  es.onmessage = (ev) => {
    const data = JSON.parse(ev.data)
    backfillProgress.value = { processed: data.processed, total: data.total }
    if (data.finished) {
      backfillRunning.value = false
      es.close()
      modalVisible.value = false
      loadGenres()
      loadUnmapped()
    }
  }
  es.onerror = (e) => {
    console.error('Backfill SSE error', e)
    backfillRunning.value = false
    es.close()
  }
}

function closeModal() {
  modalVisible.value = false
}

onMounted(() => {
  loadGenres()
})

defineExpose({ refresh: () => { loadGenres(); loadUnmapped(); } })
</script>

<style scoped>
.genre-manager {
  padding: 0;
  color: var(--text-color);
}

.page-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 32px;
}

.header-main h1 {
  margin: 0 0 16px 0;
  font-size: 28px;
  font-weight: 800;
}

.tabs {
  display: flex;
  gap: 8px;
  background: var(--background-elevated);
  padding: 4px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
}

.tabs button {
  padding: 8px 16px;
  border: none;
  background: transparent;
  color: var(--text-secondary);
  border-radius: 8px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tabs button:hover {
  background: rgba(255, 255, 255, 0.05);
  color: var(--text-color);
}

.tabs button.active {
  background: var(--surface-color);
  color: var(--primary-color);
  box-shadow: 0 2px 8px rgba(0,0,0,0.2);
}

.search-bar {
  position: relative;
  margin-bottom: 24px;
}

.search-bar .icon {
  position: absolute;
  left: 16px;
  top: 50%;
  transform: translateY(-50%);
  color: var(--text-tertiary);
}

.search-bar input {
  width: 100%;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 14px 14px 14px 48px;
  color: var(--text-color);
  font-size: 15px;
  transition: all 0.2s ease;
}

.search-bar input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 4px var(--primary-glow);
}

.genre-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
  gap: 20px;
}

.genre-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  padding: 20px;
  transition: all 0.2s ease;
  position: relative;
}

.genre-card:hover {
  border-color: var(--primary-color);
  transform: translateY(-4px);
  box-shadow: 0 12px 24px rgba(0,0,0,0.2);
}

.genre-card-header {
  display: flex;
  align-items: flex-start;
  justify-content: space-between;
  margin-bottom: 12px;
}

.genre-card h3 {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--primary-light);
}

.track-count {
  font-size: 12px;
  color: var(--text-secondary);
  background: var(--background-elevated);
  padding: 2px 8px;
  border-radius: 999px;
  font-weight: 500;
}

.more-btn {
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.more-btn:hover {
  background: var(--background-elevated);
  color: var(--text-color);
}

.genre-desc {
  font-size: 14px;
  color: var(--text-secondary);
  line-height: 1.5;
  margin-bottom: 16px;
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}

.aliases-list {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.alias-pill {
  font-size: 11px;
  background: var(--background-elevated);
  color: var(--text-tertiary);
  padding: 2px 8px;
  border-radius: 4px;
  border: 1px solid var(--border-color);
}

/* Unmapped Styles */
.unmapped-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  background: var(--surface-color);
  padding: 24px;
  border-radius: 20px;
  border: 1px solid var(--border-color);
  margin-bottom: 24px;
}

.info-content h3 { margin: 0 0 4px 0; }
.info-content p { margin: 0; color: var(--text-secondary); font-size: 14px; }

.unmapped-list {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.unmapped-item {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 20px;
}

.item-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.tag-badge {
  font-weight: 700;
  color: var(--primary-light);
  background: var(--primary-glow);
  padding: 6px 16px;
  border-radius: 999px;
  font-size: 14px;
}

.suggestions-grid {
  display: flex;
  gap: 16px;
  align-items: center;
}

.suggestions-grid .label {
  font-size: 13px;
  color: var(--text-tertiary);
  font-weight: 600;
  text-transform: uppercase;
}

.suggestion-pills {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
}

.suggestion-pill {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  color: var(--text-color);
  padding: 6px 12px;
  border-radius: 8px;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.suggestion-pill:hover {
  border-color: var(--primary-color);
  background: var(--primary-glow);
  color: var(--primary-light);
}

/* Common UI */
.btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  border-radius: 12px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  border: none;
}

.btn-sm {
  padding: 6px 12px;
  font-size: 12px;
  border-radius: 8px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  color: var(--text-secondary);
  cursor: pointer;
}

.btn-primary { 
  background: var(--primary-color); 
  color: #000; 
  box-shadow: 0 4px 12px var(--primary-glow);
}
.btn-primary:hover:not(:disabled) { transform: translateY(-2px); box-shadow: 0 6px 16px var(--primary-glow); }

.btn-secondary { background: var(--background-elevated); color: var(--text-color); border: 1px solid var(--border-color); }
.btn-danger { background: var(--error-color); color: #fff; }
.btn-danger:hover:not(:disabled) { transform: translateY(-1px); box-shadow: 0 4px 12px rgba(248, 113, 113, 0.3); }

/* Tools Tab */
.tools-header {
  margin-bottom: 24px;
}

.tools-header h3 {
  margin: 0 0 4px 0;
  font-size: 1.25rem;
  font-weight: 700;
}

.tools-description {
  margin: 0;
  color: var(--text-secondary);
  font-size: 14px;
}

.tools-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
  gap: 20px;
}

.tool-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  padding: 24px;
  display: flex;
  gap: 20px;
  transition: all 0.2s ease;
}

.tool-card:hover {
  border-color: rgba(255, 255, 255, 0.1);
  transform: translateY(-2px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.tool-icon-box {
  width: 56px;
  height: 56px;
  border-radius: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.tool-icon-primary {
  background: var(--primary-glow);
  color: var(--primary-color);
}

.tool-icon-danger {
  background: rgba(248, 113, 113, 0.15);
  color: var(--error-color);
}

.tool-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex: 1;
}

.tool-info h4 {
  margin: 0;
  font-size: 1rem;
  font-weight: 700;
  color: var(--text-color);
}

.tool-info p {
  margin: 0;
  font-size: 13px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.tool-info .btn {
  align-self: flex-start;
  margin-top: 4px;
}

/* Progress banner */
.progress-banner {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 20px;
}

.tools-progress {
  margin-top: 20px;
}

.progress-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
  margin-bottom: 12px;
}

.progress-label {
  font-weight: 600;
  font-size: 14px;
  color: var(--text-color);
}

.progress-current {
  color: var(--text-tertiary);
  font-size: 12px;
}

.progress-bar {
  height: 6px;
  background: rgba(255, 255, 255, 0.1);
  border-radius: 999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: var(--primary-color);
  border-radius: 999px;
  transition: width 0.3s ease;
}

.preview-mini {
  margin: 8px 0;
}

.preview-mini small {
  color: var(--text-tertiary);
  font-size: 12px;
}

.item-actions {
  display: flex;
  gap: 8px;
}

.spin { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }

/* Modal */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
}

.modal-card {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 24px;
  width: 100%;
  max-width: 500px;
  box-shadow: 0 32px 64px rgba(0, 0, 0, 0.5);
  overflow: hidden;
}

.modal-header {
  padding: 24px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 { margin: 0; font-size: 20px; }

.close-btn { background: transparent; border: none; color: var(--text-tertiary); cursor: pointer; }

.modal-body { padding: 24px; }

.form-group { margin-bottom: 20px; }
.form-group label { display: block; margin-bottom: 8px; font-size: 14px; font-weight: 600; color: var(--text-secondary); }
.form-group input, .form-group textarea, .form-group select {
  width: 100%;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 12px;
  color: var(--text-color);
  font-size: 15px;
}

.modal-footer { padding: 20px 24px; display: flex; justify-content: flex-end; gap: 12px; background: var(--background-elevated); }

.menu-divider { height: 1px; background: var(--border-color); margin: 4px 0; }
.warning-text { color: var(--error-color); font-size: 13px; margin-top: 4px; }

/* Modal Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-card,
.modal-leave-active .modal-card {
  transition: all 0.3s cubic-bezier(0.34, 1.56, 0.64, 1);
}

.modal-enter-from .modal-card,
.modal-leave-to .modal-card {
  transform: translateY(20px) scale(0.95);
  opacity: 0;
}
</style>
