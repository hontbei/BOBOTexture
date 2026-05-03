<script setup lang="ts">
import { onMounted, onUnmounted, ref, watch } from 'vue'
import { useReportStore } from '@/stores/report'
import { useUiStore } from '@/stores/ui'
import { drawAtlas, hitTest } from '@/composables/useCanvas'
import PreviewControls from './PreviewControls.vue'
import PreviewEmpty from './PreviewEmpty.vue'
import PreviewLoading from './PreviewLoading.vue'

const reportStore = useReportStore()
const ui = useUiStore()

const canvasRef = ref<HTMLCanvasElement | null>(null)
const containerRef = ref<HTMLElement | null>(null)
const atlasImage = ref<HTMLImageElement | null>(null)
const imageLoaded = ref(false)
const showGrid = ref(false)
const cw = ref(800)
const ch = ref(600)

let resizeObserver: ResizeObserver | null = null
let loadId = 0

function loadImage() {
  const url = reportStore.atlasPngUrl
  if (!url) {
    imageLoaded.value = false
    atlasImage.value = null
    redraw()
    return
  }
  const currentLoad = ++loadId
  const img = new Image()
  img.onload = () => {
    if (currentLoad !== loadId) return
    atlasImage.value = img
    imageLoaded.value = true
    redraw()
  }
  img.onerror = () => {
    if (currentLoad !== loadId) return
  }
  img.src = `${url}?t=${Date.now()}`
}

function redraw() {
  const canvas = canvasRef.value
  const report = reportStore.report
  if (!canvas || !report) return

  drawAtlas({
    ctx: canvas.getContext('2d')!,
    image: imageLoaded.value ? atlasImage.value : null,
    atlasWidth: report.atlasSize.width,
    atlasHeight: report.atlasSize.height,
    placed: report.placed,
    selectedId: ui.selectedSpriteId,
    hoveredId: ui.hoveredSpriteId,
    zoom: ui.zoom,
    panX: ui.panOffset.x,
    panY: ui.panOffset.y,
    showGrid: showGrid.value,
  })
}

function onCanvasClick(e: MouseEvent) {
  const report = reportStore.report
  const canvas = canvasRef.value
  if (!report || !canvas) return

  const rect = canvas.getBoundingClientRect()
  const cx = (e.clientX - rect.left) * (canvas.width / rect.width)
  const cy = (e.clientY - rect.top) * (canvas.height / rect.height)

  const id = hitTest(cx, cy, report.atlasSize.width, report.atlasSize.height, report.placed, ui.zoom, ui.panOffset.x, ui.panOffset.y, canvas.width, canvas.height)
  ui.setSelection(id)
}

function onCanvasMove(e: MouseEvent) {
  const report = reportStore.report
  const canvas = canvasRef.value
  if (!report || !canvas) return

  const rect = canvas.getBoundingClientRect()
  const cx = (e.clientX - rect.left) * (canvas.width / rect.width)
  const cy = (e.clientY - rect.top) * (canvas.height / rect.height)

  const id = hitTest(cx, cy, report.atlasSize.width, report.atlasSize.height, report.placed, ui.zoom, ui.panOffset.x, ui.panOffset.y, canvas.width, canvas.height)
  ui.setHover(id)
}

function onCanvasLeave() {
  ui.setHover(null)
}

function onWheel(e: WheelEvent) {
  e.preventDefault()
  const delta = e.deltaY > 0 ? 0.92 : 1.08
  ui.setZoom(ui.zoom * delta)
}

watch(() => reportStore.report, () => {
  loadImage()
}, { immediate: true })

watch([() => ui.zoom, () => ui.panOffset, () => ui.selectedSpriteId, () => ui.hoveredSpriteId, showGrid], () => {
  redraw()
})

onMounted(() => {
  const container = containerRef.value
  const canvas = canvasRef.value
  if (!container || !canvas) return

  resizeObserver = new ResizeObserver(() => {
    const rect = container.getBoundingClientRect()
    cw.value = Math.floor(rect.width)
    ch.value = Math.floor(rect.height)
    canvas.width = cw.value * window.devicePixelRatio
    canvas.height = ch.value * window.devicePixelRatio
    canvas.style.width = cw.value + 'px'
    canvas.style.height = ch.value + 'px'
    redraw()
  })
  resizeObserver.observe(container)

  loadImage()
})

onUnmounted(() => {
  resizeObserver?.disconnect()
})
</script>

<template>
  <div class="atlas-preview">
    <PreviewControls v-if="reportStore.report" v-model:zoom="ui.zoom" v-model:show-grid="showGrid" />
    <div ref="containerRef" class="canvas-container">
      <canvas
        ref="canvasRef"
        class="atlas-canvas"
        @click="onCanvasClick"
        @mousemove="onCanvasMove"
        @mouseleave="onCanvasLeave"
        @wheel="onWheel"
      />
      <PreviewEmpty v-if="!reportStore.report" />
      <PreviewLoading v-if="reportStore.report && !imageLoaded" />
    </div>
  </div>
</template>

<style scoped>
.atlas-preview {
  display: flex;
  flex-direction: column;
  flex: 1;
  overflow: hidden;
}

.canvas-container {
  flex: 1;
  position: relative;
  overflow: hidden;
}

.atlas-canvas {
  width: 100%;
  height: 100%;
  display: block;
}
</style>
