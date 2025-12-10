<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="isOpen" class="modal-overlay" @click="emit('close')">
        <div class="modal" @click.stop>
          <div class="modal-header">
            <div class="modal-icon">
              <Icon name="settings" :size="24" />
            </div>
            <h3>Admin Login</h3>
            <button class="modal-close" @click="emit('close')">
              <Icon name="x" :size="20" />
            </button>
          </div>
          
          <form class="modal-body" @submit.prevent="handleSubmit">
            <p class="modal-description">
              Enter your admin token to manage playlists, edit songs, and access admin features.
            </p>
            
            <div class="form-group">
              <label for="admin-token">Admin Token</label>
              <div class="input-wrapper">
                <input
                  id="admin-token"
                  v-model="tokenInput"
                  :type="showToken ? 'text' : 'password'"
                  placeholder="Enter your admin token"
                  autocomplete="off"
                  :disabled="isLoading"
                />
                <button 
                  type="button" 
                  class="toggle-visibility"
                  @click="showToken = !showToken"
                  :title="showToken ? 'Hide token' : 'Show token'"
                >
                  <Icon :name="showToken ? 'eye-off' : 'eye'" :size="18" />
                </button>
              </div>
            </div>

            <Transition name="fade">
              <div v-if="error" class="error-message">
                <Icon name="alert-circle" :size="16" />
                <span>{{ error }}</span>
              </div>
            </Transition>

            <div class="form-actions">
              <button type="button" class="btn btn-secondary" @click="emit('close')" :disabled="isLoading">
                Cancel
              </button>
              <button type="submit" class="btn btn-primary" :disabled="!tokenInput.trim() || isLoading">
                <span v-if="isLoading" class="loading-spinner"></span>
                {{ isLoading ? 'Verifying...' : 'Login' }}
              </button>
            </div>
          </form>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import Icon from './Icons.vue'

interface Props {
  isOpen: boolean
  error?: string | null
  isLoading?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{
  close: []
  submit: [token: string]
}>()

const tokenInput = ref('')
const showToken = ref(false)

// Reset form when modal opens
watch(() => props.isOpen, (isOpen) => {
  if (isOpen) {
    tokenInput.value = ''
    showToken.value = false
  }
})

const handleSubmit = () => {
  if (tokenInput.value.trim()) {
    emit('submit', tokenInput.value.trim())
  }
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.75);
  backdrop-filter: blur(8px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.modal {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 20px;
  width: 100%;
  max-width: 420px;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.4);
  overflow: hidden;
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 14px;
  padding: 24px 24px 0;
}

.modal-icon {
  width: 48px;
  height: 48px;
  background: var(--primary-glow);
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  color: var(--primary-color);
}

.modal-header h3 {
  margin: 0;
  font-size: 1.25rem;
  font-weight: 700;
  flex: 1;
}

.modal-close {
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 8px;
  transition: all 0.2s ease;
}

.modal-close:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.modal-body {
  padding: 24px;
}

.modal-description {
  margin: 0 0 20px 0;
  color: var(--text-secondary);
  font-size: 14px;
  line-height: 1.6;
}

.form-group {
  margin-bottom: 20px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 600;
  font-size: 13px;
  color: var(--text-secondary);
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.input-wrapper input {
  width: 100%;
  padding: 14px 48px 14px 16px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  color: var(--text-color);
  font-family: inherit;
  font-size: 14px;
  transition: all 0.2s ease;
}

.input-wrapper input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.input-wrapper input:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.toggle-visibility {
  position: absolute;
  right: 8px;
  background: none;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 8px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 6px;
  transition: all 0.2s ease;
}

.toggle-visibility:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.error-message {
  display: flex;
  align-items: flex-start;
  gap: 10px;
  padding: 14px 16px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.3);
  border-radius: 10px;
  color: #f87171;
  font-size: 13px;
  line-height: 1.5;
  margin-bottom: 20px;
}

.error-message svg {
  flex-shrink: 0;
  margin-top: 1px;
}

.form-actions {
  display: flex;
  gap: 12px;
  justify-content: flex-end;
}

.btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 24px;
  border: none;
  border-radius: 10px;
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  min-width: 100px;
}

.btn-primary {
  background: linear-gradient(135deg, var(--primary-color), var(--accent-color));
  color: white;
  box-shadow: 0 4px 15px var(--accent-muted);
}

.btn-primary:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 8px 25px var(--accent-muted);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.btn-secondary {
  background: var(--background-elevated);
  color: var(--text-color);
  border: 1px solid var(--border-color);
}

.btn-secondary:hover:not(:disabled) {
  background: var(--surface-hover);
  border-color: var(--border-hover);
}

.btn-secondary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid rgba(255, 255, 255, 0.3);
  border-top-color: white;
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.3s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal,
.modal-leave-to .modal {
  transform: scale(0.95) translateY(-20px);
}

.fade-enter-active,
.fade-leave-active {
  transition: all 0.2s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}
</style>
