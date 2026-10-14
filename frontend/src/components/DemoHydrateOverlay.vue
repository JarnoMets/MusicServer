<template>
  <div v-if="visible" class="hydrate-overlay">
    <div class="hydrate-card">
      <p class="eyebrow">Local demo</p>
      <h2>Preparing library</h2>
      <p>{{ phase }}</p>
      <p v-if="error" class="err">{{ error }}</p>
    </div>
  </div>
</template>

<script setup lang="ts">
defineProps<{ visible: boolean; phase: string; error?: string }>()
</script>

<style scoped>
.hydrate-overlay {
  position: fixed;
  inset: 0;
  z-index: 5000;
  display: grid;
  place-items: center;
  background: rgba(0, 0, 0, 0.72);
  backdrop-filter: blur(4px);
  padding: 1rem;
}
.hydrate-card {
  background: #181818;
  border: 1px solid rgba(30, 215, 96, 0.35);
  border-radius: 14px;
  padding: 1.5rem;
  max-width: 24rem;
  width: 100%;
  color: #fff;
}
.eyebrow {
  margin: 0 0 0.35rem;
  font-size: 0.75rem;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--spotify-green, #1ed760);
}
h2 { margin: 0 0 0.5rem; font-size: 1.25rem; }
p { margin: 0; color: #b3b3b3; }
.err { margin-top: 0.75rem; color: #f07178; }
</style>
