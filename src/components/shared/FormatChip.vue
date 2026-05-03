<script setup lang="ts">
defineProps<{
  label: string
  selected: boolean
  hint?: string
}>()

const emit = defineEmits<{
  toggle: []
}>()
</script>

<template>
  <button
    class="format-chip"
    :class="{ 'format-chip-active': selected }"
    @click="emit('toggle')"
  >
    <span class="chip-indicator">{{ selected ? '✓' : '' }}</span>
    <div class="chip-text">
      <span class="chip-label">{{ label }}</span>
      <span v-if="hint" class="chip-hint">{{ hint }}</span>
    </div>
  </button>
</template>

<style scoped>
.format-chip {
  display: flex;
  align-items: center;
  gap: 8px;
  width: 100%;
  padding: 8px 10px;
  border: 1px solid var(--border-color);
  border-radius: var(--radius-sm);
  background: var(--bg-input);
  cursor: pointer;
  text-align: left;
  transition: border-color 0.15s, background 0.15s;
}

.format-chip:hover {
  border-color: var(--accent);
}

.format-chip-active {
  border-color: var(--accent);
  background: var(--accent-light);
}

.chip-indicator {
  width: 18px;
  height: 18px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  border: 2px solid var(--border-color);
  font-size: 12px;
  color: var(--accent);
  flex-shrink: 0;
}

.format-chip-active .chip-indicator {
  border-color: var(--accent);
  background: var(--accent);
  color: white;
}

.chip-text {
  display: flex;
  flex-direction: column;
}

.chip-label {
  font-size: 12px;
  font-weight: 500;
  color: var(--text-primary);
}

.chip-hint {
  font-size: 10px;
  color: var(--text-muted);
  margin-top: 1px;
}
</style>
