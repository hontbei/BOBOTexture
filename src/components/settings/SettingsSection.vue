<script setup lang="ts">
import { useUiStore } from '@/stores/ui'

const props = defineProps<{
  title: string
  sectionId: string
}>()

const ui = useUiStore()

function toggle() {
  ui.toggleSection(props.sectionId)
}
</script>

<template>
  <div class="settings-section">
    <button class="section-header" @click="toggle">
      <span class="section-chevron">{{ ui.isSectionOpen(sectionId) ? '▾' : '▸' }}</span>
      <span class="section-title">{{ title }}</span>
    </button>
    <div v-if="ui.isSectionOpen(sectionId)" class="section-body">
      <slot />
    </div>
  </div>
</template>

<style scoped>
.settings-section {
  border: 1px solid var(--divider);
  border-radius: var(--radius-md);
  background: var(--bg-panel);
  overflow: hidden;
}

.settings-section + .settings-section {
  margin-top: 8px;
}

.section-header {
  display: flex;
  align-items: center;
  gap: 6px;
  width: 100%;
  padding: 10px 12px;
  border: none;
  background: transparent;
  font-size: 13px;
  font-weight: 600;
  color: var(--text-primary);
  cursor: pointer;
}

.section-header:hover {
  background: var(--bg-hover);
}

.section-chevron {
  font-size: 10px;
  color: var(--text-muted);
  width: 12px;
  transition: transform 0.15s;
}

.section-title {
  flex: 1;
  text-align: left;
}

.section-body {
  padding: 8px 12px 12px;
  border-top: 1px solid var(--divider);
}
</style>
