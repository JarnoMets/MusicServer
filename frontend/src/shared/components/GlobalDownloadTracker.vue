<template>
  <div v-if="downloadStore.isDownloading" class="global-download-tracker">
    <div class="tracker-badge" @click="goToDownloader">
      <div class="progress-ring">
        <svg viewBox="0 0 36 36">
          <circle class="ring-bg" cx="18" cy="18" r="16" />
          <circle 
            class="ring-fill" 
            cx="18" 
            cy="18" 
            r="16" 
            :stroke-dasharray="`${progress}, 100`"
          />
        </svg>
        <span class="icon-center">
            <Icon name="download" :size="14" />
        </span>
      </div>
      <div class="tracker-info">
        <span class="file-name">{{ downloadStore.currentProgress?.current_file || 'Downloading...' }}</span>
        <span class="progress-text">{{ progress }}%</span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { useRouter } from 'vue-router'
import { useDownloadStore } from '../../stores/downloadStore'
import Icon from './Icons.vue'

const downloadStore = useDownloadStore()
const router = useRouter()

const progress = computed(() => Math.round(downloadStore.currentProgress?.progress || 0))

const goToDownloader = () => {
  router.push({ name: 'Admin', query: { tab: 'downloader' } })
}
</script>

<style scoped>
.global-download-tracker {
  display: flex;
  align-items: center;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: 30px;
  padding: 4px 12px 4px 6px;
  cursor: pointer;
  transition: all 0.2s ease;
  max-width: 200px;
  height: 36px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.global-download-tracker:hover {
  background: var(--surface-hover);
  border-color: var(--primary-color);
  transform: translateY(-2px);
}

.tracker-badge {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  overflow: hidden;
}

.progress-ring {
  width: 26px;
  height: 26px;
  position: relative;
  flex-shrink: 0;
}

.progress-ring svg {
  transform: rotate(-90deg);
}

.ring-bg {
  fill: none;
  stroke: var(--border-color);
  stroke-width: 4;
}

.ring-fill {
  fill: none;
  stroke: var(--primary-color);
  stroke-width: 4;
  stroke-linecap: round;
  transition: stroke-dasharray 0.3s ease;
}

.icon-center {
  position: absolute;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  color: var(--primary-light);
  line-height: 0;
}

.tracker-info {
  display: flex;
  flex-direction: column;
  min-width: 0;
}

.file-name {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-color);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  line-height: 1.2;
}

.progress-text {
  font-size: 9px;
  color: var(--text-tertiary);
  font-weight: 700;
  line-height: 1;
}

@keyframes pulse {
  0% { opacity: 1; }
  50% { opacity: 0.7; }
  100% { opacity: 1; }
}

.global-download-tracker {
  animation: pulse 2s infinite ease-in-out;
}
</style>
