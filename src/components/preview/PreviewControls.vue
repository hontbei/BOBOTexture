<script setup lang="ts">
import { computed } from 'vue'

const zoom = defineModel<number>('zoom', { default: 1.0 })
const showGrid = defineModel<boolean>('showGrid', { default: false })

const emit = defineEmits<{
  fit: []
}>()

function fitToCanvas() {
  emit('fit')
}

function resetZoom() {
  zoom.value = 1.0
}

const zoomPercent = computed(() => Math.round(zoom.value * 100) + '%')
</script>

<template>
  <div class="preview-controls">
    <button class="ctrl-btn" @click="fitToCanvas" :title="$t('preview.fit')">◱</button>
    <button class="ctrl-btn" @click="resetZoom" :title="$t('preview.zoom100')">1:1</button>
    <input
      type="range"
      class="zoom-slider"
      :value="zoom"
      min="0.1"
      max="4"
      step="0.05"
      @input="zoom = parseFloat(($event.target as HTMLInputElement).value)"
      title="Zoom"
    />
    <span class="zoom-label">{{ zoomPercent }}</span>
    <label class="grid-toggle" :title="$t('preview.grid')">
      <input v-model="showGrid" type="checkbox" />
      <span>{{ $t('preview.grid') }}</span>
    </label>
  </div>
</template>

<style scoped>
.preview-controls {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 6px 12px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--divider);
  flex-shrink: 0;
}

.ctrl-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: 14px;
}

.ctrl-btn:hover {
  background: var(--bg-hover);
}

.zoom-slider {
  width: 100px;
  accent-color: var(--accent);
}

.zoom-label {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  min-width: 36px;
}

.grid-toggle {
  display: flex;
  align-items: center;
  gap: 4px;
  font-size: 11px;
  color: var(--text-muted);
  cursor: pointer;
}
</style>
