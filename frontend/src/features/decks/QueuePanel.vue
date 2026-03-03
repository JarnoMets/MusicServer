<template>
  <div class="modal-overlay" @click.self="$emit('close')">
    <div class="queue-panel">
      <div class="panel-header">
        <h3>
          <Icon name="list" :size="18" />
          DJ Queue
        </h3>
        <div class="header-actions">
          <span class="queue-count">{{ store.autoPlayPlaylist.length }} tracks</span>
          <button
            v-if="pendingTracks.length > 0"
            class="btn-clear"
            @click="store.clearPendingQueue()"
            title="Clear all pending tracks"
          >
            <Icon name="trash" :size="13" />
            Clear Pending
          </button>
          <button class="btn-close" @click="$emit('close')">
            <Icon name="x" :size="18" />
          </button>
        </div>
      </div>

      <!-- Legend -->
      <div class="queue-legend">
        <span class="legend-item played"><span class="legend-dot"></span> Played</span>
        <span class="legend-item loaded"><span class="legend-dot"></span> Loaded</span>
        <span class="legend-item pending"><span class="legend-dot"></span> Pending</span>
      </div>

      <div class="queue-list" ref="listRef">
        <div v-if="store.autoPlayPlaylist.length === 0" class="queue-empty">
          <Icon name="disc" :size="32" />
          <p>No tracks in queue</p>
          <span class="hint">Add tracks from the Library, Playlists, Genres or Artists</span>
        </div>

        <TransitionGroup name="queue-item" tag="div">
          <div
            v-for="(track, index) in store.autoPlayPlaylist"
            :key="track.id + '-' + index"
            class="queue-item"
            :class="[
              trackStatus(index),
              { 'drag-over': dragOverIndex === index },
              { 'dragging': dragIndex === index },
            ]"
            :draggable="trackStatus(index) === 'pending'"
            @dragstart="onDragStart(index, $event)"
            @dragover.prevent="onDragOver(index, $event)"
            @dragleave="onDragLeave"
            @drop.prevent="onDrop(index)"
            @dragend="onDragEnd"
          >
            <span class="queue-index">{{ index + 1 }}</span>

            <div class="queue-status-indicator">
              <Icon
                v-if="trackStatus(index) === 'played'"
                name="check-circle"
                :size="14"
              />
              <Icon
                v-else-if="trackStatus(index) === 'loaded'"
                name="disc"
                :size="14"
              />
              <span v-else class="pending-dot"></span>
            </div>

            <div class="queue-info">
              <span class="queue-title">{{ track.title }}</span>
              <span class="queue-artist">{{ track.artist || 'Unknown Artist' }}</span>
            </div>

            <div class="queue-meta">
              <span v-if="track.bpm" class="queue-bpm">{{ Math.round(track.bpm) }}</span>
              <span v-if="track.initial_key && track.initial_key !== 'NONE'" class="queue-key">{{ track.initial_key }}</span>
            </div>

            <!-- Drag handle for pending tracks -->
            <button
              v-if="trackStatus(index) === 'pending'"
              class="btn-drag"
              title="Drag to reorder"
            >
              <Icon name="move" :size="14" />
            </button>

            <!-- Remove button for pending tracks -->
            <button
              v-if="trackStatus(index) === 'pending'"
              class="btn-remove"
              @click="store.removeFromQueue(index)"
              title="Remove from queue"
            >
              <Icon name="x" :size="14" />
            </button>
          </div>
        </TransitionGroup>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { useDjStore } from '../../stores/djStore'
import Icon from '../../shared/components/Icons.vue'

defineEmits<{
  close: []
}>()

const store = useDjStore()

const dragIndex = ref<number | null>(null)
const dragOverIndex = ref<number | null>(null)

const trackStatus = (index: number) => store.getTrackQueueStatus(index)

const pendingTracks = computed(() =>
  store.autoPlayPlaylist.filter((_, i) => trackStatus(i) === 'pending')
)

// ─── Drag & Drop ──────────────────────────────────────────────

const onDragStart = (index: number, e: DragEvent) => {
  if (trackStatus(index) !== 'pending') {
    e.preventDefault()
    return
  }
  dragIndex.value = index
  if (e.dataTransfer) {
    e.dataTransfer.effectAllowed = 'move'
    e.dataTransfer.setData('text/plain', String(index))
  }
}

const onDragOver = (index: number, e: DragEvent) => {
  if (dragIndex.value === null) return
  if (trackStatus(index) !== 'pending') return
  e.dataTransfer!.dropEffect = 'move'
  dragOverIndex.value = index
}

const onDragLeave = () => {
  dragOverIndex.value = null
}

const onDrop = (toIndex: number) => {
  if (dragIndex.value === null) return
  if (trackStatus(toIndex) !== 'pending') return
  store.reorderQueue(dragIndex.value, toIndex)
  dragIndex.value = null
  dragOverIndex.value = null
}

const onDragEnd = () => {
  dragIndex.value = null
  dragOverIndex.value = null
}
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.8);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
}

.queue-panel {
  background: #1a1a1a;
  width: 520px;
  max-height: 85vh;
  border-radius: 16px;
  border: 1px solid #333;
  display: flex;
  flex-direction: column;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.5);
}

.panel-header {
  padding: 16px 20px;
  border-bottom: 1px solid #333;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.panel-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 700;
  display: flex;
  align-items: center;
  gap: 8px;
  color: #fff;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 10px;
}

.queue-count {
  font-size: 11px;
  color: #888;
  font-weight: 600;
  padding: 3px 8px;
  background: rgba(255,255,255,0.05);
  border-radius: 10px;
}

.btn-clear {
  display: flex;
  align-items: center;
  gap: 5px;
  background: rgba(239, 68, 68, 0.1);
  border: 1px solid rgba(239, 68, 68, 0.2);
  color: #f87171;
  padding: 4px 10px;
  border-radius: 6px;
  cursor: pointer;
  font-size: 11px;
  font-weight: 600;
  transition: all 0.15s;
}

.btn-clear:hover {
  background: rgba(239, 68, 68, 0.2);
  border-color: rgba(239, 68, 68, 0.4);
}

.btn-close {
  background: none;
  border: none;
  color: #888;
  cursor: pointer;
  padding: 4px;
  border-radius: 6px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}

.btn-close:hover {
  color: #fff;
  background: rgba(255,255,255,0.1);
}

/* Legend */
.queue-legend {
  display: flex;
  gap: 16px;
  padding: 10px 20px;
  border-bottom: 1px solid rgba(255,255,255,0.04);
  font-size: 10px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 5px;
}

.legend-dot {
  width: 8px;
  height: 8px;
  border-radius: 50%;
}

.legend-item.played { color: #22c55e; }
.legend-item.played .legend-dot { background: #22c55e; }

.legend-item.loaded { color: #f59e0b; }
.legend-item.loaded .legend-dot { background: #f59e0b; }

.legend-item.pending { color: #888; }
.legend-item.pending .legend-dot { background: #555; }

/* Queue List */
.queue-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.queue-empty {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  padding: 48px 24px;
  color: #555;
  text-align: center;
}

.queue-empty p {
  margin: 12px 0 4px;
  font-size: 14px;
  font-weight: 600;
  color: #888;
}

.queue-empty .hint {
  font-size: 11px;
  color: #555;
}

/* Queue Item */
.queue-item {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 8px 12px;
  border-radius: 8px;
  transition: all 0.15s;
  border: 1px solid transparent;
}

.queue-item:hover {
  background: rgba(255,255,255,0.03);
}

/* Played status */
.queue-item.played {
  opacity: 0.5;
  border-left: 3px solid #22c55e;
}

.queue-item.played .queue-title {
  text-decoration: line-through;
  color: #22c55e;
}

.queue-item.played .queue-status-indicator {
  color: #22c55e;
}

/* Loaded status */
.queue-item.loaded {
  background: rgba(245, 158, 11, 0.08);
  border-left: 3px solid #f59e0b;
}

.queue-item.loaded .queue-title {
  color: #f59e0b;
}

.queue-item.loaded .queue-status-indicator {
  color: #f59e0b;
}

/* Pending status */
.queue-item.pending {
  cursor: grab;
}

.queue-item.pending:active {
  cursor: grabbing;
}

.queue-item.dragging {
  opacity: 0.3;
}

.queue-item.drag-over {
  border: 1px dashed #4f46e5;
  background: rgba(79, 70, 229, 0.1);
}

.queue-index {
  font-size: 11px;
  color: #555;
  font-weight: 700;
  width: 24px;
  text-align: right;
  flex-shrink: 0;
}

.queue-status-indicator {
  flex-shrink: 0;
  width: 16px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.pending-dot {
  width: 6px;
  height: 6px;
  border-radius: 50%;
  background: #555;
}

.queue-info {
  flex: 1;
  min-width: 0;
  display: flex;
  flex-direction: column;
}

.queue-title {
  color: #fff;
  font-size: 13px;
  font-weight: 600;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.queue-artist {
  color: #888;
  font-size: 11px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.queue-meta {
  display: flex;
  gap: 6px;
  align-items: center;
  flex-shrink: 0;
}

.queue-bpm {
  font-size: 10px;
  font-weight: 700;
  color: #4f46e5;
  font-family: 'Courier New', monospace;
}

.queue-key {
  font-size: 10px;
  font-weight: 600;
  color: #22c55e;
}

.btn-drag {
  background: none;
  border: none;
  color: #444;
  cursor: grab;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.queue-item.pending:hover .btn-drag {
  opacity: 1;
}

.btn-drag:hover {
  color: #888;
  background: rgba(255,255,255,0.05);
}

.btn-remove {
  background: none;
  border: none;
  color: #555;
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0;
  transition: all 0.15s;
  flex-shrink: 0;
}

.queue-item.pending:hover .btn-remove {
  opacity: 1;
}

.btn-remove:hover {
  color: #f87171;
  background: rgba(239, 68, 68, 0.15);
}

/* TransitionGroup animations */
.queue-item-enter-active,
.queue-item-leave-active {
  transition: all 0.25s ease;
}

.queue-item-enter-from {
  opacity: 0;
  transform: translateX(-20px);
}

.queue-item-leave-to {
  opacity: 0;
  transform: translateX(20px);
}

.queue-item-move {
  transition: transform 0.25s ease;
}

/* Scrollbar */
.queue-list::-webkit-scrollbar {
  width: 5px;
}

.queue-list::-webkit-scrollbar-track {
  background: transparent;
}

.queue-list::-webkit-scrollbar-thumb {
  background: rgba(255,255,255,0.1);
  border-radius: 3px;
}

.queue-list::-webkit-scrollbar-thumb:hover {
  background: rgba(255,255,255,0.2);
}
</style>
