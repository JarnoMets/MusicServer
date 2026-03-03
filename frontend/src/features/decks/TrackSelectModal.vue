<script setup lang="ts">
import TrackBrowser from '../../features/decks/TrackBrowser.vue'
import type { DeckId } from '../../types/dj'
import Icon from '../../shared/components/Icons.vue'

defineProps<{
  show: boolean
  deckId: DeckId
}>()

const emit = defineEmits<{
  close: []
}>()

const handleClose = () => {
  emit('close')
}

const handleBackdropClick = (e: MouseEvent) => {
  if (e.target === e.currentTarget) {
    handleClose()
  }
}

const handleLoaded = () => {
  // Optionally close modal after load, or keep open
  handleClose()
}
</script>

<template>
  <Teleport to="body">
    <Transition name="modal">
      <div v-if="show" class="modal-backdrop" @click="handleBackdropClick">
        <div class="modal-content">
          <div class="modal-header">
            <h3>Load Track to Deck {{ deckId }}</h3>
            <button class="btn-close" @click="handleClose">
              <Icon name="x" :size="20" />
            </button>
          </div>
          <div class="modal-body">
            <TrackBrowser :target-deck-id="deckId" @loaded="handleLoaded" />
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
  z-index: 5000;
  padding: 20px;
}

.modal-content {
  background: var(--surface-color, #1a1a2e);
  border: 1px solid var(--border-color, #2a2a3e);
  border-radius: 12px;
  width: 100%;
  max-width: 800px;
  height: 80vh;
  display: flex;
  flex-direction: column;
  box-shadow: 0 20px 60px rgba(0,0,0,0.5);
}

.modal-header {
  padding: 16px;
  border-bottom: 1px solid rgba(255,255,255,0.1);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.modal-header h3 {
  margin: 0;
  font-size: 18px;
  color: var(--text-color, #fff);
}

.btn-close {
  background: none;
  border: none;
  color: var(--text-secondary, #aaa);
  cursor: pointer;
  padding: 4px;
}

.btn-close:hover {
  color: white;
}

.modal-body {
  flex: 1;
  overflow: hidden;
  padding: 16px;
  display: flex;
  flex-direction: column;
}

/* Animations */
.modal-enter-active,
.modal-leave-active {
  transition: opacity 0.2s ease;
}

.modal-enter-from,
.modal-leave-to {
  opacity: 0;
}

.modal-enter-active .modal-content,
.modal-leave-active .modal-content {
  transition: transform 0.2s ease;
}

.modal-enter-from .modal-content,
.modal-leave-to .modal-content {
  transform: scale(0.95);
}
</style>
