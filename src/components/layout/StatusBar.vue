<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { useProjectStore } from '@/stores/project'
import { useReportStore } from '@/stores/report'
import { usePackStore } from '@/stores/pack'

const project = useProjectStore()
const reportStore = useReportStore()
const pack = usePackStore()
const { t } = useI18n()

const spriteCount = computed(() => project.sourceCount)

const atlasSize = computed(() => {
  const r = reportStore.report
  if (!r) return null
  return { width: r.atlasSize.width, height: r.atlasSize.height }
})

const formatDisplay = computed(() => {
  if (!pack.formats.length) return t('status.noFormat')
  const labels: Record<string, string> = {
    png_only: 'PNG',
    json_array: 'JSON',
    tmp_sprite_asset: 'TMP',
  }
  return pack.formats.map(f => labels[f] || f).join(' + ')
})
</script>

<template>
  <div class="statusbar">
    <div class="status-section">
      <span class="status-label">{{ $t('status.sprites') + ': ' }}</span>
      <span class="status-value">{{ spriteCount }}</span>
    </div>
    <div class="status-divider">|</div>
    <div class="status-section">
      <span class="status-label">{{ $t('status.atlas') + ': ' }}</span>
      <span class="status-value text-mono">{{ atlasSize ? `${atlasSize.width}\u00D7${atlasSize.height}` : '\u2014' }}</span>
    </div>
    <div class="status-spacer" />
    <div v-if="reportStore.lastPackDuration" class="status-section">
      <span class="status-value text-mono">{{ reportStore.lastPackDuration }}ms</span>
    </div>
    <div class="status-section">
      <span class="status-value">{{ formatDisplay }}</span>
    </div>
  </div>
</template>

<style scoped>
.statusbar {
  display: flex;
  align-items: center;
  height: var(--statusbar-height);
  padding: 0 16px;
  background: var(--bg-toolbar);
  gap: 8px;
  font-size: 11px;
}

.status-section {
  display: flex;
  align-items: center;
  gap: 4px;
  white-space: nowrap;
}

.status-label {
  color: var(--text-muted);
}

.status-value {
  color: var(--text-secondary);
  font-weight: 500;
}

.status-divider {
  color: var(--border-color);
  font-size: 10px;
}

.status-spacer {
  flex: 1;
}
</style>
