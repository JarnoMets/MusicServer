/**
 * Composable for managing confirm dialogs
 * Provides a Promise-based API for showing confirm modals
 */
import { reactive } from 'vue'

interface ConfirmOptions {
  title?: string
  message: string
  confirmText?: string
  cancelText?: string
  variant?: 'danger' | 'warning' | 'info'
}

interface ConfirmState {
  show: boolean
  title: string
  message: string
  confirmText: string
  cancelText: string
  variant: 'danger' | 'warning' | 'info'
  resolve: ((value: boolean) => void) | null
}

// Singleton state
const state = reactive<ConfirmState>({
  show: false,
  title: 'Confirm',
  message: '',
  confirmText: 'Confirm',
  cancelText: 'Cancel',
  variant: 'danger',
  resolve: null,
})

const confirm = (options: ConfirmOptions): Promise<boolean> => {
  return new Promise((resolve) => {
    state.show = true
    state.title = options.title || 'Confirm'
    state.message = options.message
    state.confirmText = options.confirmText || 'Confirm'
    state.cancelText = options.cancelText || 'Cancel'
    state.variant = options.variant || 'danger'
    state.resolve = resolve
  })
}

const handleConfirm = () => {
  if (state.resolve) {
    state.resolve(true)
  }
  state.show = false
  state.resolve = null
}

const handleCancel = () => {
  if (state.resolve) {
    state.resolve(false)
  }
  state.show = false
  state.resolve = null
}

export const useConfirm = () => {
  return {
    state,
    confirm,
    handleConfirm,
    handleCancel,
  }
}
