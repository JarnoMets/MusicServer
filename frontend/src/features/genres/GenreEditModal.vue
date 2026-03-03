<script setup lang="ts">
import { ref, watch } from 'vue'
import { musicAPI } from '../../api/music'
import { useToast } from '../../composables/useToast'
import Icon from '../../shared/components/Icons.vue'

interface Props {
  isOpen: boolean
  genre?: { id?: string, name: string, description?: string } | null
}

const props = defineProps<Props>()

const emit = defineEmits<{
  close: []
  saved: []
}>()

const { success, error: showError } = useToast()
const loading = ref(false)
const formData = ref({
  name: '',
  description: ''
})

watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    formData.value = {
      name: props.genre?.name || '',
      description: props.genre?.description || ''
    }
  }
})

const handleSave = async () => {
  if (!formData.value.name.trim()) {
    showError('Error', 'Genre name is required')
    return
  }

  loading.value = true
  try {
    if (props.genre?.id) {
      await musicAPI.updateGenre(props.genre.id, formData.value.name, formData.value.description)
      success('Success', 'Genre updated successfully')
    } else {
      await musicAPI.createGenre(formData.value.name, formData.value.description)
      success('Success', 'Genre created successfully')
    }
    emit('saved')
    emit('close')
  } catch (err: any) {
    showError('Error', err.response?.data?.error || 'Failed to save genre')
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div v-if="isOpen" class="modal-overlay" @click.self="emit('close')">
    <div class="modal-content">
      <div class="modal-header">
        <h3>{{ genre?.id ? 'Edit Genre' : 'Create Genre' }}</h3>
        <button class="close-btn" @click="emit('close')" :disabled="loading">
          <Icon name="x" :size="20" />
        </button>
      </div>

      <form @submit.prevent="handleSave" class="modal-body">
        <div class="form-group">
          <label for="genre-name">Name</label>
          <input 
            id="genre-name"
            v-model="formData.name"
            type="text" 
            placeholder="e.g. Deep House"
            class="text-input"
            required
            :disabled="loading"
          />
        </div>

        <div class="form-group">
          <label for="genre-desc">Description (Optional)</label>
          <textarea
            id="genre-desc"
            v-model="formData.description"
            placeholder="Describe the genre..."
            class="text-input textarea"
            :disabled="loading"
            rows="3"
          ></textarea>
        </div>

        <div class="modal-footer">
          <button type="button" class="btn btn-outline" @click="emit('close')" :disabled="loading">
            Cancel
          </button>
          <button type="submit" class="btn btn-primary" :disabled="loading || !formData.name.trim()">
            {{ loading ? 'Saving...' : (genre?.id ? 'Save Changes' : 'Create Genre') }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.6);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 2000;
  padding: 20px;
}

.modal-content {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
  width: 90%;
  max-width: 450px;
  animation: slideUp 0.3s ease;
}

@keyframes fadeIn { from { opacity: 0; } to { opacity: 1; } }
@keyframes slideUp { from { transform: translateY(20px); opacity: 0; } to { transform: translateY(0); opacity: 1; } }

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
}

.close-btn {
  background: none;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  padding: 4px;
  border-radius: 50%;
  display: flex;
}

.modal-body {
  padding: 24px;
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.form-group label {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.text-input {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 12px 14px;
  color: var(--text-color);
  font-family: inherit;
  font-size: 15px;
  transition: all 0.2s ease;
}

.text-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.textarea {
  resize: vertical;
  min-height: 80px;
}

.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: 8px;
}

.btn {
  padding: 10px 20px;
  border: none;
  border-radius: 10px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
}

.btn-outline {
  background: transparent;
  border: 1px solid var(--border-color);
  color: var(--text-color);
}

.btn-primary {
  background: var(--primary-color);
  color: white;
}

.btn-primary:hover:not(:disabled) {
  background: var(--primary-light);
  transform: translateY(-1px);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}
</style>
