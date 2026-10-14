import { ref } from 'vue'

const visible = ref(false)

export function isDemoHost(): boolean {
  if (typeof window === 'undefined') return false
  return window.location.hostname === 'demo.jarnomets.com'
}

export function useDemoReadonly() {
  function showDemoReadonly() {
    visible.value = true
  }

  function dismissDemoReadonly() {
    visible.value = false
  }

  return {
    visible,
    showDemoReadonly,
    dismissDemoReadonly
  }
}

export function isDemoReadonlyError(err: unknown): boolean {
  const data = (err as { response?: { status?: number; data?: { error?: string } } })?.response
  return data?.status === 403 && data?.data?.error === 'demo_readonly'
}
