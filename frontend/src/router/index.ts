import { createRouter, createWebHistory } from 'vue-router'

const routes = [
  {
    path: '/',
    name: 'browse',
    component: () => import('../views/Browse.vue')
  },
  {
    path: '/admin/genres',
    name: 'genre-mapper',
    component: () => import('../features/admin/GenreMapper.vue')
  },
  {
    path: '/admin/downloader',
    name: 'youtube-downloader',
    component: () => import('../features/admin/DownloaderTab.vue')
  },
  {
    path: '/upload',
    name: 'upload',
    component: () => import('../views/Upload.vue')
  }
]

const router = createRouter({
  history: createWebHistory(),
  routes
})

export default router
