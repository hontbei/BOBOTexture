<script setup lang="ts">
import { onMounted, watch } from 'vue'
import { useAppStore } from './stores/app'
import { useProjectStore } from './stores/project'
import { usePackStore } from './stores/pack'
import { useReportStore } from './stores/report'
import { useKeyboard } from './composables/useKeyboard'
import Shell from './components/layout/Shell.vue'

const app = useAppStore()
const project = useProjectStore()
const pack = usePackStore()
const report = useReportStore()

useKeyboard()

onMounted(async () => {
  await app.initLogListener()
  await app.bootstrap()
})

let autoPackTimer: ReturnType<typeof setTimeout> | null = null

watch(
  [() => project.sources, () => ({
    algorithm: pack.algorithm,
    maxWidth: pack.maxWidth,
    maxHeight: pack.maxHeight,
    autoSize: pack.autoSize,
    borderPadding: pack.padding.borderPadding,
    shapePadding: pack.padding.shapePadding,
    extrude: pack.padding.extrude,
    trim: pack.trim,
    alphaThreshold: pack.alphaThreshold,
    formats: pack.formats,
    scaleVariants: pack.scaleVariants,
    outputDir: pack.outputDir,
    atlasName: pack.atlasName,
    allowRotation: pack.allowRotation,
  })],
  () => {
    if (!project.sourceCount) {
      report.clearReport()
      return
    }
    if (autoPackTimer) clearTimeout(autoPackTimer)
    autoPackTimer = setTimeout(() => {
      pack.executeAutoPack()
    }, 800)
  },
  { deep: true }
)
</script>

<template>
  <Shell />
</template>
