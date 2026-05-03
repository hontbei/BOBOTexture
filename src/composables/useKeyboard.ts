import { onMounted, onUnmounted } from 'vue'
import { useProjectStore } from '@/stores/project'

export function useKeyboard() {
  const project = useProjectStore()

  function onKeyDown(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey

    if (ctrl && e.key === 'z' && !e.shiftKey) {
      e.preventDefault()
      project.undo()
      return
    }
    if (ctrl && e.key === 'z' && e.shiftKey) {
      e.preventDefault()
      project.redo()
    }
  }

  onMounted(() => {
    window.addEventListener('keydown', onKeyDown)
  })

  onUnmounted(() => {
    window.removeEventListener('keydown', onKeyDown)
  })
}
