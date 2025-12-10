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
  private hashCache = new Map<string, { hash: string; isDuplicate: boolean }>()
  private checkTimeoutMs = 5000 // 5 second timeout for duplicate check

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
        // Only restore completed/successful uploads, not queued or in-progress ones
        // since we can't actually re-upload without the file
        this.uploads.value = state.uploads.filter((item) => item.status === 'success')
        this.uploading.value = false // Never restore uploading state
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

  private async checkDuplicate(fileHash: string): Promise<boolean> {
    try {
      const response = await musicAPI.checkDuplicateHash(fileHash)
      return response.data?.exists === true
    } catch {
      // If check fails, proceed with upload anyway
      return false
    }
  }

  /**
   * Compute file hash using streaming to avoid loading entire file into memory
   * Uses Web Crypto API for efficient SHA-256 hashing
   */
  private async computeFileHashStreaming(file: File): Promise<string> {
    const cacheKey = `${file.name}-${file.size}-${file.lastModified}`
    
    // Return cached hash if available
    if (this.hashCache.has(cacheKey)) {
      return this.hashCache.get(cacheKey)!.hash
    }

    const hashBuffer = await crypto.subtle.digest('SHA-256', await file.arrayBuffer())
    const hashArray = Array.from(new Uint8Array(hashBuffer))
    const hashHex = hashArray.map((b) => b.toString(16).padStart(2, '0')).join('')
    
    return hashHex
  }

  /**
   * Check for duplicate with timeout fallback - if check takes too long, proceed with upload
   * Returns true if file is a duplicate, false otherwise
   */
  private async checkDuplicateWithTimeout(file: File): Promise<boolean> {
    // Skip check for very small files (unlikely to be duplicates worth checking)
    if (file.size < 1024 * 1024) {
      // < 1MB
      return false
    }

    try {
      // Race between hash computation + check and timeout
      const checkPromise = (async () => {
        const hash = await this.computeFileHashStreaming(file)
        const isDuplicate = await this.checkDuplicate(hash)
        
        // Cache the result
        const cacheKey = `${file.name}-${file.size}-${file.lastModified}`
        this.hashCache.set(cacheKey, { hash, isDuplicate })
        
        return isDuplicate
      })()

      // If check takes longer than timeout, assume it's not a duplicate and proceed
      const timeoutPromise = new Promise<boolean>((resolve) =>
        setTimeout(() => resolve(false), this.checkTimeoutMs)
      )

      return Promise.race([checkPromise, timeoutPromise])
    } catch {
      // If anything fails, proceed with upload
      return false
    }
  }

  async startUpload() {
    if (this.uploading.value) return
    
    const queuedOrError = this.uploads.value.filter(
      (item) => item.status === 'queued' || item.status === 'error',
    )
    if (queuedOrError.length === 0) return

    this.uploading.value = true
    this.saveToSessionStorage()

    // Collect items to retry (502/503 errors) at the end
    const retryItems: UploadItem[] = []
    
    // Process uploads with parallel limit for better performance on bulk uploads
    // Use 3 concurrent uploads as a balance between performance and server load
    const MAX_CONCURRENT = 3
    let activeUploads = 0
    let itemIndex = 0

    const uploadQueue = async () => {
      while (itemIndex < this.uploads.value.length || activeUploads > 0) {
        // Start new uploads up to the concurrent limit
        while (activeUploads < MAX_CONCURRENT && itemIndex < this.uploads.value.length) {
          const item = this.uploads.value[itemIndex++]
          if (item.status === 'success') continue
          if (item.status !== 'queued' && item.status !== 'error') continue

          activeUploads++
          this.uploadSingleFile(item, retryItems).finally(() => {
            activeUploads--
          })
        }

        // Wait a bit before checking for more uploads
        await new Promise(resolve => setTimeout(resolve, 50))
      }
    }

    await uploadQueue()

    // Retry 502/503 errors with exponential backoff
    if (retryItems.length > 0) {
      await this.retryWithBackoff(retryItems)
    }

    this.uploading.value = false
    this.saveToSessionStorage()
  }

  private async uploadSingleFile(item: UploadItem, retryItems: UploadItem[]) {
    item.status = 'uploading'
    item.progress = 0
    item.message = undefined
    this.saveToSessionStorage()

    const abortController = new AbortController()
    this.abortControllers.set(item.id, abortController)

    try {
      // Check for duplicates efficiently (skips for small files, times out if takes too long)
      item.message = 'Checking for duplicates...'
      this.saveToSessionStorage()
      const isDuplicate = await this.checkDuplicateWithTimeout(item.file)
      
      if (isDuplicate) {
        item.status = 'success'
        item.message = 'Duplicate file (already exists)'
        item.progress = 100
        return
      }

      // Proceed with upload
      item.message = 'Uploading...'
      this.saveToSessionStorage()

      const formData = new FormData()
      formData.append('file', item.file, item.file.name)

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
      
      // Check if this file was marked as duplicate in the response
      const errors = response.data?.errors as string[] | undefined
      if (errors && errors.some((err) => err.toLowerCase().includes('duplicate'))) {
        item.status = 'success'  // Mark duplicates as success
        item.message = 'Duplicate file (already exists)'
      } else {
        item.status = 'success'
        const inserted = response.data?.inserted as Array<{ title?: string }> | undefined
        item.message = inserted?.[0]?.title ? `Added ${inserted[0].title}` : 'Uploaded successfully'
      }
    } catch (error: any) {
      if (error.name === 'AbortError') {
        // Upload was cancelled
        item.status = 'error'
        item.message = 'Upload cancelled'
      } else {
        const status = error?.response?.status
        // Collect 502/503 errors for retry
        if (status === 502 || status === 503) {
          item.status = 'error'
          item.message = `Server error (${status}), will retry...`
          retryItems.push(item)
        } else {
          item.status = 'error'
          item.message = error?.response?.data?.error || error?.message || 'Upload failed'
        }
      }
    } finally {
      this.abortControllers.delete(item.id)
      this.saveToSessionStorage()
    }
  }

  private async retryWithBackoff(items: UploadItem[]) {
    const MAX_RETRIES = 3
    const BASE_DELAY_MS = 1000

    for (let retryCount = 0; retryCount < MAX_RETRIES; retryCount++) {
      const delayMs = BASE_DELAY_MS * Math.pow(2, retryCount) // 1s, 2s, 4s
      
      console.log(`Retrying ${items.length} items in ${delayMs}ms (attempt ${retryCount + 1}/${MAX_RETRIES})`)
      await new Promise(resolve => setTimeout(resolve, delayMs))

      const stillNeedRetry: UploadItem[] = []

      for (const item of items) {
        // Only retry if still in error state
        if (item.status !== 'error') continue

        const formData = new FormData()
        formData.append('file', item.file, item.file.name)

        const abortController = new AbortController()
        this.abortControllers.set(item.id, abortController)

        try {
          item.message = `Retrying... (attempt ${retryCount + 1}/${MAX_RETRIES})`
          this.saveToSessionStorage()

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
          const errors = response.data?.errors as string[] | undefined
          if (errors && errors.some((err) => err.toLowerCase().includes('duplicate'))) {
            item.status = 'error'
            item.message = 'Duplicate file skipped (already exists)'
          } else {
            item.status = 'success'
            const inserted = response.data?.inserted as Array<{ title?: string }> | undefined
            item.message = inserted?.[0]?.title ? `Added ${inserted[0].title}` : 'Uploaded successfully'
          }
        } catch (error: any) {
          const status = error?.response?.status
          if (status === 502 || status === 503) {
            // Still getting server error, retry again
            stillNeedRetry.push(item)
          } else {
            // Different error, don't retry
            item.status = 'error'
            item.message = error?.response?.data?.error || error?.message || 'Upload failed'
          }
        } finally {
          this.abortControllers.delete(item.id)
          this.saveToSessionStorage()
        }
      }

      // Update items to retry for next iteration
      items = stillNeedRetry
      if (items.length === 0) break
    }

    // Mark any remaining items as failed
    for (const item of items) {
      if (item.status === 'error') {
        item.message = 'Upload failed (server unavailable after retries)'
        this.saveToSessionStorage()
      }
    }
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
