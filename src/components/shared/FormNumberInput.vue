<script setup lang="ts">
defineProps<{
  label: string
  modelValue: number
  min?: number
  max?: number
  step?: number
}>()

const emit = defineEmits<{
  'update:modelValue': [value: number]
}>()
</script>

<template>
  <label class="form-field">
    <span class="form-label">{{ label }}</span>
    <input
      type="number"
      class="form-input"
      :value="modelValue"
      :min="min"
      :max="max"
      :step="step ?? 1"
      @input="emit('update:modelValue', Number(($event.target as HTMLInputElement).value))"
    />
  </label>
</template>

<style scoped>
.form-field {
  display: flex;
  align-items: center;
  justify-content: space-between;
  gap: 8px;
  margin-top: 0;
}

.form-label {
  font-size: 12px;
  color: var(--text-secondary);
  flex-shrink: 0;
}

.form-input {
  width: 80px;
  height: 28px;
  text-align: right;
}
</style>
