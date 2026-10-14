<template>
  <div v-if="show" class="demo-banner" role="status">
    <span>Demo — changes stay in this browser only. Playback still streams from the server.</span>
    <button type="button" class="demo-banner-reset" @click="resetLocal">Reset local data</button>
  </div>
</template>

<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { isDemoHost } from '../composables/useDemoReadonly'
import { setDemoMeta } from '../demo/db'

const show = ref(false)
onMounted(() => {
  show.value = isDemoHost()
})

async function resetLocal() {
  await setDemoMeta({ hydrated: false, hydratedAt: null })
  window.location.reload()
}
</script>

<style scoped>
.demo-banner {
  position: sticky;
  top: 0;
  z-index: 4000;
  display: flex;
  flex-wrap: wrap;
  align-items: center;
  justify-content: center;
  gap: 0.75rem 1.25rem;
  padding: 0.55rem 1rem;
  background: #0d1a12;
  color: #b7e0c4;
  font-size: 0.85rem;
  border-bottom: 1px solid #1e3d2a;
  text-align: center;
}

.demo-banner-reset {
  appearance: none;
  border: 1px solid var(--spotify-green, #1ed760);
  background: transparent;
  color: var(--spotify-green, #1ed760);
  border-radius: 999px;
  padding: 0.25rem 0.75rem;
  font: inherit;
  font-size: 0.8rem;
  cursor: pointer;
}
</style>
