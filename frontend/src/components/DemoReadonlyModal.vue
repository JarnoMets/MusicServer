<template>
  <div
    v-if="visible"
    class="modal-overlay"
    role="dialog"
    aria-modal="true"
    aria-labelledby="demo-readonly-title"
    @click.self="dismissDemoReadonly"
  >
    <div class="modal demo-modal">
      <p class="demo-eyebrow">Read-only demo</p>
      <h2 id="demo-readonly-title">This action isn’t available here</h2>
      <p>
        The public demo is browse-only. Uploads, edits, playlist changes, and
        other write actions are blocked on this host.
      </p>
      <div class="modal-actions">
        <button type="button" class="btn-primary" @click="dismissDemoReadonly">
          Got it
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useDemoReadonly } from '../composables/useDemoReadonly'

const { visible, dismissDemoReadonly } = useDemoReadonly()
</script>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.65);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 3000;
  backdrop-filter: blur(4px);
  padding: 1rem;
}

.modal {
  background: #181818;
  border: 1px solid rgba(30, 215, 96, 0.35);
  border-radius: 14px;
  padding: 1.5rem;
  max-width: 28rem;
  width: 100%;
  box-shadow: 0 24px 48px rgba(0, 0, 0, 0.45);
}

.demo-eyebrow {
  margin: 0 0 0.35rem;
  font-size: 0.75rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--spotify-green, #1ed760);
}

.demo-modal h2 {
  margin: 0 0 0.75rem;
  color: #fff;
}

.demo-modal p {
  margin: 0 0 1.25rem;
  color: #b3b3b3;
  line-height: 1.55;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
}

.btn-primary {
  appearance: none;
  border: none;
  border-radius: 999px;
  padding: 0.65rem 1.25rem;
  background: var(--spotify-green, #1ed760);
  color: #121212;
  font: inherit;
  font-weight: 600;
  cursor: pointer;
}

.btn-primary:hover {
  filter: brightness(1.08);
}
</style>
