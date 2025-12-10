/**
 * Composable for music data fetching and state management
 */
import { computed, onMounted, reactive, ref, watch } from 'vue'
import { musicAPI } from '../api/music'
import type { MusicFile, PlaylistSummary, MusicFilters, PaginationState } from '../types/MusicTab'

export const useMusicData = () => {
  const musicFiles = ref<MusicFile[]>([])
  const playlists = ref<PlaylistSummary[]>([])
  const genres = ref<string[]>([])
  const loading = ref(false)
  const hasMore = computed(() => musicFiles.value.length === pagination.limit)
  const playlistMenuOpen = ref<string | null>(null)

  const filters = reactive<MusicFilters>({
    search: '',
    genre: '',
    sort: 'title',
    order: 'asc',
  })

  const pagination = reactive<PaginationState>({
    page: 1,
    limit: 50,
  })

  const fetchGenres = async () => {
    try {
      const response = await musicAPI.listGenres()
      genres.value = response.data.map((g: any) => g.name)
    } catch (error) {
      console.warn('Failed to load genres', error)
    }
  }

  const fetchPlaylists = async () => {
    try {
      const response = await musicAPI.getPlaylists()
      playlists.value = response.data
    } catch (error) {
      console.warn('Failed to load playlists', error)
    }
  }

  const fetchMusic = async () => {
    loading.value = true
    playlistMenuOpen.value = null
    try {
      const response = await musicAPI.getMusicFiles({
        search: filters.search || undefined,
        genre: filters.genre === 'unconfirmed' ? undefined : (filters.genre || undefined),
        sort: filters.sort,
        order: filters.order,
        limit: pagination.limit,
        offset: (pagination.page - 1) * pagination.limit,
        unconfirmed_only: filters.genre === 'unconfirmed' ? true : undefined,
      })
      musicFiles.value = response.data
    } catch (error) {
      console.error('Error fetching music files:', error)
    } finally {
      loading.value = false
    }
  }

  const refreshAll = () => {
    fetchMusic()
    fetchPlaylists()
    fetchGenres()
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
      fetchMusic()
    }
  }

  const prevPage = () => {
    if (pagination.page > 1) {
      pagination.page -= 1
      fetchMusic()
    }
  }

  // Initialize on mount
  onMounted(() => {
    fetchMusic()
    fetchPlaylists()
    fetchGenres()
  })

  return {
    // State
    musicFiles,
    playlists,
    genres,
    loading,
    hasMore,
    playlistMenuOpen,
    filters,
    pagination,
    // Methods
    fetchMusic,
    fetchPlaylists,
    fetchGenres,
    refreshAll,
    resetFilters,
    nextPage,
    prevPage,
  }
}
