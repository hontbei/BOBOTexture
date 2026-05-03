<script setup lang="ts">
import { computed } from 'vue'
import type { SpriteSource } from '@/types'
import OriginBadge from './OriginBadge.vue'

const props = defineProps<{
  source: SpriteSource
  selected: boolean
  hovered: boolean
}>()

const emit = defineEmits<{
  remove: []
  select: []
  hover: []
  leave: []
}>()

const displaySize = computed(() => `${props.source.subRect.width}×${props.source.subRect.height}`)
const pathDisplay = computed(() => {
  const parts = props.source.sourcePath.replace(/\\/g, '/').split('/')
  return parts.slice(-2).join('/')
})
</script>

<template>
  <div
    class="sprite-row"
    :class="{ 'sprite-row-selected': selected, 'sprite-row-hovered': hovered }"
    @click="emit('select')"
    @mouseenter="emit('hover')"
    @mouseleave="emit('leave')"
  >
    <span class="sprite-col sprite-col-name truncate" :title="source.name">{{ source.name }}</span>
    <span class="sprite-col sprite-col-origin">
      <OriginBadge :origin="source.origin" />
    </span>
    <span class="sprite-col sprite-col-size text-mono">{{ displaySize }}</span>
    <span class="sprite-col sprite-col-path truncate text-muted" :title="source.sourcePath">{{ pathDisplay }}</span>
    <button class="sprite-col sprite-col-remove" @click.stop="emit('remove')" title="Remove">×</button>
  </div>
</template>

<style scoped>
.sprite-row {
  display: grid;
  grid-template-columns: minmax(120px, 1.2fr) 70px 80px minmax(140px, 2fr) 32px;
  align-items: center;
  min-height: 32px;
  padding: 0 8px;
  border-bottom: 1px solid var(--divider);
  cursor: pointer;
  transition: background 0.1s ease;
  gap: 4px;
}

.sprite-row:nth-child(even) {
  background: #fafafa;
}

.sprite-row:hover {
  background: var(--bg-hover);
}

.sprite-row-selected {
  background: var(--bg-selected) !important;
  border-left: 3px solid var(--accent);
  padding-left: 5px;
}

.sprite-row-hovered {
  background: var(--bg-hover);
}

.sprite-col {
  font-size: 12px;
}

.sprite-col-origin {
  text-align: center;
}

.sprite-col-size {
  text-align: right;
  color: var(--text-secondary);
}

.sprite-col-remove {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 24px;
  height: 24px;
  border-radius: 50%;
  border: none;
  background: transparent;
  color: var(--text-muted);
  font-size: 16px;
  cursor: pointer;
  transition: all 0.1s ease;
}

.sprite-col-remove:hover {
  background: #fce8e6;
  color: var(--danger);
}
</style>
