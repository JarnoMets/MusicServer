<template>
  <div class="genre-mapper">
    <h2>Unmapped Detected Genres</h2>
    <div class="reprocess">
      <button @click="startReprocess" :disabled="reprocessing">Start reprocess missing artists</button>
      <div v-if="reprocessing">Progress: {{ progress.processed }} / {{ progress.total }} <span v-if="progress.current">(current: {{ progress.current }})</span></div>
    </div>
    <div v-if="loading">Loading...</div>
    <div v-else>
      <div v-if="unmapped.length === 0">No unmapped tags found.</div>
      <ul>
        <li v-for="tag in unmapped" :key="tag" class="tag-item">
          <div class="tag-header">
            <strong>{{ tag }}</strong>
            <button @click="refresh">Refresh</button>
            <button @click="preview(tag)">Preview Backfill</button>
          </div>
          <div v-if="previewCounts[tag]">
            <small>Will update {{ previewCounts[tag].music_rows }} music rows and {{ previewCounts[tag].artist_rows }} artist cache rows</small>
          </div>
          <div class="suggestions">
            <em>Suggestions:</em>
            <ul>
              <li v-for="(s, i) in suggestions[tag] || []" :key="i">
                {{ s[0] }} (score: {{ s[1].toFixed(2) }})
                <button @click="mapTo(tag, s[0])">Map</button>
              </li>
            </ul>
          </div>
          <div class="map-create">
            <input v-model="newGenreName[tag]" placeholder="Create new canonical genre" />
            <button @click="confirmCreateAndMap(tag)">Create & Map</button>
          </div>
        </li>
      </ul>
    </div>
  </div>

  <div v-if="modalVisible" class="modal">
    <div class="modal-content">
      <h3>Confirm Backfill</h3>
      <p>Alias: <strong>{{ modalData.alias }}</strong></p>
      <p>Canonical: <strong>{{ modalData.canonical }}</strong></p>
      <p>Will update approximately: {{ modalData.music_rows }} music rows, {{ modalData.artist_rows }} artist rows</p>
      <div>
        <button @click="startBackfillConfirmed">Start Backfill</button>
        <button @click="closeModal">Cancel</button>
      </div>
      <div v-if="backfillRunning">Backfill Progress: {{ backfillProgress.processed }} / {{ backfillProgress.total }}</div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { musicAPI } from '@/api/music'
import { useToast } from '../../composables/useToast'

const { error: showError } = useToast()

const unmapped = ref<string[]>([])
const suggestions = ref<Record<string, [string, number][]>>({})
const newGenreName = ref<Record<string, string>>({})
const loading = ref(false)
const previewCounts = ref<Record<string, { music_rows: number; artist_rows: number }>>({})
interface ModalData {
  alias: string
  canonical: string
  music_rows: number
  artist_rows: number
  genre_id?: string
}

const modalVisible = ref(false)
const modalData = ref<ModalData>({ alias: '', canonical: '', music_rows: 0, artist_rows: 0 })
const backfillRunning = ref(false)
const backfillProgress = ref({ processed: 0, total: 0 })
const reprocessing = ref(false)
const progress = ref({ processed: 0, total: 0, current: null as string | null, finished: false })

async function load() {
  loading.value = true
  const res = await musicAPI.listUnmappedGenres()
  unmapped.value = res.data || []
  // load suggestions for each
  for (const tag of unmapped.value) {
    const sres = await musicAPI.suggestGenres(tag)
    suggestions.value[tag] = sres.data || []
  }
  loading.value = false
}

let es: EventSource | null = null

async function startReprocess() {
  reprocessing.value = true
  const res = await musicAPI.startReprocessMissing()
  const sessionId = res.data.session_id
  es = musicAPI.getReprocessStream(sessionId)
  es.onmessage = (ev) => {
    const data = JSON.parse(ev.data)
    progress.value = data
    if (data.finished) {
      reprocessing.value = false
      es?.close()
      es = null
      load()
    }
  }
  es.onerror = (e) => {
    console.error('SSE error', e)
    reprocessing.value = false
    es?.close()
    es = null
  }
}

function refresh() {
  load()
}

async function mapTo(raw: string, canonicalName: string) {
  // find the genre id for canonicalName
  const list = await musicAPI.listGenres()
  const genre = list.data.find((g: any) => g.name === canonicalName)
  if (genre) {
    // Create alias and backfill
    await musicAPI.addGenreAliasBackfill(raw, genre.id)
    await load()
  } else {
    showError('Genre Not Found', 'The selected genre could not be found')
  }
}

async function preview(raw: string) {
  const res = await musicAPI.previewBackfill(raw)
  previewCounts.value[raw] = res.data
}

async function confirmCreateAndMap(raw: string) {
  const name = newGenreName.value[raw]
  if (!name) {
    showError('Missing Name', 'Please provide a genre name')
    return
  }
  // create genre first
  const created = await musicAPI.createGenre(name)
  const gid = created.data.id
  // preview counts
  const pres = await musicAPI.previewBackfill(raw)
  modalData.value = {
    alias: raw,
    canonical: name,
    music_rows: pres.data.music_rows,
    artist_rows: pres.data.artist_rows,
    genre_id: String(gid),
  }
  modalVisible.value = true
}

function closeModal() {
  modalVisible.value = false
}

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
      load()
    }
  }
  es.onerror = (e) => {
    console.error('Backfill SSE error', e)
    backfillRunning.value = false
    es.close()
  }
}

onMounted(() => load())
</script>

<style scoped>
.genre-mapper {
  padding: 0;
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.genre-mapper h2 {
  margin: 0;
  font-size: 24px;
  font-weight: 700;
  color: var(--text-color);
}

.reprocess {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 20px 24px;
  display: flex;
  align-items: center;
  gap: 20px;
  flex-wrap: wrap;
}

.reprocess button {
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-dark) 100%);
  color: #fff;
  border: none;
  border-radius: 12px;
  padding: 12px 24px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  box-shadow: 0 4px 16px var(--primary-glow);
}

.reprocess button:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 24px var(--primary-glow);
}

.reprocess button:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.reprocess > div {
  color: var(--text-secondary);
  font-size: 14px;
  padding: 10px 16px;
  background: var(--background-elevated);
  border-radius: 10px;
}

.genre-mapper > ul {
  list-style: none;
  padding: 0;
  margin: 0;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.tag-item {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 24px;
  transition: all 0.2s ease;
}

.tag-item:hover {
  border-color: var(--primary-color);
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.tag-header {
  display: flex;
  gap: 16px;
  align-items: center;
  flex-wrap: wrap;
  margin-bottom: 16px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
}

.tag-header strong {
  font-size: 18px;
  color: var(--primary-light);
  background: var(--primary-glow);
  padding: 6px 14px;
  border-radius: 999px;
}

.tag-header button {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 8px 16px;
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.tag-header button:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
}

.tag-item > div > small {
  display: block;
  margin-bottom: 16px;
  color: var(--text-tertiary);
  font-size: 13px;
  padding: 10px 14px;
  background: var(--background-elevated);
  border-radius: 10px;
}

.suggestions {
  margin-bottom: 20px;
}

.suggestions em {
  display: block;
  margin-bottom: 12px;
  color: var(--text-secondary);
  font-size: 13px;
  font-style: normal;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.suggestions ul {
  list-style: none;
  padding: 0;
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
}

.suggestions ul li {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  font-size: 14px;
  color: var(--text-color);
}

.suggestions ul li button {
  background: var(--primary-color);
  color: #fff;
  border: none;
  border-radius: 8px;
  padding: 6px 14px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s ease;
}

.suggestions ul li button:hover {
  transform: scale(1.05);
  box-shadow: 0 4px 12px var(--primary-glow);
}

.map-create {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.map-create input {
  flex: 1;
  min-width: 200px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 12px 16px;
  color: var(--text-color);
  font-size: 14px;
  transition: all 0.2s ease;
}

.map-create input:hover {
  border-color: var(--primary-color);
}

.map-create input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.map-create button {
  background: linear-gradient(135deg, var(--accent-color) 0%, #0891b2 100%);
  color: #fff;
  border: none;
  border-radius: 12px;
  padding: 12px 24px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.map-create button:hover {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px rgba(6, 182, 212, 0.3);
}

/* Modal */
.modal {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 100;
  padding: 20px;
}

.modal-content {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  padding: 32px;
  max-width: 500px;
  width: 100%;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
  animation: modalSlideIn 0.3s ease;
}

@keyframes modalSlideIn {
  from {
    opacity: 0;
    transform: scale(0.95) translateY(-20px);
  }
  to {
    opacity: 1;
    transform: scale(1) translateY(0);
  }
}

.modal-content h3 {
  margin: 0 0 20px 0;
  font-size: 22px;
  color: var(--text-color);
}

.modal-content p {
  margin: 12px 0;
  color: var(--text-secondary);
  font-size: 14px;
}

.modal-content p strong {
  color: var(--primary-light);
}

.modal-content > div {
  display: flex;
  gap: 12px;
  margin-top: 24px;
}

.modal-content > div button {
  flex: 1;
  padding: 14px 24px;
  border: none;
  border-radius: 12px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.modal-content > div button:first-child {
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-dark) 100%);
  color: #fff;
  box-shadow: 0 4px 16px var(--primary-glow);
}

.modal-content > div button:first-child:hover {
  transform: translateY(-2px);
}

.modal-content > div button:last-child {
  background: var(--background-elevated);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.modal-content > div button:last-child:hover {
  background: var(--surface-color);
  color: var(--text-color);
}

.modal-content > div:last-of-type:not(:first-of-type) {
  margin-top: 16px;
  padding: 14px;
  background: var(--background-elevated);
  border-radius: 12px;
  color: var(--text-secondary);
  font-size: 14px;
}
</style>
