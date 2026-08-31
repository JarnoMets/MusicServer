import { createRouter, createWebHistory } from 'vue-router'
import { useAuthStore } from '../stores/auth'
import LoginView from '../views/LoginView.vue'
import GoogleCallbackView from '../views/GoogleCallbackView.vue'

const routes = [
  {
    path: '/login',
    name: 'login',
    component: LoginView,
    meta: {
      title: 'Login',
      guest: true
    }
  },
  {
    path: '/auth/google/callback',
    name: 'google-callback',
    component: GoogleCallbackView,
    meta: {
      title: 'Authenticating...',
      guest: true
    }
  },
  {
    path: '/',
    name: 'browse',
    component: () => import('../views/Browse.vue'),
    meta: {
      title: 'Library',
      requiresAuth: true
    }
  },
  {
    path: '/admin/genres',
    name: 'genre-mapper',
    component: () => import('../features/admin/GenreMapper.vue'),
    meta: {
      title: 'Genre Manager',
      requiresAuth: true,
      requiresAdmin: true
    }
  },
  {
    path: '/admin/downloader',
    name: 'youtube-downloader',
    component: () => import('../features/admin/DownloaderTab.vue'),
    meta: {
      title: 'YouTube Downloader',
      requiresAuth: true,
      requiresAdmin: true
    }
  },
  {
    path: '/admin/bulk-operations',
    name: 'bulk-operations',
    component: () => import('../features/admin/BulkOperations.vue'),
    meta: {
      title: 'Bulk Operations',
      requiresAuth: true,
      requiresAdmin: true
    }
  },
  {
    path: '/admin/history',
    name: 'audit-history',
    component: () => import('../features/admin/HistoryTab.vue'),
    meta: {
      title: 'Change History',
      requiresAuth: true,
      requiresAdmin: true
    }
  },
  {
    path: '/admin/metadata',
    name: 'metadata-settings',
    component: () => import('../features/admin/MetadataSettings.vue'),
    meta: {
      title: 'Metadata Settings',
      requiresAuth: true,
      requiresAdmin: true
    }
  },
  {
    path: '/admin/metadata/suggestions',
    name: 'metadata-suggestions',
    component: () => import('../features/metadata/MetadataSuggestions.vue'),
    meta: {
      title: 'Metadata Suggestions',
      requiresAuth: true,
      requiresAdmin: true
    }
  },
  {
    path: '/decks',
    name: 'decks',
    component: () => import('../features/decks/DecksView.vue'),
    meta: {
      title: 'DJ Decks',
      requiresAuth: true
    }
  },
  {
    path: '/upload',
    name: 'upload',
    component: () => import('../views/Upload.vue'),
    meta: {
      title: 'Upload',
      requiresAuth: true
    }
  },
  {
    path: '/edit/:id',
    name: 'edit-track',
    component: () => import('../views/EditTrack.vue'),
    meta: {
      title: 'Edit Track',
      requiresAuth: true
    }
  },
  {
    path: '/profile/tokens',
    name: 'access-tokens',
    component: () => import('../features/tokens/TokenManager.vue'),
    meta: {
      title: 'Access Tokens',
      requiresAuth: true
    }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

// Navigation guard
router.beforeEach(async (to, _from, next) => {
  const authStore = useAuthStore()
  
  // Set window title
  document.title = (to.meta.title as string) || 'Music Server'

  const requiresAuth = to.matched.some(record => record.meta.requiresAuth)
  const requiresAdmin = to.matched.some(record => record.meta.requiresAdmin)
  const isGuest = to.matched.some(record => record.meta.guest)

  if (requiresAuth && !authStore.isAuthenticated) {
    // If we have a token but not a user, try to fetch the user
    if (authStore.token && !authStore.user) {
      const success = await authStore.fetchUser()
      if (success) {
        if (requiresAdmin && !authStore.isAdmin) {
          next('/')
        } else {
          next()
        }
      } else {
        next('/login')
      }
    } else {
      next('/login')
    }
  } else if (requiresAdmin && !authStore.isAdmin) {
    next('/')
  } else if (isGuest && authStore.isAuthenticated) {
    next('/')
  } else {
    next()
  }
})

export default router
