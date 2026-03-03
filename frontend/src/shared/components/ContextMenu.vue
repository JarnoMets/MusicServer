<template>
  <div v-show="modelValue" class="context-menu" :style="style" ref="menuRef">
    <slot></slot>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed, watch } from 'vue'

const props = defineProps<{
  modelValue: boolean
  x: number
  y: number
}>()

const emit = defineEmits(['update:modelValue'])

const menuRef = ref<HTMLElement | null>(null)

const style = computed(() => ({
  top: `${props.y}px`,
  left: `${props.x}px`
}))

const close = () => {
  emit('update:modelValue', false)
}

const handleClickOutside = (e: MouseEvent) => {
  if (menuRef.value && !menuRef.value.contains(e.target as Node)) {
    close()
  }
}

onMounted(() => {
  document.addEventListener('mousedown', handleClickOutside)
  document.addEventListener('contextmenu', close) // Close on next right click too
})

onUnmounted(() => {
  document.removeEventListener('mousedown', handleClickOutside)
  document.removeEventListener('contextmenu', close)
})

watch(() => props.modelValue, (val) => {
  if (val) {
    // Ensure visibility within viewport
    setTimeout(() => {
      if (!menuRef.value) return
      const rect = menuRef.value.getBoundingClientRect()
      const winW = window.innerWidth
      const winH = window.innerHeight
      
      let finalX = props.x
      let finalY = props.y
      
      if (props.x + rect.width > winW) {
        finalX = winW - rect.width - 10
      }
      if (props.y + rect.height > winH) {
        finalY = winH - rect.height - 10
      }
      
      menuRef.value.style.left = `${finalX}px`
      menuRef.value.style.top = `${finalY}px`
    }, 0)
  }
})
</script>

<style scoped>
.context-menu {
  position: fixed;
  background: var(--surface-color);
  border: 1px solid var(--border-color);
  border-radius: 12px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.4);
  z-index: 1000;
  min-width: 180px;
  overflow: hidden;
  padding: 8px;
  animation: fadeIn 0.1s ease;
}

@keyframes fadeIn {
  from { opacity: 0; transform: scale(0.95); }
  to { opacity: 1; transform: scale(1); }
}
</style>
