<script setup lang="ts">
import Icon from './Icons.vue'

interface Props {
  show: boolean
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'danger' | 'warning' | 'info'
}

withDefaults(defineProps<Props>(), {
  title: 'Confirm',
  confirmText: 'Confirm',
  cancelText: 'Cancel',
  variant: 'danger'
})

const emit = defineEmits<{
  confirm: []
  cancel: []
}>()

const handleConfirm = () => {
  emit('confirm')
}

const handleCancel = () => {
  emit('cancel')
}

const handleBackdropClick = (e: MouseEvent) => {
  if (e.target === e.currentTarget) {
    handleCancel()
  }
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-backdrop" @click="handleBackdropClick">
        <div class="modal-content" role="dialog" aria-modal="true">
          <div class="modal-header">
            <div :class="['modal-icon', variant]">
              <Icon v-if="variant === 'danger'" name="trash" :size="24" />
              <Icon v-else-if="variant === 'warning'" name="alert-triangle" :size="24" />
              <Icon v-else name="info" :size="24" />
            </div>
            <h3 class="modal-title">{{ title }}</h3>
          </div>
          
          <p class="modal-message">{{ message }}</p>
          
          <div class="modal-actions">
            <button 
              type="button" 
              class="btn btn-secondary" 
              @click="handleCancel"
            >
              {{ cancelText }}
            </button>
            <button 
              type="button" 
              :class="['btn', `btn-${variant}`]"
              @click="handleConfirm"
            >
              {{ confirmText }}
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </Teleport>
</template>

<style scoped>
.modal-backdrop {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.7);
  backdrop-filter: blur(4px);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 4000;
  padding: 20px;
}

.modal-content {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 28px;
  max-width: 420px;
  width: 100%;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.modal-header {
  display: flex;
  align-items: center;
  gap: 16px;
  margin-bottom: 16px;
}

.modal-icon {
  width: 48px;
  height: 48px;
  border-radius: 12px;
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
}

.modal-icon.danger {
  background: rgba(239, 68, 68, 0.15);
  color: #ef4444;
}

.modal-icon.warning {
  background: rgba(245, 158, 11, 0.15);
  color: #f59e0b;
}

.modal-icon.info {
  background: var(--primary-glow);
  color: var(--primary-color);
}

.modal-title {
  margin: 0;
  font-size: 18px;
  font-weight: 700;
  color: var(--text-color);
}

.modal-message {
  margin: 0 0 24px 0;
  color: var(--text-secondary);
  font-size: 15px;
  line-height: 1.6;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 12px;
}

.btn {
  border: none;
  border-radius: 10px;
  padding: 12px 20px;
  cursor: pointer;
  font-weight: 600;
  font-size: 14px;
  transition: all 0.2s ease;
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

.btn-danger {
  background: #ef4444;
  color: white;
}

.btn-danger:hover {
  background: #dc2626;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
}

.btn-warning {
  background: #f59e0b;
  color: white;
}

.btn-warning:hover {
  background: #d97706;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(245, 158, 11, 0.4);
}

.btn-info {
  background: var(--primary-color);
  color: white;
}

.btn-info:hover {
  background: var(--primary-dark);
  transform: translateY(-1px);
  box-shadow: 0 4px 12px var(--primary-glow);
}

/* Transitions */
.modal-enter-active,
.modal-leave-active {
  transition: all 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: scale(0.95) translateY(-10px);
}

.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.2s ease;
}
</style>
