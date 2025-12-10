<template>
  <div class="sync-panel">
    <h3>Library Sync</h3>
    <div>
      <input v-model="folder" placeholder="Optional folder to sync" />
      <button @click="startSync" :disabled="running">Start Sync</button>
      <button @click="cancelSync" :disabled="!running">Cancel</button>
    </div>
    <div v-if="running">
      <p>{{ status }}</p>
      <progress :value="progress" max="100"></progress>
      <p>Inserted: {{ inserted }} Failed: {{ failed }}</p>
    </div>
  </div>
</template>

<script lang="ts">
import { defineComponent, ref } from 'vue'
import { musicAPI } from '@/api/music'
import { getAPIBaseURL } from '@/utils/api'

export default defineComponent({
  name: 'SyncPanel',
  setup() {
    const folder = ref('')
    const running = ref(false)
    const status = ref('')
    const progress = ref(0)
    const inserted = ref(0)
    const failed = ref(0)
    const sessionId = ref<string | null>(null)
    let evt: EventSource | null = null

    const startSync = async () => {
      running.value = true
      status.value = 'Starting...'
      try {
        const res = await musicAPI.syncMusicFolder(folder.value || undefined)
        sessionId.value = res.data.session_id
        if (sessionId.value) {
          const apiBaseUrl = getAPIBaseURL()
          evt = new EventSource(`${apiBaseUrl}/music/sync/stream/${sessionId.value}`)
          evt.onmessage = (e) => {
            try {
              const d = JSON.parse(e.data)
              status.value = d.status || status.value
              if (d.progress !== undefined && d.progress !== null) progress.value = d.progress
              if (d.inserted_files !== undefined) inserted.value = d.inserted_files
              if (d.failed_files !== undefined) failed.value = d.failed_files
              if (d.is_cancelled) {
                stopStream()
              }
            } catch (err) {
              console.warn('Invalid SSE data', err)
            }
          }
          evt.onerror = (err) => {
            console.warn('SSE error', err)
            stopStream()
          }
        } else {
          status.value = 'Sync started (no session)'
          running.value = false
        }
      } catch (e) {
        status.value = 'Error starting sync'
        running.value = false
      }
    }

    const cancelSync = async () => {
      if (!sessionId.value) return
      try {
        await musicAPI.cancelSync(sessionId.value)
      } catch (e) {
        console.warn('Cancel request failed', e)
      }
      stopStream()
    }

    function stopStream() {
      if (evt) {
        evt.close()
        evt = null
      }
      running.value = false
      sessionId.value = null
    }

    return { folder, running, status, progress, inserted, failed, startSync, cancelSync }
  }
})
</script>

<style scoped>
.sync-panel {
  padding: 1rem;
  border: 1px solid #ddd;
  border-radius: 8px;
}
</style>
