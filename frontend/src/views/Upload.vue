<template>
  <div class="upload-view">
    <header class="upload-header">
      <div>
        <p class="eyebrow">Library tools</p>
        <h1>Upload music files</h1>
        <p class="subtitle">
          Use drag & drop or the picker below. We will add files to your library right after upload.
        </p>
      </div>
      <div class="header-actions">
        <button class="btn btn-secondary" :disabled="!hasCompleted" @click="clearCompleted">
          Clear completed
        </button>
        <button class="btn btn-primary" :disabled="!canUpload || uploading" @click="startUpload">
          {{ uploading ? 'Uploading…' : 'Start upload' }}
        </button>
      </div>
    </header>

    <section class="picker-card">
      <label class="picker-label" for="file-picker">File picker</label>
      <input
        id="file-picker"
        ref="fileInput"
        class="picker-input"
        type="file"
        accept="audio/*"
        multiple
        @change="onFileChange"
      />
      <p class="picker-hint">Supports FLAC, MP3, WAV, AIFF, OGG, M4A and more. Select multiple files at once.</p>
    </section>

    <section
      class="dropzone"
      :class="{ 'dropzone--active': isDragging }"
      @dragenter.prevent="handleDrag(true)"
      @dragover.prevent
      @dragleave.prevent="handleDrag(false)"
      @drop.prevent="handleDrop"
    >
      <div class="dropzone__content">
        <Icon name="upload" :size="48" class="drop-icon" />
        <h2>Drag files anywhere in this area</h2>
        <p>Or use the picker above</p>
        <button class="btn btn-secondary" @click="triggerFileDialog">Browse files</button>
      </div>
    </section>

    <section class="stats-grid">
      <article class="stat-card">
        <p class="label">Queued</p>
        <p class="value">{{ queueCount }}</p>
      </article>
      <article class="stat-card">
        <p class="label">Uploaded</p>
        <p class="value success">{{ successCount }}</p>
      </article>
      <article class="stat-card">
        <p class="label">Errors</p>
        <p class="value error">{{ errorCount }}</p>
      </article>
    </section>

    <section v-if="uploads.length" class="queue">
      <div class="queue__header">
        <h3>Upload queue</h3>
        <span>{{ uploads.length }} file{{ uploads.length === 1 ? '' : 's' }}</span>
      </div>
      <div class="queue__list">
        <article v-for="item in uploads" :key="item.id" class="queue-item">
          <div class="queue-item__left">
            <div class="file-icon">
              <Icon name="disc" :size="20" />
            </div>
            <div>
              <p class="file-name">{{ item.file.name }}</p>
              <p class="file-meta">
                {{ formatBytes(item.file.size) }} · {{ itemStatusLabel(item.status) }}
              </p>
            </div>
          </div>
          <div class="queue-item__right">
            <div v-if="item.status !== 'queued'" class="progress">
              <div class="progress__bar" :style="{ width: item.progress + '%' }"></div>
            </div>
            <p v-if="item.message" :class="['status-message', item.status]">{{ item.message }}</p>
            <div class="queue-item__actions">
              <button
                v-if="item.status === 'error'"
                class="btn btn-secondary"
                :disabled="uploading"
                @click="retryItem(item.id)"
              >
                Retry
              </button>
              <button
                class="btn btn-ghost"
                :disabled="uploading && item.status === 'uploading'"
                @click="removeItem(item.id)"
              >
                Remove
              </button>
            </div>
          </div>
        </article>
      </div>
    </section>
  </div>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { musicAPI } from '../api/music'
import Icon from '../shared/components/Icons.vue'

interface UploadItem {
  id: string
  file: File
  progress: number
  status: 'queued' | 'uploading' | 'success' | 'error'
  message?: string
}

const uploads = ref<UploadItem[]>([])
const uploading = ref(false)
const isDragging = ref(false)
const fileInput = ref<HTMLInputElement | null>(null)

const queueCount = computed(() => uploads.value.filter((item) => item.status === 'queued').length)
const successCount = computed(() => uploads.value.filter((item) => item.status === 'success').length)
const errorCount = computed(() => uploads.value.filter((item) => item.status === 'error').length)
const hasCompleted = computed(() => successCount.value > 0)
const canUpload = computed(() => uploads.value.some((item) => item.status === 'queued' || item.status === 'error'))

const AUDIO_EXTENSIONS = ['.mp3', '.flac', '.wav', '.aiff', '.ogg', '.m4a']

function triggerFileDialog() {
  fileInput.value?.click()
}

function onFileChange(event: Event) {
  const target = event.target as HTMLInputElement
  if (target.files?.length) {
    addFiles(target.files)
    target.value = ''
  }
}

function handleDrag(active: boolean) {
  isDragging.value = active
}

function handleDrop(event: DragEvent) {
  handleDrag(false)
  if (event.dataTransfer?.files?.length) {
    addFiles(event.dataTransfer.files)
  }
}

function addFiles(fileList: FileList | File[]) {
  const newItems: UploadItem[] = []
  Array.from(fileList).forEach((file) => {
    if (!isAudioFile(file)) return
    const exists = uploads.value.some((item) => item.file.name === file.name && item.file.size === file.size)
    if (exists) return
    newItems.push({
      id: crypto?.randomUUID ? crypto.randomUUID() : `${Date.now()}-${Math.random()}`,
      file,
      progress: 0,
      status: 'queued',
    })
  })
  uploads.value = [...uploads.value, ...newItems]
}

function isAudioFile(file: File) {
  if (file.type.startsWith('audio/')) return true
  const lower = file.name.toLowerCase()
  return AUDIO_EXTENSIONS.some((ext) => lower.endsWith(ext))
}

function formatBytes(bytes: number) {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
}

function itemStatusLabel(status: UploadItem['status']) {
  switch (status) {
    case 'queued':
      return 'Queued'
    case 'uploading':
      return 'Uploading'
    case 'success':
      return 'Uploaded'
    case 'error':
      return 'Error'
    default:
      return ''
  }
}

function removeItem(id: string) {
  uploads.value = uploads.value.filter((item) => item.id !== id)
}

function clearCompleted() {
  uploads.value = uploads.value.filter((item) => item.status !== 'success')
}

function retryItem(id: string) {
  const target = uploads.value.find((item) => item.id === id)
  if (!target) return
  target.status = 'queued'
  target.progress = 0
  target.message = undefined
}

async function startUpload() {
  if (!canUpload.value) return
  uploading.value = true

  for (const item of uploads.value) {
    if (item.status === 'success') continue
    if (item.status !== 'queued' && item.status !== 'error') continue

    item.status = 'uploading'
    item.progress = 0
    item.message = undefined

    const formData = new FormData()
    formData.append('file', item.file, item.file.name)

    try {
      const response = await musicAPI.uploadMusicFiles(formData, {
        onUploadProgress: (event) => {
          if (event.total) {
            item.progress = Math.round((event.loaded / event.total) * 100)
          }
        },
      })
      item.progress = 100
      item.status = 'success'
      const inserted = response.data?.inserted as Array<{ title?: string }> | undefined
      item.message = inserted?.[0]?.title ? `Added ${inserted[0].title}` : 'Uploaded successfully'
    } catch (error: any) {
      item.status = 'error'
      item.message = error?.response?.data?.error || error?.message || 'Upload failed'
    }
  }

  uploading.value = false
}
</script>

<style scoped>
.upload-view {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.upload-header {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  padding: 32px;
  border-radius: 20px;
  border: 1px solid var(--border-color);
  background: linear-gradient(145deg, var(--primary-glow), var(--surface-color));
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.2);
}

.eyebrow {
  text-transform: uppercase;
  letter-spacing: 0.3em;
  font-size: 0.75rem;
  color: var(--primary-light);
  margin-bottom: 8px;
  font-weight: 600;
}

.upload-header h1 {
  margin: 0;
  font-size: 2rem;
  color: var(--text-color);
}

.subtitle {
  margin-top: 10px;
  color: var(--text-secondary);
  line-height: 1.5;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  align-items: center;
}

.btn {
  border: none;
  border-radius: 12px;
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
  box-shadow: 0 4px 16px var(--primary-glow);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 24px var(--primary-glow);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
}

.btn-secondary {
  background: var(--surface-color);
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
}

.btn-secondary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-ghost {
  background: transparent;
  color: var(--text-secondary);
  border: 1px solid transparent;
}

.btn-ghost:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

.picker-card {
  display: flex;
  flex-direction: column;
  gap: 12px;
  padding: 28px;
  border-radius: 16px;
  border: 1px solid var(--border-color);
  background: var(--surface-color);
}

.picker-label {
  font-weight: 700;
  color: var(--text-color);
  font-size: 15px;
}

.picker-input {
  width: 100%;
  padding: 16px;
  border-radius: 12px;
  border: 2px dashed var(--border-color);
  background: var(--background-elevated);
  color: var(--text-secondary);
  transition: all 0.2s ease;
}

.picker-input:hover {
  border-color: var(--primary-color);
  background: var(--primary-glow);
}

.picker-input::file-selector-button {
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-dark) 100%);
  border: none;
  color: #fff;
  padding: 10px 20px;
  margin-right: 16px;
  border-radius: 10px;
  cursor: pointer;
  font-weight: 600;
  transition: all 0.2s ease;
}

.picker-input::file-selector-button:hover {
  transform: scale(1.02);
}

.picker-hint {
  font-size: 0.85rem;
  color: var(--text-tertiary);
}

.dropzone {
  border: 2px dashed var(--border-color);
  border-radius: 20px;
  padding: 56px;
  text-align: center;
  background: var(--surface-color);
  transition: all 0.3s ease;
}

.dropzone:hover {
  border-color: var(--primary-color);
}

.dropzone--active {
  border-color: var(--accent-color);
  background: linear-gradient(145deg, var(--primary-glow), var(--surface-color));
  box-shadow: 0 30px 60px var(--primary-glow);
  transform: scale(1.01);
}

.dropzone__content {
  display: flex;
  flex-direction: column;
  gap: 16px;
  align-items: center;
}

.dropzone__content h2 {
  color: var(--text-color);
  font-size: 1.5rem;
  margin: 0;
}

.dropzone__content p {
  color: var(--text-secondary);
  margin: 0;
}

.drop-icon {
  color: var(--primary-color);
  opacity: 0.8;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 16px;
}

.stat-card {
  padding: 24px;
  border-radius: 16px;
  border: 1px solid var(--border-color);
  background: var(--surface-color);
  text-align: center;
  transition: all 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-4px);
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.stat-card .label {
  font-size: 0.85rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-tertiary);
  margin-bottom: 8px;
  font-weight: 600;
}

.stat-card .value {
  font-size: 2.5rem;
  font-weight: 700;
  color: var(--text-color);
}

.stat-card .value.success {
  color: var(--success-color);
}

.stat-card .value.error {
  color: var(--error-color);
}

.queue {
  border: 1px solid var(--border-color);
  border-radius: 20px;
  background: var(--surface-color);
  overflow: hidden;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.15);
}

.queue__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px 28px;
  border-bottom: 1px solid var(--border-color);
  background: var(--background-elevated);
}

.queue__header h3 {
  margin: 0;
  color: var(--text-color);
  font-size: 16px;
}

.queue__header span {
  color: var(--text-tertiary);
  font-size: 14px;
}

.queue__list {
  max-height: 400px;
  overflow-y: auto;
}

.queue-item {
  display: flex;
  justify-content: space-between;
  gap: 20px;
  padding: 20px 28px;
  flex-wrap: wrap;
  transition: background 0.2s ease;
}

.queue-item:hover {
  background: var(--background-elevated);
}

.queue-item:not(:last-child) {
  border-bottom: 1px solid var(--border-color);
}

.queue-item__left {
  display: flex;
  gap: 16px;
  align-items: center;
  min-width: 240px;
  flex: 1;
}

.file-icon {
  font-size: 2rem;
  width: 48px;
  height: 48px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary-glow);
  border-radius: 12px;
}

.file-name {
  font-size: 1rem;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 4px 0;
}

.file-meta {
  font-size: 0.85rem;
  color: var(--text-secondary);
  margin: 0;
}

.queue-item__right {
  display: flex;
  flex-direction: column;
  gap: 10px;
  min-width: 220px;
  align-items: flex-end;
}

.queue-item__actions {
  display: flex;
  gap: 8px;
}

.progress {
  width: 100%;
  height: 8px;
  background: var(--background-elevated);
  border-radius: 999px;
  overflow: hidden;
}

.progress__bar {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--accent-color));
  border-radius: inherit;
  transition: width 0.3s ease;
  position: relative;
}

.progress__bar::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255,255,255,0.3), transparent);
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(100%); }
}

.status-message {
  font-size: 0.85rem;
  color: var(--text-secondary);
}

.status-message.success {
  color: var(--success-color);
}

.status-message.error {
  color: var(--error-color);
}

@media (max-width: 900px) {
  .upload-header {
    flex-direction: column;
    padding: 24px;
  }

  .header-actions {
    width: 100%;
    justify-content: flex-start;
  }

  .dropzone {
    padding: 40px 24px;
  }

  .queue-item {
    padding: 16px 20px;
  }
}
</style>
