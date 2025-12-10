<template>
  <div class="upload-view">
    <header class="upload-header">
      <div>
        <p class="eyebrow">Library tools</p>
        <h1>Upload music files</h1>
        <p class="subtitle">
          Select files and start the upload. Files will be added to your library immediately. You can safely leave this page during upload.
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

    <!-- Collapsible file picker section -->
    <section class="picker-section">
      <button class="picker-toggle" @click="isPickerOpen = !isPickerOpen">
        <Icon :name="isPickerOpen ? 'chevron-down' : 'chevron-right'" :size="18" />
        <span>Add files</span>
        <span class="toggle-hint">(Drag & drop or browse)</span>
      </button>

      <Transition name="expand">
        <div v-if="isPickerOpen" class="picker-content">
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
          <p class="picker-hint">
            Supports FLAC, MP3, WAV, AIFF, OGG, M4A and more. Select multiple files at once.
          </p>

          <section
            class="dropzone"
            :class="{ 'dropzone--active': isDragging }"
            @dragenter.prevent="handleDrag(true)"
            @dragover.prevent
            @dragleave.prevent="handleDrag(false)"
            @drop.prevent="handleDrop"
          >
            <div class="dropzone__content">
              <Icon name="upload" :size="40" class="drop-icon" />
              <h2>Drag files here</h2>
              <p>Or use the picker above</p>
              <button class="btn btn-secondary" @click="triggerFileDialog">Browse files</button>
            </div>
          </section>
        </div>
      </Transition>
    </section>

    <!-- Stats -->
    <section class="stats-grid">
      <article class="stat-card">
        <p class="label">Queued</p>
        <p class="value">{{ queueCount }}</p>
      </article>
      <article class="stat-card">
        <p class="label">Uploading</p>
        <p class="value uploading">{{ uploadingCount }}</p>
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

    <!-- Queue -->
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

    <!-- Empty state -->
    <section v-if="!uploads.length" class="empty-state">
      <Icon name="inbox" :size="48" class="empty-icon" />
      <h2>No files queued</h2>
      <p>Add files using the form above to get started</p>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useUploadManager } from '../composables/useUploadManager'
import Icon from '../shared/components/Icons.vue'

const {
  uploads,
  uploading,
  queueCount,
  successCount,
  errorCount,
  uploadingCount,
  hasCompleted,
  canUpload,
  addFiles,
  removeItem,
  clearCompleted,
  retryItem,
  startUpload,
} = useUploadManager()

const fileInput = ref<HTMLInputElement | null>(null)
const isDragging = ref(false)
const isPickerOpen = ref(false)

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

function formatBytes(bytes: number) {
  if (bytes === 0) return '0 B'
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return `${(bytes / Math.pow(k, i)).toFixed(1)} ${sizes[i]}`
}

function itemStatusLabel(status: string) {
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
</script>

<style scoped>
.upload-view {
  display: flex;
  flex-direction: column;
  gap: 24px;
  max-width: 900px;
}

.upload-header {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  padding: 28px;
  border-radius: 16px;
  border: 1px solid var(--border-color);
  background: linear-gradient(145deg, var(--primary-glow), var(--surface-color));
  box-shadow: 0 4px 24px rgba(0, 0, 0, 0.1);
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
  font-size: 1.8rem;
  color: var(--text-color);
}

.subtitle {
  margin-top: 8px;
  margin-bottom: 0;
  color: var(--text-secondary);
  line-height: 1.5;
  font-size: 0.95rem;
}

.header-actions {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
  align-items: center;
  justify-content: flex-end;
}

.btn {
  border: none;
  border-radius: 10px;
  padding: 10px 20px;
  cursor: pointer;
  font-weight: 600;
  font-size: 13px;
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
  padding: 8px 12px;
}

.btn-ghost:hover:not(:disabled) {
  background: rgba(239, 68, 68, 0.1);
  color: #ef4444;
  border-color: rgba(239, 68, 68, 0.3);
}

/* Picker section - collapsible */
.picker-section {
  border: 1px solid var(--border-color);
  border-radius: 12px;
  overflow: hidden;
  background: var(--surface-color);
}

.picker-toggle {
  width: 100%;
  padding: 16px 20px;
  background: transparent;
  border: none;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  align-items: center;
  gap: 12px;
  cursor: pointer;
  font-weight: 600;
  color: var(--text-color);
  font-size: 14px;
  transition: background 0.2s ease;
}

.picker-toggle:hover {
  background: var(--background-elevated);
}

.toggle-hint {
  font-size: 12px;
  color: var(--text-tertiary);
  font-weight: 400;
  margin-left: auto;
}

.picker-content {
  padding: 20px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.picker-label {
  font-weight: 700;
  color: var(--text-color);
  font-size: 14px;
  margin-bottom: 4px;
}

.picker-input {
  width: 100%;
  padding: 12px;
  border-radius: 8px;
  border: 2px dashed var(--border-color);
  background: var(--background-elevated);
  color: var(--text-secondary);
  transition: all 0.2s ease;
  font-size: 13px;
}

.picker-input:hover {
  border-color: var(--primary-color);
  background: var(--primary-glow);
}

.picker-input::file-selector-button {
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-dark) 100%);
  border: none;
  color: #fff;
  padding: 8px 16px;
  margin-right: 12px;
  border-radius: 8px;
  cursor: pointer;
  font-weight: 600;
  font-size: 12px;
  transition: all 0.2s ease;
}

.picker-input::file-selector-button:hover {
  transform: scale(1.02);
}

.picker-hint {
  font-size: 0.8rem;
  color: var(--text-tertiary);
  margin: 0;
}

.dropzone {
  border: 2px dashed var(--border-color);
  border-radius: 12px;
  padding: 32px 24px;
  text-align: center;
  background: var(--background-elevated);
  transition: all 0.3s ease;
}

.dropzone:hover {
  border-color: var(--primary-color);
}

.dropzone--active {
  border-color: var(--accent-color);
  background: linear-gradient(145deg, var(--primary-glow), var(--background-elevated));
  box-shadow: 0 20px 40px var(--primary-glow);
  transform: scale(1.01);
}

.dropzone__content {
  display: flex;
  flex-direction: column;
  gap: 12px;
  align-items: center;
}

.dropzone__content h2 {
  color: var(--text-color);
  font-size: 1.2rem;
  margin: 0;
}

.dropzone__content p {
  color: var(--text-secondary);
  margin: 0;
  font-size: 0.9rem;
}

.drop-icon {
  color: var(--primary-color);
  opacity: 0.8;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

.stat-card {
  padding: 16px;
  border-radius: 12px;
  border: 1px solid var(--border-color);
  background: var(--surface-color);
  text-align: center;
  transition: all 0.2s ease;
}

.stat-card:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.stat-card .label {
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-tertiary);
  margin-bottom: 6px;
  font-weight: 600;
}

.stat-card .value {
  font-size: 2rem;
  font-weight: 700;
  color: var(--text-color);
}

.stat-card .value.success {
  color: #10b981;
}

.stat-card .value.uploading {
  color: var(--primary-light);
}

.stat-card .value.error {
  color: #ef4444;
}

.queue {
  border: 1px solid var(--border-color);
  border-radius: 12px;
  background: var(--surface-color);
  overflow: hidden;
  box-shadow: 0 2px 12px rgba(0, 0, 0, 0.08);
}

.queue__header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  background: var(--background-elevated);
}

.queue__header h3 {
  margin: 0;
  color: var(--text-color);
  font-size: 14px;
  font-weight: 600;
}

.queue__header span {
  color: var(--text-tertiary);
  font-size: 12px;
}

.queue__list {
  max-height: 500px;
  overflow-y: auto;
}

.queue-item {
  display: flex;
  justify-content: space-between;
  gap: 16px;
  padding: 16px 20px;
  flex-wrap: wrap;
  transition: background 0.2s ease;
  align-items: center;
}

.queue-item:hover {
  background: var(--background-elevated);
}

.queue-item:not(:last-child) {
  border-bottom: 1px solid var(--border-color);
}

.queue-item__left {
  display: flex;
  gap: 12px;
  align-items: center;
  min-width: 180px;
  flex: 1;
}

.file-icon {
  font-size: 1.5rem;
  width: 40px;
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--primary-glow);
  border-radius: 8px;
  flex-shrink: 0;
}

.file-name {
  font-size: 0.95rem;
  font-weight: 600;
  color: var(--text-color);
  margin: 0 0 2px 0;
  word-break: break-word;
}

.file-meta {
  font-size: 0.8rem;
  color: var(--text-secondary);
  margin: 0;
}

.queue-item__right {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 150px;
  align-items: flex-end;
}

.queue-item__actions {
  display: flex;
  gap: 6px;
}

.progress {
  width: 100%;
  height: 6px;
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
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.status-message {
  font-size: 0.8rem;
  color: var(--text-secondary);
}

.status-message.success {
  color: #10b981;
}

.status-message.error {
  color: #ef4444;
}

/* Empty state */
.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: var(--text-tertiary);
}

.empty-icon {
  color: var(--border-color);
  margin-bottom: 16px;
}

.empty-state h2 {
  margin: 0 0 8px 0;
  font-size: 1.1rem;
  color: var(--text-secondary);
}

.empty-state p {
  margin: 0;
  font-size: 0.9rem;
}

/* Animations */
.expand-enter-active,
.expand-leave-active {
  transition: all 0.3s ease;
}

.expand-enter-from {
  opacity: 0;
  max-height: 0;
}

.expand-leave-to {
  opacity: 0;
  max-height: 0;
}

@media (max-width: 900px) {
  .upload-header {
    flex-direction: column;
    padding: 20px;
  }

  .header-actions {
    width: 100%;
  }

  .upload-header h1 {
    font-size: 1.5rem;
  }

  .subtitle {
    font-size: 0.9rem;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .queue-item {
    flex-direction: column;
    align-items: flex-start;
  }

  .queue-item__right {
    width: 100%;
    align-items: flex-start;
  }

  .queue-item__left {
    width: 100%;
  }

  .dropzone {
    padding: 24px 16px;
  }
}

@media (max-width: 600px) {
  .upload-view {
    gap: 16px;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .stat-card {
    padding: 12px;
  }

  .stat-card .value {
    font-size: 1.5rem;
  }
}
</style>
