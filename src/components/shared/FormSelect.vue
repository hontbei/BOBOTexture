<script setup lang="ts">
defineProps<{
  label: string
  modelValue: string
  options: { value: string; label: string }[]
}>()

const emit = defineEmits<{
  'update:modelValue': [value: string]
}>()
</script>

<template>
  <label class="form-field">
    <span class="form-label">{{ label }}</span>
    <select
      class="form-select"
      :value="modelValue"
      @change="emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
    >
      <option v-for="opt in options" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
    </select>
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

.form-select {
  flex: 1;
  max-width: 180px;
  height: 28px;
  font-size: 12px;
}
</style>
