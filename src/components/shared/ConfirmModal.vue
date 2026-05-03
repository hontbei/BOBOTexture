<script setup lang="ts">
import { ref } from 'vue'

const visible = ref(false)
const resolvePromise = ref<((value: string) => void) | null>(null)

function show(): Promise<string> {
  visible.value = true
  return new Promise((resolve) => {
    resolvePromise.value = resolve
  })
}

function choose(action: string) {
  visible.value = false
  resolvePromise.value?.(action)
  resolvePromise.value = null
}

defineExpose({ show })
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-overlay" @click.self="choose('cancel')">
      <div class="modal-box">
        <div class="modal-title">{{ $t('confirm.title') }}</div>
        <div class="modal-text">{{ $t('confirm.message') }}</div>
        <div class="modal-actions">
          <button class="modal-btn modal-btn-cancel" @click="choose('cancel')">{{ $t('confirm.cancel') }}</button>
          <button class="modal-btn modal-btn-discard" @click="choose('discard')">{{ $t('confirm.discard') }}</button>
          <button class="modal-btn modal-btn-save" @click="choose('save')">{{ $t('confirm.save') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed;
  inset: 0;
  background: rgba(0, 0, 0, 0.35);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 9999;
  backdrop-filter: blur(2px);
}

.modal-box {
  background: var(--bg-panel);
  border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.18);
  padding: 24px;
  min-width: 360px;
  max-width: 440px;
}

.modal-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.modal-text {
  font-size: 13px;
  color: var(--text-secondary);
  margin-bottom: 20px;
  line-height: 1.4;
}

.modal-actions {
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.modal-btn {
  padding: 8px 16px;
  border-radius: 4px;
  font-size: 12px;
  font-weight: 500;
  cursor: pointer;
  border: 1px solid var(--border-color);
  background: var(--bg-input);
  color: var(--text-primary);
  transition: background 0.1s;
}

.modal-btn:hover {
  background: var(--bg-hover);
}

.modal-btn-save {
  background: var(--accent);
  color: var(--text-inverse);
  border-color: var(--accent);
}

.modal-btn-save:hover {
  background: var(--accent-hover);
}

.modal-btn-discard {
  border-color: var(--danger);
  color: var(--danger);
}

.modal-btn-discard:hover {
  background: #fce8e6;
}

.modal-btn-cancel {
  color: var(--text-secondary);
}
</style>
