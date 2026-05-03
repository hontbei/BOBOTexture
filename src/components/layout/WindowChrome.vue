<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

const appWindow = getCurrentWindow()

function minimize() {
  appWindow.minimize()
}

function toggleMaximize() {
  appWindow.toggleMaximize()
}

function close() {
  appWindow.close()
}
</script>

<template>
  <div class="chrome-bar" data-tauri-drag-region>
    <div class="chrome-title" data-tauri-drag-region>
      BOBOTexture V2
    </div>
    <div class="chrome-controls">
      <button class="chrome-btn chrome-btn-min" @click="minimize" title="Minimize">
        <svg width="10" height="1" viewBox="0 0 10 1">
          <rect width="10" height="1" fill="currentColor" />
        </svg>
      </button>
      <button class="chrome-btn chrome-btn-max" @click="toggleMaximize" title="Maximize">
        <svg width="10" height="10" viewBox="0 0 10 10">
          <rect x="0.5" y="0.5" width="9" height="9" fill="none" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
      <button class="chrome-btn chrome-btn-close" @click="close" title="Close">
        <svg width="10" height="10" viewBox="0 0 10 10">
          <path d="M0.5,0.5 L9.5,9.5 M9.5,0.5 L0.5,9.5" stroke="currentColor" stroke-width="1" />
        </svg>
      </button>
    </div>
  </div>
</template>

<style scoped>
.chrome-bar {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 32px;
  padding: 0 4px 0 12px;
  background: var(--bg-toolbar);
}

.chrome-title {
  font-size: 12px;
  color: var(--text-secondary);
  font-weight: 500;
}

.chrome-controls {
  display: flex;
  gap: 0;
}

.chrome-btn {
  display: flex;
  align-items: center;
  justify-content: center;
  width: 46px;
  height: 32px;
  color: var(--text-secondary);
  background: transparent;
  border: none;
  border-radius: 0;
  cursor: pointer;
  transition: background 0.1s ease;
}

.chrome-btn:hover {
  background: rgba(0, 0, 0, 0.06);
}

.chrome-btn-min:hover {
  background: rgba(0, 0, 0, 0.08);
}

.chrome-btn-max:hover {
  background: rgba(0, 0, 0, 0.08);
}

.chrome-btn-close:hover {
  background: #e81123;
  color: #ffffff;
}
</style>
