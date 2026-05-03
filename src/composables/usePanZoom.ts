import { ref, onMounted, onUnmounted, type Ref } from 'vue'

export function usePanZoom(canvasRef: Ref<HTMLCanvasElement | null>) {
  const zoom = ref(1.0)
  const panOffset = ref({ x: 0, y: 0 })
  const isPanning = ref(false)
  let lastX = 0
  let lastY = 0

  function onWheel(e: WheelEvent) {
    e.preventDefault()
    const delta = e.deltaY > 0 ? 0.9 : 1.1
    zoom.value = Math.max(0.1, Math.min(4.0, zoom.value * delta))
  }

  function onMouseDown(e: MouseEvent) {
    if (e.button === 1 || (e.button === 0 && e.altKey)) {
      isPanning.value = true
      lastX = e.clientX
      lastY = e.clientY
      if (canvasRef.value) {
        canvasRef.value.style.cursor = 'grabbing'
      }
    }
  }

  function onMouseMove(e: MouseEvent) {
    if (!isPanning.value) return
    panOffset.value = {
      x: panOffset.value.x + (e.clientX - lastX),
      y: panOffset.value.y + (e.clientY - lastY),
    }
    lastX = e.clientX
    lastY = e.clientY
  }

  function onMouseUp() {
    isPanning.value = false
    if (canvasRef.value) {
      canvasRef.value.style.cursor = 'default'
    }
  }

  onMounted(() => {
    const el = canvasRef.value
    if (!el) return
    el.addEventListener('wheel', onWheel, { passive: false })
    el.addEventListener('mousedown', onMouseDown)
    window.addEventListener('mousemove', onMouseMove)
    window.addEventListener('mouseup', onMouseUp)
  })

  onUnmounted(() => {
    const el = canvasRef.value
    if (el) {
      el.removeEventListener('wheel', onWheel)
      el.removeEventListener('mousedown', onMouseDown)
    }
    window.removeEventListener('mousemove', onMouseMove)
    window.removeEventListener('mouseup', onMouseUp)
  })

  return { zoom, panOffset }
}
