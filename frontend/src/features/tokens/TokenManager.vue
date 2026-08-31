<template>
  <div class="token-manager">
    <div class="page-header">
      <div class="header-content">
        <h1>Personal Access Tokens</h1>
        <p class="subtitle">
          Generate tokens that allow external programs, scripts, or integrations to connect
          to your music library. Each token can be scoped to specific operations.
        </p>
      </div>
      <button class="btn-primary" @click="showCreate = true">
        <Icon name="plus" :size="18" />
        New Token
      </button>
    </div>

    <!-- Token list -->
    <div v-if="loading" class="loading-state">
      <Icon name="loader" class="spin" :size="24" />
      <span>Loading tokens…</span>
    </div>

    <div v-else-if="tokens.length === 0" class="empty-state">
      <Icon name="key" :size="48" />
      <h3>No tokens yet</h3>
      <p>Create a personal access token to connect external apps and scripts.</p>
      <button class="btn-primary" @click="showCreate = true">
        <Icon name="plus" :size="18" />
        Create your first token
      </button>
    </div>

    <div v-else class="tokens-list">
      <div v-for="token in tokens" :key="token.id" class="token-card">
        <div class="token-card-main">
          <div class="token-info">
            <div class="token-name">
              <Icon name="key" :size="16" />
              {{ token.name }}
            </div>
            <div class="token-meta">
              <span class="meta-item">Created {{ formatDate(token.created_at) }}</span>
              <span v-if="token.last_used_at" class="meta-item">
                Last used {{ formatDate(token.last_used_at) }}
              </span>
              <span v-else class="meta-item muted">Never used</span>
              <span v-if="token.expires_at" class="meta-item" :class="isExpired(token.expires_at) ? 'expired' : 'expiry'">
                {{ isExpired(token.expires_at) ? 'Expired' : 'Expires' }} {{ formatDate(token.expires_at) }}
              </span>
              <span v-else class="meta-item muted">No expiry</span>
            </div>
          </div>

          <!-- Permission badges -->
          <div class="token-perms">
            <span
              v-for="perm in getPermBadges(token)"
              :key="perm.label"
              class="perm-badge"
              :class="perm.active ? 'active' : 'inactive'"
              :title="perm.description"
            >
              <Icon :name="perm.icon" :size="12" />
              {{ perm.label }}
            </span>
          </div>
        </div>

        <div class="token-card-actions">
          <button class="btn-icon" title="Edit token" @click="startEdit(token)">
            <Icon name="edit" :size="16" />
          </button>
          <button class="btn-icon danger" title="Revoke token" @click="confirmDelete(token)">
            <Icon name="trash" :size="16" />
          </button>
        </div>
      </div>
    </div>

    <!-- ===== Create / Edit Modal ===== -->
    <Transition name="modal">
      <div v-if="showCreate || editingToken" class="modal-overlay" @click.self="closeModal">
        <div class="modal">
          <div class="modal-header">
            <h2>{{ editingToken ? 'Edit Token' : 'Create Access Token' }}</h2>
            <button class="btn-icon" @click="closeModal">
              <Icon name="x" :size="20" />
            </button>
          </div>

          <div class="modal-body">
            <!-- Name -->
            <div class="form-group">
              <label class="form-label">Token Name <span class="required">*</span></label>
              <input
                v-model="form.name"
                type="text"
                class="form-input"
                placeholder="e.g. Home Assistant, API client, CLI script"
                maxlength="255"
                autofocus
              />
              <p class="form-hint">A descriptive name so you remember what this token is for.</p>
            </div>

            <!-- Permissions grid -->
            <div class="form-group">
              <label class="form-label">Permissions</label>
              <p class="form-hint">Choose what this token is allowed to do.</p>
              <div class="perm-grid">
                <label
                  v-for="perm in permissionOptions"
                  :key="perm.key"
                  class="perm-option"
                  :class="{ checked: form[perm.key] }"
                >
                  <input
                    type="checkbox"
                    v-model="form[perm.key]"
                    class="perm-checkbox"
                  />
                  <div class="perm-option-content">
                    <div class="perm-option-header">
                      <div class="perm-icon-wrap">
                        <Icon :name="perm.icon" :size="18" />
                      </div>
                      <span class="perm-name">{{ perm.label }}</span>
                    </div>
                    <span class="perm-desc">{{ perm.description }}</span>
                  </div>
                  <div class="perm-check-indicator">
                    <Icon v-if="form[perm.key]" name="check" :size="14" />
                  </div>
                </label>
              </div>
            </div>

            <!-- Expiry -->
            <div class="form-group">
              <label class="form-label">Expiry</label>
              <div class="expiry-row">
                <label class="radio-option">
                  <input type="radio" v-model="expiryMode" value="never" />
                  <span>Never expires</span>
                </label>
                <label class="radio-option">
                  <input type="radio" v-model="expiryMode" value="date" />
                  <span>Expires on</span>
                </label>
                <input
                  v-if="expiryMode === 'date'"
                  v-model="form.expires_at"
                  type="date"
                  class="form-input date-input"
                  :min="today"
                />
              </div>
            </div>

            <p v-if="formError" class="form-error">{{ formError }}</p>
          </div>

          <div class="modal-footer">
            <button class="btn-ghost" @click="closeModal" :disabled="saving">Cancel</button>
            <button class="btn-primary" @click="submitForm" :disabled="saving || !form.name.trim()">
              <Icon v-if="saving" name="loader" class="spin" :size="16" />
              {{ editingToken ? 'Save Changes' : 'Create Token' }}
            </button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- ===== New Token Display Modal (shown once after creation) ===== -->
    <Transition name="modal">
      <div v-if="newToken" class="modal-overlay">
        <div class="modal new-token-modal">
          <div class="modal-header success">
            <div class="success-icon">
              <Icon name="check-circle" :size="24" />
            </div>
            <h2>Token Created!</h2>
          </div>
          <div class="modal-body">
            <div class="token-warning">
              <Icon name="alert-triangle" :size="20" />
              <strong>Copy this token now.</strong> It will not be shown again.
            </div>
            <div class="token-display">
              <code class="token-value">{{ newToken }}</code>
              <button class="btn-copy" @click="copyToken" :class="{ copied: justCopied }">
                <Icon :name="justCopied ? 'check' : 'copy'" :size="16" />
                {{ justCopied ? 'Copied!' : 'Copy' }}
              </button>
            </div>
            <p class="token-hint">
              Use this token in the <code>Authorization</code> header of API requests:
              <code class="code-example">Authorization: Bearer {{ newToken }}</code>
            </p>
          </div>
          <div class="modal-footer">
            <button class="btn-primary" @click="newToken = null">I've saved my token</button>
          </div>
        </div>
      </div>
    </Transition>

    <!-- ===== Delete Confirmation ===== -->
    <Transition name="modal">
      <div v-if="deletingToken" class="modal-overlay" @click.self="deletingToken = null">
        <div class="modal confirm-modal">
          <div class="modal-header">
            <h2>Revoke Token</h2>
          </div>
          <div class="modal-body">
            <p>
              Are you sure you want to revoke <strong>{{ deletingToken.name }}</strong>?
              Any integrations using this token will immediately lose access.
            </p>
          </div>
          <div class="modal-footer">
            <button class="btn-ghost" @click="deletingToken = null">Cancel</button>
            <button class="btn-danger" @click="deleteToken" :disabled="saving">
              <Icon v-if="saving" name="loader" class="spin" :size="16" />
              Revoke Token
            </button>
          </div>
        </div>
      </div>
    </Transition>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue'
import { musicAPI } from '../../api/music'
import Icon from '../../shared/components/Icons.vue'

// ── Types ───────────────────────────────────────────────────────────────────

interface AccessToken {
  id: string
  name: string
  can_read: boolean
  can_create: boolean
  can_edit: boolean
  can_delete: boolean
  last_used_at: string | null
  expires_at: string | null
  created_at: string
}

interface PermForm {
  name: string
  can_read: boolean
  can_create: boolean
  can_edit: boolean
  can_delete: boolean
  expires_at: string
  [key: string]: string | boolean
}

// ── Permission options ───────────────────────────────────────────────────────

const permissionOptions = [
  {
    key: 'can_read',
    label: 'Read',
    icon: 'eye',
    description: 'Browse library, stream music, access playlists and genres.',
  },
  {
    key: 'can_create',
    label: 'Create',
    icon: 'plus-circle',
    description: 'Upload tracks, create playlists, add internet streams.',
  },
  {
    key: 'can_edit',
    label: 'Edit',
    icon: 'edit',
    description: 'Update track metadata, rename, reorder playlists.',
  },
  {
    key: 'can_delete',
    label: 'Delete',
    icon: 'trash',
    description: 'Remove tracks, playlists, and streams.',
  },
]

// ── State ────────────────────────────────────────────────────────────────────

const tokens = ref<AccessToken[]>([])
const loading = ref(false)
const saving = ref(false)
const showCreate = ref(false)
const editingToken = ref<AccessToken | null>(null)
const deletingToken = ref<AccessToken | null>(null)
const newToken = ref<string | null>(null)
const justCopied = ref(false)
const formError = ref<string | null>(null)
const expiryMode = ref<'never' | 'date'>('never')

const today = computed(() => new Date().toISOString().split('T')[0])

const form = ref<PermForm>({
  name: '',
  can_read: true,
  can_create: false,
  can_edit: false,
  can_delete: false,
  expires_at: '',
})

// ── Helpers ──────────────────────────────────────────────────────────────────

function formatDate(iso: string | null | undefined): string {
  if (!iso) return '—'
  return new Date(iso).toLocaleDateString(undefined, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
  })
}

function isExpired(iso: string | null | undefined): boolean {
  if (!iso) return false
  return new Date(iso) < new Date()
}

function getPermBadges(token: AccessToken) {
  return permissionOptions.map(p => ({
    label: p.label,
    icon: p.icon,
    description: p.description,
    active: token[p.key as keyof AccessToken] as boolean,
  }))
}

// ── API ───────────────────────────────────────────────────────────────────────

async function loadTokens() {
  loading.value = true
  try {
    const res = await musicAPI.listTokens()
    tokens.value = res.data
  } catch (e) {
    console.error('Failed to load tokens', e)
  } finally {
    loading.value = false
  }
}

// ── Create / Edit ─────────────────────────────────────────────────────────────

function resetForm() {
  form.value = {
    name: '',
    can_read: true,
    can_create: false,
    can_edit: false,
    can_delete: false,
    expires_at: '',
  }
  expiryMode.value = 'never'
  formError.value = null
}

function startEdit(token: AccessToken) {
  editingToken.value = token
  form.value = {
    name: token.name,
    can_read: token.can_read,
    can_create: token.can_create,
    can_edit: token.can_edit,
    can_delete: token.can_delete,
    expires_at: token.expires_at ? token.expires_at.split('T')[0] : '',
  }
  expiryMode.value = token.expires_at ? 'date' : 'never'
  formError.value = null
}

function closeModal() {
  showCreate.value = false
  editingToken.value = null
  resetForm()
}

async function submitForm() {
  formError.value = null

  if (!form.value.name.trim()) {
    formError.value = 'Token name is required.'
    return
  }
  if (!form.value.can_read && !form.value.can_create && !form.value.can_edit && !form.value.can_delete) {
    formError.value = 'Please enable at least one permission.'
    return
  }

  saving.value = true
  try {
    const expiresAt = expiryMode.value === 'date' && form.value.expires_at
      ? new Date(form.value.expires_at).toISOString()
      : null

    if (editingToken.value) {
      await musicAPI.updateToken(editingToken.value.id, {
        name: form.value.name.trim(),
        can_read: form.value.can_read as boolean,
        can_create: form.value.can_create as boolean,
        can_edit: form.value.can_edit as boolean,
        can_delete: form.value.can_delete as boolean,
        expires_at: expiresAt,
        clear_expires_at: expiryMode.value === 'never' && !!editingToken.value.expires_at,
      })
      closeModal()
    } else {
      const res = await musicAPI.createToken({
        name: form.value.name.trim(),
        can_read: form.value.can_read as boolean,
        can_create: form.value.can_create as boolean,
        can_edit: form.value.can_edit as boolean,
        can_delete: form.value.can_delete as boolean,
        expires_at: expiresAt,
      })
      newToken.value = res.data.token
      closeModal()
    }
    await loadTokens()
  } catch (e: any) {
    formError.value = e?.response?.data?.message || 'An error occurred. Please try again.'
  } finally {
    saving.value = false
  }
}

// ── Delete ────────────────────────────────────────────────────────────────────

function confirmDelete(token: AccessToken) {
  deletingToken.value = token
}

async function deleteToken() {
  if (!deletingToken.value) return
  saving.value = true
  try {
    await musicAPI.deleteToken(deletingToken.value.id)
    deletingToken.value = null
    await loadTokens()
  } catch (e) {
    console.error('Failed to delete token', e)
  } finally {
    saving.value = false
  }
}

// ── Copy token ───────────────────────────────────────────────────────────────

async function copyToken() {
  if (!newToken.value) return
  try {
    await navigator.clipboard.writeText(newToken.value)
    justCopied.value = true
    setTimeout(() => { justCopied.value = false }, 2000)
  } catch {
    // Fallback: select text
  }
}

// ── Lifecycle ─────────────────────────────────────────────────────────────────

onMounted(loadTokens)
</script>

<style scoped>
.token-manager {
  max-width: 860px;
  margin: 0 auto;
  padding: 2rem 1.5rem;
}

/* ── Header ─────────────────────────────────────────── */
.page-header {
  display: flex;
  align-items: flex-start;
  gap: 1.5rem;
  margin-bottom: 2rem;
}
.header-content { flex: 1; }
.page-header h1 { font-size: 1.6rem; font-weight: 700; margin: 0 0 0.4rem; }
.subtitle { color: var(--text-muted); margin: 0; font-size: 0.95rem; }

/* ── States ─────────────────────────────────────────── */
.loading-state, .empty-state {
  text-align: center;
  padding: 4rem 2rem;
  color: var(--text-muted);
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 0.75rem;
}
.empty-state h3 { font-size: 1.2rem; color: var(--text-primary); margin: 0; }
.empty-state p { margin: 0; max-width: 340px; }

/* ── Token cards ────────────────────────────────────── */
.tokens-list { display: flex; flex-direction: column; gap: 0.75rem; }

.token-card {
  background: var(--card-bg, var(--bg-secondary));
  border: 1px solid var(--border-color);
  border-radius: 10px;
  padding: 1rem 1.25rem;
  display: flex;
  align-items: center;
  gap: 1rem;
  transition: border-color 0.15s;
}
.token-card:hover { border-color: var(--spotify-green); }

.token-card-main { flex: 1; display: flex; flex-direction: column; gap: 0.5rem; }
.token-info { display: flex; flex-direction: column; gap: 0.2rem; }

.token-name {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  font-weight: 600;
  font-size: 1rem;
}

.token-meta {
  display: flex;
  flex-wrap: wrap;
  gap: 0.4rem 1rem;
}
.meta-item { font-size: 0.8rem; color: var(--text-muted); }
.meta-item.muted { opacity: 0.6; }
.meta-item.expired { color: var(--color-danger, #e74c3c); }
.meta-item.expiry { color: var(--color-warning, #f39c12); }

/* ── Permission badges ──────────────────────────────── */
.token-perms {
  display: flex;
  flex-wrap: wrap;
  gap: 0.35rem;
  margin-top: 0.15rem;
}
.perm-badge {
  display: inline-flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.15rem 0.55rem;
  border-radius: 999px;
  font-size: 0.72rem;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.04em;
  transition: all 0.15s;
}
.perm-badge.active {
  background: color-mix(in srgb, var(--spotify-green) 15%, transparent);
  color: var(--spotify-green);
  border: 1px solid color-mix(in srgb, var(--spotify-green) 40%, transparent);
}
.perm-badge.inactive {
  background: transparent;
  color: var(--text-muted);
  border: 1px solid var(--border-color);
  opacity: 0.45;
}

.token-card-actions { display: flex; gap: 0.4rem; flex-shrink: 0; }

/* ── Buttons ─────────────────────────────────────────── */
.btn-primary {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.55rem 1.1rem;
  background: var(--spotify-green);
  color: #000;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  font-size: 0.9rem;
  transition: filter 0.15s;
}
.btn-primary:hover:not(:disabled) { filter: brightness(1.1); }
.btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }

.btn-ghost {
  padding: 0.55rem 1.1rem;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 0.9rem;
  transition: background 0.15s;
}
.btn-ghost:hover:not(:disabled) { background: var(--bg-hover); }

.btn-danger {
  display: inline-flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.55rem 1.1rem;
  background: var(--color-danger, #e74c3c);
  color: #fff;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: filter 0.15s;
}
.btn-danger:hover:not(:disabled) { filter: brightness(1.1); }

.btn-icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 34px;
  height: 34px;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: transparent;
  color: var(--text-muted);
  cursor: pointer;
  transition: all 0.15s;
}
.btn-icon:hover { border-color: var(--text-primary); color: var(--text-primary); }
.btn-icon.danger:hover { border-color: var(--color-danger, #e74c3c); color: var(--color-danger, #e74c3c); }

/* ── Modal ──────────────────────────────────────────── */
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0,0,0,0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 1rem;
}
.modal {
  background: var(--bg-primary, #181818);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  width: 100%;
  max-width: 520px;
  max-height: 90vh;
  overflow-y: auto;
  display: flex;
  flex-direction: column;
}
.modal-header {
  display: flex;
  align-items: center;
  gap: 0.75rem;
  padding: 1.25rem 1.5rem;
  border-bottom: 1px solid var(--border-color);
}
.modal-header h2 { flex: 1; margin: 0; font-size: 1.15rem; }
.modal-header.success { color: var(--spotify-green); }
.success-icon { display: flex; align-items: center; }
.modal-body { padding: 1.5rem; display: flex; flex-direction: column; gap: 1.25rem; }
.modal-footer {
  display: flex;
  justify-content: flex-end;
  gap: 0.75rem;
  padding: 1rem 1.5rem;
  border-top: 1px solid var(--border-color);
}

/* ── Form ────────────────────────────────────────────── */
.form-group { display: flex; flex-direction: column; gap: 0.4rem; }
.form-label { font-weight: 600; font-size: 0.9rem; }
.required { color: var(--color-danger, #e74c3c); }
.form-input {
  padding: 0.55rem 0.75rem;
  border: 1px solid var(--border-color);
  border-radius: 6px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  font-size: 0.95rem;
  transition: border-color 0.15s;
}
.form-input:focus { outline: none; border-color: var(--spotify-green); }
.form-hint { font-size: 0.82rem; color: var(--text-muted); margin: 0; }
.form-error { color: var(--color-danger, #e74c3c); font-size: 0.85rem; margin: 0; }

/* ── Permission grid ─────────────────────────────────── */
.perm-grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 0.65rem;
}
@media (max-width: 500px) { .perm-grid { grid-template-columns: 1fr; } }

.perm-option {
  display: flex;
  align-items: flex-start;
  gap: 0.75rem;
  padding: 0.85rem;
  border: 2px solid var(--border-color);
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.15s;
  position: relative;
}
.perm-option.checked {
  border-color: var(--spotify-green);
  background: color-mix(in srgb, var(--spotify-green) 8%, transparent);
}
.perm-checkbox { display: none; }
.perm-option-content { flex: 1; }
.perm-option-header {
  display: flex;
  align-items: center;
  gap: 0.5rem;
  margin-bottom: 0.3rem;
}
.perm-icon-wrap {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border-radius: 6px;
  background: var(--bg-tertiary, var(--bg-secondary));
  color: var(--text-muted);
}
.perm-option.checked .perm-icon-wrap { background: var(--spotify-green); color: #000; }
.perm-name { font-weight: 600; font-size: 0.9rem; }
.perm-desc { font-size: 0.78rem; color: var(--text-muted); line-height: 1.4; }
.perm-check-indicator {
  position: absolute;
  top: 0.6rem;
  right: 0.6rem;
  width: 18px;
  height: 18px;
  border-radius: 4px;
  border: 2px solid var(--border-color);
  display: flex;
  align-items: center;
  justify-content: center;
  transition: all 0.15s;
}
.perm-option.checked .perm-check-indicator {
  background: var(--spotify-green);
  border-color: var(--spotify-green);
  color: #000;
}

/* ── Expiry row ──────────────────────────────────────── */
.expiry-row { display: flex; align-items: center; gap: 1rem; flex-wrap: wrap; }
.radio-option { display: flex; align-items: center; gap: 0.4rem; cursor: pointer; }
.date-input { flex: 1; min-width: 150px; }

/* ── New token display ───────────────────────────────── */
.new-token-modal { max-width: 580px; }
.token-warning {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.75rem 1rem;
  background: color-mix(in srgb, #f39c12 12%, transparent);
  border: 1px solid color-mix(in srgb, #f39c12 35%, transparent);
  border-radius: 8px;
  color: #f39c12;
  font-size: 0.9rem;
}
.token-display {
  display: flex;
  align-items: stretch;
  gap: 0;
  border: 1px solid var(--border-color);
  border-radius: 8px;
  overflow: hidden;
}
.token-value {
  flex: 1;
  padding: 0.75rem 1rem;
  font-family: monospace;
  font-size: 0.8rem;
  word-break: break-all;
  background: var(--bg-secondary);
  color: var(--text-primary);
  user-select: all;
}
.btn-copy {
  display: flex;
  align-items: center;
  gap: 0.4rem;
  padding: 0.75rem 1rem;
  border: none;
  border-left: 1px solid var(--border-color);
  background: var(--bg-tertiary, var(--bg-secondary));
  color: var(--text-muted);
  cursor: pointer;
  font-size: 0.85rem;
  white-space: nowrap;
  transition: all 0.15s;
}
.btn-copy:hover, .btn-copy.copied { background: var(--spotify-green); color: #000; }
.token-hint { font-size: 0.82rem; color: var(--text-muted); margin: 0; }
.code-example {
  display: block;
  margin-top: 0.4rem;
  padding: 0.5rem 0.75rem;
  background: var(--bg-secondary);
  border-radius: 6px;
  font-size: 0.78rem;
  word-break: break-all;
}

/* ── Confirm modal ───────────────────────────────────── */
.confirm-modal { max-width: 400px; }
.confirm-modal .modal-body p { margin: 0; line-height: 1.6; }

/* ── Animation ───────────────────────────────────────── */
.modal-enter-active, .modal-leave-active { transition: opacity 0.2s; }
.modal-enter-from, .modal-leave-to { opacity: 0; }

.spin { animation: spin 1s linear infinite; }
@keyframes spin { from { transform: rotate(0deg); } to { transform: rotate(360deg); } }
</style>
