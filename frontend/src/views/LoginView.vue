<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '../stores/auth'

const router = useRouter()
const authStore = useAuthStore()
const googleAuthUrl = ref<string | null>(null)

onMounted(async () => {
  if (authStore.isAuthenticated) {
    router.push('/')
    return
  }
  googleAuthUrl.value = await authStore.getGoogleAuthUrl()
})

const handleGoogleLogin = () => {
  if (googleAuthUrl.value) {
    window.location.href = googleAuthUrl.value
  }
}
</script>

<template>
  <div class="login-container">
    <div class="login-card">
      <div class="logo">
        <svg viewBox="0 0 24 24" fill="currentColor">
          <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm-1 14H9V8h2v8zm4 0h-2V8h2v8z"/>
        </svg>
        <div>
          <p class="page-kicker">Music Server</p>
          <h1>Sign in</h1>
        </div>
      </div>
      
      <p class="subtitle">Manage your library, playlists, genres, and DJ decks from one place.</p>

      <div v-if="authStore.error" class="error-message">
        {{ authStore.error }}
      </div>

      <button 
        v-if="googleAuthUrl"
        @click="handleGoogleLogin" 
        class="login-button google-btn"
        :disabled="authStore.loading"
        type="button"
      >
        <svg class="google-icon" viewBox="0 0 24 24">
          <path fill="#4285F4" d="M22.56 12.25c0-.78-.07-1.53-.2-2.25H12v4.26h5.92c-.26 1.37-1.04 2.53-2.21 3.31v2.77h3.57c2.08-1.92 3.28-4.74 3.28-8.09z"/>
          <path fill="#34A853" d="M12 23c2.97 0 5.46-.98 7.28-2.66l-3.57-2.77c-.98.66-2.23 1.06-3.71 1.06-2.86 0-5.29-1.93-6.16-4.53H2.18v2.84C3.99 20.53 7.7 23 12 23z"/>
          <path fill="#FBBC05" d="M5.84 14.09c-.22-.66-.35-1.36-.35-2.09s.13-1.43.35-2.09V7.07H2.18C1.43 8.55 1 10.22 1 12s.43 3.45 1.18 4.93l2.85-2.22.81-.62z"/>
          <path fill="#EA4335" d="M12 5.38c1.62 0 3.06.56 4.21 1.64l3.15-3.15C17.45 2.09 14.97 1 12 1 7.7 1 3.99 3.47 2.18 7.07l3.66 2.84c.87-2.6 3.3-4.53 6.16-4.53z"/>
        </svg>
        <span>{{ authStore.loading ? 'Signing in...' : 'Continue with Google' }}</span>
      </button>
      
      <div v-else class="loading-spinner">
        <div class="spinner"></div>
        <p>Connecting to authentication server...</p>
      </div>

      <div class="footer">
        <p>Fast browsing. Better playlists. Cleaner library management.</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.login-container {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background:
    radial-gradient(circle at 20% 20%, rgba(30, 215, 96, 0.12), transparent 35%),
    linear-gradient(135deg, #121212 0%, #080808 100%);
  color: white;
  padding: 1rem;
}

.login-card {
  background: rgba(24, 24, 24, 0.9);
  backdrop-filter: blur(10px);
  border: 1px solid rgba(255, 255, 255, 0.1);
  border-radius: 24px;
  padding: 3rem 2rem;
  width: 100%;
  max-width: 440px;
  text-align: center;
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.5);
}

.logo {
  display: flex;
  flex-direction: column;
  align-items: center;
  margin-bottom: 2rem;
}

.logo svg {
  width: 64px;
  height: 64px;
  color: #1ed760; /* Spotify Green */
  margin-bottom: 1rem;
}

.logo div {
  display: flex;
  flex-direction: column;
  gap: 6px;
  align-items: center;
}

.logo h1 {
  font-size: 2rem;
  font-weight: 800;
  letter-spacing: -1px;
  margin: 0;
}

.subtitle {
  color: #b3b3b3;
  font-size: 1rem;
  margin-bottom: 2.5rem;
  line-height: 1.6;
}

.error-message {
  background: rgba(234, 67, 53, 0.1);
  color: #ff5252;
  padding: 1rem;
  border-radius: 10px;
  margin-bottom: 1.5rem;
  font-size: 0.9rem;
  border: 1px solid rgba(234, 67, 53, 0.3);
}

.login-button {
  width: 100%;
  padding: 0.9rem;
  border-radius: 9999px;
  border: none;
  font-size: 1rem;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 0.75rem;
}

.google-btn {
  background: white;
  color: #121212;
}

.google-btn:hover:not(:disabled) {
  background: #f0f0f0;
  transform: scale(1.02);
}

.google-btn:active {
  transform: scale(0.98);
}

.google-btn:disabled {
  opacity: 0.7;
  cursor: not-allowed;
}

.google-icon {
  width: 24px;
  height: 24px;
}

.loading-spinner {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 1rem;
  color: #b3b3b3;
}

.spinner {
  width: 32px;
  height: 32px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: #1ed760;
  border-radius: 50%;
  animation: spin 1s linear infinite;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.footer {
  margin-top: 3rem;
  color: #535353;
  font-size: 0.8rem;
}

.google-btn:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
}

@media (max-width: 480px) {
  .login-card {
    padding: 2rem 1.5rem;
  }
  
  .logo h1 {
    font-size: 1.75rem;
  }
}
</style>
