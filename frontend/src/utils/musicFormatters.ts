/**
 * Utility functions for music formatting
 */

/**
 * Format duration from milliseconds to human readable string
 * @param ms Duration in milliseconds (from backend)
 */
export const formatDuration = (ms?: number | null): string => {
  if (!ms) return '&mdash;'
  // Convert milliseconds to seconds
  const totalSeconds = Math.floor(ms / 1000)
  const hrs = Math.floor(totalSeconds / 3600)
  const mins = Math.floor((totalSeconds % 3600) / 60)
  const secs = Math.floor(totalSeconds % 60)
  if (hrs > 0) {
    return `${hrs}:${String(mins).padStart(2, '0')}:${String(secs).padStart(2, '0')}`
  }
  return `${mins}:${String(secs).padStart(2, '0')}`
}

export const formatDate = (value?: string): string => {
  if (!value) return '&mdash;'
  return new Date(value).toLocaleDateString()
}

/**
 * Format a date string as a relative time ("today", "yesterday", "3 days ago")
 * with a fallback to locale date for older dates.
 */
export const formatRelativeDate = (dateString: string): string => {
  const date = new Date(dateString)
  const now = new Date()
  const diffDays = Math.floor((now.getTime() - date.getTime()) / (1000 * 60 * 60 * 24))

  if (diffDays === 0) return 'today'
  if (diffDays === 1) return 'yesterday'
  if (diffDays < 7) return `${diffDays} days ago`
  return date.toLocaleDateString()
}

/**
 * Format a byte count as a human-readable string (e.g. "1.5 MB").
 */
export const formatBytes = (bytes: number): string => {
  if (bytes === 0) return '0 Bytes'
  const k = 1024
  const sizes = ['Bytes', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  return Math.round((bytes / Math.pow(k, i)) * 100) / 100 + ' ' + sizes[i]
}

/**
 * Truncate a string to a maximum length, appending "..." if truncated.
 */
export const truncate = (text: string, length: number): string => {
  if (text.length <= length) return text
  return text.slice(0, length) + '...'
}

export const calculateTotalDuration = (musicFiles: { duration?: number | null }[]): string => {
  const totalMs = musicFiles.reduce((sum, track) => sum + (track.duration || 0), 0)
  return `Total duration: ${formatDuration(totalMs)}`
}
