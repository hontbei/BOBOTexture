import { onMounted, onUnmounted } from 'vue'
import { useProjectStore } from '@/stores/project'
import { useUiStore } from '@/stores/ui'

export interface KeyboardActions {
  onSave?: () => void
  onSaveAs?: () => void
  onNew?: () => void
  onOpen?: () => void
}

export function useKeyboard(actions: KeyboardActions) {
  const project = useProjectStore()

  function isInputFocused(): boolean {
    const tag = document.activeElement?.tagName
    return tag === 'INPUT' || tag === 'TEXTAREA' || tag === 'SELECT'
  }

  function onKeyDown(e: KeyboardEvent) {
    const ctrl = e.ctrlKey || e.metaKey

    if (e.key === 'Delete' && !isInputFocused()) {
      const ui = useUiStore()
      if (ui.selectedSpriteId) {
        e.preventDefault()
        project.removeSourceById(ui.selectedSpriteId)
        ui.setSelection(null)
      }
      return
    }

    if (ctrl && e.key === 's' && !e.shiftKey) {
      e.preventDefault()
      actions.onSave?.()
      return
    }
    if (ctrl && e.key === 's' && e.shiftKey) {
      e.preventDefault()
      actions.onSaveAs?.()
      return
    }
    if (ctrl && e.key === 'n') {
      e.preventDefault()
      actions.onNew?.()
      return
    }
    if (ctrl && e.key === 'o') {
      e.preventDefault()
      actions.onOpen?.()
      return
    }
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
