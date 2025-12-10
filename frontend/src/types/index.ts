export interface MusicFile {
  id: string
  title: string
  artist?: string
  album?: string
  duration?: number
  file_path: string
  created_at: string
  updated_at: string
}

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
  track_count: number
  created_at: string
  updated_at: string
}
