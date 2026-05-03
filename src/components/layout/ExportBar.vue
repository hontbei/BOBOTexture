<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { usePackStore } from '@/stores/pack'
import { useI18n } from 'vue-i18n'

const pack = usePackStore()
const { t } = useI18n()

const formats = [
  { key: 'png_only' as const, label: t('atlaspro.formats.pngOnly') },
  { key: 'json_array' as const, label: t('atlaspro.formats.jsonArray') },
  { key: 'tmp_sprite_asset' as const, label: t('atlaspro.formats.tmpSpriteAsset') },
]

function toggle(f: string) {
  if (pack.formats.includes(f as any)) {
    pack.formats = pack.formats.filter(x => x !== f)
  } else {
    pack.formats = [...pack.formats, f as any]
  }
}

async function browse() {
  const r = await open({ directory: true, multiple: false })
  if (typeof r === 'string') pack.outputDir = r
}

const emit = defineEmits<{ publish: [] }>()
</script>

<template>
  <div class="export-bar">
    <div class="eb-section">
      <span class="eb-label">{{ $t('panels.export') }}</span>
      <div class="eb-chips">
        <button
          v-for="f in formats" :key="f.key"
          class="eb-chip" :class="{ active: pack.formats.includes(f.key) }"
          @click="toggle(f.key)"
        >{{ f.label }}</button>
      </div>
    </div>

    <div class="eb-divider" />

    <div class="eb-section">
      <span class="eb-label">{{ $t('atlaspro.fields.outputDir') }}</span>
      <div class="eb-input-row">
        <input
          type="text" class="eb-input"
          :value="pack.outputDir"
          :placeholder="t('atlaspro.placeholders.outputDir')"
          @input="pack.outputDir = ($event.target as HTMLInputElement).value"
        />
        <button class="eb-browse" @click="browse">{{ $t('common.browse') }}</button>
      </div>
    </div>

    <div class="eb-divider" />

    <div class="eb-section eb-section-name">
      <span class="eb-label">{{ $t('atlaspro.fields.atlasName') }}</span>
      <input
        type="text" class="eb-input eb-input-sm"
        :value="pack.atlasName"
        :placeholder="t('atlaspro.placeholders.atlasName')"
        @input="pack.atlasName = ($event.target as HTMLInputElement).value"
      />
    </div>

    <div class="eb-spacer" />

    <button class="eb-publish" :disabled="!pack.canExecute" @click="emit('publish')">
      {{ pack.busy ? '...' : $t('atlaspro.execute') }}
    </button>
  </div>
</template>

<style scoped>
.export-bar {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 6px 12px;
  background: var(--bg-panel);
  border-bottom: 1px solid var(--divider);
  min-height: 44px;
  flex-shrink: 0;
}

.eb-section {
  display: flex;
  align-items: center;
  gap: 6px;
  flex-shrink: 0;
}

.eb-label {
  font-size: 12px;
  color: var(--text-muted);
  white-space: nowrap;
}

.eb-chips {
  display: flex;
  gap: 3px;
}

.eb-chip {
  padding: 4px 10px;
  font-size: 12px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-secondary);
  cursor: pointer;
  white-space: nowrap;
}
.eb-chip:hover { border-color: var(--accent); }
.eb-chip.active { border-color: var(--accent); background: var(--accent-light); color: var(--accent); font-weight: 500; }

.eb-divider {
  width: 1px; height: 24px; background: var(--divider); flex-shrink: 0;
}

.eb-input-row {
  display: flex; gap: 4px;
}

.eb-input {
  width: 180px; height: 28px; font-size: 12px;
}

.eb-input-sm { width: 100px; }

.eb-browse {
  height: 28px; padding: 0 8px; font-size: 11px;
  border: 1px solid var(--border-color); border-radius: var(--radius-sm);
  background: var(--bg-input); color: var(--text-secondary); cursor: pointer;
  white-space: nowrap;
}
.eb-browse:hover { border-color: var(--accent); }

.eb-spacer { flex: 1; }

.eb-publish {
  height: 36px; padding: 0 24px;
  font-size: 15px; font-weight: 700;
  border: none; border-radius: var(--radius-sm);
  background: var(--accent); color: var(--text-inverse);
  cursor: pointer; white-space: nowrap;
}
.eb-publish:hover:not(:disabled) { background: var(--accent-hover); }
.eb-publish:disabled { background: var(--bg-badge); color: var(--text-muted); cursor: not-allowed; }
</style>
