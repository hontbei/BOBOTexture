import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useUiStore = defineStore('ui', () => {
  const selectedSpriteId = ref<string | null>(null)
  const hoveredSpriteId = ref<string | null>(null)
  const zoom = ref(1.0)
  const panOffset = ref({ x: 0, y: 0 })
  const openSections = ref<string[]>(['layout', 'spacing', 'trim', 'export'])
  const showAdvanced = ref(false)

  function setSelection(id: string | null) {
    selectedSpriteId.value = id
  }

  function setHover(id: string | null) {
    hoveredSpriteId.value = id
  }

  function setZoom(z: number) {
    zoom.value = Math.max(0.1, Math.min(4.0, z))
  }

  function setPan(x: number, y: number) {
    panOffset.value = { x, y }
  }

  function fitZoom(canvasWidth: number, canvasHeight: number, atlasWidth: number, atlasHeight: number) {
    const scaleX = (canvasWidth - 32) / atlasWidth
    const scaleY = (canvasHeight - 32) / atlasHeight
    zoom.value = Math.min(scaleX, scaleY, 1.0)
    panOffset.value = { x: 0, y: 0 }
  }

  function toggleSection(id: string) {
    if (openSections.value.includes(id)) {
      openSections.value = openSections.value.filter(s => s !== id)
    } else {
      openSections.value = [...openSections.value, id]
    }
  }

  function isSectionOpen(id: string) {
    return openSections.value.includes(id)
  }

  return {
    selectedSpriteId,
    hoveredSpriteId,
    zoom,
    panOffset,
    openSections,
    showAdvanced,
    setSelection,
    setHover,
    setZoom,
    setPan,
    fitZoom,
    toggleSection,
    isSectionOpen,
  }
})
