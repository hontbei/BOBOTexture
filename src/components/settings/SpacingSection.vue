<script setup lang="ts">
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormNumberInput from '../shared/FormNumberInput.vue'

const pack = usePackStore()
</script>

<template>
  <SettingsSection :title="$t('panels.spacing')" section-id="spacing">
    <FormNumberInput v-model="pack.padding.shapePadding" :label="$t('atlaspro.fields.shapePadding')" :min="0" :max="128" />
    <FormNumberInput v-model="pack.padding.borderPadding" :label="$t('atlaspro.fields.borderPadding')" :min="0" :max="128" style="margin-top: 8px" />
    <FormNumberInput v-model="pack.padding.extrude" :label="$t('atlaspro.fields.extrude')" :min="0" :max="32" style="margin-top: 8px" />
  </SettingsSection>
</template>
