<script setup lang="ts">
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormSelect from '../shared/FormSelect.vue'
import FormNumberInput from '../shared/FormNumberInput.vue'
import FormCheckbox from '../shared/FormCheckbox.vue'

const pack = usePackStore()

const algorithmOptions = [
  { value: 'max_rects', label: 'MaxRects' },
  { value: 'skyline', label: 'Skyline' },
]
</script>

<template>
  <SettingsSection title="Layout" section-id="layout">
    <FormSelect v-model="pack.algorithm" label="Algorithm" :options="algorithmOptions" />
    <FormCheckbox v-model="pack.autoSize" label="Auto-size" style="margin-top: 10px" />
    <div v-if="!pack.autoSize" style="margin-top: 8px">
      <FormNumberInput v-model="pack.maxWidth" label="Max Width" :min="64" :max="4096" :step="64" />
      <FormNumberInput v-model="pack.maxHeight" label="Max Height" :min="64" :max="4096" :step="64" style="margin-top: 8px" />
    </div>
    <p v-if="pack.autoSize" style="font-size: 11px; color: var(--text-muted); margin-top: 6px">
      Auto: 256 → 4096
    </p>
  </SettingsSection>
</template>
