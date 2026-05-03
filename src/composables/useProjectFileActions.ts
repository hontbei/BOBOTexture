import { open, save, ask, message } from '@tauri-apps/plugin-dialog'
import { useProjectStore } from '@/stores/project'
import { usePackStore } from '@/stores/pack'
import { useReportStore } from '@/stores/report'
import { useAppStore } from '@/stores/app'
import { scanAtlasProInputs } from '@/ipc/atlaspro'
import { pathExists, openInExplorer } from '@/ipc/system'

export type SaveResult = 'saved' | 'cancelled' | 'failed'

let counter = 0
function freshName(): string {
  const d = new Date()
  const ds = `${d.getFullYear()}${String(d.getMonth() + 1).padStart(2, '0')}${String(d.getDate()).padStart(2, '0')}`
  counter++
  return `Untitled-${ds}-${counter}`
}

function ext(p: string): string {
  return p.endsWith('.boboproj') ? p : p + '.boboproj'
}

export function useProjectFileActions() {
  const project = useProjectStore()
  const pack = usePackStore()
  const report = useReportStore()
  const app = useAppStore()

  let modalShow: (() => Promise<string | undefined>) | null = null

  function bindModal(fn: () => Promise<string | undefined>) {
    modalShow = fn
  }

  async function writeTo(path: string): Promise<SaveResult> {
    try { await project.saveProject(path); await app.saveLastProjectPath(path); return 'saved' }
    catch { return 'failed' }
  }

  async function doSave(): Promise<SaveResult> {
    if (project.projectFilePath) return writeTo(project.projectFilePath)
    return doSaveAs()
  }

  async function doSaveAs(): Promise<SaveResult> {
    const picked = await save({
      filters: [{ name: 'BOBO Project', extensions: ['boboproj'] }],
      defaultPath: project.projectName + '.boboproj',
    })
    if (!picked) return 'cancelled'

    const dest = ext(picked)
    if (dest !== project.projectFilePath) {
      try {
        if (await pathExists(dest)) {
          if (!(await ask('File already exists. Replace it?', { title: 'Confirm Overwrite', kind: 'warning' }))) {
            return 'cancelled'
          }
        }
      } catch {}
    }
    return writeTo(dest)
  }

  async function guardDestructive(): Promise<boolean> {
    if (!project.dirty) return true
    const choice = await modalShow?.()
    if (!choice || choice === 'cancel') return false
    if (choice === 'discard') return true
    return (await doSave()) === 'saved'
  }

  async function doNew() {
    if (!(await guardDestructive())) return
    project.beginSuppress()
    project.resetProject(freshName())
    pack.resetDefaults()
    project.endSuppress()
    report.clearReport()
  }

  async function doOpen() {
    const picked = await open({
      multiple: true,
      filters: [
        { name: 'All Supported', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp', 'boboproj'] },
        { name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp', 'bmp'] },
        { name: 'BOBO Project', extensions: ['boboproj'] },
      ],
    })
    if (!picked) return
    const paths = Array.isArray(picked) ? picked : [picked]
    const projPath = paths.find(p => p.endsWith('.boboproj'))

    if (projPath) {
      if (!(await guardDestructive())) return
      try { await project.loadProject(projPath); await app.saveLastProjectPath(projPath) }
      catch { await message(`Failed to load: ${projPath}`, { kind: 'error' }) }
      return
    }
    try { const d = await scanAtlasProInputs(paths, true); project.addSources(d) }
    catch (e) { console.error('Scan failed:', e) }
  }

  async function doSetOutputDir() {
    const r = await open({ directory: true, multiple: false })
    if (typeof r === 'string') pack.outputDir = r
  }

  async function doPublish() {
    if (!pack.canExecute) return
    try { await pack.executePack(); try { await openInExplorer(pack.outputDir) } catch {} }
    catch (e) { console.error('Pack failed:', e) }
  }

  async function doAutoLoad() {
    const lp = app.settings.last_project_path
    if (!lp) return
    try {
      if (!(await pathExists(lp))) { app.clearLastProjectPath(); return }
      await project.loadProject(lp)
    } catch { app.clearLastProjectPath() }
  }

  return {
    bindModal, doSave, doSaveAs, doNew, doOpen,
    doSetOutputDir, doPublish, doAutoLoad,
  }
}
