<template>
  <Transition name="float-in">
    <div v-if="isVisible" class="upload-tracker-floating">
      <div class="tracker-container">
        <div class="tracker-header">
          <div class="tracker-title">
            <Icon name="upload" :size="16" class="tracker-icon" />
            <span>Uploading</span>
          </div>
          <button class="close-btn" @click="toggleMinimize" :title="isMinimized ? 'Expand' : 'Collapse'">
            <Icon :name="isMinimized ? 'chevron-up' : 'chevron-down'" :size="16" />
          </button>
        </div>

        <div v-if="!isMinimized" class="tracker-content">
          <div class="progress-info">
            <p class="progress-text">
              {{ uploadingCount }} uploading · {{ successCount }} completed · {{ queueCount }} queued
            </p>
            <p class="progress-percent">{{ totalProgress }}%</p>
          </div>

          <div class="progress-bar">
            <div class="progress-fill" :style="{ width: totalProgress + '%' }"></div>
          </div>

          <div class="tracker-actions">
            <button class="btn-action btn-cancel" @click="cancelAllUploads" v-if="uploading">
              Cancel all
            </button>
            <RouterLink to="/upload" class="btn-action btn-view">
              View details
            </RouterLink>
          </div>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { computed, ref } from 'vue'
import { RouterLink } from 'vue-router'
import { useUploadManager } from '../composables/useUploadManager'
import Icon from '../shared/components/Icons.vue'

const {
  totalCount,
  uploading,
  uploadingCount,
  successCount,
  queueCount,
  totalProgress,
  cancelAllUploads,
} = useUploadManager()

const isMinimized = ref(false)

const isVisible = computed(() => totalCount.value > 0)

function toggleMinimize() {
  isMinimized.value = !isMinimized.value
}
</script>

<style scoped>
.upload-tracker-floating {
  position: fixed;
  bottom: 24px;
  right: 24px;
  z-index: 999;
  max-width: 320px;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.3);
}

.tracker-container {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  overflow: hidden;
  backdrop-filter: blur(10px);
}

.tracker-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px 16px;
  background: linear-gradient(135deg, var(--primary-color), var(--primary-dark));
  color: white;
  font-weight: 600;
  font-size: 13px;
}

.tracker-title {
  display: flex;
  align-items: center;
  gap: 8px;
}

.tracker-icon {
  opacity: 0.9;
}

.close-btn {
  background: rgba(255, 255, 255, 0.2);
  border: none;
  border-radius: 8px;
  padding: 4px 6px;
  cursor: pointer;
  color: white;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: background 0.2s ease;
}

.close-btn:hover {
  background: rgba(255, 255, 255, 0.3);
}

.tracker-content {
  padding: 12px 16px;
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-text {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0;
}

.progress-percent {
  font-size: 14px;
  font-weight: 700;
  color: var(--text-color);
  margin: 0;
}

.progress-bar {
  width: 100%;
  height: 6px;
  background: var(--background-elevated);
  border-radius: 999px;
  overflow: hidden;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, var(--primary-color), var(--accent-color));
  border-radius: inherit;
  transition: width 0.3s ease;
  position: relative;
}

.progress-fill::after {
  content: '';
  position: absolute;
  inset: 0;
  background: linear-gradient(90deg, transparent, rgba(255, 255, 255, 0.3), transparent);
  animation: shimmer 1.5s infinite;
}

@keyframes shimmer {
  0% {
    transform: translateX(-100%);
  }
  100% {
    transform: translateX(100%);
  }
}

.tracker-actions {
  display: flex;
  gap: 8px;
  font-size: 12px;
}

.btn-action {
  flex: 1;
  padding: 6px 10px;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  background: var(--background-elevated);
  color: var(--text-color);
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  transition: all 0.2s ease;
  text-decoration: none;
  text-align: center;
  display: flex;
  align-items: center;
  justify-content: center;
}

.btn-action:hover {
  background: var(--primary-glow);
  border-color: var(--primary-color);
  color: var(--primary-light);
}

.btn-cancel {
  background: rgba(239, 68, 68, 0.1);
  border-color: rgba(239, 68, 68, 0.3);
  color: #ef4444;
}

.btn-cancel:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: #ef4444;
}

.float-in-enter-active,
.float-in-leave-active {
  transition: all 0.3s ease;
}

.float-in-enter-from {
  opacity: 0;
  transform: translateY(30px) translateX(30px);
}

.float-in-leave-to {
  opacity: 0;
  transform: translateY(30px) translateX(30px);
}

@media (max-width: 640px) {
  .upload-tracker-floating {
    bottom: 16px;
    right: 16px;
    max-width: 280px;
  }
}
</style>
