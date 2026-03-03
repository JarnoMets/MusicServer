// ─── Music ───────────────────────────────────────────────────────

export interface MusicFile {
  id: string
  title: string
  artist?: string | null
  album?: string | null
  genre?: string | null
  guessed_genre?: string | null
  release_date?: string | null
  duration?: number | null
  file_path: string
  bpm?: number | null
  initial_key?: string | null
  beat_grid_offset?: number | null
  beat_map?: number[] | null
  metadata_analyzed?: boolean
  key_confirmed?: boolean
  created_at: string
  updated_at: string
}

// ─── Playlists ───────────────────────────────────────────────────

export interface Playlist {
  id: string
  name: string
  description?: string
  items: MusicFile[]
  created_at: string
  updated_at: string
}

export interface PlaylistSummary {
  id: string
  name: string
  description?: string
  track_count?: number
  created_at?: string
  updated_at?: string
}

// ─── Genres ──────────────────────────────────────────────────────

export interface Genre {
  id?: string
  name: string
  description?: string
  track_count?: number
}

// ─── Artists ─────────────────────────────────────────────────────

export interface ArtistSummary {
  name: string
  genre?: string | null
  song_count: number
}

// ─── Music Filters & Pagination ──────────────────────────────────

export interface MusicFilters {
  search: string
  genre: string
  sort: 'title' | 'artist' | 'album' | 'genre' | 'created_at' | 'updated_at' | 'release_date' | 'duration' | 'bpm'
  order: 'asc' | 'desc'
  unconfirmedOnly?: boolean
}

export interface PaginationState {
  page: number
  limit: number
}

export interface EditState {
  open: boolean
  saving: boolean
  trackId: string | null
  form: {
    title: string
    artist: string
    album: string
    genre: string
    release_date: string
    bpm: number | null
    initial_key: string
    beat_grid_offset: number | null
  }
}
