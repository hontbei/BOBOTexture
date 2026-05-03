<script setup lang="ts">
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormNumberInput from '../shared/FormNumberInput.vue'

const pack = usePackStore()
</script>

<template>
  <SettingsSection title="Spacing" section-id="spacing">
    <FormNumberInput v-model="pack.padding.shapePadding" label="Shape Padding" :min="0" :max="128" />
    <FormNumberInput v-model="pack.padding.borderPadding" label="Border Padding" :min="0" :max="128" style="margin-top: 8px" />
    <FormNumberInput v-model="pack.padding.extrude" label="Extrude" :min="0" :max="32" style="margin-top: 8px" />
  </SettingsSection>
</template>
