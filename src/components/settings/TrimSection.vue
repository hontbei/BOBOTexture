<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormSelect from '../shared/FormSelect.vue'
import FormNumberInput from '../shared/FormNumberInput.vue'

const pack = usePackStore()
const { t } = useI18n()

const trimOptions = computed(() => [
  { value: 'none', label: t('atlaspro.trimModes.none') },
  { value: 'alpha', label: t('atlaspro.trimModes.alpha') },
])
</script>

<template>
  <SettingsSection :title="$t('atlaspro.fields.trim')" section-id="trim">
    <FormSelect v-model="pack.trim" :label="$t('atlaspro.fields.trim')" :options="trimOptions" />
    <FormNumberInput
      v-if="pack.trim === 'alpha'"
      v-model="pack.alphaThreshold"
      :label="$t('atlaspro.fields.alphaThreshold')"
      :min="0"
      :max="255"
      style="margin-top: 8px"
    />
  </SettingsSection>
</template>
