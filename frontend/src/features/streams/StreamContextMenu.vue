<script setup lang="ts">
import { ref, onMounted, onUnmounted, nextTick } from 'vue'
import Icon from '../../shared/components/Icons.vue'

interface Props {
  isAdmin: boolean
}

defineProps<Props>()

const emit = defineEmits<{
  play: [stream: any]
  details: [stream: any]
  edit: [stream: any]
  delete: [stream: any]
  'copy-url': [stream: any]
  close: []
}>()

const visible = ref(false)
const position = ref({ x: 0, y: 0 })
const current = ref<any | null>(null)
const menuRef = ref<HTMLElement | null>(null)

const open = async (stream: any, event: MouseEvent) => {
  event.preventDefault()
  current.value = stream
  const x = event.clientX
  const y = event.clientY
  position.value = { x, y }
  visible.value = true

  nextTick(() => {
    if (menuRef.value) {
      const rect = menuRef.value.getBoundingClientRect()
      const vw = window.innerWidth
      const vh = window.innerHeight
      if (rect.right > vw) {
        position.value.x = vw - rect.width - 8
      }
      if (rect.bottom > vh) {
        position.value.y = vh - rect.height - 8
      }
    }
  })
}

const close = () => {
  visible.value = false
  current.value = null
  emit('close')
}

const handleClickOutside = (e: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('click', handleClickOutside)
  document.addEventListener('scroll', close, true)
})

onUnmounted(() => {
  document.removeEventListener('click', handleClickOutside)
  document.removeEventListener('scroll', close, true)
})

defineExpose({ open, close })
</script>

<template>
  <Teleport to="body">
    <div
      v-if="visible && current"
      ref="menuRef"
      class="context-menu"
      :style="{ left: position.x + 'px', top: position.y + 'px' }"
    >
      <button class="menu-item" @click="emit('play', current); close()">
        <Icon name="play" :size="15" />
        <span>Play Stream</span>
      </button>

      <button class="menu-item" @click="emit('details', current); close()">
        <Icon name="chevron-right" :size="15" />
        <span>Open Stream</span>
      </button>

      <div class="menu-separator"></div>

      <button class="menu-item" @click="emit('copy-url', current); close()">
        <Icon name="copy" :size="15" />
        <span>Copy Stream URL</span>
      </button>

      <div v-if="isAdmin" class="menu-separator"></div>

      <button v-if="isAdmin" class="menu-item" @click="emit('edit', current); close()">
        <Icon name="edit" :size="15" />
        <span>Edit Stream</span>
      </button>

      <button v-if="isAdmin" class="menu-item danger" @click="emit('delete', current); close()">
        <Icon name="trash" :size="15" />
        <span>Delete Stream</span>
      </button>
    </div>
  </Teleport>
</template>

<style scoped>
.context-menu {
  position: fixed;
  z-index: 2500;
  min-width: 180px;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  padding: 6px;
  box-shadow: 0 12px 32px rgba(0, 0, 0, 0.4);
  animation: context-fade 0.15s ease-out;
}

@keyframes context-fade {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}

.menu-item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  padding: 9px 14px;
  background: transparent;
  border: none;
  border-radius: 8px;
  color: var(--text-color);
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  text-align: left;
  transition: all 0.15s;
}

.menu-item:hover { background: var(--primary-glow); color: var(--primary-light); }
.menu-item.danger { color: #ef4444; }
.menu-item.danger:hover { background: rgba(239, 68, 68, 0.1); }

.menu-separator { height: 1px; background: var(--border-color); margin: 6px 4px; }
</style>
