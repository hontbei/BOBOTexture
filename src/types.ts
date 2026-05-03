export interface AppSettings {
  language: string
  launch_animation: boolean
  particle_level: string
  window_width: number
  window_height: number
  log_to_disk: boolean
  last_project_path?: string
}

export interface LogEntry {
  level: string
  source: string
  message: string
  timestamp: string
}

export interface FileEntry {
  path: string
  name: string
  size: number
  modified_ms: number
}

// --- AtlasPacker Pro ---------------------------------------------------------

export interface PixelRect {
  x: number
  y: number
  width: number
  height: number
}

export interface PixelSize {
  width: number
  height: number
}

export interface NormalizedPoint {
  x: number
  y: number
}

export interface UnitySpriteMeta {
  spriteId: string
  internalId: number
  alignment: number
  pivot: NormalizedPoint
  border: { left: number; bottom: number; right: number; top: number }
  parentTextureGuid: string
}

export type SpriteOriginKind = 'file' | 'unity_sub_sprite'

export interface SpriteSource {
  id: string
  name: string
  sourcePath: string
  origin: SpriteOriginKind
  subRect: PixelRect
  unity?: UnitySpriteMeta
}

export type AtlasProAlgorithm = 'skyline' | 'max_rects'
export type AtlasProTrim = 'none' | 'alpha'
export type AtlasProFormat = 'png_only' | 'json_array' | 'tmp_sprite_asset'

export interface PaddingOptions {
  borderPadding: number
  shapePadding: number
  extrude: number
}

export interface ScaleVariant {
  suffix: string
  scale: number
}

export interface AtlasProExecuteRequest {
  sources: SpriteSource[]
  outputDir: string
  atlasName: string
  maxWidth: number
  maxHeight: number
  algorithm: AtlasProAlgorithm
  padding: PaddingOptions
  trim: AtlasProTrim
  alphaThreshold: number
  formats: AtlasProFormat[]
  scaleVariants: ScaleVariant[]
  autoSize?: boolean
}

export interface PackedSprite {
  id: string
  name: string
  frame: PixelRect
  sourceFrame: PixelRect
  sourceSize: PixelSize
  rotated: boolean
  trimmed: boolean
  unity?: UnitySpriteMeta
}

export interface SkippedSprite {
  id: string
  name: string
  reason: string
}

export interface EmittedFile {
  format: AtlasProFormat
  path: string
}

export interface AtlasProReport {
  atlasSize: PixelSize
  placed: PackedSprite[]
  skipped: SkippedSprite[]
  outputs: EmittedFile[]
}

export interface BoboProjectFile {
  version: number
  atlasName: string
  outputDir: string
  settings: {
    algorithm: AtlasProAlgorithm
    maxWidth: number
    maxHeight: number
    autoSize: boolean
    padding: PaddingOptions
    trim: AtlasProTrim
    alphaThreshold: number
    formats: AtlasProFormat[]
    allowRotation: boolean
  }
  scaleVariants: ScaleVariant[]
  sources: string[]
}
