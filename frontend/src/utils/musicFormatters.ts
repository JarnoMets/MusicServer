/**
 * Utility functions for music formatting
 */

/**
 * Format duration from milliseconds to human readable string
 * @param ms Duration in milliseconds (from backend)
 */
export const formatDuration = (ms?: number | null): string => {
  if (!ms) return '—'
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
  if (!value) return '—'
  return new Date(value).toLocaleDateString()
}

export const calculateTotalDuration = (musicFiles: { duration?: number | null }[]): string => {
  const totalMs = musicFiles.reduce((sum, track) => sum + (track.duration || 0), 0)
  return `Total duration: ${formatDuration(totalMs)}`
}
