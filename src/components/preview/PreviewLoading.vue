<template>
  <div class="preview-loading">
    <div class="loading-spinner"></div>
    <div class="loading-text">Packing...</div>
  </div>
</template>

<style scoped>
.preview-loading {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--divider);
  border-top-color: var(--accent);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
  margin-bottom: 12px;
}

@keyframes spin {
  to { transform: rotate(360deg); }
}

.loading-text {
  font-size: 13px;
  color: var(--text-muted);
}
</style>
