<script setup lang="ts">
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormSelect from '../shared/FormSelect.vue'
import FormNumberInput from '../shared/FormNumberInput.vue'

const pack = usePackStore()

const trimOptions = [
  { value: 'none', label: 'None' },
  { value: 'alpha', label: 'Alpha' },
]
</script>

<template>
  <SettingsSection title="Trim" section-id="trim">
    <FormSelect v-model="pack.trim" label="Trim Mode" :options="trimOptions" />
    <FormNumberInput
      v-if="pack.trim === 'alpha'"
      v-model="pack.alphaThreshold"
      label="Alpha Threshold"
      :min="0"
      :max="255"
      style="margin-top: 8px"
    />
  </SettingsSection>
</template>
