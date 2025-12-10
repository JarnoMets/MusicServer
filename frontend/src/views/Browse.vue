<template>
  <div class="browse-view">
    <!-- Tab Navigation -->
    <nav class="tab-navigation">
      <button
        v-for="tab in tabs"
        :key="tab.id"
        :class="['tab-button', { active: tab.id === activeTabId }]"
        @click="activeTabId = tab.id"
      >
        <Icon :name="tab.icon" :size="18" />
        <span class="tab-label">{{ tab.label }}</span>
      </button>
    </nav>

    <!-- Tab Content -->
    <section class="tab-content">
      <Transition name="fade" mode="out-in">
        <keep-alive>
          <component 
            :is="activeTab?.component" 
            :key="activeTabId"
            :can-edit="isLoggedIn"
          />
        </keep-alive>
      </Transition>
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
import Icon from '../shared/components/Icons.vue'
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
  gap: 24px;
}

/* Tab Navigation */
.tab-navigation {
  display: flex;
  gap: 6px;
  padding: 6px;
  background: var(--surface-color);
  border-radius: 12px;
  overflow-x: auto;
}

.tab-navigation::-webkit-scrollbar {
  height: 0;
}

.tab-button {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 20px;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--text-secondary);
  font-weight: 500;
  font-size: 14px;
  cursor: pointer;
  transition: all 0.2s ease;
  white-space: nowrap;
}

.tab-button:hover:not(.active) {
  color: var(--text-color);
  background: var(--background-elevated);
}

.tab-button.active {
  background: var(--primary-color);
  color: #fff;
}

.tab-label {
  font-size: 14px;
  font-weight: 500;
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
