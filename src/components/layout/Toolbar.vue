<script setup lang="ts">
import { useProjectStore } from '@/stores/project'
import { usePackStore } from '@/stores/pack'

const project = useProjectStore()
const pack = usePackStore()

const emit = defineEmits<{
  save: []
  'save-as': []
  new: []
  open: []
  'settings-open': []
}>()
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-group">
      <button class="toolbar-btn" :title="$t('toolbar.new') + ' (Ctrl+N)'" @click="emit('new')">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" fill="none"/></svg>
      </button>
      <button class="toolbar-btn" :title="$t('toolbar.open') + ' (Ctrl+O)'" @click="emit('open')">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M2 4l4-2h8v10H2z" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
      <button class="toolbar-btn" :title="$t('toolbar.save') + ' (Ctrl+S)'" @click="emit('save')">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M3 13h10V5l-2-2H3zM3 13V3" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M5 13V9h6v4" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
      <button class="toolbar-btn" :title="$t('toolbar.saveAs')" @click="emit('save-as')">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M3 13h10V5l-2-2H3z" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M7 7v5M5 10h4" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
    </div>
    <div class="toolbar-separator" />
    <div class="toolbar-group">
      <button class="toolbar-btn" :disabled="!project.undoStack.length" :title="$t('toolbar.undo') + ' (Ctrl+Z)'" @click="project.undo()">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M3 7h7a3 3 0 010 6H6" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M6 4l-3 3 3 3" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
    </div>
    <div class="toolbar-spacer" />
    <div class="toolbar-group">
      <button class="toolbar-btn" :title="$t('toolbar.settings')" @click="emit('settings-open')">
        <svg width="16" height="16" viewBox="0 0 16 16"><circle cx="8" cy="8" r="3" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M8 1v2m0 10v2M1 8h2m10 0h2M3.05 3.05l1.41 1.41m7.08 7.08l1.41 1.41m0-12.9l-1.41 1.41m-7.08 7.08l-1.41 1.41" stroke="currentColor" stroke-width="1.2"/></svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.toolbar { display: flex; align-items: center; height: 40px; padding: 0 8px; background: var(--bg-toolbar); gap: 4px; }
.toolbar-group { display: flex; align-items: center; gap: 0; }
.toolbar-separator { width: 1px; height: 24px; background: var(--border-color); margin: 0 6px; }
.toolbar-spacer { flex: 1; }
.toolbar-btn { display: flex; align-items: center; justify-content: center; width: 32px; height: 32px; color: var(--text-secondary); background: transparent; border: none; border-radius: var(--radius-sm); cursor: pointer; transition: background 0.1s ease; }
.toolbar-btn:hover:not(:disabled) { background: rgba(0,0,0,0.06); color: var(--text-primary); }
.toolbar-btn:active:not(:disabled) { background: rgba(0,0,0,0.1); }
.toolbar-btn:disabled { opacity: 0.35; }
.toolbar-btn-publish { width: auto; padding: 0 12px; font-size: 12px; font-weight: 600; color: var(--accent); }
.toolbar-btn-publish:hover:not(:disabled) { background: var(--accent-light); }
</style>
