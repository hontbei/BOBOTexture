<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import SettingsPanel from '../settings/SettingsPanel.vue'

const panelRef = ref<HTMLElement | null>(null)
const width = ref(320)
const dragging = ref(false)
let startX = 0
let startWidth = 0

function onMouseDown(e: MouseEvent) {
  dragging.value = true
  startX = e.clientX
  startWidth = width.value
  document.body.style.cursor = 'col-resize'
  document.body.style.userSelect = 'none'
}

function onMouseMove(e: MouseEvent) {
  if (!dragging.value) return
  const delta = startX - e.clientX
  const next = Math.max(280, Math.min(500, startWidth + delta))
  width.value = next
}

function onMouseUp() {
  if (!dragging.value) return
  dragging.value = false
  document.body.style.cursor = ''
  document.body.style.userSelect = ''
}

onMounted(() => {
  document.addEventListener('mousemove', onMouseMove)
  document.addEventListener('mouseup', onMouseUp)
})

onUnmounted(() => {
  document.removeEventListener('mousemove', onMouseMove)
  document.removeEventListener('mouseup', onMouseUp)
})
</script>

<template>
  <div ref="panelRef" class="panel-right panel-glass" :style="{ width: width + 'px' }">
    <div
      class="resize-handle resize-handle-left"
      :class="{ 'resize-active': dragging }"
      @mousedown="onMouseDown"
    />
    <div class="panel-header">
      <span class="panel-title">Settings</span>
    </div>
    <SettingsPanel />
  </div>
</template>

<style scoped>
.panel-right {
  position: relative;
  display: flex;
  flex-direction: column;
  border-right: none;
  border-left: 1px solid var(--divider);
  overflow: hidden;
}

.panel-header {
  display: flex;
  align-items: center;
  height: 40px;
  padding: 0 16px;
  border-bottom: 1px solid var(--divider);
  flex-shrink: 0;
}

.panel-title {
  font-size: 12px;
  font-weight: 600;
  text-transform: uppercase;
  letter-spacing: 0.05em;
  color: var(--text-secondary);
}

/* Resize handle */
.resize-handle {
  position: absolute;
  top: 0;
  bottom: 0;
  width: 4px;
  cursor: col-resize;
  z-index: 10;
  background: transparent;
  transition: background 0.15s ease;
}

.resize-handle-left {
  left: -2px;
}

.resize-handle:hover,
.resize-active {
  background: var(--accent);
}

.resize-active {
  background: var(--accent);
}
</style>
