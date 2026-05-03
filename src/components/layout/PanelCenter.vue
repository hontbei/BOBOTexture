<script setup lang="ts">
import AtlasPreview from '../preview/AtlasPreview.vue'
</script>

<template>
  <div class="panel-center">
    <AtlasPreview />
  </div>
</template>

<style scoped>
.panel-center {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-app);
}
</style>
