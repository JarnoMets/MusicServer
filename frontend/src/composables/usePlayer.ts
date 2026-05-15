import { reactive, readonly } from 'vue'
import { getAPIBaseURL } from '../utils/api'

const API_BASE_URL = getAPIBaseURL()

/**
 * Mirror the currently-playing track to DJ Deck 1.
 *
 * This runs lazily (next tick) so it doesn't block the regular player from
 * starting.  It only mirrors *local* tracks (not internet streams) because
 * streams don't have an ID the deck system can load.
 *
 * The DJ audio engine is a singleton — its state survives route changes, so
 * the track stays loaded in Deck 1 even when the user navigates to /decks.
 * Deck 1 is loaded in a *paused/cued* state so it doesn't produce any sound
 * until the user explicitly hits play on the deck.
 */
let mirrorQueued = false
const mirrorToDeck1 = () => {
  if (mirrorQueued) return
  mirrorQueued = true
  // Use setTimeout(0) so the import is lazy and doesn't create circular deps
  // at module-init time.
  setTimeout(async () => {
    mirrorQueued = false
    const src = state.currentSource
    if (!src || src.type !== 'local') return

    try {
      // Dynamic imports to avoid circular dependency at module level
      const { useDjStore } = await import('../stores/djStore')
      const { useDjAudioEngine } = await import('./useDjAudioEngine')
      const store = useDjStore()
      const engine = useDjAudioEngine()

      // Don't mirror if deck 1 already has this exact track loaded
      const deck1 = store.getDeck(1)
      if (deck1.track?.id === src.id) return

      // Initialise the audio engine if it hasn't been yet (requires a prior
      // user-gesture — if init fails we silently skip mirroring).
      if (!engine.isInitialized.value) {
        try { await engine.init() } catch { return }
      }

      // Build a MusicFile-like object from the player source
      const mirrorTrack = {
        id: src.id,
        title: src.title,
        artist: src.artist ?? null,
        bpm: src.bpm ?? null,
        initial_key: src.initial_key ?? null,
        duration: src.duration ?? null,
        file_path: '',
        created_at: '',
        updated_at: '',
      }

      // Load into deck 1 but keep it paused (cued)
      await engine.loadTrackToDeck(1, mirrorTrack as any)
      // The engine auto-plays on sync — force pause so deck 1 just mirrors
      engine.pause(1)
      store.setPlayState(1, 'cued')
    } catch (e) {
      // Non-critical — if mirroring fails the regular player still works fine
      console.warn('[Mirror] Failed to mirror track to Deck 1:', e)
    }
  }, 0)
}

type PlayerSource =
  | {
      type: 'local'
      id: string
      title: string
      artist?: string | null
      bpm?: number | null
      initial_key?: string | null
      duration?: number | null
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
  bpm?: number | null
  initial_key?: string | null
  duration?: number | null
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
  prefetchBuffer: Map<string, string> // trackId -> blobUrl/objectUrl
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
  prefetchBuffer: new Map(),
})

const prefetchNextTracks = () => {
  if (state.queue.length === 0 || state.currentIndex === -1) return

  // Prefetch up to 2 next tracks
  const nextIndices = [state.currentIndex + 1, state.currentIndex + 2]
  if (state.repeat === 'all') {
    // If repeat all, we might want to wrap around
    nextIndices[0] %= state.queue.length
    nextIndices[1] %= state.queue.length
  }

  nextIndices.forEach(idx => {
    if (idx >= 0 && idx < state.queue.length) {
      const track = state.queue[idx]
      prefetchTrack(track.id)
    }
  })
}

const prefetchTrack = async (id: string) => {
  if (state.prefetchBuffer.has(id)) return

  try {
    const url = buildStreamUrl(id)
    const response = await fetch(url)
    if (!response.ok) return

    const blob = await response.blob()
    const objectUrl = URL.createObjectURL(blob)
    state.prefetchBuffer.set(id, objectUrl)
    console.debug(`[Prefetch] Buffered track ${id}`)

    // Keep buffer small (e.g. 3 tracks)
    if (state.prefetchBuffer.size > 3) {
      // Remove oldest (first inserted)
      const firstKey = state.prefetchBuffer.keys().next().value
      if (firstKey) {
        URL.revokeObjectURL(state.prefetchBuffer.get(firstKey)!)
        state.prefetchBuffer.delete(firstKey)
      }
    }
  } catch (e) {
    console.warn(`[Prefetch] Failed for track ${id}:`, e)
  }
}

const buildStreamUrl = (musicId: string) => {
  const base = `${API_BASE_URL}/music/${musicId}/stream`
  try {
    // Read token from localStorage (same key used by the auth store). We append
    // it as a `token` query param because audio elements cannot set custom
    // Authorization headers.
    const token = localStorage.getItem('music_auth_token')
    if (token) {
      // Preserve existing query params if any
      return `${base}?token=${encodeURIComponent(token)}`
    }
  } catch (e) {
    // localStorage may be unavailable in some contexts; fall back to base URL
    console.warn('Could not read auth token from localStorage for stream URL', e)
  }
  return base
}

const playLocalTrack = (params: { 
  id: string; 
  title: string; 
  artist?: string | null;
  bpm?: number | null;
  initial_key?: string | null;
  duration?: number | null;
}) => {
  // Add to history if we have a current track
  if (state.currentSource?.type === 'local') {
    const currentTrack: TrackInfo = {
      id: state.currentSource.id,
      title: state.currentSource.title,
      artist: state.currentSource.artist,
      bpm: state.currentSource.bpm,
      initial_key: state.currentSource.initial_key,
      duration: state.currentSource.duration,
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
    bpm: params.bpm,
    initial_key: params.initial_key,
    duration: params.duration,
  }

  // Use prefetched URL if available
  if (state.prefetchBuffer.has(params.id)) {
    state.audioUrl = state.prefetchBuffer.get(params.id)!
  } else {
    state.audioUrl = buildStreamUrl(params.id)
  }

  state.updatedAt = Date.now()
  state.isPlaying = true

  // Update queue index if track is in queue
  const idx = state.queue.findIndex(t => t.id === params.id)
  if (idx !== -1) {
    state.currentIndex = idx
  }

  // Mirror to DJ Deck 1 so the user can seamlessly transition to decks
  mirrorToDeck1()

  // Trigger prefetch for next tracks
  prefetchNextTracks()
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
  } else {
    // If we just set the queue but didn't start playing (e.g. startIndex -1), 
    // we should still prefetch the first tracks
    prefetchNextTracks()
  }
}

const addToQueue = (track: TrackInfo) => {
  state.queue.push(track)
  if (state.queue.length === 1 || (state.currentIndex !== -1 && state.queue.length <= state.currentIndex + 3)) {
    prefetchNextTracks()
  }
}

const clearQueue = () => {
  state.queue = []
  state.currentIndex = -1
  // Clear prefetch buffer
  state.prefetchBuffer.forEach(url => URL.revokeObjectURL(url))
  state.prefetchBuffer.clear()
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
        bpm: state.currentSource.bpm,
        initial_key: state.currentSource.initial_key,
        duration: state.currentSource.duration,
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
      bpm: prevTrack.bpm,
      initial_key: prevTrack.initial_key,
      duration: prevTrack.duration,
    }
    state.audioUrl = buildStreamUrl(prevTrack.id)
    state.updatedAt = Date.now()
    state.isPlaying = true
    
    // Update index
    const idx = state.queue.findIndex(t => t.id === prevTrack.id)
    state.currentIndex = idx

    // Mirror to DJ Deck 1
    mirrorToDeck1()
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
        bpm: state.currentSource.bpm,
        initial_key: state.currentSource.initial_key,
        duration: state.currentSource.duration,
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
      bpm: nextTrack.bpm,
      initial_key: nextTrack.initial_key,
      duration: nextTrack.duration,
    }
    state.audioUrl = buildStreamUrl(nextTrack.id)
    state.updatedAt = Date.now()
    state.isPlaying = true

    // Mirror to DJ Deck 1
    mirrorToDeck1()
  } else if (state.repeat === 'all' && state.queue.length > 0) {
    // Loop back to beginning when repeat is enabled
    state.currentIndex = 0
    const nextTrack = state.queue[0]
    state.currentSource = {
      type: 'local',
      id: nextTrack.id,
      title: nextTrack.title,
      artist: nextTrack.artist,
      bpm: nextTrack.bpm,
      initial_key: nextTrack.initial_key,
      duration: nextTrack.duration,
    }
    state.audioUrl = buildStreamUrl(nextTrack.id)
    state.updatedAt = Date.now()
    state.isPlaying = true

    // Mirror to DJ Deck 1
    mirrorToDeck1()
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
    /** Mutable state — only for internal GlobalPlayer sync. Do NOT use elsewhere. */
    _mutableState: state,
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
