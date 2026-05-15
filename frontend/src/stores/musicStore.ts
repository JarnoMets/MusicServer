import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { musicAPI } from '../api/music'
import { buildSSEURL } from '../utils/api'
import type { MusicFile, ArtistSummary, PlaylistSummary } from '../types'

export const useMusicStore = defineStore('music', () => {
  const musicFiles = ref<MusicFile[]>([])
  const allCachedLoaded = ref(false)
  const loadingAll = ref(false)
  const incrementalStatus = ref<{ loaded: number; total: number | null }>({ loaded: 0, total: null })

  // Optimization: use a Map for O(1) lookups by id
  const musicMap = ref<Map<string, MusicFile>>(new Map())

  // Metadata summary
  const artists = ref<ArtistSummary[]>([])
  const loadingArtists = ref(false)

  // Playlist state
  const playlists = ref<PlaylistSummary[]>([])
  const loadingPlaylists = ref(false)

  // State for filtering/sorting if handled locally
  const filters = ref({
    search: '',
    genre: '',
    sort: 'title' as string,
    order: 'asc' as 'asc' | 'desc'
  })

  let es: EventSource | null = null
  let reconnectTimeout: ReturnType<typeof setTimeout> | null = null
  let reconnectAttempts = 0
  let initialized = false
  const MAX_RECONNECT_ATTEMPTS = 5

  // Computed: Filtered and Sorted list for UI consumption
  // This is only used if allCachedLoaded is true
  const filteredAndSortedFiles = computed(() => {
    if (!allCachedLoaded.value) return musicFiles.value

    let result = [...musicFiles.value]

    // Apply filters locally if all data is present
    if (filters.value.search) {
      const s = filters.value.search.toLowerCase()
      result = result.filter(f => 
        f.title.toLowerCase().includes(s) || 
        (f.artist && f.artist.toLowerCase().includes(s)) ||
        (f.album && f.album.toLowerCase().includes(s))
      )
    }

    if (filters.value.genre) {
      if (filters.value.genre === 'unconfirmed') {
        result = result.filter(f => f.genre_source === 'auto')
      } else {
        result = result.filter(f => f.genre_name === filters.value.genre)
      }
    }

    // Apply sorting locally
    const { sort, order } = filters.value
    const modifier = order === 'asc' ? 1 : -1

    result.sort((a, b) => {
      if (sort === 'release_date') {
        const timeA = a.release_date ? new Date(a.release_date).getTime() : 0
        const timeB = b.release_date ? new Date(b.release_date).getTime() : 0
        return modifier * (timeA - timeB)
      }
      
      if (sort === 'duration' || sort === 'bpm') {
        const valA = (a[sort] as number) || 0
        const valB = (b[sort] as number) || 0
        return modifier * (valA - valB)
      }

      if (sort === 'created_at' || sort === 'updated_at') {
        const timeA = new Date(a[sort]).getTime()
        const timeB = new Date(b[sort]).getTime()
        return modifier * (timeA - timeB)
      }

      const valA = String(((a as Record<string, unknown>)[sort] ?? '')).toLowerCase()
      const valB = String(((b as Record<string, unknown>)[sort] ?? '')).toLowerCase()
      
      return modifier * valA.localeCompare(valB)
    })

    return result
  })

  const sortedPlaylists = computed(() => {
    const result = [...playlists.value]
    // Default to newest updated first
    result.sort((a, b) => {
      const timeA = a.updated_at ? new Date(a.updated_at).getTime() : 0
      const timeB = b.updated_at ? new Date(b.updated_at).getTime() : 0
      return timeB - timeA
    })
    return result
  })

  const updateMusicFilesFromArray = (files: MusicFile[]) => {
    // Optimization: Bulk assignment
    musicFiles.value = files
    const newMap = new Map<string, MusicFile>()
    files.forEach(f => newMap.set(f.id, f))
    musicMap.value = newMap
  }

  const upsertFile = (file: MusicFile) => {
    // Update map first
    musicMap.value.set(file.id, file)

    const idx = musicFiles.value.findIndex((m) => m.id === file.id)
    if (idx >= 0) {
      musicFiles.value.splice(idx, 1, file)
    } else {
      musicFiles.value.unshift(file)
    }
  }

  const removeById = (id: string) => {
    musicMap.value.delete(id)
    const idx = musicFiles.value.findIndex((m) => m.id === id)
    if (idx >= 0) musicFiles.value.splice(idx, 1)
  }

  const refreshPlaylists = async () => {
    loadingPlaylists.value = true
    try {
      const resp = await musicAPI.getPlaylists()
      if (Array.isArray(resp.data)) {
        playlists.value = resp.data as PlaylistSummary[]
      }
    } catch (e) {
      console.warn('Failed to refresh playlists', e)
    } finally {
      loadingPlaylists.value = false
    }
  }

  const refreshAllTracks = async () => {
    if (loadingAll.value) return
    loadingAll.value = true
    
    // Reset incremental status
    incrementalStatus.value = { loaded: 0, total: null }
    
    try {
      // Step 1: Fetch a small initial batch quickly to make the UI feel responsive
      const quickResp = await musicAPI.getMusicFiles({ limit: 100, sort: 'created_at', order: 'desc' })
      if (Array.isArray(quickResp.data)) {
        updateMusicFilesFromArray(quickResp.data as MusicFile[])
        incrementalStatus.value.loaded = quickResp.data.length
      }
      
      // Step 2: Fetch stats to know the total count
      const statsResp = await musicAPI.getMusicStats()
      incrementalStatus.value.total = statsResp.data.total_count
      
      // Step 3: Fetch the full cached list in the background
      const resp = await musicAPI.getAllCachedTracks()
      if (Array.isArray(resp.data)) {
        updateMusicFilesFromArray(resp.data as MusicFile[])
        allCachedLoaded.value = resp.data.length > 0
        incrementalStatus.value.loaded = resp.data.length
      }
    } catch (e) {
      console.warn('Failed to refresh all cached tracks', e)
    } finally {
      loadingAll.value = false
    }
  }

  const refreshArtists = async () => {
    loadingArtists.value = true
    try {
      // Prefer server-side cached artists summary
      const resp = await musicAPI.getArtistsCached()
      if (Array.isArray(resp.data) && resp.data.length > 0) {
        artists.value = resp.data as ArtistSummary[]
      } else {
        const resp2 = await musicAPI.getArtists()
        if (Array.isArray(resp2.data)) artists.value = resp2.data as ArtistSummary[]
      }
    } catch (e) {
      console.warn('Failed to refresh artists summary', e)
    } finally {
      loadingArtists.value = false
    }
  }

  const setupEventSource = () => {
    // Clear any existing connection/timeout
    closeEventSource()
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout)
      reconnectTimeout = null
    }

    try {
      const url = buildSSEURL('/updates/stream')
      es = new EventSource(url)

      es.onopen = () => {
        console.log('Updates EventSource connected')
        reconnectAttempts = 0
      }

      es.addEventListener('message', (ev: MessageEvent) => {
        try {
          const data = JSON.parse(ev.data)
          const t = data.type || ''
          const payload = data.payload

          if (t === 'music_created' || t === 'music_updated') {
            if (payload) upsertFile(payload as MusicFile)
          } else if (t === 'music_deleted') {
            if (payload && payload.id) removeById(payload.id)
          } else if (t === 'music_bulk_updated') {
            // bulk operations: refresh full cache to keep consistent
            refreshAllTracks()
          } else if (t === 'playlist_created' || t === 'playlist_updated' || t === 'playlist_deleted' || t === 'playlist_items_updated') {
            // playlist status changed - refresh playlists summary
            refreshPlaylists()
          } else if (t === 'artist_genre_set' || t === 'artist_renamed') {
            // artist metadata changed - refresh artists summary and possibly full list
            try {
              // Update artists list incrementally if payload provides details
              if (t === 'artist_genre_set' && payload && payload.artist) {
                const aidx = artists.value.findIndex((a) => a.name === payload.artist)
                if (aidx >= 0) {
                  artists.value[aidx].genre = payload.genre
                } else {
                  // unknown artist - refresh full artists
                  refreshArtists()
                }
              } else if (t === 'artist_renamed' && payload && payload.old_name) {
                const aidx = artists.value.findIndex((a) => a.name === payload.old_name)
                if (aidx >= 0) {
                  // update name, and if a new_name collides, remove the old entry
                  const existsIdx = artists.value.findIndex((a) => a.name === payload.new_name)
                  if (existsIdx >= 0) {
                    // merge counts
                    artists.value[existsIdx].song_count += artists.value[aidx].song_count
                    artists.value.splice(aidx, 1)
                  } else {
                    artists.value[aidx].name = payload.new_name
                  }
                } else {
                  // not found - refresh
                  refreshArtists()
                }
              } else {
                // generic fallback
                refreshArtists()
              }
            } catch (e) {
              console.warn('Failed to incrementally apply artist update, falling back to refresh', e)
              refreshArtists()
            }
            // If full cached list is loaded, refresh it to update artist fields
            if (allCachedLoaded.value) refreshAllTracks()
          }
        } catch (e) {
          console.warn('Invalid update event received in store', e)
        }
      })

      es.addEventListener('error', (err) => {
        console.warn('Updates EventSource error (store)', err)
        
        // If it's closed (readyState === 2), try to reconnect
        if (es?.readyState === 2) {
          handleReconnect()
        }
      })
    } catch (e) {
      console.warn('Failed to open updates EventSource (store)', e)
      handleReconnect()
    }
  }

  const handleReconnect = () => {
    if (reconnectAttempts >= MAX_RECONNECT_ATTEMPTS) {
      console.error('Max EventSource reconnect attempts reached')
      return
    }

    const delay = Math.min(1000 * Math.pow(2, reconnectAttempts), 30000)
    reconnectAttempts++
    
    console.log(`Reconnecting to EventSource in ${delay}ms (attempt ${reconnectAttempts})`)
    
    if (reconnectTimeout) clearTimeout(reconnectTimeout)
    reconnectTimeout = setTimeout(() => {
      setupEventSource()
    }, delay)
  }

  const closeEventSource = () => {
    if (es) {
      es.close()
      es = null
    }
    if (reconnectTimeout) {
      clearTimeout(reconnectTimeout)
      reconnectTimeout = null
    }
  }

  const init = async () => {
    // Guard against duplicate initialization from multiple components
    if (initialized) return
    initialized = true

    // Try to load cached all-tracks; if not available, leave musicFiles empty
    if (!allCachedLoaded.value) {
      await refreshAllTracks()
    }
    
    // Always load playlists on init
    await refreshPlaylists()
    
    // Always start listening for updates so incremental patches apply
    setupEventSource()
  }

  return {
    // state
    musicFiles,
    filteredAndSortedFiles,
    allCachedLoaded,
    loadingAll,
    artists,
    loadingArtists,
    playlists,
    loadingPlaylists,
    filters,
    incrementalStatus,
    // actions
    init,
    refreshAllTracks,
    refreshArtists,
    refreshPlaylists,
    closeEventSource,
    // computed
    sortedPlaylists,
    // helpers
    upsertFile,
    removeById,
  }
})
