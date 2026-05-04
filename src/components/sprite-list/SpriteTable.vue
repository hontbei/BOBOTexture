<script setup lang="ts">
import { computed, ref } from 'vue'
import type { SpriteSource } from '@/types'
import { useUiStore } from '@/stores/ui'
import SpriteRow from './SpriteRow.vue'

const props = defineProps<{ sources: SpriteSource[]; filter: string }>()
const emit = defineEmits<{ remove: [source: SpriteSource] }>()
const ui = useUiStore()

const ctxMenu = ref<{ x: number; y: number; source: SpriteSource } | null>(null)

const filteredSources = computed(() => {
  const f = props.filter.toLowerCase().trim()
  if (!f) return props.sources
  return props.sources.filter(s => s.name.toLowerCase().includes(f))
})

function onSelect(source: SpriteSource) { ui.setSelection(source.id === ui.selectedSpriteId ? null : source.id) }
function onHover(source: SpriteSource) { ui.setHover(source.id) }
function onLeave() { ui.setHover(null) }

function onContextMenu(source: SpriteSource, e: MouseEvent) {
  ctxMenu.value = { x: e.clientX, y: e.clientY, source }
}

function doRemove() {
  if (ctxMenu.value) { emit('remove', ctxMenu.value.source); ctxMenu.value = null }
}
function closeMenu() { ctxMenu.value = null }
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
    <div class="table-body" @click="closeMenu">
      <SpriteRow
        v-for="source in filteredSources" :key="source.id"
        :source="source"
        :selected="ui.selectedSpriteId === source.id"
        :hovered="ui.hoveredSpriteId === source.id"
        @select="onSelect(source)"
        @hover="onHover(source)"
        @leave="onLeave()"
        @remove="emit('remove', source)"
        @remove-context="onContextMenu(source, $event)"
      />
      <div v-if="!filteredSources.length" class="table-empty">
        {{ filter ? $t('empty.noMatching') : $t('empty.noSprites') }}
      </div>
    </div>
    <Teleport to="body">
      <div
        v-if="ctxMenu"
        class="ctx-menu"
        :style="{ left: ctxMenu.x + 'px', top: ctxMenu.y + 'px' }"
        @click.stop
      >
        <button class="ctx-item" @click="doRemove">{{ $t('common.remove') }}</button>
      </div>
    </Teleport>
  </div>
</template>

<style scoped>
.sprite-table { display: flex; flex-direction: column; height: 100%; }
.table-header {
  display: grid; grid-template-columns: minmax(120px, 1.2fr) 70px 80px minmax(140px, 2fr) 32px;
  padding: 6px 8px; border-bottom: 2px solid var(--divider); gap: 4px; flex-shrink: 0;
}
.table-header span { font-size: 11px; font-weight: 600; color: var(--text-muted); text-transform: uppercase; }
.table-body { flex: 1; overflow-y: auto; min-height: 0; }
.table-empty { padding: 32px 16px; text-align: center; color: var(--text-muted); font-size: 13px; }

.ctx-menu {
  position: fixed; z-index: 10000;
  background: var(--bg-panel); border: 1px solid var(--border-color);
  border-radius: var(--radius-sm); box-shadow: 0 4px 16px rgba(0,0,0,0.12);
  min-width: 100px; padding: 4px 0;
}
.ctx-item {
  display: block; width: 100%; padding: 6px 12px; text-align: left;
  border: none; background: transparent; color: var(--text-primary);
  font-size: 12px; cursor: pointer;
}
.ctx-item:hover { background: var(--bg-hover); color: var(--danger); }
</style>
