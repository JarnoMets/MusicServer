import { ref, computed, readonly } from 'vue'
import { setAdminToken } from '../api/music'

// Module-level singleton state
const token = ref<string | null>(localStorage.getItem('adminToken'))
const isLoginModalOpen = ref(false)
const loginError = ref<string | null>(null)
const isLoggingIn = ref(false)

// Initialize token from storage on load
if (token.value) {
  setAdminToken(token.value)
}

export function useAuth() {
  const isLoggedIn = computed(() => !!token.value)

  const login = async (inputToken: string): Promise<boolean> => {
    isLoggingIn.value = true
    loginError.value = null

    try {
      // Test the token by making an authenticated request
      setAdminToken(inputToken)
      
      // Try to access an admin endpoint to verify token
      const response = await fetch('/api/admin/genres', {
        method: 'GET',
        headers: {
          'Authorization': `Bearer ${inputToken}`,
        },
      })

      if (response.ok || response.status === 404) {
        // Token is valid (404 just means no genres yet, but auth passed)
        token.value = inputToken
        localStorage.setItem('adminToken', inputToken)
        isLoginModalOpen.value = false
        return true
      } else if (response.status === 401) {
        setAdminToken(null)
        loginError.value = 'Invalid token. Please check your admin token and try again.'
        return false
      } else {
        setAdminToken(null)
        loginError.value = 'Authentication failed. Please try again.'
        return false
      }
    } catch (error) {
      setAdminToken(null)
      loginError.value = 'Connection error. Please check your network and try again.'
      return false
    } finally {
      isLoggingIn.value = false
    }
  }

  const logout = () => {
    token.value = null
    localStorage.removeItem('adminToken')
    setAdminToken(null)
  }

  const openLoginModal = () => {
    loginError.value = null
    isLoginModalOpen.value = true
  }

  const closeLoginModal = () => {
    isLoginModalOpen.value = false
    loginError.value = null
  }

  return {
    // State
    isLoggedIn,
    isLoginModalOpen: readonly(isLoginModalOpen),
    loginError: readonly(loginError),
    isLoggingIn: readonly(isLoggingIn),
    
    // Actions
    login,
    logout,
    openLoginModal,
    closeLoginModal,
  }
}
