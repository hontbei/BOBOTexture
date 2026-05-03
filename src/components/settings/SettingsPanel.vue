<script setup lang="ts">
import { usePackStore } from '@/stores/pack'
import LayoutSection from './LayoutSection.vue'
import SpacingSection from './SpacingSection.vue'
import TrimSection from './TrimSection.vue'
import ExportSection from './ExportSection.vue'
import AdvancedSection from './AdvancedSection.vue'
import ResultSection from './ResultSection.vue'

const pack = usePackStore()

async function handleExecute() {
  try {
    await pack.executePack()
  } catch (err) {
    console.error('Pack failed:', err)
  }
}
</script>

<template>
  <div class="settings-panel">
    <div class="settings-scroll">
      <LayoutSection />
      <SpacingSection />
      <TrimSection />
      <AdvancedSection />
      <ResultSection />
    </div>
    <div class="execute-bar">
      <button
        class="execute-btn"
        :disabled="!pack.canExecute"
        @click="handleExecute"
      >
        {{ pack.busy ? $t('common.loading') : $t('atlaspro.execute') }}
      </button>
    </div>
  </div>
</template>

<style scoped>
.settings-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.settings-scroll {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.execute-bar {
  padding: 10px 12px;
  border-top: 1px solid var(--divider);
  background: var(--bg-panel);
  flex-shrink: 0;
}

.execute-btn {
  width: 100%;
  height: 36px;
  border: none;
  border-radius: var(--radius-sm);
  background: var(--accent);
  color: var(--text-inverse);
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: background 0.15s;
}

.execute-btn:hover:not(:disabled) {
  background: var(--accent-hover);
}

.execute-btn:disabled {
  background: var(--bg-badge);
  color: var(--text-muted);
}
</style>
