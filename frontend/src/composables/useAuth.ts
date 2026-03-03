import { computed } from 'vue'
import { useAuthStore } from '../stores/auth'

export function useAuth() {
  const authStore = useAuthStore()
  
  const isLoggedIn = computed(() => authStore.isAuthenticated)
  const isAdmin = computed(() => authStore.isAdmin)
  const token = computed(() => authStore.token)

  const login = async (code: string): Promise<boolean> => {
    return authStore.loginWithGoogle(code)
  }

  const logout = () => {
    authStore.logout()
  }

  const openLoginModal = () => {
    // Redirect to login page instead of opening modal
    window.location.href = '/login'
  }

  const closeLoginModal = () => {
    // No-op for now
  }

  return {
    isLoggedIn,
    isAdmin,
    token,
    login,
    logout,
    openLoginModal,
    closeLoginModal,
  }
}
