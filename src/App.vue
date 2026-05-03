<script setup lang="ts">
import { onMounted, onUnmounted, watch, ref } from 'vue'
import { getCurrentWindow } from '@tauri-apps/api/window'
import { useAppStore } from './stores/app'
import { useProjectStore } from './stores/project'
import { usePackStore } from './stores/pack'
import { useReportStore } from './stores/report'
import { useProjectFileActions } from './composables/useProjectFileActions'
import { useKeyboard } from './composables/useKeyboard'
import Shell from './components/layout/Shell.vue'
import ConfirmModal from './components/shared/ConfirmModal.vue'
import SettingsModal from './components/settings/SettingsModal.vue'

const app = useAppStore()
const project = useProjectStore()
const pack = usePackStore()
const report = useReportStore()
const confirmModal = ref<InstanceType<typeof ConfirmModal> | null>(null)
const settingsModal = ref<InstanceType<typeof SettingsModal> | null>(null)
const actions = useProjectFileActions()
const appWindow = getCurrentWindow()

let unlistenClose: (() => void) | undefined
let closing = false

async function destroyWindow(reason: string) {
  console.log(`[close] destroy start: ${reason}`)
  await appWindow.destroy()
  console.log('[close] destroy resolved')
}

async function requestClose(source = 'custom-x') {
  if (closing) {
    console.log(`[close] ignored; already closing (${source})`)
    return
  }

  closing = true
  console.log(`[close] request from ${source}; dirty=${project.dirty}`)

  try {
    if (!project.dirty) {
      await destroyWindow('clean')
      return
    }

    const choice = await confirmModal.value?.show()
    console.log('[close] modal choice:', choice)

    if (!choice || choice === 'cancel') return

    if (choice === 'discard') {
      await destroyWindow('discard')
      return
    }

    const result = await actions.doSave()
    console.log('[close] save result:', result)

    if (result === 'saved') {
      await destroyWindow('saved')
    }
  } catch (error) {
    console.error('[close] failed:', error)
  } finally {
    closing = false
    console.log('[close] closing reset')
  }
}

onMounted(async () => {
  actions.bindModal(() => confirmModal.value!.show())

  unlistenClose = await appWindow.onCloseRequested((event) => {
    event.preventDefault()
    void requestClose()
  })

  await app.initLogListener()
  await app.bootstrap()
  await actions.doAutoLoad()
})

onUnmounted(() => {
  unlistenClose?.()
})

useKeyboard({
  onSave: () => actions.doSave(),
  onSaveAs: () => actions.doSaveAs(),
  onNew: () => actions.doNew(),
  onOpen: () => actions.doOpen(),
})

let autoPackTimer: ReturnType<typeof setTimeout> | null = null

watch(
  [() => project.sources, () => ({
    algorithm: pack.algorithm, maxWidth: pack.maxWidth, maxHeight: pack.maxHeight,
    autoSize: pack.autoSize,
    borderPadding: pack.padding.borderPadding, shapePadding: pack.padding.shapePadding,
    extrude: pack.padding.extrude, trim: pack.trim, alphaThreshold: pack.alphaThreshold,
    formats: pack.formats, scaleVariants: pack.scaleVariants,
    outputDir: pack.outputDir, atlasName: pack.atlasName,
    allowRotation: pack.allowRotation,
  })],
  () => {
    if (!project.sourceCount) { report.clearReport(); return }
    if (autoPackTimer) clearTimeout(autoPackTimer)
    autoPackTimer = setTimeout(async () => {
      console.time('auto-pack')
      await pack.executeAutoPack()
      console.timeEnd('auto-pack')
    }, 300)
  },
  { deep: true }
)

watch(
  () => ({
    atlasName: pack.atlasName, outputDir: pack.outputDir,
    algorithm: pack.algorithm, autoSize: pack.autoSize,
    maxWidth: pack.maxWidth, maxHeight: pack.maxHeight,
    borderPadding: pack.padding.borderPadding, shapePadding: pack.padding.shapePadding,
    extrude: pack.padding.extrude, trim: pack.trim, alphaThreshold: pack.alphaThreshold,
    formats: pack.formats, scaleVariants: pack.scaleVariants,
    allowRotation: pack.allowRotation,
    sources: project.sources.map(s => s.sourcePath),
  }),
  () => {
    const fp = JSON.stringify({
      sources: project.sources.map(s => s.sourcePath).sort(),
      atlasName: pack.atlasName, outputDir: pack.outputDir,
      algorithm: pack.algorithm, autoSize: pack.autoSize,
      maxWidth: pack.maxWidth, maxHeight: pack.maxHeight,
      padding: pack.padding, trim: pack.trim,
      alphaThreshold: pack.alphaThreshold,
      formats: [...pack.formats].sort(),
      allowRotation: pack.allowRotation,
      scaleVariants: pack.scaleVariants,
    })
    project.dirty = (fp !== project.savedFingerprint)
  },
  { deep: true }
)
</script>

<template>
  <Shell
    @save="actions.doSave()"
    @save-as="actions.doSaveAs()"
    @new="actions.doNew()"
    @open="actions.doOpen()"
    @publish="actions.doPublish()"
    @request-close="requestClose()"
    @settings-open="settingsModal?.show()"
  />
  <ConfirmModal ref="confirmModal" />
  <SettingsModal ref="settingsModal" />
</template>
