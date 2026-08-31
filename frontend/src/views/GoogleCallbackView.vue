<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter, useRoute } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const route = useRoute()
const authStore = useAuthStore()
const error = ref<string | null>(null)

onMounted(async () => {
  const code = route.query.code as string
  if (!code) {
    error.value = 'No authorization code found'
    setTimeout(() => router.push('/login'), 3000)
    return
  }

  const success = await authStore.loginWithGoogle(code)
  if (success) {
    router.push('/')
  } else {
    error.value = authStore.error || 'Failed to authenticate with Google'
    setTimeout(() => router.push('/login'), 5000)
  }
})
</script>

<template>
  <div class="callback-container">
    <div v-if="!error" class="callback-card">
      <div class="loading-state">
        <div class="spinner"></div>
        <p class="page-kicker">Music Server</p>
        <h1>Completing sign in</h1>
        <p>Finalizing your secure connection and loading your library.</p>
      </div>
    </div>
    <div v-else class="callback-card">
      <div class="error-state">
        <svg class="error-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
          <circle cx="12" cy="12" r="10" stroke-width="2"/>
          <line x1="12" y1="8" x2="12" y2="12" stroke-width="2" stroke-linecap="round"/>
          <line x1="12" y1="16" x2="12.01" y2="16" stroke-width="2" stroke-linecap="round"/>
        </svg>
        <h1>Authentication failed</h1>
        <p>{{ error }}</p>
        <router-link to="/login" class="retry-link">Return to login</router-link>
      </div>
    </div>
  </div>
</template>

<style scoped>
.callback-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    radial-gradient(circle at 20% 20%, rgba(30, 215, 96, 0.12), transparent 35%),
    linear-gradient(135deg, #121212 0%, #080808 100%);
  color: white;
  padding: 1rem;
  text-align: center;
}

.callback-card {
  width: 100%;
  max-width: 440px;
  background: rgba(24, 24, 24, 0.92);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 24px;
  padding: 3rem 2rem;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.loading-state, .error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
}

.spinner {
  width: 48px;
  height: 48px;
  border: 4px solid rgba(255, 255, 255, 0.1);
  border-top-color: #1ed760;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

h1 {
  font-size: 1.75rem;
  margin: 0;
  font-weight: 700;
}

p {
  color: #b3b3b3;
  font-size: 1rem;
  margin: 0;
  line-height: 1.6;
}

.error-icon {
  width: 64px;
  height: 64px;
  color: #ea4335;
}

.retry-link {
  margin-top: 1rem;
  color: #1ed760;
  text-decoration: none;
  font-weight: 600;
  padding: 0.75rem 1.5rem;
  border: 1px solid #1ed760;
  border-radius: 9999px;
  transition: all 0.2s ease;
}

.retry-link:hover {
  background: rgba(30, 215, 96, 0.1);
}

.retry-link:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
}
</style>
