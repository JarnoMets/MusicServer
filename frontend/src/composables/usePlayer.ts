import { reactive, readonly } from 'vue'
import { getAPIBaseURL } from '../utils/api'

const API_BASE_URL = getAPIBaseURL()

type PlayerSource =
  | {
      type: 'local'
      id: string
      title: string
      artist?: string | null
    }
  | {
      type: 'stream'
      title: string
      url: string
      genre?: string
    }

interface TrackInfo {
  id: string
  title: string
  artist?: string | null
}

interface PlayerState {
  currentSource: PlayerSource | null
  audioUrl: string | null
  updatedAt: number
  isPlaying: boolean
  queue: TrackInfo[]
  currentIndex: number
  history: TrackInfo[]
  autoplay: boolean
  shuffle: boolean
  repeat: 'off' | 'all' | 'one'
}

const state = reactive<PlayerState>({
  currentSource: null,
  audioUrl: null,
  updatedAt: Date.now(),
  isPlaying: false,
  queue: [],
  currentIndex: -1,
  history: [],
  autoplay: true,
  shuffle: false,
  repeat: 'off',
})

const buildStreamUrl = (musicId: string) => {
  return `${API_BASE_URL}/music/${musicId}/stream`
}

const playLocalTrack = (params: { id: string; title: string; artist?: string | null }) => {
  // Add to history if we have a current track
  if (state.currentSource?.type === 'local') {
    const currentTrack: TrackInfo = {
      id: state.currentSource.id,
      title: state.currentSource.title,
      artist: state.currentSource.artist,
    }
    // Only add if different from last history item
    if (state.history.length === 0 || state.history[state.history.length - 1].id !== currentTrack.id) {
      state.history.push(currentTrack)
      // Keep history limited
      if (state.history.length > 50) {
        state.history.shift()
      }
    }
  }

  state.currentSource = {
    type: 'local',
    id: params.id,
    title: params.title,
    artist: params.artist,
  }
  state.audioUrl = buildStreamUrl(params.id)
  state.updatedAt = Date.now()
  state.isPlaying = true

  // Update queue index if track is in queue
  const idx = state.queue.findIndex(t => t.id === params.id)
  if (idx !== -1) {
    state.currentIndex = idx
  }
}

const playInternetStream = (params: { title: string; url: string; genre?: string }) => {
  state.currentSource = {
    type: 'stream',
    title: params.title,
    url: params.url,
    genre: params.genre,
  }
  state.audioUrl = params.url
  state.updatedAt = Date.now()
  state.isPlaying = true
  // Clear queue for streams
  state.queue = []
  state.currentIndex = -1
}

const stopPlayback = () => {
  state.isPlaying = false
  state.currentSource = null
  state.audioUrl = null
  state.updatedAt = Date.now()
}

const setPlayingStatus = (value: boolean) => {
  state.isPlaying = value
}

// Queue management
const setQueue = (tracks: TrackInfo[], startIndex = 0) => {
  state.queue = tracks
  state.currentIndex = startIndex
  if (tracks.length > 0 && startIndex >= 0 && startIndex < tracks.length) {
    playLocalTrack(tracks[startIndex])
  }
}

const addToQueue = (track: TrackInfo) => {
  state.queue.push(track)
}

const clearQueue = () => {
  state.queue = []
  state.currentIndex = -1
}

const hasPreviousTrack = () => {
  // Can go back if we have history
  return state.history.length > 0
}

const hasNextTrack = () => {
  // Can go forward if there are more tracks in queue
  return state.currentIndex >= 0 && state.currentIndex < state.queue.length - 1
}

const playPreviousTrack = () => {
  if (state.history.length > 0) {
    const prevTrack = state.history.pop()!
    // Add current track back to front of queue if it exists
    if (state.currentSource?.type === 'local') {
      const currentTrack: TrackInfo = {
        id: state.currentSource.id,
        title: state.currentSource.title,
        artist: state.currentSource.artist,
      }
      // Insert at current position
      if (state.currentIndex >= 0) {
        state.queue.splice(state.currentIndex, 0, currentTrack)
      }
    }
    
    state.currentSource = {
      type: 'local',
      id: prevTrack.id,
      title: prevTrack.title,
      artist: prevTrack.artist,
    }
    state.audioUrl = buildStreamUrl(prevTrack.id)
    state.updatedAt = Date.now()
    state.isPlaying = true
    
    // Update index
    const idx = state.queue.findIndex(t => t.id === prevTrack.id)
    state.currentIndex = idx
  }
}

const playNextTrack = () => {
  if (state.currentIndex >= 0 && state.currentIndex < state.queue.length - 1) {
    const nextIndex = state.currentIndex + 1
    const nextTrack = state.queue[nextIndex]
    
    // Add current to history
    if (state.currentSource?.type === 'local') {
      const currentTrack: TrackInfo = {
        id: state.currentSource.id,
        title: state.currentSource.title,
        artist: state.currentSource.artist,
      }
      if (state.history.length === 0 || state.history[state.history.length - 1].id !== currentTrack.id) {
        state.history.push(currentTrack)
        if (state.history.length > 50) {
          state.history.shift()
        }
      }
    }
    
    state.currentIndex = nextIndex
    state.currentSource = {
      type: 'local',
      id: nextTrack.id,
      title: nextTrack.title,
      artist: nextTrack.artist,
    }
    state.audioUrl = buildStreamUrl(nextTrack.id)
    state.updatedAt = Date.now()
    state.isPlaying = true
  } else if (state.repeat === 'all' && state.queue.length > 0) {
    // Loop back to beginning when repeat is enabled
    state.currentIndex = 0
    const nextTrack = state.queue[0]
    state.currentSource = {
      type: 'local',
      id: nextTrack.id,
      title: nextTrack.title,
      artist: nextTrack.artist,
    }
    state.audioUrl = buildStreamUrl(nextTrack.id)
    state.updatedAt = Date.now()
    state.isPlaying = true
  }
}

const toggleAutoplay = () => {
  state.autoplay = !state.autoplay
}

const toggleShuffle = () => {
  state.shuffle = !state.shuffle
}

const cycleRepeat = () => {
  const modes: Array<'off' | 'all' | 'one'> = ['off', 'all', 'one']
  const currentIndex = modes.indexOf(state.repeat)
  state.repeat = modes[(currentIndex + 1) % modes.length]
}

export const usePlayer = () => {
  return {
    state: readonly(state),
    playLocalTrack,
    playInternetStream,
    stopPlayback,
    setPlayingStatus,
    setQueue,
    addToQueue,
    clearQueue,
    hasPreviousTrack,
    hasNextTrack,
    playPreviousTrack,
    playNextTrack,
    toggleAutoplay,
    toggleShuffle,
    cycleRepeat,
  }
}
