/**
 * Composable for track editing functionality
 */
import { reactive, ref } from 'vue'
import { musicAPI } from '../api/music'
import { useToast } from './useToast'
import type { MusicFile, EditState } from '../types'

export const useTrackEdit = (onSaveComplete: () => void) => {
  const { error: toastError } = useToast()
  const editState = reactive<EditState>({
    open: false,
    saving: false,
    trackId: null,
    form: {
      title: '',
      artist: '',
      album: '',
      genre: '',
      genre_id: '',
      release_date: '',
      bpm: null,
      initial_key: '',
      beat_grid_offset: null,
    },
  })

  // MusicBrainz suggestion
  const releaseDateSuggestion = ref<{ date: string; album?: string; genre?: string; confidence: number } | null>(null)
  const lookingUpReleaseDate = ref(false)

  const openEdit = (track: MusicFile) => {
    editState.trackId = track.id
    editState.form.title = track.title
    editState.form.artist = track.artist || ''
    editState.form.album = track.album || ''
    editState.form.genre = track.genre_name || ''
    editState.form.genre_id = track.genre_id || ''
    editState.form.bpm = track.bpm || null
    editState.form.initial_key = track.initial_key || ''
    editState.form.beat_grid_offset = track.beat_grid_offset || null
    // Format release_date for date input (YYYY-MM-DD)
    if (track.release_date) {
      const d = new Date(track.release_date)
      editState.form.release_date = d.toISOString().split('T')[0]
    } else {
      editState.form.release_date = ''
    }
    editState.open = true
    releaseDateSuggestion.value = null

    // Auto-lookup release date if not set
    if (!track.release_date) {
      lookupReleaseDate(track.title, track.artist || undefined)
    }
  }

  const lookupReleaseDate = async (title: string, artist?: string) => {
    lookingUpReleaseDate.value = true
    releaseDateSuggestion.value = null
    try {
      const response = await musicAPI.lookupReleaseDate(title, artist)
      const data = response.data
      if (data.release_date && data.confidence > 0.5) {
        releaseDateSuggestion.value = {
          date: data.release_date,
          album: data.album,
          genre: data.genre,
          confidence: data.confidence,
        }
      }
    } catch (error) {
      console.warn('Release date lookup failed', error)
    } finally {
      lookingUpReleaseDate.value = false
    }
  }

  const applySuggestedReleaseDate = () => {
    if (releaseDateSuggestion.value) {
      // Handle partial dates like "2020" or "2020-05"
      let dateStr = releaseDateSuggestion.value.date
      if (/^\d{4}$/.test(dateStr)) {
        dateStr = `${dateStr}-01-01`
      } else if (/^\d{4}-\d{2}$/.test(dateStr)) {
        dateStr = `${dateStr}-01`
      }
      editState.form.release_date = dateStr
      if (releaseDateSuggestion.value.album && !editState.form.album) {
        editState.form.album = releaseDateSuggestion.value.album
      }
      if (releaseDateSuggestion.value.genre && !editState.form.genre) {
        editState.form.genre = releaseDateSuggestion.value.genre
      }
    }
  }

  const closeEdit = () => {
    editState.open = false
    editState.trackId = null
    releaseDateSuggestion.value = null
  }

  const saveEdit = async () => {
    if (!editState.trackId) return
    editState.saving = true
    try {
      const updateData: Record<string, unknown> = {
        title: editState.form.title,
        artist: editState.form.artist || undefined,
        album: editState.form.album || undefined,
        genre_id: editState.form.genre_id || undefined,
        bpm: editState.form.bpm || null,
        initial_key: editState.form.initial_key || undefined,
        beat_grid_offset: editState.form.beat_grid_offset || null,
        metadata_analyzed: true, // Mark as analyzed when manually edited
      }
      if (editState.form.release_date) {
        // Handle YYYY-MM-DD or partials
        let dateVal = editState.form.release_date
        if (/^\d{4}$/.test(dateVal)) dateVal = `${dateVal}-01-01`
        else if (/^\d{4}-\d{2}$/.test(dateVal)) dateVal = `${dateVal}-01`
        
        try {
          updateData.release_date = new Date(dateVal).toISOString()
        } catch {
          console.error("Invalid date", dateVal)
        }
      }
      await musicAPI.updateMusicFile(editState.trackId, updateData)
      onSaveComplete()
      closeEdit()
    } catch (error) {
      console.error('Failed to update track', error)
      toastError('Failed to save', 'Unable to save changes, please try again.')
    } finally {
      editState.saving = false
    }
  }

  const quickSetDate = async (trackId: string, date: string, album?: string) => {
    try {
      // Handle partial dates
      let dateVal = date
      if (/^\d{4}$/.test(dateVal)) dateVal = `${dateVal}-01-01`
      else if (/^\d{4}-\d{2}$/.test(dateVal)) dateVal = `${dateVal}-01`
      
      const isoDate = new Date(dateVal).toISOString()
      const updateData: Record<string, unknown> = {
        release_date: isoDate
      }
      if (album) updateData.album = album

      await musicAPI.updateMusicFile(trackId, updateData)
      onSaveComplete()
    } catch (error) {
      console.error('Failed to quick set date', error)
    }
  }

  return {
    editState,
    releaseDateSuggestion,
    lookingUpReleaseDate,
    openEdit,
    closeEdit,
    saveEdit,
    lookupReleaseDate,
    applySuggestedReleaseDate,
    quickSetDate,
  }
}
