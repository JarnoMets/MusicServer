<template>
  <div class="metadata-settings">
    <div class="settings-header">
      <div class="header-content">
        <h1>Metadata Settings</h1>
        <p class="subtitle">Configure external API sources for music metadata like genres and release dates.</p>
      </div>
      <div class="header-actions">
        <button v-if="authStore.isAuthenticated" class="btn-primary" @click="saveConfig" :disabled="saving || !authStore.isAuthenticated">
          <Icon name="save" v-if="!saving" />
          <Icon name="loader" class="spin" v-else />
          {{ saving ? 'Saving...' : 'Save Settings' }}
        </button>
        <button v-else class="btn-primary" @click="startLogin">
          <Icon name="login" />
          Sign in to save
        </button>
      </div>
    </div>

    <div class="settings-grid">
      <section class="settings-card full-width">
        <div class="card-header">
          <Icon name="settings" :size="24" />
          <h2>Metadata Source</h2>
        </div>
        <p class="card-desc">Select the preferred source for auto-detecting all track metadata, including Album/EP, Release Date, and Genre.</p>
        <div class="source-options-row">
          <label class="source-option" :class="{ active: config.metadata_source === 'musicbrainz' }">
            <input type="radio" v-model="config.metadata_source" value="musicbrainz" />
            <div class="option-content">
              <strong>MusicBrainz</strong>
              <span>Default community-driven database. Reliable but sometimes missing newer releases.</span>
            </div>
          </label>
          <label class="source-option" :class="{ active: config.metadata_source === 'discogs' }">
            <input type="radio" v-model="config.metadata_source" value="discogs" />
            <div class="option-content">
              <strong>Discogs</strong>
              <span>Large database focus on physical releases. High accuracy for electronic music and niche genres. Uses official Styles for high resolution genre detection.</span>
            </div>
          </label>
        </div>
      </section>

      <section class="settings-card full-width">
        <div class="card-header">
          <Icon name="disc" :size="24" />
          <h2>Discogs Integration</h2>
        </div>
        <p class="card-desc">To use Discogs and avoid strict rate limits, you can provide a Personal Access Token.</p>
        <div class="form-group">
          <label>Discogs Personal Access Token</label>
          <div class="input-with-help">
            <input 
              type="password" 
              v-model="config.discogs_token" 
              placeholder="Enter your Discogs token here..." 
              class="token-input"
            />
            <div class="help-text">
              Generate one at <a href="https://www.discogs.com/settings/developers" target="_blank" rel="noopener">discogs.com/settings/developers</a>
            </div>
          </div>
        </div>
      </section>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useAuthStore } from '../../stores/auth'
import { useToast } from '../../composables/useToast'
import { musicAPI } from '../../api/music'
import Icon from '../../shared/components/Icons.vue'

const { success, error } = useToast()

const config = ref({
  metadata_source: 'musicbrainz',
  discogs_token: ''
})

const loading = ref(true)
const saving = ref(false)
const authStore = useAuthStore()

const fetchConfig = async () => {
  try {
    if (!authStore.isAuthenticated) {
      loading.value = false
      return
    }

    const { data } = await musicAPI.getMetadataConfig()
    config.value = {
      metadata_source: data.metadata_source || 'musicbrainz',
      discogs_token: data.discogs_token || ''
    }
  } catch (e) {
    console.error('Failed to load config', e)
    error('Failed to load config', 'Check server logs')
  } finally {
    loading.value = false
  }
}

const saveConfig = async () => {
  saving.value = true
  try {
    if (!authStore.isAuthenticated) {
      error('Not authenticated', 'Please sign in as an admin before saving settings')
      saving.value = false
      return
    }

    await musicAPI.updateMetadataConfig(config.value)
    success('Settings saved', 'Metadata configuration updated successfully')
  } catch (e) {
    error('Error saving', String(e))
  } finally {
    saving.value = false
  }
}

onMounted(fetchConfig)

// Helper to kick off Google SSO flow (opens auth URL)
const startLogin = async () => {
  try {
    const url = await authStore.getGoogleAuthUrl()
    if (!url) {
      error('Login failed', 'Could not get login URL')
      return
    }
    window.location.href = url
  } catch (e) {
    error('Login error', String(e))
  }
}
</script>

<style scoped>
.metadata-settings {
  padding: 8px;
}

.settings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 32px;
  background: rgba(40, 40, 40, 0.4);
  padding: 24px;
  border-radius: 12px;
}

.header-content h1 {
  margin: 0;
  font-size: 2rem;
}

.subtitle {
  color: var(--spotify-light-grey);
  margin: 4px 0 0;
}

.settings-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(400px, 1fr));
  gap: 24px;
}

.settings-card {
  background: var(--spotify-dark-grey);
  padding: 24px;
  border-radius: 12px;
  border: 1px solid #333;
}

.settings-card.full-width {
  grid-column: 1 / -1;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 12px;
  margin-bottom: 12px;
  color: var(--spotify-green);
}

.card-header h2 {
  margin: 0;
  font-size: 1.25rem;
  color: white;
}

.card-desc {
  color: var(--spotify-light-grey);
  font-size: 0.9rem;
  margin-bottom: 24px;
}

.source-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.source-options-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 12px;
}

.source-option {
  display: flex;
  align-items: flex-start;
  gap: 16px;
  padding: 16px;
  background: #252525;
  border: 2px solid transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
}

.source-option:hover {
  background: #2a2a2a;
}

.source-option.active {
  border-color: var(--spotify-green);
  background: #2d2d2d;
}

.source-option input {
  margin-top: 4px;
}

.option-content {
  display: flex;
  flex-direction: column;
}

.option-content strong {
  display: block;
  font-size: 1rem;
  margin-bottom: 4px;
}

.option-content span {
  font-size: 0.8rem;
  color: var(--spotify-light-grey);
  line-height: 1.4;
}

.form-group {
  margin-top: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 700;
  font-size: 0.9rem;
}

.token-input {
  width: 100%;
  padding: 12px;
  background: #252525;
  border: 1px solid #444;
  border-radius: 4px;
  color: white;
  font-family: monospace;
}

.help-text {
  margin-top: 8px;
  font-size: 0.8rem;
  color: var(--spotify-light-grey);
}

.help-text a {
  color: var(--spotify-green);
  text-decoration: none;
}

.help-text a:hover {
  text-decoration: underline;
}

.btn-primary {
  display: flex;
  align-items: center;
  gap: 8px;
  background: var(--spotify-green);
  color: black;
  border: none;
  padding: 12px 24px;
  border-radius: 500px;
  font-weight: 700;
  cursor: pointer;
  transition: transform 0.1s;
}

.btn-primary:hover {
  transform: scale(1.04);
  background: #1ed760;
}

.btn-primary:active {
  transform: scale(0.98);
}

.btn-primary:disabled {
  opacity: 0.6;
  cursor: not-allowed;
  transform: none;
}

.spin {
  animation: spin 1s linear infinite;
}

@keyframes spin {
  from { transform: rotate(0deg); }
  to { transform: rotate(360deg); }
}
</style>
