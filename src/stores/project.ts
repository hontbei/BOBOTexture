import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SpriteSource, BoboProjectFile } from '@/types'
import { writeTextFile, readTextFile } from '@/ipc/system'
import { scanAtlasProInputs } from '@/ipc/atlaspro'

function sourceFingerprint(source: SpriteSource): string {
  return `${source.sourcePath}::${JSON.stringify(source.subRect)}`
}

export const useProjectStore = defineStore('project', () => {
  const sources = ref<SpriteSource[]>([])
  const projectFilePath = ref<string | null>(null)
  const projectName = ref('Untitled')
  const dirty = ref(false)
  const undoStack = ref<SpriteSource[][]>([])
  const redoStack = ref<SpriteSource[][]>([])

  const sourceCount = computed(() => sources.value.length)

  const fingerprints = computed(() => new Set(sources.value.map(sourceFingerprint)))

  function snapshotForUndo() {
    undoStack.value.push([...sources.value])
    if (undoStack.value.length > 50) {
      undoStack.value.shift()
    }
    redoStack.value = []
  }

  function addSources(newSources: SpriteSource[]) {
    snapshotForUndo()
    const existing = fingerprints.value
    const toAdd = newSources.filter(s => !existing.has(sourceFingerprint(s)))
    if (toAdd.length) {
      sources.value = [...sources.value, ...toAdd]
      dirty.value = true
    }
  }

  function removeSource(target: SpriteSource) {
    snapshotForUndo()
    const fp = sourceFingerprint(target)
    sources.value = sources.value.filter(s => sourceFingerprint(s) !== fp)
    dirty.value = true
  }

  function clearSources() {
    snapshotForUndo()
    sources.value = []
    dirty.value = true
  }

  function undo() {
    if (undoStack.value.length === 0) return
    redoStack.value.push([...sources.value])
    sources.value = undoStack.value.pop()!
  }

  function redo() {
    if (redoStack.value.length === 0) return
    undoStack.value.push([...sources.value])
    sources.value = redoStack.value.pop()!
  }

  function setProjectFile(path: string | null) {
    projectFilePath.value = path
    if (path) {
      const segments = path.replace(/\\/g, '/').split('/')
      projectName.value = segments[segments.length - 1].replace(/\.[^.]+$/, '')
    }
  }

  function replaceSources(newSources: SpriteSource[]) {
    undoStack.value = []
    redoStack.value = []
    sources.value = newSources
  }

  function resetProject(name: string) {
    sources.value = []
    projectFilePath.value = null
    projectName.value = name
    dirty.value = false
    undoStack.value = []
    redoStack.value = []
  }

  function markClean() {
    dirty.value = false
  }

  function markDirty() {
    dirty.value = true
  }

  async function saveProject(path: string) {
    const { usePackStore } = await import('./pack')
    const pack = usePackStore()
    const file: BoboProjectFile = {
      version: 1,
      atlasName: pack.atlasName,
      outputDir: pack.outputDir,
      settings: {
        algorithm: pack.algorithm,
        maxWidth: pack.maxWidth,
        maxHeight: pack.maxHeight,
        autoSize: pack.autoSize,
        padding: { ...pack.padding },
        trim: pack.trim,
        alphaThreshold: pack.alphaThreshold,
        formats: [...pack.formats],
        allowRotation: pack.allowRotation,
      },
      scaleVariants: pack.scaleVariants.map(v => ({ suffix: v.suffix, scale: v.scale })),
      sources: [...new Set(sources.value.map(s => s.sourcePath))],
    }
    await writeTextFile(path, JSON.stringify(file, null, 2))
    setProjectFile(path)
    markClean()
  }

  async function loadProject(path: string) {
    const content = await readTextFile(path)
    const file: BoboProjectFile = JSON.parse(content)
    const { usePackStore } = await import('./pack')
    const pack = usePackStore()

    pack.atlasName = file.atlasName
    pack.outputDir = file.outputDir
    pack.algorithm = file.settings.algorithm
    pack.maxWidth = file.settings.maxWidth
    pack.maxHeight = file.settings.maxHeight
    pack.autoSize = file.settings.autoSize
    pack.padding = { ...file.settings.padding }
    pack.trim = file.settings.trim
    pack.alphaThreshold = file.settings.alphaThreshold
    pack.formats = [...file.settings.formats]
    pack.allowRotation = file.settings.allowRotation
    pack.scaleVariants = file.scaleVariants.map(v => ({ ...v }))

    setProjectFile(path)
    markClean()

    if (file.sources.length > 0) {
      try {
        const discovered = await scanAtlasProInputs(file.sources, true)
        replaceSources(discovered)
      } catch (e) {
        console.error('Failed to scan project sources:', e)
      }
    } else {
      replaceSources([])
    }
  }

  return {
    sources,
    projectFilePath,
    projectName,
    dirty,
    undoStack,
    redoStack,
    sourceCount,
    fingerprints,
    addSources,
    removeSource,
    clearSources,
    replaceSources,
    resetProject,
    undo,
    redo,
    setProjectFile,
    markClean,
    markDirty,
    saveProject,
    loadProject,
  }
})
