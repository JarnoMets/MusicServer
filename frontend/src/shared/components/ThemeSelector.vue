<template>
  <div class="theme-selector">
    <button 
      class="theme-trigger"
      @click="isOpen = !isOpen"
      :title="'Theme: ' + themes[currentTheme].label"
    >
      <Icon name="palette" :size="18" />
      <span class="trigger-label">{{ themes[currentTheme].label }}</span>
      <Icon name="chevron-down" :size="14" :class="['trigger-chevron', { open: isOpen }]" />
    </button>

    <Transition name="dropdown">
      <div v-if="isOpen" class="theme-dropdown" @click.stop>
        <div class="dropdown-header">
          <span class="dropdown-title">Choose Theme</span>
        </div>
        <div class="theme-grid">
          <button
            v-for="theme in themeList"
            :key="theme.name"
            :class="['theme-option', { active: theme.name === currentTheme }]"
            @click="selectTheme(theme.name)"
          >
            <div class="option-preview" :style="getPreviewStyle(theme)"></div>
            <span class="option-label">{{ theme.label }}</span>
            <Icon v-if="theme.name === currentTheme" name="check" :size="16" class="option-check" />
          </button>
        </div>
      </div>
    </Transition>

    <!-- Backdrop for closing -->
    <div v-if="isOpen" class="backdrop" @click="isOpen = false"></div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import { useTheme, themes, type ThemeName, type Theme } from '../../composables/useTheme'
import Icon from './Icons.vue'

const { currentTheme, setTheme } = useTheme()
const isOpen = ref(false)

const themeList = computed(() => Object.values(themes))

const selectTheme = (themeName: ThemeName) => {
  setTheme(themeName)
  isOpen.value = false
}

const getPreviewStyle = (theme: Theme) => ({
  background: `linear-gradient(135deg, ${theme.colors.primary} 0%, ${theme.colors.accent} 100%)`,
  borderColor: theme.colors.primary,
})

// Close on escape
const handleKeydown = (e: KeyboardEvent) => {
  if (e.key === 'Escape') {
    isOpen.value = false
  }
}

onMounted(() => {
  document.addEventListener('keydown', handleKeydown)
})

onUnmounted(() => {
  document.removeEventListener('keydown', handleKeydown)
})
</script>

<style scoped>
.theme-selector {
  position: relative;
  z-index: 200;
}

.theme-trigger {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px 14px;
  background: var(--surface-muted);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-md);
  color: var(--text-secondary);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all var(--transition-base);
}

.theme-trigger:hover {
  background: var(--surface-hover);
  border-color: var(--border-hover);
  color: var(--text-color);
}

.trigger-label {
  font-weight: 600;
}

.trigger-chevron {
  transition: transform var(--transition-base);
  opacity: 0.6;
}

.trigger-chevron.open {
  transform: rotate(180deg);
}

.backdrop {
  position: fixed;
  inset: 0;
  z-index: 150;
}

.theme-dropdown {
  position: absolute;
  top: calc(100% + 8px);
  right: 0;
  min-width: 280px;
  background: var(--background-elevated);
  border: 1px solid var(--border-color);
  border-radius: var(--radius-lg);
  box-shadow: 0 20px 60px rgba(0, 0, 0, 0.4), 0 0 0 1px rgba(255, 255, 255, 0.05);
  overflow: hidden;
  z-index: 200;
}

.dropdown-header {
  padding: 14px 16px;
  border-bottom: 1px solid var(--border-color);
  background: var(--surface-muted);
}

.dropdown-title {
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.1em;
  color: var(--text-tertiary);
}

.theme-grid {
  padding: 8px;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.theme-option {
  display: flex;
  align-items: center;
  gap: 12px;
  width: 100%;
  padding: 10px 12px;
  background: transparent;
  border: 1px solid transparent;
  border-radius: var(--radius-md);
  cursor: pointer;
  transition: all var(--transition-base);
  text-align: left;
}

.theme-option:hover {
  background: var(--surface-muted);
  border-color: var(--border-color);
}

.theme-option.active {
  background: var(--accent-muted);
  border-color: var(--primary-color);
}

.option-preview {
  width: 36px;
  height: 36px;
  border-radius: var(--radius-sm);
  border: 2px solid;
  flex-shrink: 0;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
}

.option-label {
  flex: 1;
  font-size: 14px;
  font-weight: 600;
  color: var(--text-color);
}

.option-check {
  color: var(--primary-color);
}

/* Dropdown animation */
.dropdown-enter-active,
.dropdown-leave-active {
  transition: all 0.2s ease;
}

.dropdown-enter-from,
.dropdown-leave-to {
  opacity: 0;
  transform: translateY(-8px) scale(0.95);
}

/* Responsive */
@media (max-width: 768px) {
  .trigger-label {
    display: none;
  }

  .theme-dropdown {
    min-width: 240px;
    right: -16px;
  }
}
</style>
