<template>
  <div class="history-tab">
    <div class="tab-header">
      <div class="header-info">
        <h3><Icon name="history" :size="20" /> Audit History</h3>
        <p class="description">Track metadata changes and revert if needed.</p>
      </div>
    </div>

    <!-- Filters & Search Bar -->
    <div class="filters-bar">
      <div class="search-box">
        <Icon name="search" :size="16" />
        <input 
          v-model="searchQuery" 
          type="text" 
          placeholder="Search by track name, field value..." 
          @input="debouncedFilter"
        />
        <button v-if="searchQuery" class="clear-btn" @click="searchQuery = ''; applyFilters()">
          <Icon name="x" :size="14" />
        </button>
      </div>
      <div class="filter-controls">
        <select v-model="filters.table_name" class="filter-select" @change="fetchLogs()">
          <option value="">All Tables</option>
          <option value="music_files">Music Files</option>
          <option value="playlists">Playlists</option>
        </select>
        <select v-model="filters.action" class="filter-select" @change="applyFilters()">
          <option value="">All Actions</option>
          <option value="INSERT">Insert</option>
          <option value="UPDATE">Update</option>
          <option value="DELETE">Delete</option>
        </select>
        <select v-model="sortOrder" class="filter-select" @change="applyFilters()">
          <option value="newest">Newest First</option>
          <option value="oldest">Oldest First</option>
        </select>
        <button class="btn-refresh" @click="fetchLogs()" :disabled="loading">
          <Icon name="refresh" :size="16" :class="{ 'animate-spin': loading }" />
          Refresh
        </button>
      </div>
    </div>

    <!-- Stats summary -->
    <div v-if="!loading && allLogs.length > 0" class="stats-bar">
      <span class="stat-item">{{ filteredLogs.length }} entries</span>
      <span class="stat-divider">&middot;</span>
      <span class="stat-item">{{ uniqueUsers.length }} user{{ uniqueUsers.length !== 1 ? 's' : '' }}</span>
      <span class="stat-divider">&middot;</span>
      <span class="stat-item">{{ actionCounts.UPDATE || 0 }} updates</span>
      <span class="stat-item">{{ actionCounts.INSERT || 0 }} inserts</span>
      <span class="stat-item">{{ actionCounts.DELETE || 0 }} deletes</span>
    </div>

    <div v-if="loading && allLogs.length === 0" class="loading-state">
      <div class="spinner"></div>
      <p>Loading audit logs...</p>
    </div>

    <div v-else-if="filteredLogs.length === 0 && allLogs.length > 0" class="empty-state">
      <Icon name="search" :size="48" :stroke-width="1.5" />
      <h3>No matching entries</h3>
      <p>Try adjusting your search or filters.</p>
      <button class="btn-text" @click="clearAllFilters">Clear all filters</button>
    </div>

    <div v-else-if="allLogs.length === 0" class="empty-state">
      <Icon name="history" :size="48" :stroke-width="1.5" />
      <h3>No history found</h3>
      <p>Changes will appear here as they happen.</p>
    </div>

    <div v-else class="logs-list">
      <div v-for="log in paginatedLogs" :key="log.id" class="log-card">
        <div class="log-header">
          <div class="log-meta">
            <span :class="['action-badge', log.action.toLowerCase().startsWith('revert') ? 'revert' : log.action.toLowerCase()]">
              {{ log.action.startsWith('REVERT') ? 'REVERT' : log.action }}
            </span>
            <span class="log-table">{{ formatTableName(log.table_name) }}</span>
            <span class="log-time" :title="new Date(log.created_at).toLocaleString()">{{ formatTime(log.created_at) }}</span>
          </div>
          <div class="log-actions">
            <button 
              v-if="canRevert(log)"
              class="btn-revert" 
              @click="revertLog(log.id)"
              :disabled="reverting === log.id"
            >
              <Icon v-if="reverting === log.id" name="loader" :size="16" class="animate-spin" />
              <Icon v-else name="history" :size="16" />
              Revert
            </button>
          </div>
        </div>

        <div class="log-content">
          <div v-if="log.old_values && log.new_values" class="diff-container">
            <div v-for="(_, key) in (getDiff(log) as Record<string, any>)" :key="key" class="diff-row">
              <span class="diff-key">{{ formatFieldName(String(key)) }}:</span>
              <div class="diff-values">
                <span class="val-old" v-if="log.old_values[key] !== null && log.old_values[key] !== undefined">{{ log.old_values[key] }}</span>
                <span class="val-old empty" v-else>(empty)</span>
                <Icon name="chevron-right" :size="12" />
                <span class="val-new" v-if="log.new_values[key] !== null && log.new_values[key] !== undefined">{{ log.new_values[key] }}</span>
                <span class="val-new empty" v-else>(empty)</span>
              </div>
            </div>
            <div class="summary-line">
              Target: <strong>{{ getRecordSummary(log) }}</strong>
            </div>
          </div>
          <div v-else-if="log.action === 'INSERT'" class="details">
            <p>Added a new record.</p>
            <div class="summary"><strong>{{ getRecordSummary(log) }}</strong></div>
          </div>
          <div v-else-if="log.action === 'DELETE'" class="details">
            <p>Deleted a record.</p>
            <div class="summary"><strong>{{ getRecordSummary(log) }}</strong></div>
          </div>
          <div v-else class="details">
            <div class="summary"><strong>{{ getRecordSummary(log) }}</strong></div>
          </div>
        </div>
        
        <div class="log-footer">
          <span class="user-tag">
            <Icon name="user" :size="14" />
            {{ formatUser(log.user_id) }}
          </span>
          <span class="record-id">ID: {{ log.record_id?.substring(0, 8) }}&hellip;</span>
        </div>
      </div>

      <!-- Pagination -->
      <div v-if="totalPages > 1" class="pagination-bar">
        <button class="btn-page" :disabled="currentPage <= 1" @click="currentPage--">
          <Icon name="chevron-left" :size="16" /> Previous
        </button>
        <span class="page-info">
          Page {{ currentPage }} of {{ totalPages }}
        </span>
        <button class="btn-page" :disabled="currentPage >= totalPages" @click="currentPage++">
          Next <Icon name="chevron-right" :size="16" />
        </button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import { musicAPI } from '@/api/music'
import { useToast } from '@/composables/useToast'
import { useConfirm } from '@/composables/useConfirm'
import Icon from '@/shared/components/Icons.vue'

const { success, error } = useToast()
const { confirm } = useConfirm()

const allLogs = ref<any[]>([])
const loading = ref(false)
const reverting = ref<string | null>(null)
const searchQuery = ref('')
const sortOrder = ref<'newest' | 'oldest'>('newest')
const currentPage = ref(1)
const pageSize = 20

const filters = ref({
  table_name: '',
  action: '',
})

// Debounce for search
let debounceTimer: ReturnType<typeof setTimeout> | null = null
const debouncedFilter = () => {
  if (debounceTimer) clearTimeout(debounceTimer)
  debounceTimer = setTimeout(() => {
    currentPage.value = 1
    applyFilters()
  }, 300)
}

const applyFilters = () => {
  currentPage.value = 1
}

const clearAllFilters = () => {
  searchQuery.value = ''
  filters.value.table_name = ''
  filters.value.action = ''
  sortOrder.value = 'newest'
  currentPage.value = 1
  fetchLogs()
}

// Computed filtered + sorted logs (client-side filtering on top of server data)
const filteredLogs = computed(() => {
  let result = [...allLogs.value]

  // Filter by action type
  if (filters.value.action) {
    result = result.filter(log => log.action === filters.value.action)
  }

  // Search filter
  if (searchQuery.value.trim()) {
    const q = searchQuery.value.toLowerCase()
    result = result.filter(log => {
      const summary = getRecordSummary(log).toLowerCase()
      const userId = (log.user_id || '').toLowerCase()
      const action = log.action.toLowerCase()
      const table = log.table_name.toLowerCase()
      
      // Also search in old/new values
      let valuesMatch = false
      if (log.old_values) {
        valuesMatch = JSON.stringify(log.old_values).toLowerCase().includes(q)
      }
      if (!valuesMatch && log.new_values) {
        valuesMatch = JSON.stringify(log.new_values).toLowerCase().includes(q)
      }
      
      return summary.includes(q) || userId.includes(q) || action.includes(q) || table.includes(q) || valuesMatch
    })
  }

  // Sort
  if (sortOrder.value === 'oldest') {
    result.sort((a, b) => new Date(a.created_at).getTime() - new Date(b.created_at).getTime())
  } else {
    result.sort((a, b) => new Date(b.created_at).getTime() - new Date(a.created_at).getTime())
  }

  return result
})

const paginatedLogs = computed(() => {
  const start = (currentPage.value - 1) * pageSize
  return filteredLogs.value.slice(start, start + pageSize)
})

const totalPages = computed(() => Math.ceil(filteredLogs.value.length / pageSize))

// Stats
const uniqueUsers = computed(() => {
  const users = new Set(allLogs.value.map(l => l.user_id || 'system'))
  return Array.from(users)
})

const actionCounts = computed(() => {
  const counts: Record<string, number> = {}
  for (const log of allLogs.value) {
    const action = log.action.startsWith('REVERT') ? 'REVERT' : log.action
    counts[action] = (counts[action] || 0) + 1
  }
  return counts
})

const fetchLogs = async () => {
  loading.value = true
  currentPage.value = 1
  try {
    const response = await musicAPI.getAuditLogs({
      table_name: filters.value.table_name || undefined,
    })
    allLogs.value = response.data
  } catch (err: any) {
    console.error('Audit log fetch error:', err)
    error('Failed to load history', 'Please try again later.')
  } finally {
    loading.value = false
  }
}

const revertLog = async (id: string) => {
  const isConfirmed = await confirm({
    title: 'Revert Change',
    message: 'Are you sure you want to revert this change? This will overwrite the current values with the old ones.',
    confirmText: 'Revert',
    variant: 'warning'
  })

  if (!isConfirmed) return
  
  reverting.value = id
  try {
    await musicAPI.revertAuditLog(id)
    success('Reverted', 'The change has been reverted successfully.')
    fetchLogs()
  } catch (err: any) {
    error('Failed to revert', err.response?.data?.error || 'Unknown error occurred.')
  } finally {
    reverting.value = null
  }
}

const formatTime = (dateStr: string) => {
  const d = new Date(dateStr)
  const now = new Date()
  const diffMs = now.getTime() - d.getTime()
  const diffMins = Math.floor(diffMs / 60000)
  const diffHours = Math.floor(diffMs / 3600000)
  const diffDays = Math.floor(diffMs / 86400000)

  if (diffMins < 1) return 'just now'
  if (diffMins < 60) return `${diffMins}m ago`
  if (diffHours < 24) return `${diffHours}h ago`
  if (diffDays < 7) return `${diffDays}d ago`

  return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: d.getFullYear() !== now.getFullYear() ? 'numeric' : undefined })
}

const formatTableName = (name: string) => {
  const map: Record<string, string> = {
    'music_files': 'Track',
    'playlists': 'Playlist',
  }
  return map[name] || name
}

const formatFieldName = (key: string) => {
  const map: Record<string, string> = {
    'guessed_genre': 'Guessed Genre',
    'genre': 'Genre',
    'release_date': 'Release Date',
    'file_path': 'File Path',
    'created_at': 'Created',
    'updated_at': 'Updated',
    'initial_key': 'Key',
  }
  return map[key] || key.replace(/_/g, ' ').replace(/\b\w/g, c => c.toUpperCase())
}

const formatUser = (userId: string | null | undefined) => {
  if (!userId) return 'System'
  if (userId === 'system') return 'System'
  // If it looks like an email, show it nicely
  if (userId.includes('@')) {
    return userId.split('@')[0]
  }
  // If it looks like a UUID, truncate it
  if (userId.length > 20) {
    return userId.substring(0, 8) + '&hellip;'
  }
  return userId
}

const canRevert = (log: any) => {
  return log.action === 'UPDATE' && log.old_values && !log.action.startsWith('REVERT')
}

const getDiff = (log: any) => {
  const diff: any = {}
  if (!log.old_values || !log.new_values) return diff
  
  for (const key in log.new_values) {
    if (JSON.stringify(log.old_values[key]) !== JSON.stringify(log.new_values[key])) {
      diff[key] = log.new_values[key]
    }
  }
  return diff
}

const getRecordSummary = (log: any) => {
  const data = log.new_values || log.old_values
  if (!data) return 'N/A'
  
  if (log.table_name === 'music_files') {
    return `${data.artist || 'Unknown'} - ${data.title || 'Unknown'}`
  }
  if (log.table_name === 'playlists') {
    return data.name || 'Unknown Playlist'
  }
  return JSON.stringify(data).substring(0, 60)
}

onMounted(() => {
  fetchLogs()
})
</script>

<style scoped>
.history-tab {
  display: flex;
  flex-direction: column;
  gap: 20px;
  max-width: 1000px;
  margin: 0 auto;
  padding: 20px;
}

.tab-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-info h3 {
  display: flex;
  align-items: center;
  gap: 8px;
  margin: 0 0 4px 0;
  color: var(--primary-color);
}

.header-info .description {
  color: var(--text-tertiary);
  font-size: 0.9rem;
  margin: 0;
}

/* Filters bar */
.filters-bar {
  display: flex;
  flex-direction: column;
  gap: 12px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 16px;
  padding: 16px;
}

.search-box {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 10px 16px;
  background: var(--background-base);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  transition: all 0.2s ease;
}

.search-box:focus-within {
  border-color: var(--primary-color);
  box-shadow: 0 0 0 3px var(--primary-glow);
}

.search-box input {
  flex: 1;
  border: none;
  background: none;
  color: var(--text-color);
  font-size: 14px;
  outline: none;
}

.search-box input::placeholder {
  color: var(--text-tertiary);
}

.clear-btn {
  background: transparent;
  border: none;
  color: var(--text-tertiary);
  cursor: pointer;
  padding: 4px;
  border-radius: 4px;
}

.clear-btn:hover {
  color: var(--text-color);
}

.filter-controls {
  display: flex;
  gap: 10px;
  flex-wrap: wrap;
  align-items: center;
}

.filter-select {
  padding: 8px 12px;
  background: var(--background-base);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-color);
  font-size: 13px;
  font-weight: 500;
  min-width: 130px;
  cursor: pointer;
}

.btn-refresh {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 14px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  color: var(--text-color);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  margin-left: auto;
}

.btn-refresh:hover:not(:disabled) {
  background: var(--surface-hover);
}

/* Stats bar */
.stats-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 10px 16px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  font-size: 13px;
  color: var(--text-tertiary);
}

.stat-item {
  font-weight: 500;
}

.stat-divider {
  color: var(--border-color);
}

/* Log list */
.logs-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.log-card {
  border-radius: 16px;
  padding: 20px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  transition: border-color 0.2s ease;
}

.log-card:hover {
  border-color: rgba(255, 255, 255, 0.1);
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.log-meta {
  display: flex;
  align-items: center;
  gap: 10px;
  flex-wrap: wrap;
}

.action-badge {
  font-size: 0.7rem;
  font-weight: 800;
  text-transform: uppercase;
  padding: 3px 10px;
  border-radius: 6px;
  letter-spacing: 0.05em;
}

.action-badge.insert { background: rgba(34, 197, 94, 0.15); color: #4ade80; }
.action-badge.update { background: rgba(234, 179, 8, 0.15); color: #facc15; }
.action-badge.delete { background: rgba(239, 68, 68, 0.15); color: #f87171; }
.action-badge.revert { background: rgba(74, 158, 255, 0.15); color: #4a9eff; }

.log-table {
  color: var(--text-tertiary);
  font-size: 0.8rem;
  font-weight: 600;
  background: rgba(255, 255, 255, 0.05);
  padding: 3px 10px;
  border-radius: 6px;
}

.log-time {
  color: var(--text-tertiary);
  font-size: 0.8rem;
}

.btn-revert {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: rgba(74, 158, 255, 0.1);
  color: #4a9eff;
  border: 1px solid rgba(74, 158, 255, 0.2);
  border-radius: 8px;
  font-size: 0.8rem;
  font-weight: 600;
  cursor: pointer;
}

.btn-revert:hover:not(:disabled) {
  background: rgba(74, 158, 255, 0.2);
}

.log-content {
  margin-bottom: 12px;
}

.diff-container {
  background: rgba(0, 0, 0, 0.25);
  border-radius: 10px;
  padding: 14px;
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.diff-row {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.diff-key {
  font-size: 0.7rem;
  font-weight: 700;
  color: var(--text-tertiary);
  text-transform: uppercase;
  letter-spacing: 0.05em;
}

.diff-values {
  display: flex;
  align-items: center;
  gap: 10px;
  font-size: 0.9rem;
}

.val-old {
  color: #f87171;
  text-decoration: line-through;
  word-break: break-all;
}

.val-new {
  color: #4ade80;
  word-break: break-all;
}

.val-old.empty, .val-new.empty {
  font-style: italic;
  opacity: 0.4;
}

.summary-line {
  font-size: 0.85rem;
  padding-top: 8px;
  border-top: 1px solid rgba(255, 255, 255, 0.05);
  color: var(--text-secondary);
}

.details p {
  color: var(--text-secondary);
  font-size: 0.9rem;
  margin-bottom: 4px;
}

.summary {
  font-weight: 600;
  color: var(--text-secondary);
}

.log-footer {
  padding-top: 12px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 0.75rem;
  color: var(--text-tertiary);
}

.user-tag {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  background: rgba(255, 255, 255, 0.05);
  padding: 3px 10px;
  border-radius: 6px;
  font-weight: 500;
}

.record-id {
  font-family: monospace;
  font-size: 0.7rem;
}

/* Pagination */
.pagination-bar {
  display: flex;
  justify-content: center;
  align-items: center;
  gap: 16px;
  padding: 16px;
  margin-top: 8px;
}

.btn-page {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 8px 16px;
  background: var(--surface-color);
  color: var(--text-color);
  border: 1px solid var(--border-color);
  border-radius: 10px;
  font-weight: 600;
  font-size: 13px;
  cursor: pointer;
}

.btn-page:hover:not(:disabled) {
  background: var(--surface-hover);
}

.btn-page:disabled {
  opacity: 0.4;
  cursor: not-allowed;
}

.page-info {
  font-size: 13px;
  font-weight: 600;
  color: var(--text-secondary);
}

.btn-text {
  background: transparent;
  border: none;
  color: var(--primary-color);
  font-weight: 600;
  font-size: 14px;
  cursor: pointer;
  margin-top: 12px;
}

.btn-text:hover {
  text-decoration: underline;
}

/* States */
.empty-state {
  text-align: center;
  padding: 60px 0;
  color: var(--text-tertiary);
  background: var(--surface-color);
  border-radius: 16px;
  border: 1px dashed var(--border-color);
}

.empty-state h3 {
  margin: 16px 0 8px 0;
  color: var(--text-secondary);
}

.loading-state {
  text-align: center;
  padding: 40px 0;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid rgba(255, 255, 255, 0.1);
  border-top-color: var(--primary-color);
  border-radius: 50%;
  margin: 0 auto 16px auto;
  animation: spin 1s infinite linear;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.animate-spin {
  animation: spin 1s infinite linear;
}

@media (max-width: 768px) {
  .history-tab {
    padding: 12px;
  }
  
  .filter-controls {
    flex-direction: column;
    align-items: stretch;
  }

  .filter-select {
    min-width: unset;
  }

  .btn-refresh {
    margin-left: 0;
  }

  .log-meta {
    flex-direction: column;
    align-items: flex-start;
    gap: 6px;
  }

  .diff-values {
    flex-wrap: wrap;
    gap: 6px;
  }

  .stats-bar {
    flex-wrap: wrap;
  }
}
</style>
