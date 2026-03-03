/**
 * Composable for track actions (play, delete, playlist operations)
 */
import { musicAPI } from '../api/music'
import { usePlayer } from './usePlayer'
import { useToast } from './useToast'
import { useConfirm } from './useConfirm'
import type { MusicFile } from '../types'

export const useTrackActions = (onActionComplete: () => void) => {
  const { playLocalTrack, setQueue } = usePlayer()
  const { success, error } = useToast()
  const { confirm } = useConfirm()

  const playTrack = (track: MusicFile) => {
    playLocalTrack({ 
      id: track.id, 
      title: track.title, 
      artist: track.artist,
      bpm: track.bpm,
      initial_key: track.initial_key,
      duration: track.duration
    })
  }

  const playAll = (tracks: MusicFile[], startIndex = 0) => {
    const queue = tracks.map(t => ({ 
      id: t.id, 
      title: t.title, 
      artist: t.artist,
      bpm: t.bpm,
      initial_key: t.initial_key,
      duration: t.duration
    }))
    setQueue(queue, startIndex)
  }

  const deleteTrack = async (track: MusicFile) => {
    const confirmed = await confirm({
      title: 'Delete Track',
      message: `Delete "${track.title}"? This cannot be undone.`,
      confirmText: 'Delete',
      cancelText: 'Cancel',
      variant: 'danger'
    })
    if (!confirmed) return
    
    try {
      await musicAPI.deleteMusicFile(track.id)
      success('Track deleted', `"${track.title}" has been removed from your library`)
      onActionComplete()
    } catch (err: unknown) {
      console.error('Error deleting track:', err)
      const e = err as { response?: { data?: { error?: string } }; message?: string }
      error('Failed to delete track', e.response?.data?.error || e.message || 'Unknown error')
    }
  }

  const addTrackToPlaylist = async (trackId: string, playlistId: string) => {
    try {
      await musicAPI.addPlaylistTrack(playlistId, { music_file_id: trackId })
      success('Added to playlist', 'Track has been added to the playlist')
    } catch (err: unknown) {
      console.error('Failed to add track to playlist', err)
      const e = err as { response?: { data?: { error?: string } }; message?: string }
      error('Failed to add track', e.response?.data?.error || e.message || 'Unknown error')
    }
  }

  const removeTrackFromPlaylist = async (trackId: string, playlistId: string) => {
    try {
      await musicAPI.removePlaylistTrack(playlistId, trackId)
      success('Removed from playlist', 'Track has been removed from the playlist')
    } catch (err: unknown) {
      console.error('Failed to remove track from playlist', err)
      const e = err as { response?: { data?: { error?: string } }; message?: string }
      error('Failed to remove track', e.response?.data?.error || e.message || 'Unknown error')
    }
  }

  const setTrackGenre = async (track: MusicFile, genre: string) => {
    try {
      await musicAPI.updateMusicFile(track.id, { genre })
      success('Genre updated', `"${track.title}" is now tagged as ${genre}`)
      onActionComplete()
    } catch (err: unknown) {
      console.error('Failed to update genre', err)
      const e = err as { response?: { data?: { error?: string } }; message?: string }
      error('Update failed', e.response?.data?.error || e.message || 'Unknown error')
    }
  }

  return {
    playTrack,
    playAll,
    deleteTrack,
    addTrackToPlaylist,
    removeTrackFromPlaylist,
    setTrackGenre,
  }
}
