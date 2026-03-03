<template>
  <div class="menu-item" :class="{ 'danger': danger }" @click="$emit('click')">
    <div class="icon">
      <slot name="icon"></slot>
    </div>
    <span class="label">
      <slot></slot>
    </span>
  </div>
</template>

<script setup lang="ts">
defineProps<{
  danger?: boolean
}>()
defineEmits(['click'])
</script>

<style scoped>
.menu-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 10px 14px;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s ease;
  font-size: 14px;
  color: var(--text-color);
}

.menu-item:hover {
  background: var(--background-elevated);
  transform: translateX(4px);
}

.menu-item.danger {
  color: var(--error-color);
}

.context-menu-item.danger:hover {
  background: rgba(255, 68, 68, 0.1);
}

.icon {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 18px;
  height: 18px;
  opacity: 0.7;
}

.label {
  flex: 1;
}
</style>
