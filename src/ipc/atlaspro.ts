import { invoke } from '@tauri-apps/api/core'

import type { AtlasProExecuteRequest, AtlasProReport, SpriteSource } from '../types'

export function scanAtlasProInputs(inputs: string[], recursive: boolean) {
  return invoke<SpriteSource[]>('scan_atlaspro_inputs', {
    request: { inputs, recursive },
  })
}

export function executeAtlasPro(request: AtlasProExecuteRequest) {
  return invoke<AtlasProReport>('execute_atlaspro', { request })
}
