import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import { convertFileSrc } from '@tauri-apps/api/core'
import type { AtlasProFormat, AtlasProReport } from '@/types'

export const useReportStore = defineStore('report', () => {
  const report = ref<AtlasProReport | null>(null)
  const lastPackDuration = ref(0)

  const formatLabels: Record<string, string> = {
    png_only: 'PNG Only',
    json_array: 'JSON Array',
    tmp_sprite_asset: 'TMP Sprite Asset',
  }

  const groupedOutputs = computed(() => {
    const groups = new Map<AtlasProFormat, string[]>()
    for (const output of report.value?.outputs ?? []) {
      const current = groups.get(output.format)
      if (current) {
        current.push(output.path)
      } else {
        groups.set(output.format, [output.path])
      }
    }
    return Array.from(groups.entries()).map(([format, paths]) => ({
      format,
      label: formatLabels[format] ?? format,
      paths,
    }))
  })

  const atlasPngUrl = computed(() => {
    if (!report.value) return null
    const pngOutput = report.value.outputs.find(
      o => o.format === 'png_only' || o.format === 'json_array' || o.format === 'tmp_sprite_asset'
    )
    if (!pngOutput) return null
    if (pngOutput.path.endsWith('.png')) {
      try { return convertFileSrc(pngOutput.path) } catch { return null }
    }
    const pngFile = report.value.outputs.find(o => o.path.endsWith('.png'))
    if (pngFile) {
      try { return convertFileSrc(pngFile.path) } catch { return null }
    }
    return null
  })

  const tmpExamples = computed<string[]>(() => {
    const r = report.value
    if (!r) return []
    const hasTmpOutput = r.outputs.some(o => o.format === 'tmp_sprite_asset')
    if (!hasTmpOutput || !r.placed.length) return []

    return r.placed.map(s =>
      `front<sprite name="${s.name}">back`
    )
  })

  function setReport(r: AtlasProReport, durationMs = 0) {
    report.value = r
    lastPackDuration.value = durationMs
  }

  function clearReport() {
    report.value = null
    lastPackDuration.value = 0
  }

  return {
    report,
    lastPackDuration,
    groupedOutputs,
    atlasPngUrl,
    tmpExamples,
    setReport,
    clearReport,
  }
})
