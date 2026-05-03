<script setup lang="ts">
import { ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useReportStore } from '@/stores/report'
import SettingsSection from './SettingsSection.vue'

const reportStore = useReportStore()
const { t } = useI18n()
const copiedTagKey = ref<string | null>(null)
let copyTimer: ReturnType<typeof setTimeout> | null = null

async function copyTag(tag: string, key: string) {
  try {
    await navigator.clipboard.writeText(tag)
    copiedTagKey.value = key
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => { copiedTagKey.value = null }, 1600)
  } catch {
    // clipboard not available
  }
}

function formatLabel(format: string) {
  if (format === 'png_only') return t('atlaspro.formats.pngOnly')
  if (format === 'json_array') return t('atlaspro.formats.jsonArray')
  if (format === 'tmp_sprite_asset') return t('atlaspro.formats.tmpSpriteAsset')
  return format
}

function exampleLabel(key: string) {
  if (key === 'default') return t('atlaspro.result.defaultAssetTag')
  if (key === 'explicit') return t('atlaspro.result.explicitAssetTag')
  if (key === 'index') return t('atlaspro.result.indexTag')
  return key
}
</script>

<template>
  <div v-if="reportStore.report">
    <SettingsSection :title="$t('atlaspro.sections.result')" section-id="result">
      <div class="result-summary">
        <div class="stat">
          <span class="stat-value">{{ reportStore.report.placed.length }}</span>
          <span class="stat-label">{{ $t('atlaspro.result.packedCount') }}</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ reportStore.report.skipped.length }}</span>
          <span class="stat-label">{{ $t('atlaspro.result.skippedCount') }}</span>
        </div>
        <div class="stat">
          <span class="stat-value">{{ reportStore.report.outputs.length }}</span>
          <span class="stat-label">{{ $t('atlaspro.result.filesCount', { count: 0 }) }}</span>
        </div>
      </div>

      <div v-if="reportStore.groupedOutputs.length" class="output-list">
        <div v-for="group in reportStore.groupedOutputs" :key="group.format" class="output-group">
          <div class="output-format">{{ formatLabel(group.format) }}</div>
          <div v-for="path in group.paths" :key="path" class="output-path" :title="path">
            {{ path.split('/').pop() || path.split('\\').pop() }}
          </div>
        </div>
      </div>

      <div v-if="reportStore.report.skipped.length" class="skipped-list">
        <div class="skipped-title">{{ $t('atlaspro.result.skippedTitle') }}</div>
        <div v-for="s in reportStore.report.skipped" :key="s.id" class="skipped-item">
          <strong>{{ s.name }}</strong>: {{ s.reason }}
        </div>
      </div>

      <div v-if="reportStore.tmpExamples.length" class="tmp-examples">
        <div class="tmp-title">{{ $t('atlaspro.result.tmpUsageTitle') }}</div>
        <div v-for="ex in reportStore.tmpExamples" :key="ex.key" class="tmp-example">
          <div class="tmp-example-header">
            <span>{{ exampleLabel(ex.key) }}</span>
            <button class="copy-btn" @click="copyTag(ex.tag, ex.key)">
              {{ copiedTagKey === ex.key ? $t('atlaspro.result.copied') : $t('atlaspro.result.copyTag') }}
            </button>
          </div>
          <code class="tmp-code">{{ ex.tag }}</code>
        </div>
      </div>
    </SettingsSection>
  </div>
</template>

<style scoped>
.result-summary {
  display: flex;
  gap: 12px;
}

.stat {
  text-align: center;
  flex: 1;
}

.stat-value {
  display: block;
  font-size: 18px;
  font-weight: 700;
  color: var(--accent);
  font-family: var(--font-mono);
}

.stat-label {
  font-size: 10px;
  color: var(--text-muted);
  text-transform: uppercase;
}

.output-list {
  margin-top: 10px;
}

.output-group {
  margin-bottom: 6px;
}

.output-format {
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.output-path {
  font-size: 11px;
  color: var(--text-muted);
  font-family: var(--font-mono);
  padding-left: 8px;
}

.skipped-list {
  margin-top: 10px;
}

.skipped-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--warning);
  margin-bottom: 4px;
}

.skipped-item {
  font-size: 11px;
  color: var(--text-secondary);
  padding: 2px 0;
}

.tmp-examples {
  margin-top: 10px;
}

.tmp-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 6px;
}

.tmp-example {
  margin-bottom: 8px;
}

.tmp-example-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  font-size: 11px;
  color: var(--text-secondary);
  margin-bottom: 3px;
}

.copy-btn {
  font-size: 10px;
  padding: 2px 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--accent);
}

.copy-btn:hover {
  background: var(--accent-light);
}

.tmp-code {
  display: block;
  padding: 6px 8px;
  background: #f1f3f4;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  overflow-x: auto;
}
</style>
