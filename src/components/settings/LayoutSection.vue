<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormSelect from '../shared/FormSelect.vue'
import FormNumberInput from '../shared/FormNumberInput.vue'
import FormCheckbox from '../shared/FormCheckbox.vue'

const pack = usePackStore()
const { t } = useI18n()

const algorithmOptions = computed(() => [
  { value: 'max_rects', label: t('atlaspro.algorithms.maxRects') },
  { value: 'skyline', label: t('atlaspro.algorithms.skyline') },
])
</script>

<template>
  <SettingsSection :title="$t('panels.layout')" section-id="layout">
    <FormSelect v-model="pack.algorithm" :label="$t('atlaspro.fields.algorithm')" :options="algorithmOptions" />
    <FormCheckbox v-model="pack.autoSize" :label="$t('layoutSection.autoSize')" style="margin-top: 10px" />
    <div v-if="!pack.autoSize" style="margin-top: 8px">
      <FormNumberInput v-model="pack.maxWidth" :label="$t('atlaspro.fields.maxWidth')" :min="64" :max="4096" :step="64" />
      <FormNumberInput v-model="pack.maxHeight" :label="$t('atlaspro.fields.maxHeight')" :min="64" :max="4096" :step="64" style="margin-top: 8px" />
    </div>
    <p v-if="pack.autoSize" style="font-size: 11px; color: var(--text-muted); margin-top: 6px">
      {{ $t('layoutSection.autoSizeHint') }}
    </p>
  </SettingsSection>
</template>
