import { useRouter } from 'vue-router'
import { watch } from 'vue'

export function useFavicon() {
  const router = useRouter()

  const faviconMap: Record<string, string> = {
    'browse': '/favicons/music.svg',
    'genre-mapper': '/favicons/genres.svg',
    'youtube-downloader': '/favicons/downloader.svg',
    'bulk-operations': '/favicons/operations.svg',
    'upload': '/favicons/upload.svg',
  }

  const setFavicon = (routeName: string) => {
    const faviconPath = faviconMap[routeName] || '/favicons/music.svg'
    const link = document.querySelector('link[rel="icon"]') as HTMLLinkElement
    
    if (link) {
      link.href = faviconPath
      link.type = 'image/svg+xml'
    } else {
      const newLink = document.createElement('link')
      newLink.rel = 'icon'
      newLink.type = 'image/svg+xml'
      newLink.href = faviconPath
      document.head.appendChild(newLink)
    }
  }

  watch(
    () => router.currentRoute.value.name,
    (routeName) => {
      if (routeName) {
        setFavicon(routeName as string)
      }
    },
    { immediate: true }
  )

  return { setFavicon }
}
