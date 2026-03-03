import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { User, AuthResponse } from '../types/auth'
import api from '../api'

const TOKEN_KEY = 'music_auth_token'
const USER_KEY = 'music_user'

function getStoredUser(): User | null {
  try {
    const stored = localStorage.getItem(USER_KEY)
    return stored ? JSON.parse(stored) : null
  } catch {
    return null
  }
}

export const useAuthStore = defineStore('auth', () => {
  const token = ref<string | null>(localStorage.getItem(TOKEN_KEY))
  const user = ref<User | null>(getStoredUser())
  const loading = ref(false)
  const error = ref<string | null>(null)

  const isAuthenticated = computed(() => !!token.value && !!user.value)
  const isAdmin = computed(() => user.value?.is_admin || false)

  function setAuth(authResponse: AuthResponse) {
    token.value = authResponse.token
    user.value = authResponse.user
    localStorage.setItem(TOKEN_KEY, authResponse.token)
    localStorage.setItem(USER_KEY, JSON.stringify(authResponse.user))
    // Set default auth header
    api.defaults.headers.common['Authorization'] = `Bearer ${authResponse.token}`
  }

  function clearAuth() {
    token.value = null
    user.value = null
    localStorage.removeItem(TOKEN_KEY)
    localStorage.removeItem(USER_KEY)
    delete api.defaults.headers.common['Authorization']
  }

  async function loginWithGoogle(code: string) {
    loading.value = true
    error.value = null
    try {
      const response = await api.post<AuthResponse>('/auth/google/callback', { code })
      setAuth(response.data)
      return true
    } catch (err: unknown) {
      // Try to extract a helpful error message from axios-like error
      const maybe = err as { response?: { data?: { error?: string } }; message?: string }
      error.value = maybe.response?.data?.error || maybe.message || 'Google login failed'
      return false
    } finally {
      loading.value = false
    }
  }

  async function getGoogleAuthUrl(): Promise<string | null> {
    try {
      const response = await api.get<{ url: string }>('/auth/google/url')
      return response.data.url
    } catch (err) {
      console.error('Failed to get Google auth URL:', err)
      return null
    }
  }

  async function fetchUser() {
    if (!token.value) return false
    
    try {
      const response = await api.get<User>('/me')
      user.value = response.data
      localStorage.setItem(USER_KEY, JSON.stringify(response.data))
      return true
    } catch {
      // On any error while fetching user, clear stored auth
      clearAuth()
      return false
    }
  }

  function logout() {
    clearAuth()
  }

  // Initialize auth header if token exists
  if (token.value) {
    api.defaults.headers.common['Authorization'] = `Bearer ${token.value}`
  }

  return {
    token,
    user,
    loading,
    error,
    isAuthenticated,
    isAdmin,
    loginWithGoogle,
    getGoogleAuthUrl,
    fetchUser,
    logout
  }
})
