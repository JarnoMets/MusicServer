export interface MusicFile {
  id: string
  title: string
  artist?: string | null
  album?: string | null
  genre?: string | null
  guessed_genre?: string | null
  duration?: number | null
  file_path: string
  created_at: string
  updated_at: string
}

export interface PlaylistSummary {
  id: string
  name: string
}

export interface MusicFilters {
  search: string
  genre: string
  sort: 'title' | 'artist' | 'album' | 'created_at' | 'updated_at'
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
    duration: number | undefined
  }
}
