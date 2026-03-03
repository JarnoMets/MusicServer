import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { musicAPI } from '../api/music'

export interface DownloadProgress {
  status: string
  progress: number | null
  current_file: string | null
  total_files: number | null
  completed_files: number | null
  failed_files: number | null
  is_cancelled: boolean | null
}

export const useDownloadStore = defineStore('download', () => {
  const activeSessionId = ref<string | null>(null)
  const currentProgress = ref<DownloadProgress | null>(null)
  const lastCompletedAt = ref<number>(0)
  
  const isDownloading = computed(() => {
    if (!activeSessionId.value || !currentProgress.value) return false
    return currentProgress.value.progress !== null && 
           currentProgress.value.progress < 100 && 
           !currentProgress.value.is_cancelled
  })

  let eventSource: EventSource | null = null

  function setSession(sessionId: string) {
    if (eventSource) {
      eventSource.close()
    }
    
    activeSessionId.value = sessionId
    const stream = musicAPI.getYoutubeProgressStream(sessionId)
    
    stream.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data)
        currentProgress.value = data
        
        if (data.progress === 100 || data.is_cancelled) {
          stream.close()
          activeSessionId.value = null
          lastCompletedAt.value = Date.now()
        }
      } catch (e) {
        console.error('Error parsing download progress:', e)
      }
    }
    
    stream.onerror = () => {
      stream.close()
      activeSessionId.value = null
      currentProgress.value = null
    }
    
    eventSource = stream
  }

  function cancelDownload() {
    if (activeSessionId.value) {
      musicAPI.cancelYoutubeDownload(activeSessionId.value)
    }
    if (eventSource) {
      eventSource.close()
      eventSource = null
    }
    activeSessionId.value = null
    currentProgress.value = null
  }

  return {
    activeSessionId,
    currentProgress,
    isDownloading,
    lastCompletedAt,
    setSession,
    cancelDownload
  }
})
