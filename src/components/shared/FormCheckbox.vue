<script setup lang="ts">
defineProps<{
  label: string
  modelValue: boolean
}>()

const emit = defineEmits<{
  'update:modelValue': [value: boolean]
}>()
</script>

<template>
  <label class="form-checkbox">
    <input
      type="checkbox"
      :checked="modelValue"
      @change="emit('update:modelValue', ($event.target as HTMLInputElement).checked)"
    />
    <span>{{ label }}</span>
  </label>
</template>

<style scoped>
.form-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 12px;
  color: var(--text-secondary);
  cursor: pointer;
  margin-top: 0;
}
</style>
