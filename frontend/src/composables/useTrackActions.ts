/**
 * Composable for track actions (play, delete, playlist operations)
 */
import { musicAPI } from '../api/music'
import { usePlayer } from './usePlayer'
import { useToast } from './useToast'
import { useConfirm } from './useConfirm'
import type { MusicFile } from '../types/MusicTab'

export const useTrackActions = (onActionComplete: () => void) => {
  const { playLocalTrack } = usePlayer()
  const { success, error } = useToast()
  const { confirm } = useConfirm()

  const playTrack = (track: MusicFile) => {
    playLocalTrack({ id: track.id, title: track.title, artist: track.artist })
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
    } catch (err: any) {
      console.error('Error deleting track:', err)
      error('Failed to delete track', err?.response?.data?.error || err?.message)
    }
  }

  const addTrackToPlaylist = async (trackId: string, playlistId: string) => {
    try {
      await musicAPI.addPlaylistTrack(playlistId, { music_file_id: trackId })
      success('Added to playlist', 'Track has been added to the playlist')
    } catch (err: any) {
      console.error('Failed to add track to playlist', err)
      error('Failed to add track', err?.response?.data?.error || err?.message)
    }
  }

  return {
    playTrack,
    deleteTrack,
    addTrackToPlaylist,
  }
}
