<template>
  <div class="preview-empty">
    <div class="empty-icon">⌖</div>
    <div class="empty-title">No atlas yet</div>
    <div class="empty-text">Drop sprites or click Open to add images</div>
  </div>
</template>

<style scoped>
.preview-empty {
  position: absolute;
  inset: 0;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  color: var(--text-muted);
}

.empty-icon {
  font-size: 48px;
  margin-bottom: 12px;
  opacity: 0.25;
}

.empty-title {
  font-size: 14px;
  font-weight: 600;
  color: var(--text-secondary);
  margin-bottom: 4px;
}

.empty-text {
  font-size: 12px;
}
</style>
