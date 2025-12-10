<script setup lang="ts">
import type { MusicFilters } from '../../types/MusicTab'
import Icon from '../../shared/components/Icons.vue'

interface Props {
  filters: MusicFilters
  genres: string[]
  loading: boolean
  pageSize: number
}

defineProps<Props>()

const emit = defineEmits<{
  'update:search': [value: string]
  'update:genre': [value: string]
  'update:unconfirmedOnly': [value: boolean]
  'update:sort': [value: MusicFilters['sort']]
  'update:order': [value: MusicFilters['order']]
  'update:pageSize': [value: number]
  reset: []
}>()

const sortOptions = [
  { value: 'title', label: 'Title' },
  { value: 'artist', label: 'Artist' },
  { value: 'album', label: 'Album' },
  { value: 'created_at', label: 'Date Added' },
  { value: 'updated_at', label: 'Last Updated' },
]
const pageSizeOptions = [25, 50, 100, 250]
</script>

<template>
  <section class="filters">
    <div class="filter-group search-group">
      <label for="search-input">
        <Icon name="search" :size="14" />
        Search
      </label>
      <div class="input-wrapper">
        <input
          id="search-input"
          :value="filters.search"
          @input="emit('update:search', ($event.target as HTMLInputElement).value)"
          type="text"
          placeholder="Search by title, artist, or album..."
          class="text-input"
        />
        <button 
          v-if="filters.search" 
          class="clear-btn"
          @click="emit('update:search', '')"
          title="Clear search"
        >
          <Icon name="x" :size="14" />
        </button>
      </div>
    </div>

    <div class="filter-group">
      <label for="genre-filter">
        <Icon name="tag" :size="14" />
        Genre
      </label>
      <select
        id="genre-filter"
        :value="filters.genre"
        @change="emit('update:genre', ($event.target as HTMLSelectElement).value)"
        class="select-input"
      >
        <option value="">All Genres</option>
        <option value="unconfirmed">Unconfirmed (with *)</option>
        <option v-for="genre in genres" :key="genre" :value="genre">{{ genre }}</option>
      </select>
    </div>

    <div class="filter-group">
      <label for="sort-filter">
        <Icon name="filter" :size="14" />
        Sort By
      </label>
      <select
        id="sort-filter"
        :value="filters.sort"
        @change="emit('update:sort', ($event.target as HTMLSelectElement).value as any)"
        class="select-input"
      >
        <option v-for="option in sortOptions" :key="option.value" :value="option.value">
          {{ option.label }}
        </option>
      </select>
    </div>

    <div class="filter-group small">
      <label for="order-filter">Order</label>
      <div class="order-buttons">
        <button
          :class="['order-btn', { active: filters.order === 'asc' }]"
          @click="emit('update:order', 'asc')"
          title="Ascending"
        >
          ↑ ASC
        </button>
        <button
          :class="['order-btn', { active: filters.order === 'desc' }]"
          @click="emit('update:order', 'desc')"
          title="Descending"
        >
          ↓ DESC
        </button>
      </div>
    </div>

    <div class="filter-group small">
      <label for="page-size">Show</label>
      <select
        id="page-size"
        :value="pageSize"
        @change="emit('update:pageSize', Number(($event.target as HTMLSelectElement).value))"
        class="select-input"
      >
        <option v-for="size in pageSizeOptions" :key="size" :value="size">{{ size }} tracks</option>
      </select>
    </div>

    <div class="filter-actions">
      <button class="reset-btn" @click="emit('reset')" title="Reset all filters">
        <span>↺</span> Reset
      </button>
    </div>
  </section>
</template>

<style scoped>
.filters {
  display: flex;
  flex-wrap: wrap;
  gap: 16px;
  align-items: flex-end;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  padding: 20px;
}

.filter-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 160px;
}

.filter-group.search-group {
  flex: 2;
  min-width: 280px;
}

.filter-group.small {
  min-width: 130px;
}

.filter-group label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.input-wrapper {
  position: relative;
  display: flex;
  align-items: center;
}

.text-input,
.select-input {
  width: 100%;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  padding: 10px 14px;
  color: var(--text-color);
  font-family: inherit;
  font-size: 14px;
  transition: all var(--transition-base);
}

.text-input:focus,
.select-input:focus {
  outline: none;
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--accent-muted);
}

.text-input::placeholder {
  color: var(--text-tertiary);
}

.clear-btn {
  position: absolute;
  right: 10px;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: none;
  background: var(--surface-muted);
  color: var(--text-tertiary);
  font-size: 16px;
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all var(--transition-base);
}

.clear-btn:hover {
  background: var(--error-color);
  color: white;
}

.select-input {
  cursor: pointer;
  padding-right: 32px;
  appearance: none;
  background-image: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' width='12' height='12' viewBox='0 0 12 12' fill='%2394a3b8'%3E%3Cpath d='M2 4l4 4 4-4'/%3E%3C/svg%3E");
  background-repeat: no-repeat;
  background-position: right 12px center;
}

.order-buttons {
  display: flex;
  gap: 4px;
}

.order-btn {
  flex: 1;
  padding: 10px 12px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  color: var(--text-tertiary);
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all var(--transition-base);
}

.order-btn:first-child {
  border-radius: var(--radius-md) 0 0 var(--radius-md);
}

.order-btn:last-child {
  border-radius: 0 var(--radius-md) var(--radius-md) 0;
  border-left: none;
}

.order-btn:hover {
  background: var(--surface-hover);
  color: var(--text-color);
}

.order-btn.active {
  background: var(--primary-color);
  border-color: var(--primary-color);
  color: white;
}

.filter-actions {
  display: flex;
  align-items: flex-end;
}

.reset-btn {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 10px 16px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-base);
}

.reset-btn:hover {
  background: var(--surface-muted);
  border-color: var(--border-hover);
  color: var(--text-color);
}

@media (max-width: 900px) {
  .filters {
    gap: 12px;
    padding: 16px;
  }

  .filter-group {
    min-width: 100%;
  }

  .filter-group.search-group {
    min-width: 100%;
  }

  .filter-group.small {
    min-width: calc(50% - 6px);
    flex: 1;
  }

  .filter-actions {
    width: 100%;
    justify-content: center;
  }

  .reset-btn {
    flex: 1;
    justify-content: center;
  }
}
</style>
