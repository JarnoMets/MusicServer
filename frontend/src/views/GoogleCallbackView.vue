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
    <div v-if="!error" class="loading-state">
      <div class="spinner"></div>
      <h1>Completing Sign in...</h1>
      <p>Finalizing your secure connection</p>
    </div>
    <div v-else class="error-state">
      <svg class="error-icon" viewBox="0 0 24 24" fill="none" stroke="currentColor">
        <circle cx="12" cy="12" r="10" stroke-width="2"/>
        <line x1="12" y1="8" x2="12" y2="12" stroke-width="2" stroke-linecap="round"/>
        <line x1="12" y1="16" x2="12.01" y2="16" stroke-width="2" stroke-linecap="round"/>
      </svg>
      <h1>Authentication Failed</h1>
      <p>{{ error }}</p>
      <router-link to="/login" class="retry-link">Return to Login</router-link>
    </div>
  </div>
</template>

<style scoped>
.callback-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #121212;
  color: white;
  padding: 1rem;
  text-align: center;
}

.loading-state, .error-state {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1.5rem;
  max-width: 400px;
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
  font-size: 1.1rem;
  margin: 0;
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
  border-radius: 50px;
  transition: all 0.2s ease;
}

.retry-link:hover {
  background: rgba(30, 215, 96, 0.1);
}
</style>
