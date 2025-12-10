import { ref, readonly } from 'vue'

export type ToastType = 'success' | 'error' | 'warning' | 'info'

export interface Toast {
  id: string
  type: ToastType
  title: string
  message?: string
  duration?: number
}

const toasts = ref<Toast[]>([])
let toastId = 0

const addToast = (toast: Omit<Toast, 'id'>) => {
  const id = `toast-${++toastId}`
  const newToast: Toast = {
    ...toast,
    id,
    duration: toast.duration ?? 4000,
  }
  
  toasts.value.push(newToast)

  // Auto remove after duration
  const duration = newToast.duration ?? 4000
  if (duration > 0) {
    setTimeout(() => {
      removeToast(id)
    }, newToast.duration)
  }

  return id
}

const removeToast = (id: string) => {
  const index = toasts.value.findIndex(t => t.id === id)
  if (index > -1) {
    toasts.value.splice(index, 1)
  }
}

const clearAllToasts = () => {
  toasts.value = []
}

// Convenience methods
const success = (title: string, message?: string, duration?: number) => {
  return addToast({ type: 'success', title, message, duration })
}

const error = (title: string, message?: string, duration?: number) => {
  return addToast({ type: 'error', title, message, duration: duration ?? 6000 })
}

const warning = (title: string, message?: string, duration?: number) => {
  return addToast({ type: 'warning', title, message, duration })
}

const info = (title: string, message?: string, duration?: number) => {
  return addToast({ type: 'info', title, message, duration })
}

export const useToast = () => {
  return {
    toasts: readonly(toasts),
    addToast,
    removeToast,
    clearAllToasts,
    success,
    error,
    warning,
    info,
  }
}
