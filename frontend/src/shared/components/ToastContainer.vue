<template>
  <Teleport to="body">
    <div class="toast-container" aria-live="polite">
      <TransitionGroup name="toast">
        <div
          v-for="toast in toasts"
          :key="toast.id"
          :class="['toast', `toast-${toast.type}`]"
          @click="removeToast(toast.id)"
        >
          <div class="toast-icon">
            <Icon v-if="toast.type === 'success'" name="check" :size="16" />
            <Icon v-else-if="toast.type === 'error'" name="x" :size="16" />
            <Icon v-else-if="toast.type === 'warning'" name="alert-triangle" :size="16" />
            <Icon v-else name="info" :size="16" />
          </div>
          <div class="toast-content">
            <div class="toast-title">{{ toast.title }}</div>
            <div v-if="toast.message" class="toast-message">{{ toast.message }}</div>
          </div>
          <button class="toast-close" @click.stop="removeToast(toast.id)">
            <Icon name="x" :size="16" />
          </button>
        </div>
      </TransitionGroup>
    </div>
  </Teleport>
</template>

<script setup lang="ts">
import { useToast } from '../../composables/useToast'
import Icon from './Icons.vue'

const { toasts, removeToast } = useToast()
</script>

<style scoped>
.toast-container {
  position: fixed;
  bottom: 120px;
  right: 24px;
  display: flex;
  flex-direction: column-reverse;
  gap: 12px;
  z-index: 9999;
  max-width: 400px;
  width: calc(100vw - 48px);
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  padding: 14px 16px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.05);
  cursor: pointer;
  pointer-events: auto;
  backdrop-filter: blur(12px);
  transition: all var(--transition-base);
}

.toast:hover {
  transform: translateX(-4px);
  box-shadow: 0 15px 50px rgba(0, 0, 0, 0.5);
}

.toast-icon {
  width: 28px;
  height: 28px;
  border-radius: var(--radius-sm);
  display: flex;
  align-items: center;
  justify-content: center;
  flex-shrink: 0;
  font-size: 14px;
  font-weight: 700;
}

.toast-success .toast-icon {
  background: rgba(6, 214, 160, 0.15);
  color: var(--success-color);
  border: 1px solid rgba(6, 214, 160, 0.3);
}

.toast-error .toast-icon {
  background: rgba(248, 113, 113, 0.15);
  color: var(--error-color);
  border: 1px solid rgba(248, 113, 113, 0.3);
}

.toast-warning .toast-icon {
  background: rgba(251, 191, 36, 0.15);
  color: var(--warning-color);
  border: 1px solid rgba(251, 191, 36, 0.3);
}

.toast-info .toast-icon {
  background: rgba(56, 189, 248, 0.15);
  color: var(--info-color);
  border: 1px solid rgba(56, 189, 248, 0.3);
}

.toast-content {
  flex: 1;
  min-width: 0;
}

.toast-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color);
  line-height: 1.3;
}

.toast-message {
  font-size: 13px;
  color: var(--text-secondary);
  margin-top: 4px;
  line-height: 1.4;
}

.toast-close {
  background: none;
  border: none;
  color: var(--text-tertiary);
  font-size: 18px;
  line-height: 1;
  cursor: pointer;
  padding: 0;
  margin: -4px -4px -4px 0;
  width: 24px;
  height: 24px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: var(--radius-sm);
  transition: all var(--transition-base);
}

.toast-close:hover {
  background: var(--surface-muted);
  color: var(--text-color);
}

/* Animations */
.toast-enter-active {
  animation: toast-in 0.3s ease-out;
}

.toast-leave-active {
  animation: toast-out 0.2s ease-in forwards;
}

@keyframes toast-in {
  from {
    opacity: 0;
    transform: translateX(100%);
  }
  to {
    opacity: 1;
    transform: translateX(0);
  }
}

@keyframes toast-out {
  from {
    opacity: 1;
    transform: translateX(0);
  }
  to {
    opacity: 0;
    transform: translateX(100%);
  }
}

/* Responsive */
@media (max-width: 480px) {
  .toast-container {
    bottom: 140px;
    right: 12px;
    left: 12px;
    width: auto;
    max-width: none;
  }

  .toast {
    padding: 12px 14px;
  }
}
</style>
