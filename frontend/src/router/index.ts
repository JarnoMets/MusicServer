import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'browse',
    component: () => import('../views/Browse.vue'),
    meta: {
      title: 'Library',
      favicon: '🎵'
    }
  },
  {
    path: '/admin/genres',
    name: 'genre-mapper',
    component: () => import('../features/admin/GenreMapper.vue'),
    meta: {
      title: 'Genre Manager',
      favicon: '🏷️'
    }
  },
  {
    path: '/admin/downloader',
    name: 'youtube-downloader',
    component: () => import('../features/admin/DownloaderTab.vue'),
    meta: {
      title: 'YouTube Downloader',
      favicon: '📥'
    }
  },
  {
    path: '/admin/bulk-operations',
    name: 'bulk-operations',
    component: () => import('../features/admin/BulkOperations.vue'),
    meta: {
      title: 'Bulk Operations',
      favicon: '⚡'
    }
  },
  {
    path: '/upload',
    name: 'upload',
    component: () => import('../views/Upload.vue'),
    meta: {
      title: 'Upload',
      favicon: '📤'
    }
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
