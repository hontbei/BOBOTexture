<script setup lang="ts">
import { onMounted, onUnmounted, ref } from 'vue'
import SpriteListPanel from '../sprite-list/SpriteListPanel.vue'

const panelRef = ref<HTMLElement | null>(null)
const width = ref(300)
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
  const delta = e.clientX - startX
  const next = Math.max(240, Math.min(600, startWidth + delta))
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
  <div ref="panelRef" class="panel-left panel-glass" :style="{ width: width + 'px' }">
    <div class="panel-header">
      <span class="panel-title">{{ $t('panels.sprites') }}</span>
    </div>
    <SpriteListPanel />
    <div
      class="resize-handle resize-handle-right"
      :class="{ 'resize-active': dragging }"
      @mousedown="onMouseDown"
    />
  </div>
</template>

<style scoped>
.panel-left {
  position: relative;
  display: flex;
  flex-direction: column;
  min-height: 0;
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

.resize-handle-right {
  right: -2px;
}

.resize-handle:hover,
.resize-active {
  background: var(--accent);
}

.resize-active {
  background: var(--accent);
}
</style>
