<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import WaveSurfer from 'wavesurfer.js'
import RegionsPlugin from 'wavesurfer.js/dist/plugins/regions.js'
import { musicAPI } from '../api/music'
import { getAPIBaseURL } from '../utils/api'
import { formatTimePrecise as formatTime } from '../utils/audioHelpers'
import { useToast } from '../composables/useToast'
import { useConfirm } from '../composables/useConfirm'
import Icon from '@/shared/components/Icons.vue'

const route = useRoute()
const router = useRouter()
const { success, error } = useToast()
const { confirm: confirmAction } = useConfirm()

const trackId = route.params.id as string
const track = ref<any>(null)
const waveformContainer = ref<HTMLElement | null>(null)
const wavesurfer = ref<WaveSurfer | null>(null)
const regions = ref<any>(null)
const isLoading = ref(true)
const isProcessing = ref(false)

const startPos = ref(0)
const endPos = ref(0)
const duration = ref(0)

const initWaveSurfer = async () => {
  if (!waveformContainer.value) return

  wavesurfer.value = WaveSurfer.create({
    container: waveformContainer.value,
    waveColor: '#4f46e5',
    progressColor: '#818cf8',
    cursorColor: '#ffffff',
    height: 128,
    normalize: true,
    backend: 'WebAudio',
  })

  regions.value = wavesurfer.value.registerPlugin(RegionsPlugin.create())

  const apiBase = getAPIBaseURL()
  const audioUrl = `${apiBase}/music/${trackId}/stream`

  wavesurfer.value.load(audioUrl)

  wavesurfer.value.on('ready', () => {
    isLoading.value = false
    duration.value = wavesurfer.value!.getDuration()
    
    // Create initial region for the whole track
    regions.value.addRegion({
      start: 0,
      end: duration.value,
      color: 'rgba(79, 70, 229, 0.2)',
      drag: true,
      resize: true,
    })

    startPos.value = 0
    endPos.value = duration.value
  })

  regions.value.on('region-updated', (region: any) => {
    startPos.value = region.start
    endPos.value = region.end
  })

  // Also support clicking to seek
  wavesurfer.value.on('interaction', () => {
    wavesurfer.value?.play()
  })
}

const togglePlay = () => {
  wavesurfer.value?.playPause()
}

const handleCut = async () => {
  if (isProcessing.value) return
  
  const confirmed = await confirmAction({
    title: 'Cut Audio',
    message: `Are you sure you want to cut this track to ${formatTime(startPos.value)} - ${formatTime(endPos.value)}? This will replace the original file and cannot be undone.`,
    confirmText: 'Cut and Replace',
    cancelText: 'Cancel',
    variant: 'danger'
  })
  if (!confirmed) return

  isProcessing.value = true
  try {
    await musicAPI.cutMusicFile(trackId, startPos.value, endPos.value)
    success('Track cut successfully', 'The audio file has been updated.')
    router.push('/')
  } catch (err: any) {
    console.error('Error cutting track:', err)
    error('Failed to cut track', err?.response?.data?.error || err?.message)
  } finally {
    isProcessing.value = false
  }
}

onMounted(async () => {
  try {
    const response = await musicAPI.getMusicFile(trackId)
    track.value = response.data
    await initWaveSurfer()
  } catch (err) {
    console.error('Failed to load track metadata:', err)
    error('Error', 'Failed to load track information')
  }
})

onUnmounted(() => {
  wavesurfer.value?.destroy()
})
</script>

<template>
  <div class="edit-track-container page-shell p-6">
    <div class="edit-track-hero surface-card">
      <div>
        <p class="page-kicker">Track editor</p>
        <h1>Edit track</h1>
        <p v-if="track" class="page-description">{{ track.artist }} — {{ track.title }}</p>
      </div>
      <div class="flex gap-4">
        <button 
          @click="router.back()" 
          class="px-4 py-2 rounded-lg bg-gray-800 text-white hover:bg-gray-700 transition"
          type="button"
        >
          Cancel
        </button>
        <button 
          @click="handleCut" 
          :disabled="isProcessing || isLoading"
          class="px-4 py-2 rounded-lg bg-indigo-600 text-white hover:bg-indigo-500 transition disabled:opacity-50 disabled:cursor-not-allowed flex items-center gap-2"
          type="button"
        >
          <Icon v-if="isProcessing" name="loader" class="animate-spin" :size="18" />
          {{ isProcessing ? 'Processing...' : 'Apply Cut' }}
        </button>
      </div>
    </div>

    <div class="edit-track-card surface-card">
      <div v-if="isLoading" class="h-32 flex items-center justify-center">
        <div class="animate-spin rounded-full h-8 w-8 border-b-2 border-indigo-500"></div>
      </div>
      
      <div ref="waveformContainer" class="mb-4"></div>
      
      <div class="edit-track-meta">
        <div class="flex gap-6">
          <div>
            <span class="block text-xs uppercase tracking-wider text-gray-500 mb-1">Start</span>
            <span class="text-lg font-mono text-white">{{ formatTime(startPos) }}</span>
          </div>
          <div>
            <span class="block text-xs uppercase tracking-wider text-gray-500 mb-1">End</span>
            <span class="text-lg font-mono text-white">{{ formatTime(endPos) }}</span>
          </div>
          <div>
            <span class="block text-xs uppercase tracking-wider text-gray-500 mb-1">Duration</span>
            <span class="text-lg font-mono text-indigo-400">{{ formatTime(endPos - startPos) }}</span>
          </div>
        </div>
        
        <div class="flex items-center gap-4">
          <button 
            @click="togglePlay" 
            class="w-12 h-12 rounded-full bg-white text-black flex items-center justify-center hover:bg-gray-200 transition"
            type="button"
          >
            <Icon v-if="wavesurfer?.isPlaying()" name="pause" :size="24" />
            <Icon v-else name="play" :size="24" />
          </button>
        </div>
      </div>
    </div>

    <div class="edit-track-help surface-card">
      <h3 class="text-sm font-semibold text-indigo-300 mb-2 flex items-center gap-2">
        <Icon name="info" :size="16" /> How to use
      </h3>
      <ul class="text-xs text-gray-400 space-y-1">
        <li>&bull; Drag the edges of the highlighted region to select the part you want to KEEP.</li>
        <li>&bull; Drag the entire region to move the selection.</li>
        <li>&bull; Click anywhere on the waveform to seek and listen.</li>
        <li>&bull; <strong>Apply Cut</strong> will remove everything outside the selection and update the file.</li>
      </ul>
    </div>
  </div>
</template>

<style scoped>
.edit-track-container {
  max-width: 1000px;
  margin: 0 auto;
}

.edit-track-hero {
  display: flex;
  justify-content: space-between;
  gap: 24px;
  align-items: center;
  padding: 24px;
}

.edit-track-hero h1 {
  margin: 0;
  font-size: 2rem;
}

.edit-track-card,
.edit-track-help {
  padding: 24px;
}

.edit-track-meta {
  display: flex;
  justify-content: space-between;
  gap: 20px;
  align-items: center;
  margin-top: 20px;
}

.edit-track-meta span {
  color: var(--text-secondary);
}

.edit-track-card :deep(.wavesurfer-region) {
  border-radius: 12px;
}

:deep(.wavesurfer-region) {
  border: 2px solid rgba(79, 70, 229, 0.4) !important;
}

:deep(.wavesurfer-handle) {
  width: 10px !important;
  background-color: #4f46e5 !important;
}

@media (max-width: 768px) {
  .edit-track-hero,
  .edit-track-meta {
    flex-direction: column;
    align-items: stretch;
  }

  .edit-track-help,
  .edit-track-card,
  .edit-track-hero {
    padding: 20px;
  }
}
</style>
