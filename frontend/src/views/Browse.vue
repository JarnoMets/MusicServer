<template>
  <div class="browse-view">
    <header class="browse-header">
      <div class="browse-hero">
        <p class="browse-kicker">Your library</p>
        <h1>{{ activeTab?.label }}</h1>
        <p class="browse-subtitle">{{ activeTab?.description }}</p>
      </div>
      <nav class="tab-navigation">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="['tab-pill', { active: tab.id === activeTabId }]"
          :aria-pressed="tab.id === activeTabId"
          type="button"
          @click="activeTabId = tab.id"
        >
          <Icon :name="tab.icon" :size="16" />
          <span class="tab-label">{{ tab.label }}</span>
        </button>
      </nav>
    </header>

    <section class="tab-content">
      <keep-alive>
        <component 
          :is="activeTab?.component" 
          :key="activeTabId"
          :can-edit="isLoggedIn"
        />
      </keep-alive>
    </section>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, type Component } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import MusicTab from '../features/music/MusicTab.vue'
import ArtistsTab from '../features/artists/ArtistsTab.vue'
import PlaylistsTab from '../features/playlists/PlaylistsTab.vue'
import GenresTab from '../features/genres/GenresTab.vue'
import StreamsTab from '../features/streams/StreamsTab.vue'
import { useAuth } from '../composables/useAuth'
import Icon from '../shared/components/Icons.vue'

const { isLoggedIn } = useAuth()
const route = useRoute()
const router = useRouter()

interface TabDefinition {
  id: string
  label: string
  icon: string
  description: string
  component: Component
}

const tabs: TabDefinition[] = [
  { id: 'music', label: 'All Tracks', icon: 'disc', description: 'Browse, search, and play your full library', component: MusicTab },
  { id: 'artists', label: 'Artists', icon: 'users', description: 'Organize your library by artist', component: ArtistsTab },
  { id: 'playlists', label: 'Playlists', icon: 'list', description: 'Build and reorder collections', component: PlaylistsTab },
  { id: 'genres', label: 'Genres', icon: 'tag', description: 'Explore music by genre', component: GenresTab },
  { id: 'streams', label: 'Streams', icon: 'radio', description: 'Switch between radio sources', component: StreamsTab },
]

const normalizeTabId = (value: unknown) => {
  if (typeof value !== 'string') return 'music'
  return tabs.some((tab) => tab.id === value) ? value : 'music'
}

const activeTabId = ref(normalizeTabId(route.query.tab))
const activeTab = computed(() => tabs.find((tab) => tab.id === activeTabId.value) || tabs[0])

watch(
  () => route.query.tab,
  (tab) => {
    const nextTab = normalizeTabId(tab)
    if (nextTab !== activeTabId.value) {
      activeTabId.value = nextTab
    }
  },
)

watch(activeTabId, (tab) => {
  if (route.query.tab !== tab) {
    router.replace({ query: { ...route.query, tab } })
  }
})
</script>

<style scoped>
.browse-view {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.browse-header {
  padding: 24px;
  border: 1px solid var(--border-color);
  border-radius: 24px;
  background: linear-gradient(135deg, rgba(29, 185, 84, 0.16), rgba(18, 18, 18, 0.92) 45%, rgba(24, 24, 24, 0.96));
  box-shadow: 0 20px 50px rgba(0, 0, 0, 0.25);
}

.browse-hero {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 20px;
}

.browse-kicker {
  margin: 0;
  color: var(--primary-light);
  font-size: 0.75rem;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.14em;
}

.browse-hero h1 {
  font-size: 3rem;
  font-weight: 800;
  margin: 0;
}

.browse-subtitle {
  margin: 0;
  color: var(--text-secondary);
  max-width: 56ch;
}

.tab-navigation {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.tab-pill {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: rgba(255, 255, 255, 0.06);
  border: 1px solid transparent;
  border-radius: 9999px;
  color: var(--text-secondary);
  font-weight: 600;
  font-size: 0.875rem;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.tab-pill:hover:not(.active) {
  background: rgba(255, 255, 255, 0.1);
  color: var(--text-color);
}

.tab-pill.active {
  background: var(--primary-color);
  color: #081b0f;
  border-color: var(--primary-color);
  box-shadow: 0 10px 24px rgba(30, 215, 96, 0.25);
}

.tab-pill:focus-visible {
  outline: 2px solid var(--primary-color);
  outline-offset: 2px;
}

/* Tab Content */
.tab-content {
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 24px;
  padding: 24px;
  min-height: 400px;
}

/* Fade Transition */
.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.15s ease;
}

.fade-enter-from,
.fade-leave-to {
  opacity: 0;
}

@media (max-width: 768px) {
  .browse-header {
    padding: 18px;
    border-radius: 20px;
  }

  .browse-hero h1 {
    font-size: 2rem;
  }

  .tab-navigation {
    padding: 4px;
    gap: 4px;
  }

  .tab-pill {
    padding: 9px 12px;
  }

  .tab-label {
    font-size: 0.8rem;
  }

  .tab-content {
    padding: 16px;
    border-radius: 18px;
  }
}
</style>
