import api from '../api/music'
import { isDemoHost } from '../composables/useDemoReadonly'
import { isDemoHydrated, setDemoMeta } from './db'
import { saveMusicDemoState, type MusicDemoState } from './handlers'

export type HydrateProgress = { phase: string; done: boolean; error?: string }

let hydratePromise: Promise<void> | null = null

export async function ensureMusicDemoHydrated(
  onProgress?: (p: HydrateProgress) => void
): Promise<void> {
  if (!isDemoHost()) return
  if (await isDemoHydrated()) {
    onProgress?.({ phase: 'ready', done: true })
    return
  }
  if (hydratePromise) return hydratePromise

  hydratePromise = (async () => {
    try {
      onProgress?.({ phase: 'Loading library…', done: false })
      let tracks: Record<string, unknown>[] = []
      try {
        const cached = await api.get('/music/all-cached')
        tracks = Array.isArray(cached.data) ? cached.data : []
      } catch {
        const page = await api.get('/music', { params: { limit: 500, offset: 0 } })
        tracks = Array.isArray(page.data) ? page.data : []
      }

      onProgress?.({ phase: 'Loading playlists…', done: false })
      const playlistsRes = await api.get('/playlists')
      const playlists = Array.isArray(playlistsRes.data) ? playlistsRes.data : []
      const playlistDetails: MusicDemoState['playlistDetails'] = {}
      for (const pl of playlists) {
        const id = String((pl as { id: string }).id)
        try {
          const detail = await api.get(`/playlists/${id}`)
          playlistDetails[id] = detail.data as Record<string, unknown>
        } catch {
          playlistDetails[id] = { ...(pl as object), tracks: [], items: [] }
        }
      }

      let artists: unknown = []
      let genres: unknown = []
      let stats: unknown = { total: tracks.length }
      let me: unknown = null
      try {
        artists = (await api.get('/artists')).data
      } catch {
        /* optional */
      }
      try {
        genres = (await api.get('/genres')).data
      } catch {
        /* optional */
      }
      try {
        stats = (await api.get('/music/stats')).data
      } catch {
        /* optional */
      }
      try {
        me = (await api.get('/me')).data
      } catch {
        /* optional */
      }

      await saveMusicDemoState({
        tracks,
        playlists: playlists as Record<string, unknown>[],
        playlistDetails,
        artists,
        genres,
        stats,
        me
      })
      await setDemoMeta({
        hydrated: true,
        hydratedAt: new Date().toISOString(),
        version: 1
      })
      onProgress?.({ phase: 'ready', done: true })
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : 'Hydrate failed'
      onProgress?.({ phase: msg, done: false, error: msg })
      throw e
    } finally {
      hydratePromise = null
    }
  })()

  return hydratePromise
}
