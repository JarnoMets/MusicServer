<script setup lang="ts">
import type { EditState } from '../../types/MusicTab'

interface Props {
  editState: EditState
  saving: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  'update:title': [value: string]
  'update:artist': [value: string]
  'update:album': [value: string]
  'update:genre': [value: string]
  'update:duration': [value: number | undefined]
  save: []
  close: []
}>()

const handleDurationInput = (e: Event) => {
  const target = e.target as HTMLInputElement
  emit('update:duration', target.value ? Number(target.value) : undefined)
}
</script>

<template>
  <div v-if="editState.open" class="edit-drawer">
    <div class="drawer-content">
      <header>
        <h3>Edit Track</h3>
        <button class="btn-icon" @click="emit('close')">✕</button>
      </header>
      <form @submit.prevent="emit('save')" class="edit-form">
        <label>
          Title
          <input
            :value="editState.form.title"
            @input="emit('update:title', ($event.target as HTMLInputElement).value)"
            type="text"
            class="text-input"
            required
          />
        </label>
        <label>
          Artist
          <input
            :value="editState.form.artist"
            @input="emit('update:artist', ($event.target as HTMLInputElement).value)"
            type="text"
            class="text-input"
          />
        </label>
        <label>
          Album
          <input
            :value="editState.form.album"
            @input="emit('update:album', ($event.target as HTMLInputElement).value)"
            type="text"
            class="text-input"
          />
        </label>
        <label>
          Genre
          <input
            :value="editState.form.genre"
            @input="emit('update:genre', ($event.target as HTMLInputElement).value)"
            type="text"
            class="text-input"
          />
        </label>
        <label>
          Duration (seconds)
          <input
            :value="editState.form.duration ?? ''"
            @input="handleDurationInput"
            type="number"
            min="0"
            class="text-input"
          />
        </label>
        <div class="drawer-actions">
          <button type="button" class="btn btn-secondary" @click="emit('close')">Cancel</button>
          <button type="submit" class="btn btn-primary" :disabled="saving">
            {{ saving ? 'Saving…' : 'Save changes' }}
          </button>
        </div>
      </form>
    </div>
  </div>
</template>

<style scoped>
.edit-drawer {
  position: fixed;
  top: 0;
  right: 0;
  height: 100vh;
  width: 380px;
  background: var(--surface-color);
  border-left: 1px solid var(--border-color);
  box-shadow: -12px 0 40px rgba(0, 0, 0, 0.5);
  z-index: 50;
  overflow-y: auto;
  animation: slideInRight 0.3s ease;
}

@keyframes slideInRight {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.drawer-content {
  padding: 28px;
  display: flex;
  flex-direction: column;
  gap: 24px;
  min-height: 100%;
}

.drawer-content header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 12px;
  padding-bottom: 16px;
  border-bottom: 1px solid var(--border-color);
}

.drawer-content h3 {
  margin: 0;
  font-size: 20px;
  font-weight: 700;
  color: var(--text-color);
}

.btn-icon {
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  width: 38px;
  height: 38px;
  display: flex;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  transition: all 0.2s ease;
  flex-shrink: 0;
  color: var(--text-secondary);
  font-size: 16px;
}

.btn-icon:hover {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #ef4444;
  transform: scale(1.05);
}

.edit-form {
  display: flex;
  flex-direction: column;
  gap: 20px;
}

.edit-form label {
  display: flex;
  flex-direction: column;
  gap: 8px;
  font-size: 13px;
  color: var(--text-secondary);
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
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

.text-input:hover {
  border-color: var(--primary-color);
}

.text-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
  background: var(--surface-color);
}

.text-input::placeholder {
  color: var(--text-tertiary);
}

.drawer-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
  margin-top: auto;
  padding-top: 24px;
  border-top: 1px solid var(--border-color);
}

.btn {
  border: none;
  border-radius: 10px;
  padding: 12px 24px;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  gap: 8px;
}

.btn-primary {
  background: linear-gradient(135deg, var(--primary-color) 0%, var(--primary-dark) 100%);
  color: #fff;
  box-shadow: 0 4px 12px var(--primary-glow);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 6px 20px var(--primary-glow);
}

.btn-primary:active:not(:disabled) {
  transform: translateY(0);
}

.btn-primary:disabled {
  opacity: 0.5;
  cursor: not-allowed;
  transform: none;
  box-shadow: none;
}

.btn-secondary {
  background: var(--background-elevated);
  color: var(--text-secondary);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover {
  background: var(--surface-color);
  color: var(--text-color);
  border-color: var(--text-tertiary);
}

@media (max-width: 900px) {
  .edit-drawer {
    width: 100%;
    max-width: 100vw;
  }
}
</style>
