<script setup lang="ts">
import { computed } from 'vue'
import type { SpriteSource } from '@/types'
import { useUiStore } from '@/stores/ui'
import { useProjectStore } from '@/stores/project'
import SpriteRow from './SpriteRow.vue'

const props = defineProps<{
  sources: SpriteSource[]
  filter: string
}>()

const emit = defineEmits<{
  remove: [source: SpriteSource]
}>()

const ui = useUiStore()

const filteredSources = computed(() => {
  const f = props.filter.toLowerCase().trim()
  if (!f) return props.sources
  return props.sources.filter(s => s.name.toLowerCase().includes(f))
})

function onSelect(source: SpriteSource) {
  ui.setSelection(source.id === ui.selectedSpriteId ? null : source.id)
}

function onHover(source: SpriteSource) {
  ui.setHover(source.id)
}

function onLeave() {
  ui.setHover(null)
}
</script>

<template>
  <div class="sprite-table">
    <div class="table-header">
      <span>{{ $t('atlaspro.table.name') }}</span>
      <span>{{ $t('atlaspro.table.origin') }}</span>
      <span>{{ $t('atlaspro.table.size') }}</span>
      <span>{{ $t('atlaspro.table.path') }}</span>
      <span>{{ $t('atlaspro.table.actions') }}</span>
    </div>
    <div class="table-body">
      <SpriteRow
        v-for="source in filteredSources"
        :key="source.id"
        :source="source"
        :selected="ui.selectedSpriteId === source.id"
        :hovered="ui.hoveredSpriteId === source.id"
        @select="onSelect(source)"
        @hover="onHover(source)"
        @leave="onLeave()"
        @remove="emit('remove', source)"
        @remove-context="emit('remove', source)"
      />
      <div v-if="!filteredSources.length" class="table-empty">
        {{ filter ? $t('empty.noMatching') : $t('empty.noSprites') }}
      </div>
    </div>
  </div>
</template>

<style scoped>
.sprite-table {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.table-header {
  display: grid;
  grid-template-columns: minmax(120px, 1.2fr) 70px 80px minmax(140px, 2fr) 32px;
  padding: 6px 8px;
  border-bottom: 2px solid var(--divider);
  gap: 4px;
  flex-shrink: 0;
}

.table-header span {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-muted);
  text-transform: uppercase;
}

.table-body {
  flex: 1;
  overflow-y: auto;
  min-height: 0;
}

.table-empty {
  padding: 32px 16px;
  text-align: center;
  color: var(--text-muted);
  font-size: 13px;
}
</style>
