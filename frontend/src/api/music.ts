import axios, { type AxiosRequestConfig } from 'axios'
import { getAPIBaseURL } from '../utils/api'

const API_BASE_URL = getAPIBaseURL()

const api = axios.create({
  baseURL: API_BASE_URL,
})

// admin token helper: set the Authorization header for admin requests
export function setAdminToken(token: string | null) {
  if (token) {
    api.defaults.headers.common['Authorization'] = `Bearer ${token}`
  } else {
    delete api.defaults.headers.common['Authorization']
  }
}

export const musicAPI = {
  // Health check
  getHealth: () => api.get('/health'),
  getDbHealth: () => api.get('/health/db'),

  // Playlists
  getPlaylists: () => api.get('/playlists'),
  getPlaylist: (id: string) => api.get(`/playlists/${id}`),
  createPlaylist: (name: string, description?: string) =>
    api.post('/playlists', { name, description }),
  updatePlaylist: (id: string, data: { name?: string; description?: string }) =>
    api.patch(`/playlists/${id}`, data),
  deletePlaylist: (id: string) => api.delete(`/playlists/${id}`),
  addPlaylistTrack: (playlistId: string, payload: { music_file_id: string; position?: number }) =>
    api.post(`/playlists/${playlistId}/tracks`, payload),
  removePlaylistTrack: (playlistId: string, trackId: string) =>
    api.delete(`/playlists/${playlistId}/tracks/${trackId}`),

  // Music files
  getMusicFiles: (params?: {
    search?: string
    genre?: string
    artist?: string
    sort?: 'title' | 'artist' | 'album' | 'created_at' | 'updated_at'
    order?: 'asc' | 'desc'
    limit?: number
    offset?: number
    unconfirmed_only?: boolean
  }) => api.get('/music', { params }),
  createMusicFile: (data: {
    title: string
    artist?: string
    album?: string
    duration?: number
    file_path: string
  }) => api.post('/music', data),
  getMusicFile: (id: string) => api.get(`/music/${id}`),
  updateMusicFile: (
    id: string,
    data: Partial<{
      title: string
      artist: string
      album: string
      genre: string
      guessed_genre: string
      duration: number
    }>,
  ) => api.patch(`/music/${id}`, data),
  deleteMusicFile: (id: string) => api.delete(`/music/${id}`),
  uploadMusicFiles: (formData: FormData, config?: AxiosRequestConfig) =>
    api.post('/music/upload', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
      ...config,
    }),
  checkDuplicateHash: (hash: string) =>
    api.post('/music/check-duplicate', { hash }),
  confirmGenre: (trackId: string, genre: string) =>
    api.post('/music/confirm-genre', { track_id: trackId, genre }),

  // YouTube Downloads
  startYoutubeDownload: (data: {
    url: string
    output_dir: string
    limit?: number
    max_concurrent?: number
    audio_quality?: string
  }) => api.post('/youtube/download', data),

  getYoutubeProgress: (sessionId: string) => api.get(`/youtube/progress/${sessionId}`),

  getYoutubeProgressStream: (sessionId: string) => {
    return new EventSource(`${API_BASE_URL}/youtube/stream/${sessionId}`)
  },

  cancelYoutubeDownload: (sessionId: string) => api.post(`/youtube/cancel/${sessionId}`),

  getDownloadedVideos: () => api.get('/youtube/downloads'),

  getDownloadStats: () => api.get('/youtube/stats'),

  removeDownloadRecord: (videoId: string) => api.delete(`/youtube/downloads/${videoId}`),

  // YouTube Playlists (saved playlist links)
  listYoutubePlaylists: () => api.get('/youtube/playlists'),
  getYoutubePlaylist: (id: string) => api.get(`/youtube/playlists/${id}`),
  createYoutubePlaylist: (data: { name: string; url: string; description?: string; autoDownload?: boolean }) =>
    api.post('/admin/youtube/playlists', data),
  updateYoutubePlaylist: (id: string, data: { name?: string; url?: string; description?: string; autoDownload?: boolean }) =>
    api.patch(`/admin/youtube/playlists/${id}`, data),
  deleteYoutubePlaylist: (id: string) => api.delete(`/admin/youtube/playlists/${id}`),
  syncYoutubePlaylist: (id: string) => api.post(`/admin/youtube/playlists/${id}/sync`),

  // Artists
  getArtists: () => api.get('/artists'),
  getArtistMusic: (artist: string) => api.get(`/artists/${encodeURIComponent(artist)}`),
  setArtistGenre: (artist: string, genre: string) =>
    api.put(`/artists/${encodeURIComponent(artist)}/genre`, { genre }),
  renameArtist: (oldName: string, newName: string) =>
    api.put(`/admin/artists/${encodeURIComponent(oldName)}/rename`, { new_name: newName }),
  reprocessArtists: () => api.post('/admin/artists/reprocess'),

  // Genre Detection & Cache
  detectGenre: (artistName: string) =>
    api.post('/genres/detect', { artist_name: artistName }),
  getGenreCache: () => api.get('/genres/cache'),
  clearGenreCache: () => api.delete('/genres/cache'),
  // Canonical genres & aliases (admin)
  listGenres: () => api.get('/genres'),
  createGenre: (name: string, description?: string) => api.post('/admin/genres', { name, description }),
  addGenreAlias: (alias: string, genre_id: string) => api.post('/admin/genres/aliases', { alias, genre_id }),
  addGenreAliasBackfill: (alias: string, genre_id: string) => api.post('/admin/genres/aliases/backfill', { alias, genre_id }),
  listUnmappedGenres: () => api.get('/genres/unmapped'),
  suggestGenres: (raw: string) => api.get(`/genres/suggest/${encodeURIComponent(raw)}`),
  previewBackfill: (alias: string) => api.get(`/genres/aliases/preview/${encodeURIComponent(alias)}`),
  startBackfill: (alias: string, genre_id: string) => api.post('/admin/genres/aliases/backfill/start', { alias, genre_id }),
  getBackfillStream: (sessionId: string) => new EventSource(`${API_BASE_URL}/genres/aliases/backfill/${sessionId}/stream`),
  // Reprocess missing artists (background)
  startReprocessMissing: () => api.post('/genres/reprocess-missing'),
  getReprocessStream: (sessionId: string) => new EventSource(`${API_BASE_URL}/genres/reprocess/${sessionId}/stream`),
  // Sync music folder
  syncMusicFolder: (folder?: string) => api.post('/music/sync', folder ? { folder } : {}),
  cancelSync: (sessionId: string) => api.post(`/music/sync/cancel/${sessionId}`),

  // Internet Streams
  listStreams: () => api.get('/streams'),
  createStream: (payload: { name: string; url: string; genre?: string; description?: string }) =>
    api.post('/streams', payload),
  updateStream: (id: string, payload: Partial<{ name: string; url: string; genre?: string; description?: string }>) =>
    api.patch(`/streams/${id}`, payload),
  deleteStream: (id: string) => api.delete(`/streams/${id}`),

  // Auto-download configuration (admin)
  getAutoDownloadConfig: () => api.get('/admin/auto-download/config'),
  updateAutoDownloadConfig: (data: {
    enabled?: boolean
    check_interval_minutes?: number
    max_concurrent_downloads?: number
    delay_between_downloads_seconds?: number
    allowed_start_hour?: number | null
    allowed_end_hour?: number | null
  }) => api.put('/admin/auto-download/config', data),
  getAutoDownloadStatus: () => api.get('/admin/auto-download/status'),
  triggerAutoDownload: () => api.post('/admin/auto-download/trigger'),
  stopAutoDownload: () => api.post('/admin/auto-download/stop'),

  // Bulk operations (admin)
  bulkRenameByRegex: (data: {
    field: 'title' | 'artist' | 'album'
    pattern: string
    replacement: string
  }) => api.post('/admin/music/bulk-rename', data),
  bulkAddToPlaylistByRegex: (data: {
    playlist_id: string
    field: 'title' | 'artist' | 'album'
    pattern: string
  }) => api.post('/admin/music/bulk-add-to-playlist', data),
}

export default api
