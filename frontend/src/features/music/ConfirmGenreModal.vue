<script setup lang="ts">
import { ref, computed } from 'vue'
import type { MusicFile } from '../../types/MusicTab'
import Icon from '../../shared/components/Icons.vue'
import { musicAPI } from '../../api/music'

interface Props {
  isOpen: boolean
  track?: MusicFile
}

const props = withDefaults(defineProps<Props>(), {
  isOpen: false,
})

const emit = defineEmits<{
  close: []
  confirm: [genre: string]
}>()

const isLoading = ref(false)
const error = ref<string | null>(null)

const genreToConfirm = computed(() => props.track?.guessed_genre || '')
const trackTitle = computed(() => props.track?.title || '')
const artistName = computed(() => props.track?.artist || 'Unknown Artist')

const handleConfirm = async () => {
  if (!props.track || !genreToConfirm.value) return

  isLoading.value = true
  error.value = null

  try {
    await musicAPI.confirmGenre(props.track.id, genreToConfirm.value)
    emit('confirm', genreToConfirm.value)
    handleClose()
  } catch (err) {
    error.value = err instanceof Error ? err.message : 'Failed to confirm genre'
  } finally {
    isLoading.value = false
  }
}

const handleClose = () => {
  error.value = null
  emit('close')
}
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="handleClose">
    <div class="modal-content">
      <div class="modal-header">
        <h3>Confirm Genre</h3>
        <button class="close-btn" @click="handleClose" :disabled="isLoading">
          <Icon name="x" :size="20" />
        </button>
      </div>

      <div class="modal-body">
        <div class="info-section">
          <p class="label">Track</p>
          <p class="value">{{ trackTitle }}</p>
        </div>

        <div class="info-section">
          <p class="label">Artist</p>
          <p class="value">{{ artistName }}</p>
        </div>

        <div class="info-section">
          <p class="label">Guessed Genre</p>
          <p class="value genre-highlight">{{ genreToConfirm }}</p>
          <p class="note">This genre will be confirmed for this track.</p>
          <p class="note">If the artist has no confirmed genre yet, it will be set as their default.</p>
        </div>

        <div v-if="error" class="error-message">
          <Icon name="alert-circle" :size="16" />
          {{ error }}
        </div>
      </div>

      <div class="modal-footer">
        <button class="btn btn-outline" @click="handleClose" :disabled="isLoading">
          Cancel
        </button>
        <button class="btn btn-primary" @click="handleConfirm" :disabled="isLoading">
          <Icon v-if="isLoading" name="loader" :size="16" />
          {{ isLoading ? 'Confirming...' : 'Confirm Genre' }}
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
  }
  to {
    opacity: 1;
  }
}

.modal-content {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
  max-width: 500px;
  width: 90%;
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    transform: translateY(20px);
    opacity: 0;
  }
  to {
    transform: translateY(0);
    opacity: 1;
  }
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 24px;
  border-bottom: 1px solid var(--border-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--text-color);
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.2s ease;
  padding: 4px;
  border-radius: 8px;
}

.close-btn:hover:not(:disabled) {
  background: var(--background-elevated);
  color: var(--text-color);
}

.close-btn:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.modal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.info-section {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.label {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
  margin: 0;
}

.value {
  font-size: 15px;
  color: var(--text-color);
  margin: 0;
  font-weight: 500;
}

.genre-highlight {
  display: inline-block;
  padding: 8px 12px;
  background: var(--primary-glow);
  border: 1px solid var(--primary-color);
  border-radius: 8px;
  color: var(--primary-light);
  font-weight: 600;
  width: fit-content;
}

.note {
  font-size: 13px;
  color: var(--text-tertiary);
  margin: 0;
  font-style: italic;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 12px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 8px;
  color: var(--error-color, #ef4444);
  font-size: 14px;
}

.modal-footer {
  display: flex;
  gap: 12px;
  padding: 24px;
  border-top: 1px solid var(--border-color);
  justify-content: flex-end;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 10px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-color);
}

.btn-outline:hover:not(:disabled) {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-outline:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-light);
  box-shadow: 0 4px 12px var(--primary-glow);
  transform: translateY(-2px);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 600px) {
  .modal-content {
    width: 95%;
    max-width: 100%;
  }

  .modal-header,
  .modal-body,
  .modal-footer {
    padding: 16px;
  }

  .modal-footer {
    flex-direction: column-reverse;
  }

  .btn {
    width: 100%;
    justify-content: center;
  }
}
</style>
