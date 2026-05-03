import { invoke } from '@tauri-apps/api/core'

import type { FileEntry, LogEntry } from '../types'

export function collectFiles(inputs: string[], recursive = true) {
  return invoke<FileEntry[]>('collect_files', {
    request: { inputs, recursive },
  })
}

export function exportLogs(outputPath: string, logs?: LogEntry[]) {
  const content = logs
    ? logs.map((entry) => `[${entry.timestamp}] [${entry.level}] [${entry.source}] ${entry.message}`).join('\n')
    : undefined

  return invoke('export_logs', {
    request: { output_path: outputPath, content },
  })
}

export function openInExplorer(path: string) {
  return invoke<void>('open_in_explorer', { path })
}
