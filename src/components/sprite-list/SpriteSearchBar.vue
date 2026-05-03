<script setup lang="ts">
import { ref, watch } from 'vue'

const model = defineModel<string>({ default: '' })

const emit = defineEmits<{
  update: [value: string]
}>()

const query = ref(model.value)

let debounce: ReturnType<typeof setTimeout> | null = null

watch(model, (v) => { query.value = v })

function onInput(e: Event) {
  const value = (e.target as HTMLInputElement).value
  query.value = value
  if (debounce) clearTimeout(debounce)
  debounce = setTimeout(() => {
    model.value = value
    emit('update', value)
  }, 200)
}
</script>

<template>
  <div class="search-bar">
    <svg class="search-icon" width="14" height="14" viewBox="0 0 14 14"><circle cx="6" cy="6" r="4.5" fill="none" stroke="#80868b" stroke-width="1.2"/><path d="M9.5 9.5L13 13" stroke="#80868b" stroke-width="1.2"/></svg>
    <input
      type="text"
      :value="query"
      :placeholder="$t('atlaspro.table.empty')"
      class="search-input"
      @input="onInput"
    />
  </div>
</template>

<style scoped>
.search-bar {
  position: relative;
  margin: 0 12px 8px;
}

.search-icon {
  position: absolute;
  left: 10px;
  top: 50%;
  transform: translateY(-50%);
  pointer-events: none;
}

.search-input {
  width: 100%;
  height: 32px;
  padding: 0 10px 0 32px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  font-size: 12px;
  outline: none;
}

.search-input:focus {
  border-color: var(--accent);
  box-shadow: 0 0 0 2px rgba(26, 115, 232, 0.15);
}
</style>
