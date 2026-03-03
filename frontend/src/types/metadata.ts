export interface MetadataSuggestion {
  music_file_id: string
  release_date?: string
  album?: string
  genre?: string
  confidence: number
  created_at: string
  updated_at: string
}

export interface ReleaseDateLookupResponse {
  release_date?: string
  album?: string
  genre?: string
  confidence: number
  error?: string
}
