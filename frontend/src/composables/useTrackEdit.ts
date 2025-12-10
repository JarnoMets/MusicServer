/**
 * Composable for track editing functionality
 */
import { reactive } from 'vue'
import { musicAPI } from '../api/music'
import type { MusicFile, EditState } from '../types/MusicTab'

export const useTrackEdit = (onSaveComplete: () => void) => {
  const editState = reactive<EditState>({
    open: false,
    saving: false,
    trackId: null,
    form: {
      title: '',
      artist: '',
      album: '',
      genre: '',
      duration: undefined,
    },
  })

  const openEdit = (track: MusicFile) => {
    editState.trackId = track.id
    editState.form.title = track.title
    editState.form.artist = track.artist || ''
    editState.form.album = track.album || ''
    editState.form.genre = track.genre || track.guessed_genre || ''
    editState.form.duration = track.duration ?? undefined
    editState.open = true
  }

  const closeEdit = () => {
    editState.open = false
    editState.trackId = null
  }

  const saveEdit = async () => {
    if (!editState.trackId) return
    editState.saving = true
    try {
      await musicAPI.updateMusicFile(editState.trackId, {
        title: editState.form.title,
        artist: editState.form.artist || undefined,
        album: editState.form.album || undefined,
        genre: editState.form.genre || undefined,
        duration: editState.form.duration,
      })
      onSaveComplete()
      closeEdit()
    } catch (error) {
      console.error('Failed to update track', error)
      alert('Unable to save changes')
    } finally {
      editState.saving = false
    }
  }

  return {
    editState,
    openEdit,
    closeEdit,
    saveEdit,
  }
}
