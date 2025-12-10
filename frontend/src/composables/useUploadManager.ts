/**
 * Global upload manager composable - handles persistent upload state across page navigation
 * Uses session storage to persist upload data and allows uploads to continue even when user leaves the upload page
 */

import { computed, ref } from 'vue'
import { musicAPI } from '../api/music'

export interface UploadItem {
  id: string
  file: File
  progress: number
  status: 'queued' | 'uploading' | 'success' | 'error'
  message?: string
}

interface UploadManagerState {
  uploads: UploadItem[]
  uploading: boolean
  isInitialized: boolean
}

// Singleton state
let uploadManagerInstance: UploadManager | null = null

class UploadManager {
  private uploads = ref<UploadItem[]>([])
  private uploading = ref(false)
  private isInitialized = ref(false)
  private abortControllers = new Map<string, AbortController>()

  constructor() {
    this.loadFromSessionStorage()
  }

  private getSessionStorageKey() {
    return 'musicserver_uploads'
  }

  private loadFromSessionStorage() {
    try {
      const stored = sessionStorage.getItem(this.getSessionStorageKey())
      if (stored) {
        const state = JSON.parse(stored) as Omit<UploadManagerState, 'isInitialized'>
        // Don't restore files since File objects can't be serialized
        // Just restore metadata
        this.uploads.value = state.uploads.map((item) => ({
          ...item,
          file: new File([], 'restored'), // Placeholder - will be set if retry is attempted
        }))
        this.uploading.value = state.uploading
      }
    } catch (e) {
      console.warn('Failed to load upload state from session storage:', e)
    }
    this.isInitialized.value = true
  }

  private saveToSessionStorage() {
    try {
      // Only serialize metadata, not File objects
      const state: Omit<UploadManagerState, 'isInitialized'> = {
        uploads: this.uploads.value.map((item) => ({
          ...item,
          file: undefined as any, // Don't serialize File objects
        })),
        uploading: this.uploading.value,
      }
      sessionStorage.setItem(this.getSessionStorageKey(), JSON.stringify(state))
    } catch (e) {
      console.warn('Failed to save upload state to session storage:', e)
    }
  }

  getUploads() {
    return this.uploads
  }

  getUploading() {
    return this.uploading
  }

  getIsInitialized() {
    return this.isInitialized
  }

  addFiles(fileList: FileList | File[]) {
    const newItems: UploadItem[] = []
    Array.from(fileList).forEach((file) => {
      if (!this.isAudioFile(file)) return
      const exists = this.uploads.value.some(
        (item) => item.file.name === file.name && item.file.size === file.size,
      )
      if (exists) return
      newItems.push({
        id: crypto?.randomUUID ? crypto.randomUUID() : `${Date.now()}-${Math.random()}`,
        file,
        progress: 0,
        status: 'queued',
      })
    })
    this.uploads.value = [...this.uploads.value, ...newItems]
    this.saveToSessionStorage()
  }

  removeItem(id: string) {
    this.uploads.value = this.uploads.value.filter((item) => item.id !== id)
    this.saveToSessionStorage()
  }

  clearCompleted() {
    this.uploads.value = this.uploads.value.filter((item) => item.status !== 'success')
    this.saveToSessionStorage()
  }

  retryItem(id: string) {
    const target = this.uploads.value.find((item) => item.id === id)
    if (!target) return
    target.status = 'queued'
    target.progress = 0
    target.message = undefined
    this.saveToSessionStorage()
  }

  async startUpload() {
    if (this.uploading.value) return
    
    const queuedOrError = this.uploads.value.filter(
      (item) => item.status === 'queued' || item.status === 'error',
    )
    if (queuedOrError.length === 0) return

    this.uploading.value = true
    this.saveToSessionStorage()

    for (const item of this.uploads.value) {
      if (item.status === 'success') continue
      if (item.status !== 'queued' && item.status !== 'error') continue

      item.status = 'uploading'
      item.progress = 0
      item.message = undefined

      const formData = new FormData()
      formData.append('file', item.file, item.file.name)

      const abortController = new AbortController()
      this.abortControllers.set(item.id, abortController)

      try {
        const response = await musicAPI.uploadMusicFiles(formData, {
          onUploadProgress: (event) => {
            if (event.total) {
              item.progress = Math.round((event.loaded / event.total) * 100)
              this.saveToSessionStorage()
            }
          },
          signal: abortController.signal,
        })
        item.progress = 100
        item.status = 'success'
        const inserted = response.data?.inserted as Array<{ title?: string }> | undefined
        item.message = inserted?.[0]?.title ? `Added ${inserted[0].title}` : 'Uploaded successfully'
      } catch (error: any) {
        if (error.name === 'AbortError') {
          // Upload was cancelled
          item.status = 'error'
          item.message = 'Upload cancelled'
        } else {
          item.status = 'error'
          item.message = error?.response?.data?.error || error?.message || 'Upload failed'
        }
      } finally {
        this.abortControllers.delete(item.id)
        this.saveToSessionStorage()
      }
    }

    this.uploading.value = false
    this.saveToSessionStorage()
  }

  cancelUpload(id: string) {
    const controller = this.abortControllers.get(id)
    if (controller) {
      controller.abort()
      this.abortControllers.delete(id)
    }
  }

  cancelAllUploads() {
    this.abortControllers.forEach((controller) => controller.abort())
    this.abortControllers.clear()
    this.uploading.value = false
    this.saveToSessionStorage()
  }

  clearAllUploads() {
    this.uploads.value = []
    this.uploading.value = false
    this.abortControllers.clear()
    sessionStorage.removeItem(this.getSessionStorageKey())
  }

  private isAudioFile(file: File) {
    const AUDIO_EXTENSIONS = ['.mp3', '.flac', '.wav', '.aiff', '.ogg', '.m4a']
    if (file.type.startsWith('audio/')) return true
    const lower = file.name.toLowerCase()
    return AUDIO_EXTENSIONS.some((ext) => lower.endsWith(ext))
  }
}

/**
 * Global upload manager - ensures only one instance exists
 */
function getUploadManagerInstance() {
  if (!uploadManagerInstance) {
    uploadManagerInstance = new UploadManager()
  }
  return uploadManagerInstance
}

export function useUploadManager() {
  const manager = getUploadManagerInstance()

  const uploads = computed(() => manager.getUploads().value)
  const uploading = computed(() => manager.getUploading().value)
  const isInitialized = computed(() => manager.getIsInitialized().value)

  const queueCount = computed(() => uploads.value.filter((item) => item.status === 'queued').length)
  const successCount = computed(() => uploads.value.filter((item) => item.status === 'success').length)
  const errorCount = computed(() => uploads.value.filter((item) => item.status === 'error').length)
  const uploadingCount = computed(
    () => uploads.value.filter((item) => item.status === 'uploading').length,
  )
  const totalCount = computed(() => uploads.value.length)

  const hasCompleted = computed(() => successCount.value > 0)
  const canUpload = computed(
    () => uploads.value.some((item) => item.status === 'queued' || item.status === 'error'),
  )

  const totalProgress = computed(() => {
    if (uploads.value.length === 0) return 0
    const totalProgress = uploads.value.reduce((sum, item) => sum + item.progress, 0)
    return Math.round(totalProgress / uploads.value.length)
  })

  return {
    // State
    uploads,
    uploading,
    isInitialized,

    // Computed
    queueCount,
    successCount,
    errorCount,
    uploadingCount,
    totalCount,
    hasCompleted,
    canUpload,
    totalProgress,

    // Methods
    addFiles: manager.addFiles.bind(manager),
    removeItem: manager.removeItem.bind(manager),
    clearCompleted: manager.clearCompleted.bind(manager),
    retryItem: manager.retryItem.bind(manager),
    startUpload: manager.startUpload.bind(manager),
    cancelUpload: manager.cancelUpload.bind(manager),
    cancelAllUploads: manager.cancelAllUploads.bind(manager),
    clearAllUploads: manager.clearAllUploads.bind(manager),
  }
}
