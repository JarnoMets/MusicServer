/**
 * Composable for music data fetching and state management
 */
import { computed, onMounted, onUnmounted, reactive, ref, watch } from 'vue'
import { musicAPI } from '../api/music'
import { useMusicStore } from '../stores/musicStore'
import type { MusicFilters, PaginationState } from '../types'

export interface MusicStats {
  total_count: number
  total_duration_ms: number
}

export const useMusicData = () => {
  const store = useMusicStore()
  
  // Local reactive copy of filters, but we will sync it with store if needed
  const filters = reactive<MusicFilters>({
    search: '',
    genre: '',
    sort: 'title',
    order: 'asc',
  })

  // Computed: Source of truth for music files for the UI
  // If the store is using its full cache, use its filtered list and apply local pagination.
  // Otherwise, use the store.musicFiles array (which is populated via paginated API)
  const musicFiles = computed(() => {
    if (!store.allCachedLoaded) return store.musicFiles

    const all = store.filteredAndSortedFiles
    const start = (pagination.page - 1) * pagination.limit
    const end = start + pagination.limit
    
    // Update stats locally based on full filtered list
    stats.value = {
      total_count: all.length,
      total_duration_ms: all.reduce((sum, f) => sum + (f.duration || 0), 0)
    }
    
    return all.slice(start, end)
  })
  
  const playlists = computed(() => store.playlists)
  const genres = ref<string[]>([])
  const canonicalGenres = ref<{ id: string; name: string; description?: string }[]>([])
  const loading = ref(false)
  
  const stats = ref<MusicStats>({ total_count: 0, total_duration_ms: 0 })

  const pagination = reactive<PaginationState>({
    page: 1,
    limit: 50,
  })

  const hasMore = computed(() => {
    if (store.allCachedLoaded) {
      return (pagination.page * pagination.limit) < stats.value.total_count
    }
    return musicFiles.value.length === pagination.limit
  })

  const playlistMenuOpen = ref<string | null>(null)

  // AbortController for cancellable fetches (prevents race conditions when switching tabs fast)
  let currentFetchController: AbortController | null = null

  const fetchGenres = async () => {
    try {
      const response = await musicAPI.listGenres()
      // Filter out genres with 0 tracks just to be safe in the UI dropdown
      genres.value = response.data
        .filter((g: { track_count: number }) => g.track_count > 0)
        .map((g: { name: string }) => g.name)
    } catch (error) {
      console.warn('Failed to load genres', error)
    }
  }

  const fetchCanonicalGenres = async () => {
    try {
      // Use the canonical list for editing, which contains ALL genres even those with 0 tracks
      const response = await musicAPI.listCanonicalGenres()
      canonicalGenres.value = response.data
    } catch (error) {
      console.warn('Failed to load canonical genres', error)
    }
  }

  const fetchPlaylists = async () => {
    await store.refreshPlaylists()
  }

  const fetchStats = async () => {
    try {
      const response = await musicAPI.getMusicStats({
        search: filters.search || undefined,
        genre: filters.genre === 'unconfirmed' ? undefined : (filters.genre || undefined),
        unconfirmed_only: filters.genre === 'unconfirmed' ? true : undefined,
      })
      stats.value = response.data
    } catch (error) {
      console.warn('Failed to load stats', error)
    }
  }

  const fetchMusic = async () => {
    // If the store has the full cache loaded, just update local search/filter filters
    // and let the store's computed properties handle the UI update.
    if (store.allCachedLoaded) {
      store.filters.search = filters.search
      store.filters.genre = filters.genre
      store.filters.sort = filters.sort
      store.filters.order = filters.order
      return
    }

    // Cancel any previous in-flight fetch to avoid races
    if (currentFetchController) {
      try { currentFetchController.abort() } catch { /* ignore */ }
      currentFetchController = null
    }

    const controller = new AbortController()
    currentFetchController = controller

    loading.value = true
    playlistMenuOpen.value = null
    try {
      const [musicResponse] = await Promise.all([
        musicAPI.getMusicFiles({
          search: filters.search || undefined,
          genre: filters.genre === 'unconfirmed' ? undefined : (filters.genre || undefined),
          sort: filters.sort,
          order: filters.order,
          limit: pagination.limit,
          offset: (pagination.page - 1) * pagination.limit,
          unconfirmed_only: filters.genre === 'unconfirmed' ? true : undefined,
        }, { signal: controller.signal }),
        fetchStats(),
      ])

      // If this fetch was aborted, don't apply the result
      if (controller.signal.aborted) return

      // assign into the store-backed array
      // Pinia unwraps store properties, so we can assign directly
      // and avoid double-wrapping refs
      store.musicFiles = musicResponse.data
    } catch (err: unknown) {
      // Ignore abort errors - they are expected when cancelling prior requests
      const maybe = err as { name?: string; message?: string }
      if (maybe && (maybe.name === 'CanceledError' || maybe.message === 'canceled' || maybe.message === 'AbortError')) {
        // no-op
      } else {
        console.error('Error fetching music files:', err)
      }
    } finally {
      // clear controller only if it's the one we created
      if (currentFetchController === controller) currentFetchController = null
      loading.value = false
    }
  }

  // The store handles trying to load a cached "all tracks" list and subscribes to updates.
  // If the store didn't provide a cached all list, this composable will fall back to paginated fetch.

  const refreshAll = () => {
    fetchMusic()
    fetchPlaylists()
    fetchGenres()
    fetchCanonicalGenres()
  }

  const resetFilters = () => {
    filters.search = ''
    filters.genre = ''
    filters.sort = 'title'
    filters.order = 'asc'
    pagination.page = 1
    fetchMusic()
  }

  const debouncedFetch = (() => {
    let handle: number | undefined
    return () => {
      window.clearTimeout(handle)
      handle = window.setTimeout(() => {
        pagination.page = 1
        fetchMusic()
      }, 350)
    }
  })()

  // Watchers
  watch(() => filters.search, () => debouncedFetch())
  watch(
    () => [filters.genre, filters.sort, filters.order],
    () => {
      pagination.page = 1
      fetchMusic()
    },
  )
  watch(() => pagination.limit, () => {
    pagination.page = 1
    fetchMusic()
  })

  const nextPage = () => {
    if (hasMore.value) {
      pagination.page += 1
      if (!store.allCachedLoaded) fetchMusic()
    }
  }

  const prevPage = () => {
    if (pagination.page > 1) {
      pagination.page -= 1
      if (!store.allCachedLoaded) fetchMusic()
    }
  }

  // Initialize on mount
  onMounted(async () => {
    // Let the store try to initialize cached all-tracks and subscribe to updates.
    // store.init() is idempotent - safe to call from multiple components.
    await store.init()
    
    // Sync initial state if it was loaded from cache
    if (store.allCachedLoaded) {
      store.filters.search = filters.search
      store.filters.genre = filters.genre
      store.filters.sort = filters.sort
      store.filters.order = filters.order
    } else {
      await fetchMusic()
    }

    fetchGenres()
    fetchCanonicalGenres()
  })

  onUnmounted(() => {
    // Abort any in-flight fetch when component using this composable unmounts
    if (currentFetchController) {
      try { currentFetchController.abort() } catch { /* ignore */ }
      currentFetchController = null
    }
    // Do NOT close the global store EventSource here - the store manages its own lifecycle.
  })

  return {
    // State
    musicFiles,
    playlists,
    genres,
    canonicalGenres,
    loading,
    hasMore,
    playlistMenuOpen,
    filters,
    pagination,
    stats,
    // Methods
    fetchMusic,
    fetchPlaylists,
    fetchGenres,
    fetchCanonicalGenres,
    refreshAll,
    resetFilters,
    nextPage,
    prevPage,
  }
}
