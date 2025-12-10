<template>
  <div class="app-container">
    <header class="app-header">
      <div class="header-left">
        <router-link to="/" class="logo">
          <Icon name="music" :size="28" />
          <h1 class="logo-text">Music Server</h1>
        </router-link>
      </div>
      <nav class="nav-main">
        <router-link to="/" class="nav-link">
          <Icon name="disc" :size="18" />
          <span>Library</span>
        </router-link>
        <router-link v-if="isLoggedIn" to="/upload" class="nav-link">
          <Icon name="upload" :size="18" />
          <span>Upload</span>
        </router-link>
        <div class="nav-separator"></div>
        <ThemeSelector />
        
        <!-- Admin gear icon (hidden until hover/click) -->
        <div class="admin-menu-wrapper" @mouseenter="showAdminMenu = true" @mouseleave="showAdminMenu = false">
          <button 
            class="admin-gear-btn" 
            :class="{ 'is-visible': showAdminMenu || isLoggedIn }"
            @click="handleAdminClick"
            :title="isLoggedIn ? 'Admin menu' : 'Admin login'"
          >
            <Icon name="settings" :size="20" />
          </button>
          
          <!-- Admin dropdown menu -->
          <Transition name="dropdown">
            <div v-if="showAdminMenu && isLoggedIn" class="admin-dropdown">
              <router-link to="/admin/genres" class="dropdown-item" @click="showAdminMenu = false">
                <Icon name="tag" :size="16" />
                <span>Genre Manager</span>
              </router-link>
              <router-link to="/admin/downloader" class="dropdown-item" @click="showAdminMenu = false">
                <Icon name="download" :size="16" />
                <span>YouTube Downloader</span>
              </router-link>
              <router-link to="/admin/bulk-operations" class="dropdown-item" @click="showAdminMenu = false">
                <Icon name="zap" :size="16" />
                <span>Bulk Operations</span>
              </router-link>
              <div class="dropdown-separator"></div>
              <button class="dropdown-item logout" @click="handleLogout">
                <Icon name="log-out" :size="16" />
                <span>Logout</span>
              </button>
            </div>
          </Transition>
        </div>
      </nav>
    </header>
    <main class="app-main">
      <router-view />
    </main>
    <ToastContainer />
    <GlobalPlayer />
    <UploadTrackerFloating />
    
    <!-- Login Modal -->
    <LoginModal
      :is-open="isLoginModalOpen"
      :error="loginError"
      :is-loading="isLoggingIn"
      @close="closeLoginModal"
      @submit="handleLogin"
    />
    
    <!-- Global Confirm Modal -->
    <ConfirmModal
      :show="confirmState.show"
      :title="confirmState.title"
      :message="confirmState.message"
      :confirm-text="confirmState.confirmText"
      :cancel-text="confirmState.cancelText"
      :variant="confirmState.variant"
      @confirm="handleConfirmModalConfirm"
      @cancel="handleConfirmModalCancel"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { useRouter } from 'vue-router'
import GlobalPlayer from './shared/components/GlobalPlayer.vue'
import ThemeSelector from './shared/components/ThemeSelector.vue'
import ToastContainer from './shared/components/ToastContainer.vue'
import LoginModal from './shared/components/LoginModal.vue'
import ConfirmModal from './shared/components/ConfirmModal.vue'
import UploadTrackerFloating from './components/UploadTrackerFloating.vue'
import Icon from './shared/components/Icons.vue'
import { useTheme } from './composables/useTheme'
import { useAuth } from './composables/useAuth'
import { useToast } from './composables/useToast'
import { useConfirm } from './composables/useConfirm'

// Initialize theme system
useTheme()

// Handle dynamic page titles and favicons
const router = useRouter()

const updatePageTitle = (title?: string, favicon?: string) => {
  const finalTitle = title ? `${title} - Music Server` : 'Music Server'
  document.title = finalTitle

  // Update favicon using emoji or dynamic approach
  if (favicon) {
    updateFavicon(favicon)
  }
}

const updateFavicon = (emoji: string) => {
  const canvas = document.createElement('canvas')
  canvas.width = 64
  canvas.height = 64
  const ctx = canvas.getContext('2d')
  if (ctx) {
    ctx.fillStyle = 'rgba(0, 0, 0, 0)'
    ctx.fillRect(0, 0, 64, 64)
    ctx.font = '48px Arial'
    ctx.fillText(emoji, 8, 52)
    const link = document.querySelector("link[rel*='icon']") as HTMLLinkElement || document.createElement('link')
    link.type = 'image/x-icon'
    link.rel = 'shortcut icon'
    link.href = canvas.toDataURL()
    document.head.appendChild(link)
  }
}

// Watch for route changes
watch(
  () => router.currentRoute.value,
  (route) => {
    const meta = route.meta as { title?: string; favicon?: string } | undefined
    updatePageTitle(meta?.title, meta?.favicon)
  },
  { immediate: true }
)

// Auth
const { 
  isLoggedIn, 
  isLoginModalOpen, 
  loginError, 
  isLoggingIn,
  login, 
  logout, 
  openLoginModal, 
  closeLoginModal 
} = useAuth()
const { success } = useToast()
const { state: confirmState, handleConfirm: handleConfirmModalConfirm, handleCancel: handleConfirmModalCancel } = useConfirm()

const showAdminMenu = ref(false)

const handleAdminClick = () => {
  if (isLoggedIn.value) {
    showAdminMenu.value = !showAdminMenu.value
  } else {
    openLoginModal()
  }
}

const handleLogin = async (token: string) => {
  const result = await login(token)
  if (result) {
    success('Logged in', 'You now have admin access')
  }
}

const handleLogout = () => {
  logout()
  showAdminMenu.value = false
  success('Logged out', 'Admin session ended')
}
</script>

<style scoped>
.app-container {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
  background: var(--background-color);
}

.app-header {
  background: var(--surface-color);
  border-bottom: 1px solid var(--border-color);
  padding: 0 32px;
  height: 64px;
  display: flex;
  justify-content: space-between;
  align-items: center;
  position: sticky;
  top: 0;
  z-index: 100;
  backdrop-filter: blur(12px);
}

.header-left {
  display: flex;
  align-items: center;
}

.logo {
  display: flex;
  align-items: center;
  gap: 12px;
  text-decoration: none;
  color: var(--primary-color);
  transition: opacity 0.2s;
}

.logo:hover {
  opacity: 0.8;
}

.logo-text {
  font-size: 20px;
  font-weight: 700;
  background: linear-gradient(120deg, var(--primary-color) 0%, var(--accent-color) 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
  margin: 0;
  letter-spacing: -0.3px;
}

.nav-main {
  display: flex;
  gap: 4px;
  align-items: center;
}

.nav-separator {
  width: 1px;
  height: 24px;
  background: var(--border-color);
  margin: 0 12px;
}

.nav-link {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 16px;
  color: var(--text-secondary);
  text-decoration: none;
  border-radius: 8px;
  font-weight: 500;
  font-size: 14px;
  transition: all 0.2s;
  border: 1px solid transparent;
}

.nav-link:hover {
  color: var(--text-color);
  background: var(--surface-hover);
}

.nav-link.router-link-active {
  color: var(--primary-light);
  background: var(--primary-glow, rgba(139, 92, 246, 0.15));
  border-color: var(--primary-color);
}

/* Admin gear button and dropdown */
.admin-menu-wrapper {
  position: relative;
}

.admin-gear-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 40px;
  height: 40px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: 10px;
  color: var(--text-tertiary);
  cursor: pointer;
  transition: all 0.3s ease;
  opacity: 0.3;
}

.admin-gear-btn:hover,
.admin-gear-btn.is-visible {
  opacity: 1;
  color: var(--text-color);
  background: var(--surface-hover);
  border-color: var(--border-color);
}

.admin-gear-btn:hover {
  transform: rotate(45deg);
}

.admin-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  min-width: 200px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 8px;
  box-shadow: 0 12px 40px rgba(0, 0, 0, 0.3);
  z-index: 200;
}

.dropdown-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 10px 14px;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--text-color);
  font-size: 14px;
  font-weight: 500;
  text-decoration: none;
  cursor: pointer;
  transition: all 0.2s ease;
}

.dropdown-item:hover {
  background: var(--primary-glow);
  color: var(--primary-light);
}

.dropdown-item.logout {
  color: #f87171;
}

.dropdown-item.logout:hover {
  background: rgba(239, 68, 68, 0.1);
  color: #f87171;
}

.dropdown-separator {
  height: 1px;
  background: var(--border-color);
  margin: 8px 0;
}

/* Dropdown transition */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px);
}

.app-main {
  flex: 1;
  padding: 32px;
  width: 100%;
  box-sizing: border-box;
  padding-bottom: 120px;
}

@media (max-width: 768px) {
  .app-header {
    padding: 0 16px;
    height: auto;
    flex-direction: column;
    gap: 12px;
    padding: 16px;
  }

  .nav-main {
    width: 100%;
    justify-content: center;
    flex-wrap: wrap;
  }

  .nav-separator {
    display: none;
  }

  .nav-link span {
    display: none;
  }

  .nav-link {
    padding: 10px 12px;
  }

  .app-main {
    padding: 16px;
    padding-bottom: 140px;
  }

  .logo-text {
    font-size: 18px;
  }
}
</style>
