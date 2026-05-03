<script setup lang="ts">
import { open } from '@tauri-apps/plugin-dialog'
import { useProjectStore } from '@/stores/project'
import { usePackStore } from '@/stores/pack'
import { useReportStore } from '@/stores/report'
import { scanAtlasProInputs } from '@/ipc/atlaspro'
import { openInExplorer } from '@/ipc/system'

const project = useProjectStore()
const pack = usePackStore()
const report = useReportStore()

function handleNew() {
  project.clearSources()
  report.clearReport()
}

async function handleOpen() {
  const result = await open({
    multiple: true,
    filters: [
      { name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] },
      { name: 'All Files', extensions: ['*'] },
    ],
  })
  if (!result) return
  const paths = Array.isArray(result) ? result : [result]
  if (paths.length) {
    try {
      const discovered = await scanAtlasProInputs(paths, true)
      project.addSources(discovered)
    } catch (err) {
      console.error('Scan failed:', err)
    }
  }
}

async function handleSaveAs() {
  const result = await open({ directory: true, multiple: false })
  if (typeof result === 'string') {
    pack.outputDir = result
  }
}

async function handlePublish() {
  if (!pack.canExecute) return
  try {
    await pack.executePack()
    try { await openInExplorer(pack.outputDir) } catch {}
  } catch (err) {
    console.error('Pack failed:', err)
  }
}
</script>

<template>
  <div class="toolbar">
    <div class="toolbar-group">
      <button class="toolbar-btn" title="New (Ctrl+N)" @click="handleNew">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M8 2v12M2 8h12" stroke="currentColor" stroke-width="1.5" fill="none"/></svg>
      </button>
      <button class="toolbar-btn" title="Open (Ctrl+O)" @click="handleOpen">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M2 4l4-2h8v10H2z" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
      <button class="toolbar-btn" title="Save" @click="handleSaveAs">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M3 13h10V5l-2-2H3zM3 13V3" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M5 13V9h6v4" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
      <button class="toolbar-btn" title="Save As (Ctrl+Shift+S)" @click="handleSaveAs">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M3 13h10V5l-2-2H3z" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M7 7v5M5 10h4" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
    </div>
    <div class="toolbar-separator" />
    <div class="toolbar-group">
      <button class="toolbar-btn" :disabled="!project.undoStack.length" title="Undo (Ctrl+Z)" @click="project.undo()">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M3 7h7a3 3 0 010 6H6" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M6 4l-3 3 3 3" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
      <button class="toolbar-btn" :disabled="!project.redoStack.length" title="Redo (Ctrl+Shift+Z)" @click="project.redo()">
        <svg width="16" height="16" viewBox="0 0 16 16"><path d="M13 7H6a3 3 0 000 6h4" stroke="currentColor" stroke-width="1.2" fill="none"/><path d="M10 4l3 3-3 3" stroke="currentColor" stroke-width="1.2" fill="none"/></svg>
      </button>
    </div>
    <div class="toolbar-spacer" />
    <div class="toolbar-group">
      <button class="toolbar-btn toolbar-btn-publish" :disabled="!pack.canExecute" title="Publish" @click="handlePublish">
        Publish
      </button>
    </div>
  </div>
</template>

<style scoped>
.toolbar {
  display: flex;
  align-items: center;
  height: 40px;
  padding: 0 8px;
  background: var(--bg-toolbar);
  gap: 4px;
}

.toolbar-group {
  display: flex;
  align-items: center;
  gap: 0;
}

.toolbar-separator {
  width: 1px;
  height: 24px;
  background: var(--border-color);
  margin: 0 6px;
}

.toolbar-spacer {
  flex: 1;
}

.toolbar-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-radius: var(--radius-sm);
  cursor: pointer;
  transition: background 0.1s ease;
}

.toolbar-btn:hover {
  background: rgba(0, 0, 0, 0.06);
  color: var(--text-primary);
}

.toolbar-btn:active {
  background: rgba(0, 0, 0, 0.1);
}

.toolbar-btn-publish {
  width: auto;
  padding: 0 12px;
  font-size: 12px;
  font-weight: 600;
  color: var(--accent);
}

.toolbar-btn-publish:hover {
  background: var(--accent-light);
}
</style>
