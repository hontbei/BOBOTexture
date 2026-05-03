<script setup lang="ts">
import { ref, computed } from 'vue'
import { useAppStore } from '@/stores/app'

const app = useAppStore()
const visible = ref(false)

const localLang = ref(app.settings.language)

const langOptions = [
  { value: 'zh', label: '中文' },
  { value: 'en', label: 'English' },
]

function show() { visible.value = true; localLang.value = app.settings.language }
function close() { visible.value = false }

async function applyLanguage(val: string) {
  localLang.value = val
  await app.saveSettings({ ...app.settings, language: val })
}

defineExpose({ show })
</script>

<template>
  <Teleport to="body">
    <div v-if="visible" class="modal-overlay" @click.self="close">
      <div class="modal-box">
        <div class="modal-title">{{ $t('settings.title') }}</div>

        <div class="modal-field">
          <span class="field-label">{{ $t('settings.language') }}</span>
          <select class="field-select" :value="localLang" @change="applyLanguage(($event.target as HTMLSelectElement).value)">
            <option v-for="opt in langOptions" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
          </select>
        </div>

        <div class="modal-actions">
          <button class="modal-btn modal-btn-close" @click="close">{{ $t('settingsModal.close') }}</button>
        </div>
      </div>
    </div>
  </Teleport>
</template>

<style scoped>
.modal-overlay {
  position: fixed; inset: 0; background: rgba(0,0,0,0.35);
  display: flex; align-items: center; justify-content: center;
  z-index: 9999; backdrop-filter: blur(2px);
}
.modal-box {
  background: var(--bg-panel); border-radius: 8px;
  box-shadow: 0 8px 32px rgba(0,0,0,0.18); padding: 24px;
  min-width: 300px; max-width: 400px;
}
.modal-title { font-size: 14px; font-weight: 600; color: var(--text-primary); margin-bottom: 16px; }
.modal-field { display: flex; align-items: center; justify-content: space-between; gap: 12px; margin-bottom: 20px; }
.field-label { font-size: 13px; color: var(--text-secondary); }
.field-select { width: 140px; }
.modal-actions { display: flex; justify-content: flex-end; }
.modal-btn {
  padding: 8px 16px; border-radius: 4px; font-size: 12px; font-weight: 500;
  cursor: pointer; border: 1px solid var(--border-color);
  background: var(--bg-input); color: var(--text-primary);
}
.modal-btn:hover { background: var(--bg-hover); }
</style>
