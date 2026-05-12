<template>
  <div class="app-layout" :class="{ 'is-logged-in': isAuthenticated }">
    <!-- Login view doesn't get the sidebar/header layout -->
    <router-view v-if="!isAuthenticated && $route.name === 'login'" />
    <router-view v-else-if="!isAuthenticated && $route.name === 'google-callback'" />
    
    <template v-else>
      <!-- Main Content Area -->
      <div class="app-container">
        <!-- Persistent Sidebar -->
        <aside class="app-sidebar hide-md">
          <div class="sidebar-nav">
            <router-link to="/" class="sidebar-item logo">
              <Icon name="music" :size="28" color="var(--spotify-green)" />
              <span class="logo-text">Music Server</span>
            </router-link>
            
            <div class="nav-section">
              <router-link to="/" class="sidebar-item">
                <Icon name="home" :size="24" />
                <span>Home</span>
              </router-link>
              <router-link to="/" class="sidebar-item" :class="{ 'active': $route.name === 'browse' }">
                <Icon name="disc" :size="24" />
                <span>Library</span>
              </router-link>
              <router-link to="/decks" class="sidebar-item" :class="{ 'active': $route.name === 'decks' }">
                <Icon name="headphones" :size="24" />
                <span>Decks</span>
              </router-link>
              <router-link to="/upload" class="sidebar-item">
                <Icon name="upload" :size="24" />
                <span>Upload</span>
              </router-link>
            </div>

            <div v-if="isAdmin" class="nav-section">
              <div class="sidebar-header">Admin</div>
              <router-link to="/admin/genres" class="sidebar-item">
                <Icon name="tag" :size="20" />
                <span>Genre Manager</span>
              </router-link>
              <router-link to="/admin/downloader" class="sidebar-item">
                <Icon name="download" :size="20" />
                <span>YouTube Downloader</span>
              </router-link>
              <router-link to="/admin/bulk-operations" class="sidebar-item">
                <Icon name="zap" :size="20" />
                <span>Bulk Operations</span>
              </router-link>
                            <router-link to="/admin/history" class="sidebar-item">
                <Icon name="history" :size="20" />
                <span>History</span>
              </router-link>
              <router-link to="/admin/metadata" class="sidebar-item">
                <Icon name="settings" :size="20" />
                <span>Metadata Settings</span>
              </router-link>
              <router-link to="/admin/metadata/suggestions" class="sidebar-item">
                <Icon name="sparkles" :size="20" />
                <span>Suggestions</span>
              </router-link>
            </div>
          </div>

          <div class="sidebar-footer">
            <GlobalDownloadTracker v-if="isAdmin" />
          </div>
        </aside>

        <div class="main-wrapper">
          <header class="main-header" :class="{ 'scrolled': isScrolled }">
            <div class="header-nav-btns">
              <button class="nav-btn" @click="$router.back()"><Icon name="chevron-left" :size="24" /></button>
              <button class="nav-btn" @click="$router.forward()"><Icon name="chevron-right" :size="24" /></button>
            </div>

            <div class="header-actions">
              <ThemeSelector />
              <div class="user-menu" v-if="user">
                <div class="user-profile" @click="showUserMenu = !showUserMenu">
                  <img v-if="user.avatar_url" :src="user.avatar_url" alt="Avatar" class="avatar" />
                  <div v-else class="avatar-ph">{{ user.name.charAt(0) }}</div>
                  <span class="user-name show-lg">{{ user.name }}</span>
                  <Icon name="chevron-down" :size="16" />
                </div>
                
                <Transition name="fade">
                  <div v-if="showUserMenu" class="user-dropdown" @mouseleave="showUserMenu = false">
                    <div class="dropdown-header">
                      <strong>{{ user.name }}</strong>
                      <span>{{ user.email }}</span>
                    </div>
                    <div class="dropdown-divider"></div>
                    <router-link class="dropdown-btn" to="/profile/tokens" @click="showUserMenu = false">Access Tokens</router-link>
                    <button class="dropdown-btn" @click="handleLogout">Log out</button>
                  </div>
                </Transition>
              </div>
            </div>
          </header>

          <main class="content-area" ref="scrollContainer" @scroll="handleScroll">
            <router-view />
          </main>
        </div>
      </div>

      <!-- Player Bar (Persistent) — hidden on DJ Decks page -->
      <footer v-if="$route.name !== 'decks'" class="app-player">
        <GlobalPlayer />
      </footer>

      <!-- Mobile Bottom Nav -->
      <nav class="mobile-nav show-md">
        <router-link to="/" class="mobile-nav-item">
          <Icon name="home" :size="24" />
          <span>Home</span>
        </router-link>
        <router-link to="/" class="mobile-nav-item">
          <Icon name="disc" :size="24" />
          <span>Library</span>
        </router-link>
        <router-link to="/decks" class="mobile-nav-item">
          <Icon name="headphones" :size="24" />
          <span>Decks</span>
        </router-link>
        <router-link to="/upload" class="mobile-nav-item">
          <Icon name="upload" :size="24" />
          <span>Upload</span>
        </router-link>
        <button class="mobile-nav-item" @click="showMobileAdmin = !showMobileAdmin" v-if="isAdmin">
          <Icon name="settings" :size="24" />
          <span>Admin</span>
        </button>
      </nav>

      <Transition name="slide-up">
        <div v-if="showMobileAdmin" class="mobile-admin-overlay" @click.self="showMobileAdmin = false">
          <div class="mobile-admin-menu">
            <div class="menu-header">Admin Menu</div>
            <router-link to="/admin/genres" class="menu-item" @click="showMobileAdmin = false">Genre Manager</router-link>
            <router-link to="/admin/downloader" class="menu-item" @click="showMobileAdmin = false">YouTube Downloader</router-link>
            <router-link to="/admin/bulk-operations" class="menu-item" @click="showMobileAdmin = false">Bulk Operations</router-link>
            <router-link to="/admin/history" class="menu-item" @click="showMobileAdmin = false">History</router-link>
            <button @click="handleLogout" class="menu-item logout">Log out</button>
          </div>
        </div>
      </Transition>
    </template>

    <ToastContainer />
    <UploadTrackerFloating />
    
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
import { ref, onMounted } from 'vue'
import { useRouter } from 'vue-router'
import { storeToRefs } from 'pinia'
import { useAuthStore } from './stores/auth'
import { useTheme } from './composables/useTheme'
import { useToast } from './composables/useToast'
import { useConfirm } from './composables/useConfirm'
import { useFavicon } from './composables/useFavicon'

import GlobalPlayer from './shared/components/GlobalPlayer.vue'
import ThemeSelector from './shared/components/ThemeSelector.vue'
import ToastContainer from './shared/components/ToastContainer.vue'
import ConfirmModal from './shared/components/ConfirmModal.vue'
import UploadTrackerFloating from './components/UploadTrackerFloating.vue'
import Icon from './shared/components/Icons.vue'
import GlobalDownloadTracker from './shared/components/GlobalDownloadTracker.vue'

// Initialize systems
useTheme()
useFavicon()
const router = useRouter()
const authStore = useAuthStore()
const { user, isAuthenticated, isAdmin } = storeToRefs(authStore)
const { success } = useToast()
const { state: confirmState, handleConfirm: handleConfirmModalConfirm, handleCancel: handleConfirmModalCancel } = useConfirm()

const showUserMenu = ref(false)
const showMobileAdmin = ref(false)
const isScrolled = ref(false)
const scrollContainer = ref<HTMLElement | null>(null)

const handleScroll = () => {
  if (scrollContainer.value) {
    isScrolled.value = scrollContainer.value.scrollTop > 10
  }
}

const handleLogout = () => {
  authStore.logout()
  router.push('/login')
  success('Logged out', 'Session ended')
}

onMounted(() => {
  if (isAuthenticated.value && !user.value) {
    authStore.fetchUser()
  }
})
</script>

<style scoped>
.app-layout {
  height: 100vh;
  display: flex;
  flex-direction: column;
  background: black;
  overflow: hidden;
}

.app-container {
  display: flex;
  flex: 1;
  overflow: hidden;
  padding: 8px;
  gap: 8px;
}

/* Sidebar */
.app-sidebar {
  width: 280px;
  background: var(--spotify-black);
  border-radius: 8px;
  display: flex;
  flex-direction: column;
  padding: 8px 0;
}

.nav-section {
  background: var(--spotify-dark-grey);
  margin: 0 8px 8px;
  padding: 12px;
  border-radius: 8px;
}

.sidebar-item {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 12px;
  color: var(--spotify-light-grey);
  text-decoration: none;
  font-weight: 700;
  transition: color 0.2s;
}

.sidebar-item:hover, .sidebar-item.router-link-active, .sidebar-item.active {
  color: white;
}

.sidebar-item svg {
  color: inherit;
}

.logo {
  padding: 16px 24px;
  margin-bottom: 8px;
}

.logo-text {
  font-size: 1.25rem;
  font-weight: 900;
  color: white;
  letter-spacing: -0.5px;
}

.sidebar-header {
  color: var(--spotify-light-grey);
  font-size: 0.75rem;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  padding: 8px 12px;
  margin-top: 8px;
}

.sidebar-footer {
  margin-top: auto;
  padding: 16px;
}

/* Main Content Wrapper */
.main-wrapper {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--spotify-dark-grey);
  border-radius: 8px;
  overflow: hidden;
  position: relative;
}

.main-header {
  height: 64px;
  padding: 0 24px;
  display: flex;
  align-items: center;
  justify-content: space-between;
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  z-index: 10;
  transition: background 0.3s;
}

.main-header.scrolled {
  background: rgba(18, 18, 18, 0.8);
  backdrop-filter: blur(20px);
}

.header-nav-btns {
  display: flex;
  gap: 8px;
}

.nav-btn {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  background: rgba(0,0,0,0.7);
  display: flex;
  align-items: center;
  justify-content: center;
  color: white;
  border: none;
  cursor: pointer;
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 16px;
}

.user-menu {
  position: relative;
}

.user-profile {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 4px 8px 4px 4px;
  background: rgba(0,0,0,0.7);
  border-radius: 20px;
  cursor: pointer;
  transition: transform 0.1s;
}

.user-profile:hover {
  transform: scale(1.02);
  background: #282828;
}

.avatar, .avatar-ph {
  width: 28px;
  height: 28px;
  border-radius: 50%;
}

.avatar-ph {
  background: var(--spotify-grey);
  display: flex;
  align-items: center;
  justify-content: center;
  font-weight: 700;
  font-size: 14px;
}

.user-name {
  font-size: 0.875rem;
  font-weight: 700;
}

.user-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  width: 200px;
  background: #282828;
  border-radius: 4px;
  padding: 4px;
  box-shadow: 0 16px 24px rgba(0,0,0,.3);
}

.dropdown-header {
  padding: 12px;
  display: flex;
  flex-direction: column;
}

.dropdown-header span {
  font-size: 0.75rem;
  color: var(--spotify-light-grey);
}

.dropdown-divider {
  height: 1px;
  background: #404040;
  margin: 4px 0;
}

.dropdown-btn {
  width: 100%;
  padding: 12px;
  text-align: left;
  background: transparent;
  color: white;
  border-radius: 2px;
  font-size: 0.875rem;
}

.dropdown-btn:hover {
  background: #3e3e3e;
}

/* Content Area */
.content-area {
  flex: 1;
  overflow-y: auto;
  padding: 64px 24px 24px;
  background: linear-gradient(to bottom, #1e1e1e 0%, #121212 300px);
}

/* Full-height mode for DJ Decks (no extra padding, maximize space) */
.content-area:has(.decks-view) {
  padding: 64px 8px 8px;
  overflow: hidden;
}

/* No player footer on decks, so remove the gap */
.app-layout:has(.decks-view) .app-player {
  display: none;
}

/* Player Bar */
.app-player {
  height: 90px;
  background: black;
  display: flex;
  align-items: center;
}

/* Mobile Nav */
.mobile-nav {
  height: 64px;
  background: black;
  display: flex;
  border-top: 1px solid #282828;
}

.mobile-nav-item {
  flex: 1;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  gap: 4px;
  color: var(--spotify-light-grey);
  text-decoration: none;
  font-size: 10px;
}

.mobile-nav-item.router-link-active {
  color: white;
}

.mobile-admin-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.7);
  z-index: 1000;
  display: flex;
  align-items: flex-end;
}

.mobile-admin-menu {
  width: 100%;
  background: #282828;
  padding: 16px;
  border-radius: 12px 12px 0 0;
}

.menu-header {
  font-weight: 700;
  padding: 16px;
  text-align: center;
  border-bottom: 1px solid #404040;
}

.menu-item {
  display: block;
  width: 100%;
  padding: 16px;
  text-decoration: none;
  color: white;
  background: transparent;
  text-align: left;
  font-size: 1.1rem;
}

.menu-item.logout {
  color: #ea4335;
}

/* Utils */
.hide-md {
  @media (max-width: 1024px) { display: none !important; }
}

.show-md {
  @media (min-width: 1025px) { display: none !important; }
}

.show-lg {
  @media (max-width: 1280px) { display: none !important; }
}

.fade-enter-active, .fade-leave-active { transition: opacity 0.2s; }
.fade-enter-from, .fade-leave-to { opacity: 0; }

.slide-up-enter-active, .slide-up-leave-active { transition: transform 0.3s ease-out; }
.slide-up-enter-from, .slide-up-leave-to { transform: translateY(100%); }
</style>
