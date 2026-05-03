<script setup lang="ts">
import type { SpriteOriginKind } from '@/types'

defineProps<{
  origin: SpriteOriginKind
}>()
</script>

<template>
  <span class="origin-badge" :class="origin">{{ origin === 'unity_sub_sprite' ? 'Unity' : 'File' }}</span>
</template>

<style scoped>
.origin-badge {
  display: inline-flex;
  align-items: center;
  padding: 2px 8px;
  border-radius: 999px;
  font-size: 11px;
  font-weight: 500;
  background: var(--bg-badge);
  color: var(--text-secondary);
}

.origin-badge.unity_sub_sprite {
  background: var(--bg-badge-unity);
  color: #7c4dff;
}
</style>
