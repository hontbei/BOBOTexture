<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProjectStore } from '@/stores/project'
import { scanAtlasProInputs } from '@/ipc/atlaspro'
import DropZone from './DropZone.vue'
import SpriteSearchBar from './SpriteSearchBar.vue'
import SpriteTable from './SpriteTable.vue'

const { t } = useI18n()
const project = useProjectStore()
const filter = ref('')

async function handleDrop(paths: string[]) {
  try {
    const discovered = await scanAtlasProInputs(paths, true)
    project.addSources(discovered)
  } catch (err) {
    console.error('Scan failed:', err)
  }
}
</script>

<template>
  <div class="sprite-list-panel">
    <DropZone
      :title="t('atlaspro.drop')"
      :subtitle="t('atlaspro.dropSub')"
      @submit="handleDrop"
    />

    <div class="list-controls">
      <span class="count-pill">{{ t('atlaspro.spriteCount', { count: project.sourceCount }) }}</span>
      <div class="list-actions">
        <button class="action-btn" :disabled="!project.sourceCount" @click="project.clearSources()">
          {{ t('atlaspro.clearList') }}
        </button>
      </div>
    </div>

    <SpriteSearchBar v-model="filter" />

    <SpriteTable
      :sources="project.sources"
      :filter="filter"
      @remove="project.removeSource"
    />
  </div>
</template>

<style scoped>
.sprite-list-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.list-controls {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 0 12px 8px;
  gap: 8px;
}

.count-pill {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 999px;
  background: var(--accent-light);
  color: var(--accent);
  font-size: 12px;
  font-weight: 600;
}

.list-actions {
  display: flex;
  align-items: center;
  gap: 8px;
}

.action-btn {
  padding: 3px 10px;
  border-radius: var(--radius-sm);
  border: 1px solid var(--border-color);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: 11px;
}

.action-btn:hover:not(:disabled) {
  border-color: var(--danger);
  color: var(--danger);
}
</style>
