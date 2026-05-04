<script setup lang="ts">
import LayoutSection from './LayoutSection.vue'
import SpacingSection from './SpacingSection.vue'
import TrimSection from './TrimSection.vue'
import AdvancedSection from './AdvancedSection.vue'
import ResultSection from './ResultSection.vue'
</script>

<template>
  <div class="settings-panel">
    <div class="settings-scroll">
      <LayoutSection />
      <SpacingSection />
      <TrimSection />
      <AdvancedSection />
      <ResultSection />
    </div>
  </div>
</template>

<style scoped>
.settings-panel { display: flex; flex-direction: column; height: 100%; min-height: 0; }
.settings-scroll { flex: 1; overflow-y: auto; padding: 8px; min-height: 0; }
</style>
