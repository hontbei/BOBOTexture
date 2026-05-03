import { onMounted, onUnmounted, ref } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { getCurrentWindow } from '@tauri-apps/api/window'

export function useFileDrop(onPaths: (paths: string[]) => void) {
  const isDragging = ref(false)
  const unlisteners: Array<() => void> = []
  let lastDropSignature = ''
  let lastDropAt = 0

  function emitPaths(paths: string[], source: string) {
    if (!paths.length) return

    const now = Date.now()
    const signature = paths.join('|')
    const isDuplicate = now - lastDropAt < 250 && signature === lastDropSignature

    if (isDuplicate) return

    lastDropSignature = signature
    lastDropAt = now
    onPaths(paths)
  }

  function handleDragPayload(source: string, payload: { type: string; paths?: string[] }) {
    if (payload.type === 'enter' || payload.type === 'over') {
      isDragging.value = true
      return
    }

    if (payload.type === 'leave') {
      isDragging.value = false
      return
    }

    if (payload.type === 'drop') {
      isDragging.value = false
      emitPaths(payload.paths ?? [], source)
    }
  }

  function extractHtml5Paths(event: DragEvent) {
    const files = Array.from(event.dataTransfer?.files ?? [])
    return files
      .map((file) => (file as File & { path?: string }).path)
      .filter((path): path is string => Boolean(path))
  }

  onMounted(async () => {
    try {
      const currentWindow = getCurrentWindow()
      const stopWindow = await currentWindow.onDragDropEvent((event) => {
        handleDragPayload('window', event.payload as { type: string; paths?: string[] })
      })
      unlisteners.push(stopWindow)
    } catch (error) {
      console.warn('window drag-drop listener failed:', error)
    }

    try {
      const webview = getCurrentWebview()
      const stopWebview = await webview.onDragDropEvent((event) => {
        handleDragPayload('webview', event.payload as { type: string; paths?: string[] })
      })
      unlisteners.push(stopWebview)
    } catch (error) {
      console.warn('webview drag-drop listener failed:', error)
    }
  })

  onUnmounted(() => {
    for (const unlisten of unlisteners.splice(0)) {
      unlisten()
    }
  })

  function onDragOver(event: DragEvent) {
    event.preventDefault()
    isDragging.value = true
  }

  function onDragLeave() {
    isDragging.value = false
  }

  function onDrop(event: DragEvent) {
    event.preventDefault()
    isDragging.value = false
    emitPaths(extractHtml5Paths(event), 'html5')
  }

  return { isDragging, onDragOver, onDragLeave, onDrop }
}
