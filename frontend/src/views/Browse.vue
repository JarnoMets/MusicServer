<template>
  <div class="browse-view">
    <header class="browse-header">
      <div class="header-content">
        <h1 v-if="activeTabId === 'music'">Library</h1>
        <h1 v-else-if="activeTabId === 'artists'">Artists</h1>
        <h1 v-else-if="activeTabId === 'playlists'">Playlists</h1>
        <h1 v-else-if="activeTabId === 'genres'">Genres</h1>
        <h1 v-else-if="activeTabId === 'streams'">Radio Streams</h1>
      </div>
      
      <nav class="tab-navigation">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          :class="['tab-pill', { active: tab.id === activeTabId }]"
          @click="activeTabId = tab.id"
        >
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
import { ref, computed, type Component } from 'vue'
import MusicTab from '../features/music/MusicTab.vue'
import ArtistsTab from '../features/artists/ArtistsTab.vue'
import PlaylistsTab from '../features/playlists/PlaylistsTab.vue'
import GenresTab from '../features/genres/GenresTab.vue'
import StreamsTab from '../features/streams/StreamsTab.vue'
import { useAuth } from '../composables/useAuth'

const { isLoggedIn } = useAuth()

interface TabDefinition {
  id: string
  label: string
  icon: string
  component: Component
}

const tabs: TabDefinition[] = [
  { id: 'music', label: 'All Tracks', icon: 'disc', component: MusicTab },
  { id: 'artists', label: 'Artists', icon: 'users', component: ArtistsTab },
  { id: 'playlists', label: 'Playlists', icon: 'list', component: PlaylistsTab },
  { id: 'genres', label: 'Genres', icon: 'tag', component: GenresTab },
  { id: 'streams', label: 'Streams', icon: 'radio', component: StreamsTab },
]

const activeTabId = ref('music')
const activeTab = computed(() => tabs.find((tab) => tab.id === activeTabId.value) || tabs[0])
</script>

<style scoped>
.browse-view {
  display: flex;
  flex-direction: column;
  gap: 32px;
}

.browse-header {
  margin-bottom: 8px;
}

.header-content h1 {
  font-size: 3rem;
  font-weight: 800;
  margin-bottom: 24px;
}

/* Spotify-style Tab Pills */
.tab-navigation {
  display: flex;
  gap: 8px;
  overflow-x: auto;
  padding-bottom: 4px;
}

.tab-pill {
  padding: 8px 16px;
  background: var(--spotify-grey);
  border: none;
  border-radius: 50px;
  color: white;
  font-weight: 600;
  font-size: 0.875rem;
  cursor: pointer;
  transition: background 0.2s;
  white-space: nowrap;
}

.tab-pill:hover:not(.active) {
  background: #333333;
}

.tab-pill.active {
  background: white;
  color: black;
}

/* Tab Content */
.tab-content {
  background: var(--surface-color);
  border-radius: 16px;
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
  .tab-navigation {
    padding: 4px;
    gap: 4px;
  }

  .tab-button {
    padding: 10px 14px;
  }

  .tab-label {
    display: none;
  }

  .tab-content {
    padding: 16px;
    border-radius: 12px;
  }
}
</style>
