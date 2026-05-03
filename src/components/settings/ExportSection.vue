<script setup lang="ts">
import { ref } from 'vue'
import { open } from '@tauri-apps/plugin-dialog'
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormatChip from '../shared/FormatChip.vue'

const pack = usePackStore()
const submitAttempted = ref(false)

const formatOptions = [
  { value: 'png_only' as const, label: 'PNG Only', hint: null },
  { value: 'json_array' as const, label: 'JSON Array', hint: 'TexturePacker format' },
  { value: 'tmp_sprite_asset' as const, label: 'TMP Sprite Asset', hint: 'Unity TextMeshPro' },
]

function toggleFormat(format: string) {
  if (pack.formats.includes(format as any)) {
    pack.formats = pack.formats.filter(f => f !== format)
  } else {
    pack.formats = [...pack.formats, format as any]
  }
}

function isSelected(format: string) {
  return pack.formats.includes(format as any)
}

async function browseOutputDir() {
  const result = await open({ directory: true, multiple: false })
  if (typeof result === 'string') {
    pack.outputDir = result
  }
}
</script>

<template>
  <SettingsSection title="Export" section-id="export">
    <div class="format-chips">
      <FormatChip
        v-for="opt in formatOptions"
        :key="opt.value"
        :label="opt.label"
        :hint="opt.hint ?? undefined"
        :selected="isSelected(opt.value)"
        @toggle="toggleFormat(opt.value)"
      />
    </div>

    <label class="form-field" style="margin-top: 12px">
      <span class="form-label">Output Dir</span>
      <div class="input-row">
        <input
          type="text"
          class="form-input flex-input"
          :value="pack.outputDir"
          placeholder="Select output folder..."
          @input="pack.outputDir = ($event.target as HTMLInputElement).value"
        />
        <button class="browse-btn" @click="browseOutputDir">…</button>
      </div>
    </label>

    <label class="form-field" style="margin-top: 8px">
      <span class="form-label">Atlas Name</span>
      <input
        type="text"
        class="form-input flex-input"
        :value="pack.atlasName"
        placeholder="atlas"
        @input="pack.atlasName = ($event.target as HTMLInputElement).value"
      />
    </label>
  </SettingsSection>
</template>

<style scoped>
.format-chips {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.form-field {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.form-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.form-input {
  width: 100%;
  height: 28px;
  font-size: 12px;
}

.flex-input {
  flex: 1;
}

.input-row {
  display: flex;
  gap: 4px;
}

.browse-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 28px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-secondary);
  font-size: 16px;
  flex-shrink: 0;
}

.browse-btn:hover {
  background: var(--bg-hover);
}
</style>
