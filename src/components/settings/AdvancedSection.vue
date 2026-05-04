<script setup lang="ts">
import { usePackStore } from '@/stores/pack'
import SettingsSection from './SettingsSection.vue'
import FormCheckbox from '../shared/FormCheckbox.vue'

const pack = usePackStore()
</script>

<template>
  <SettingsSection :title="$t('atlaspro.sections.advanced')" section-id="advanced">
    <FormCheckbox v-model="pack.allowRotation" :label="$t('allowRotation')" />
    <p v-if="pack.tmpSelected && pack.allowRotation" class="rotation-warn">
      {{ $t('rotationDisabled') }}
    </p>
  </SettingsSection>
</template>

<style scoped>
.rotation-warn {
  font-size: 11px; color: var(--warning); margin-top: 6px;
}
</style>
