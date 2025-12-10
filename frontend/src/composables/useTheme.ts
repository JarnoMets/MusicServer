import { ref, onMounted } from 'vue'

export type ThemeName = 'midnight' | 'aurora' | 'sunset' | 'forest' | 'ocean' | 'lavender' | 'light'

export interface Theme {
  name: ThemeName
  label: string
  colors: {
    // Brand
    primary: string
    primaryHover: string
    primaryDark: string
    primaryLight: string
    accent: string
    accentDark: string
    accentMuted: string
    // Background
    background: string
    backgroundElevated: string
    surface: string
    surfaceHover: string
    surfaceMuted: string
    // Border
    border: string
    borderHover: string
    // Text
    text: string
    textSecondary: string
    textTertiary: string
    // Status
    success: string
    warning: string
    error: string
    info: string
    // Gradients
    headerGradient: string
    bodyGradient: string
  }
}

export const themes: Record<ThemeName, Theme> = {
  midnight: {
    name: 'midnight',
    label: 'Midnight',
    colors: {
      primary: '#8b5cf6',
      primaryHover: '#a78bfa',
      primaryDark: '#6d28d9',
      primaryLight: '#c4b5fd',
      accent: '#22d3ee',
      accentDark: '#0ea5e9',
      accentMuted: 'rgba(34, 211, 238, 0.15)',
      background: '#04050c',
      backgroundElevated: '#0f172a',
      surface: '#152139',
      surfaceHover: '#1f2d4a',
      surfaceMuted: 'rgba(255, 255, 255, 0.04)',
      border: 'rgba(148, 163, 184, 0.2)',
      borderHover: 'rgba(148, 163, 184, 0.4)',
      text: '#f8fafc',
      textSecondary: '#cbd5f5',
      textTertiary: '#94a3b8',
      success: '#06d6a0',
      warning: '#fbbf24',
      error: '#f87171',
      info: '#38bdf8',
      headerGradient: 'linear-gradient(135deg, rgba(15, 23, 42, 0.85), rgba(30, 41, 59, 0.85))',
      bodyGradient: 'radial-gradient(circle at 20% 20%, rgba(93, 63, 211, 0.35), transparent 40%), radial-gradient(circle at 80% 0%, rgba(14, 165, 233, 0.25), transparent 45%), linear-gradient(180deg, #030712 0%, #02040a 100%)',
    },
  },
  aurora: {
    name: 'aurora',
    label: 'Aurora',
    colors: {
      primary: '#10b981',
      primaryHover: '#34d399',
      primaryDark: '#059669',
      primaryLight: '#6ee7b7',
      accent: '#f472b6',
      accentDark: '#db2777',
      accentMuted: 'rgba(244, 114, 182, 0.15)',
      background: '#030712',
      backgroundElevated: '#111827',
      surface: '#1f2937',
      surfaceHover: '#374151',
      surfaceMuted: 'rgba(255, 255, 255, 0.05)',
      border: 'rgba(75, 85, 99, 0.5)',
      borderHover: 'rgba(107, 114, 128, 0.6)',
      text: '#f9fafb',
      textSecondary: '#d1d5db',
      textTertiary: '#9ca3af',
      success: '#10b981',
      warning: '#fbbf24',
      error: '#ef4444',
      info: '#3b82f6',
      headerGradient: 'linear-gradient(135deg, rgba(17, 24, 39, 0.9), rgba(31, 41, 55, 0.85))',
      bodyGradient: 'radial-gradient(circle at 15% 50%, rgba(16, 185, 129, 0.2), transparent 40%), radial-gradient(circle at 85% 20%, rgba(244, 114, 182, 0.15), transparent 40%), linear-gradient(180deg, #030712 0%, #0a0a0a 100%)',
    },
  },
  sunset: {
    name: 'sunset',
    label: 'Sunset',
    colors: {
      primary: '#f97316',
      primaryHover: '#fb923c',
      primaryDark: '#ea580c',
      primaryLight: '#fdba74',
      accent: '#ec4899',
      accentDark: '#db2777',
      accentMuted: 'rgba(236, 72, 153, 0.15)',
      background: '#0c0a09',
      backgroundElevated: '#1c1917',
      surface: '#292524',
      surfaceHover: '#44403c',
      surfaceMuted: 'rgba(255, 255, 255, 0.05)',
      border: 'rgba(120, 113, 108, 0.4)',
      borderHover: 'rgba(168, 162, 158, 0.5)',
      text: '#fafaf9',
      textSecondary: '#e7e5e4',
      textTertiary: '#a8a29e',
      success: '#22c55e',
      warning: '#fbbf24',
      error: '#ef4444',
      info: '#3b82f6',
      headerGradient: 'linear-gradient(135deg, rgba(28, 25, 23, 0.9), rgba(41, 37, 36, 0.85))',
      bodyGradient: 'radial-gradient(circle at 20% 80%, rgba(249, 115, 22, 0.25), transparent 45%), radial-gradient(circle at 80% 20%, rgba(236, 72, 153, 0.2), transparent 40%), linear-gradient(180deg, #0c0a09 0%, #0a0908 100%)',
    },
  },
  forest: {
    name: 'forest',
    label: 'Forest',
    colors: {
      primary: '#22c55e',
      primaryHover: '#4ade80',
      primaryDark: '#16a34a',
      primaryLight: '#86efac',
      accent: '#84cc16',
      accentDark: '#65a30d',
      accentMuted: 'rgba(132, 204, 22, 0.15)',
      background: '#052e16',
      backgroundElevated: '#14532d',
      surface: '#166534',
      surfaceHover: '#15803d',
      surfaceMuted: 'rgba(255, 255, 255, 0.06)',
      border: 'rgba(74, 222, 128, 0.25)',
      borderHover: 'rgba(74, 222, 128, 0.4)',
      text: '#f0fdf4',
      textSecondary: '#bbf7d0',
      textTertiary: '#86efac',
      success: '#4ade80',
      warning: '#facc15',
      error: '#f87171',
      info: '#38bdf8',
      headerGradient: 'linear-gradient(135deg, rgba(20, 83, 45, 0.9), rgba(22, 101, 52, 0.85))',
      bodyGradient: 'radial-gradient(circle at 30% 70%, rgba(34, 197, 94, 0.2), transparent 45%), radial-gradient(circle at 70% 20%, rgba(132, 204, 22, 0.15), transparent 40%), linear-gradient(180deg, #052e16 0%, #022c22 100%)',
    },
  },
  ocean: {
    name: 'ocean',
    label: 'Ocean',
    colors: {
      primary: '#0ea5e9',
      primaryHover: '#38bdf8',
      primaryDark: '#0284c7',
      primaryLight: '#7dd3fc',
      accent: '#06b6d4',
      accentDark: '#0891b2',
      accentMuted: 'rgba(6, 182, 212, 0.15)',
      background: '#020617',
      backgroundElevated: '#0f172a',
      surface: '#1e3a5f',
      surfaceHover: '#1e40af',
      surfaceMuted: 'rgba(255, 255, 255, 0.05)',
      border: 'rgba(56, 189, 248, 0.2)',
      borderHover: 'rgba(56, 189, 248, 0.4)',
      text: '#f0f9ff',
      textSecondary: '#bae6fd',
      textTertiary: '#7dd3fc',
      success: '#10b981',
      warning: '#fbbf24',
      error: '#f87171',
      info: '#38bdf8',
      headerGradient: 'linear-gradient(135deg, rgba(15, 23, 42, 0.9), rgba(30, 58, 95, 0.85))',
      bodyGradient: 'radial-gradient(circle at 50% 100%, rgba(14, 165, 233, 0.25), transparent 50%), radial-gradient(circle at 80% 20%, rgba(6, 182, 212, 0.2), transparent 40%), linear-gradient(180deg, #020617 0%, #0a0a1a 100%)',
    },
  },
  lavender: {
    name: 'lavender',
    label: 'Lavender',
    colors: {
      primary: '#a855f7',
      primaryHover: '#c084fc',
      primaryDark: '#9333ea',
      primaryLight: '#d8b4fe',
      accent: '#e879f9',
      accentDark: '#d946ef',
      accentMuted: 'rgba(232, 121, 249, 0.15)',
      background: '#0a0118',
      backgroundElevated: '#1e1033',
      surface: '#2d1f4a',
      surfaceHover: '#3d2a66',
      surfaceMuted: 'rgba(255, 255, 255, 0.05)',
      border: 'rgba(168, 85, 247, 0.25)',
      borderHover: 'rgba(168, 85, 247, 0.45)',
      text: '#faf5ff',
      textSecondary: '#e9d5ff',
      textTertiary: '#c4b5fd',
      success: '#10b981',
      warning: '#fbbf24',
      error: '#f87171',
      info: '#38bdf8',
      headerGradient: 'linear-gradient(135deg, rgba(30, 16, 51, 0.9), rgba(45, 31, 74, 0.85))',
      bodyGradient: 'radial-gradient(circle at 25% 25%, rgba(168, 85, 247, 0.3), transparent 45%), radial-gradient(circle at 75% 75%, rgba(232, 121, 249, 0.2), transparent 45%), linear-gradient(180deg, #0a0118 0%, #050008 100%)',
    },
  },
  light: {
    name: 'light',
    label: 'Light',
    colors: {
      primary: '#6366f1',
      primaryHover: '#818cf8',
      primaryDark: '#4f46e5',
      primaryLight: '#a5b4fc',
      accent: '#06b6d4',
      accentDark: '#0891b2',
      accentMuted: 'rgba(6, 182, 212, 0.1)',
      background: '#f8fafc',
      backgroundElevated: '#ffffff',
      surface: '#f1f5f9',
      surfaceHover: '#e2e8f0',
      surfaceMuted: 'rgba(0, 0, 0, 0.03)',
      border: 'rgba(148, 163, 184, 0.3)',
      borderHover: 'rgba(148, 163, 184, 0.5)',
      text: '#0f172a',
      textSecondary: '#475569',
      textTertiary: '#64748b',
      success: '#10b981',
      warning: '#f59e0b',
      error: '#ef4444',
      info: '#3b82f6',
      headerGradient: 'linear-gradient(135deg, rgba(255, 255, 255, 0.95), rgba(241, 245, 249, 0.9))',
      bodyGradient: 'linear-gradient(180deg, #f8fafc 0%, #f1f5f9 100%)',
    },
  },
}

const STORAGE_KEY = 'music_server_theme'
const currentTheme = ref<ThemeName>('midnight')

const applyTheme = (themeName: ThemeName) => {
  const theme = themes[themeName]
  if (!theme) return

  const root = document.documentElement
  const { colors } = theme

  root.style.setProperty('--primary-color', colors.primary)
  root.style.setProperty('--primary-hover', colors.primaryHover)
  root.style.setProperty('--primary-dark', colors.primaryDark)
  root.style.setProperty('--primary-light', colors.primaryLight)
  root.style.setProperty('--accent-color', colors.accent)
  root.style.setProperty('--accent-dark', colors.accentDark)
  root.style.setProperty('--accent-muted', colors.accentMuted)
  root.style.setProperty('--background-color', colors.background)
  root.style.setProperty('--background-elevated', colors.backgroundElevated)
  root.style.setProperty('--surface-color', colors.surface)
  root.style.setProperty('--surface-hover', colors.surfaceHover)
  root.style.setProperty('--surface-muted', colors.surfaceMuted)
  root.style.setProperty('--border-color', colors.border)
  root.style.setProperty('--border-hover', colors.borderHover)
  root.style.setProperty('--text-color', colors.text)
  root.style.setProperty('--text-secondary', colors.textSecondary)
  root.style.setProperty('--text-tertiary', colors.textTertiary)
  root.style.setProperty('--success-color', colors.success)
  root.style.setProperty('--warning-color', colors.warning)
  root.style.setProperty('--error-color', colors.error)
  root.style.setProperty('--info-color', colors.info)
  root.style.setProperty('--header-gradient', colors.headerGradient)
  root.style.setProperty('--body-gradient', colors.bodyGradient)

  // Set a class on body for additional theme-specific styling
  document.body.setAttribute('data-theme', themeName)
}

const setTheme = (themeName: ThemeName) => {
  currentTheme.value = themeName
  localStorage.setItem(STORAGE_KEY, themeName)
  applyTheme(themeName)
}

const loadTheme = () => {
  const stored = localStorage.getItem(STORAGE_KEY) as ThemeName | null
  if (stored && themes[stored]) {
    currentTheme.value = stored
  }
  applyTheme(currentTheme.value)
}

export const useTheme = () => {
  onMounted(() => {
    loadTheme()
  })

  return {
    currentTheme,
    themes,
    setTheme,
    loadTheme,
  }
}

// Initialize theme immediately (before Vue mounts)
if (typeof window !== 'undefined') {
  const stored = localStorage.getItem(STORAGE_KEY) as ThemeName | null
  if (stored && themes[stored]) {
    applyTheme(stored)
  } else {
    applyTheme('midnight')
  }
}
