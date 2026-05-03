import { defineStore } from 'pinia'
import { ref, computed, watch } from 'vue'
import type { AtlasProAlgorithm, AtlasProFormat, AtlasProTrim, PaddingOptions, ScaleVariant } from '@/types'
import { executeAtlasPro } from '@/ipc/atlaspro'
import { useProjectStore } from './project'
import { useReportStore } from './report'

export const usePackStore = defineStore('pack', () => {
  const algorithm = ref<AtlasProAlgorithm>('max_rects')
  const maxWidth = ref(4096)
  const maxHeight = ref(4096)
  const autoSize = ref(true)
  const padding = ref<PaddingOptions>({ borderPadding: 2, shapePadding: 2, extrude: 0 })
  const trim = ref<AtlasProTrim>('none')
  const alphaThreshold = ref(0)
  const formats = ref<AtlasProFormat[]>(['png_only', 'json_array', 'tmp_sprite_asset'])
  const scaleVariants = ref<ScaleVariant[]>([])
  const allowRotation = ref(false)
  const outputDir = ref('')
  const atlasName = ref('atlas')
  const busy = ref(false)

  const canExecute = computed(() => {
    return !busy.value
      && outputDir.value.trim().length > 0
      && atlasName.value.trim().length > 0
      && formats.value.length > 0
      && useProjectStore().sourceCount > 0
  })

  const tmpSelected = computed(() => formats.value.includes('tmp_sprite_asset'))

  watch(tmpSelected, (selected) => {
    if (selected && allowRotation.value) {
      allowRotation.value = false
    }
  })

  type PackSettings = {
    algorithm: AtlasProAlgorithm
    maxWidth: number
    maxHeight: number
    autoSize: boolean
    padding: PaddingOptions
    trim: AtlasProTrim
    alphaThreshold: number
    formats: AtlasProFormat[]
    scaleVariants: ScaleVariant[]
    allowRotation: boolean
    outputDir: string
    atlasName: string
  }

  async function executePack() {
    const project = useProjectStore()
    const reportStore = useReportStore()

    if (!canExecute.value) return

    busy.value = true
    try {
      const report = await executeAtlasPro({
        sources: project.sources,
        outputDir: outputDir.value.trim(),
        atlasName: atlasName.value.trim(),
        maxWidth: maxWidth.value,
        maxHeight: maxHeight.value,
        algorithm: algorithm.value,
        padding: { ...padding.value },
        trim: trim.value,
        alphaThreshold: alphaThreshold.value,
        formats: [...formats.value],
        scaleVariants: scaleVariants.value.map(v => ({ suffix: v.suffix.trim(), scale: v.scale })),
        autoSize: autoSize.value,
      })
      reportStore.setReport(report)
    } catch (err) {
      throw err
    } finally {
      busy.value = false
    }
  }

  let autoPackTimer: ReturnType<typeof setTimeout> | null = null

  function scheduleAutoPack() {
    if (autoPackTimer) clearTimeout(autoPackTimer)
    autoPackTimer = setTimeout(() => executeAutoPack(), 800)
  }

  async function executeAutoPack() {
    const project = useProjectStore()
    if (!project.sourceCount) return

    const autoDir = outputDir.value.trim() || '/tmp/bobotexture-autopack'
    busy.value = true
    try {
      const report = await executeAtlasPro({
        sources: project.sources,
        outputDir: autoDir,
        atlasName: atlasName.value.trim() || 'atlas',
        maxWidth: maxWidth.value,
        maxHeight: maxHeight.value,
        algorithm: algorithm.value,
        padding: { ...padding.value },
        trim: trim.value,
        alphaThreshold: alphaThreshold.value,
        formats: [...formats.value],
        scaleVariants: scaleVariants.value.map(v => ({ suffix: v.suffix.trim(), scale: v.scale })),
        autoSize: autoSize.value,
      })
      useReportStore().setReport(report)
    } catch {
      // Auto-pack errors are non-fatal
    } finally {
      busy.value = false
    }
  }

  return {
    algorithm,
    maxWidth,
    maxHeight,
    autoSize,
    padding,
    trim,
    alphaThreshold,
    formats,
    scaleVariants,
    allowRotation,
    outputDir,
    atlasName,
    busy,
    canExecute,
    tmpSelected,
    executePack,
    scheduleAutoPack,
    executeAutoPack,
  }
})
