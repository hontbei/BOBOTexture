<script setup lang="ts">
import { computed, ref } from 'vue'
import { useI18n } from 'vue-i18n'
import { useReportStore } from '@/stores/report'
import SettingsSection from './SettingsSection.vue'

const reportStore = useReportStore()
const { t } = useI18n()
const copied = ref(false)
let copyTimer: ReturnType<typeof setTimeout> | null = null

const allTags = computed(() => reportStore.tmpExamples.join('\n'))

async function copyAll() {
  try {
    await navigator.clipboard.writeText(allTags.value)
    copied.value = true
    if (copyTimer) clearTimeout(copyTimer)
    copyTimer = setTimeout(() => { copied.value = false }, 1600)
  } catch {}
}

function formatLabel(format: string) {
  if (format === 'png_only') return t('atlaspro.formats.pngOnly')
  if (format === 'json_array') return t('atlaspro.formats.jsonArray')
  if (format === 'tmp_sprite_asset') return t('atlaspro.formats.tmpSpriteAsset')
  return format
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
        <div class="tmp-header">
          <span class="tmp-title">{{ $t('atlaspro.result.tmpUsageTitle') }}</span>
          <button class="copy-btn" @click="copyAll">
            {{ copied ? $t('atlaspro.result.copied') : $t('atlaspro.result.copyTag') }}
          </button>
        </div>
        <code class="tmp-code">{{ allTags }}</code>
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

.tmp-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.tmp-title {
  font-size: 11px;
  font-weight: 600;
  color: var(--text-primary);
}

.copy-btn {
  font-size: 10px;
  padding: 2px 8px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--accent);
  cursor: pointer;
}

.copy-btn:hover {
  background: var(--accent-light);
}

.tmp-code {
  display: block;
  padding: 8px 10px;
  background: #f1f3f4;
  border-radius: var(--radius-sm);
  font-size: 11px;
  font-family: var(--font-mono);
  color: var(--text-primary);
  white-space: pre-wrap;
  line-height: 1.7;
  max-height: 200px;
  overflow-y: auto;
}
</style>
