import { ref } from 'vue'
import { useProjectStore } from '@/stores/project'
import { scanAtlasProInputs } from '@/ipc/atlaspro'

export function useFileDrop() {
  const busy = ref(false)
  const project = useProjectStore()

  async function handlePaths(paths: string[], recursive = true) {
    if (!paths.length) return
    busy.value = true
    try {
      const discovered = await scanAtlasProInputs(paths, recursive)
      project.addSources(discovered)
    } catch (err) {
      console.error('Scan failed:', err)
    } finally {
      busy.value = false
    }
  }

  return { busy, handlePaths }
}
