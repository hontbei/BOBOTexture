<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { useFileDrop } from '../../composables/useFileDrop'

defineProps<{
  title: string
  subtitle?: string
}>()

const emit = defineEmits<{
  submit: [paths: string[]]
}>()

const { isDragging, onDragOver, onDragLeave, onDrop } = useFileDrop((paths) => {
  emit('submit', paths)
})

async function handleClick() {
  const selected = await open({
    multiple: true,
    filters: [
      { name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] },
      { name: 'All Files', extensions: ['*'] },
    ],
  })
  if (!selected) return
  const paths = Array.isArray(selected) ? selected : [selected]
  emit('submit', paths)
}
</script>

<template>
  <button
    type="button"
    class="dropzone"
    :class="{ 'dropzone-active': isDragging }"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
    @click="handleClick"
  >
    <div class="dropzone-icon">+</div>
    <div class="dropzone-title">{{ title }}</div>
    <div v-if="subtitle" class="dropzone-subtitle">{{ subtitle }}</div>
  </button>
</template>

<style scoped>
.dropzone {
  display: block;
  width: calc(100% - 24px);
  margin: 12px;
  padding: 20px 16px;
  border: 2px dashed var(--border-color);
  border-radius: var(--radius-md);
  text-align: center;
  cursor: pointer;
  background: transparent;
  color: inherit;
  font-family: inherit;
  font-size: inherit;
  transition: border-color 0.15s ease, background 0.15s ease;
}

.dropzone:hover,
.dropzone-active {
  border-color: var(--accent);
  background: var(--accent-light);
}

.dropzone-icon {
  font-size: 24px;
  color: var(--text-muted);
  margin-bottom: 4px;
}

.dropzone-title {
  font-size: 13px;
  color: var(--text-secondary);
}

.dropzone-subtitle {
  font-size: 11px;
  color: var(--text-muted);
  margin-top: 4px;
}
</style>
