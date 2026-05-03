import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { SpriteSource } from '@/types'

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

  function markClean() {
    dirty.value = false
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
    undo,
    redo,
    setProjectFile,
    markClean,
  }
})
