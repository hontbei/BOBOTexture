<script setup lang="ts">
import { ref } from 'vue'

const props = defineProps<{
  title: string
  subtitle?: string
}>()

const emit = defineEmits<{
  submit: [paths: string[]]
}>()

const dragging = ref(false)

function onDragOver(e: DragEvent) {
  e.preventDefault()
  dragging.value = true
}

function onDragLeave() {
  dragging.value = false
}

function onDrop(e: DragEvent) {
  e.preventDefault()
  dragging.value = false
  const files: string[] = []
  if (e.dataTransfer?.files) {
    for (let i = 0; i < e.dataTransfer.files.length; i++) {
      const file = e.dataTransfer.files[i]
      if ('path' in file) {
        files.push((file as any).path as string)
      }
    }
  }
  if (files.length) {
    emit('submit', files)
  }
}
</script>

<template>
  <div
    class="dropzone"
    :class="{ 'dropzone-active': dragging }"
    @dragover="onDragOver"
    @dragleave="onDragLeave"
    @drop="onDrop"
  >
    <div class="dropzone-icon">+</div>
    <div class="dropzone-title">{{ title }}</div>
    <div v-if="subtitle" class="dropzone-subtitle">{{ subtitle }}</div>
  </div>
</template>

<style scoped>
.dropzone {
  margin: 12px;
  padding: 20px 16px;
  border: 2px dashed var(--border-color);
  border-radius: var(--radius-md);
  text-align: center;
  cursor: pointer;
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
