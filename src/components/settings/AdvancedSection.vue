<script setup lang="ts">
import { usePackStore } from '@/stores/pack'
import { useUiStore } from '@/stores/ui'
import SettingsSection from './SettingsSection.vue'
import FormCheckbox from '../shared/FormCheckbox.vue'

const pack = usePackStore()
const ui = useUiStore()

function addVariant() {
  pack.scaleVariants = [...pack.scaleVariants, { suffix: '', scale: 1 }]
}

function addPresets() {
  const variants = [...pack.scaleVariants]
  if (!variants.some(v => v.suffix.trim() === '@2x')) {
    variants.push({ suffix: '@2x', scale: 2 })
  }
  if (!variants.some(v => v.suffix.trim() === '@0.5x')) {
    variants.push({ suffix: '@0.5x', scale: 0.5 })
  }
  pack.scaleVariants = variants
}

function removeVariant(index: number) {
  pack.scaleVariants = pack.scaleVariants.filter((_, i) => i !== index)
}
</script>

<template>
  <SettingsSection title="Advanced" section-id="advanced">
    <FormCheckbox
      v-model="pack.allowRotation"
      label="Allow Rotation"
      style="margin-bottom: 10px"
    />
    <p v-if="pack.tmpSelected && pack.allowRotation" style="font-size: 11px; color: var(--warning); margin-top: -6px; margin-bottom: 8px">
      Rotation disabled when TMP is selected
    </p>

    <div class="variant-section">
      <div class="variant-header">
        <span class="form-label">Scale Variants</span>
        <div class="variant-actions">
          <button class="mini-btn" @click="addPresets">Presets</button>
          <button class="mini-btn" @click="addVariant">+ Add</button>
        </div>
      </div>
      <div v-for="(v, i) in pack.scaleVariants" :key="i" class="variant-row">
        <input
          type="text"
          class="variant-input suffix"
          :value="v.suffix"
          placeholder="@2x"
          @input="pack.scaleVariants[i] = { ...pack.scaleVariants[i], suffix: ($event.target as HTMLInputElement).value }"
        />
        <input
          type="number"
          class="variant-input scale"
          :value="v.scale"
          min="0.01"
          step="0.1"
          @input="pack.scaleVariants[i] = { ...pack.scaleVariants[i], scale: parseFloat(($event.target as HTMLInputElement).value) }"
        />
        <button class="remove-btn" @click="removeVariant(i)">×</button>
      </div>
    </div>
  </SettingsSection>
</template>

<style scoped>
.variant-section {
  margin-top: 8px;
}

.variant-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 6px;
}

.form-label {
  font-size: 12px;
  color: var(--text-secondary);
}

.variant-actions {
  display: flex;
  gap: 4px;
}

.mini-btn {
  padding: 2px 8px;
  font-size: 11px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  color: var(--text-secondary);
}

.mini-btn:hover {
  border-color: var(--accent);
  color: var(--accent);
}

.variant-row {
  display: flex;
  gap: 4px;
  align-items: center;
  margin-top: 4px;
}

.variant-input {
  height: 28px;
  font-size: 12px;
}

.variant-input.suffix {
  flex: 2;
}

.variant-input.scale {
  flex: 1;
  width: 60px;
}

.remove-btn {
  width: 24px;
  height: 24px;
  border: none;
  border-radius: 50%;
  background: transparent;
  color: var(--text-muted);
  font-size: 16px;
  flex-shrink: 0;
}

.remove-btn:hover {
  background: #fce8e6;
  color: var(--danger);
}
</style>
