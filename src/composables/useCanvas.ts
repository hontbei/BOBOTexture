import type { PackedSprite } from '@/types'

export interface DrawAtlasOptions {
  ctx: CanvasRenderingContext2D
  image: HTMLImageElement | null
  atlasWidth: number
  atlasHeight: number
  placed: PackedSprite[]
  selectedId: string | null
  hoveredId: string | null
  zoom: number
  panX: number
  panY: number
  showGrid: boolean
}

export function drawAtlas(options: DrawAtlasOptions) {
  const { ctx, image, atlasWidth, atlasHeight, placed, selectedId, hoveredId, zoom, panX, panY, showGrid } = options
  const canvas = ctx.canvas
  const cw = canvas.width
  const ch = canvas.height

  ctx.clearRect(0, 0, cw, ch)
  ctx.save()
  ctx.translate(cw / 2 + panX, ch / 2 + panY)
  ctx.scale(zoom, zoom)
  ctx.translate(-atlasWidth / 2, -atlasHeight / 2)

  // Draw atlas image
  if (image) {
    ctx.drawImage(image, 0, 0, atlasWidth, atlasHeight)
  } else {
    ctx.fillStyle = '#e8e8e8'
    ctx.fillRect(0, 0, atlasWidth, atlasHeight)
  }

  // Draw grid
  if (showGrid) {
    const scale = zoom
    ctx.strokeStyle = 'rgba(0,0,0,0.12)'
    ctx.lineWidth = 1
    const gridSize = 32
    for (let x = 0; x <= atlasWidth; x += gridSize) {
      ctx.beginPath()
      ctx.moveTo(x, 0); ctx.lineTo(x, atlasHeight)
      ctx.lineWidth = (x % 128 === 0) ? 1.5 / scale : 0.5 / scale
      ctx.stroke()
    }
    for (let y = 0; y <= atlasHeight; y += gridSize) {
      ctx.beginPath()
      ctx.moveTo(0, y); ctx.lineTo(atlasWidth, y)
      ctx.lineWidth = (y % 128 === 0) ? 1.5 / scale : 0.5 / scale
      ctx.stroke()
    }
  }

  // Draw sprite outlines
  const scale = zoom
  ctx.lineWidth = 1 / scale

  for (const sprite of placed) {
    const { x, y, width, height } = sprite.frame
    const isSelected = sprite.id === selectedId
    const isHovered = sprite.id === hoveredId

    if (isSelected) {
      ctx.strokeStyle = '#1a73e8'
      ctx.lineWidth = 2 / scale
      ctx.strokeRect(x + 0.5, y + 0.5, width, height)
      ctx.fillStyle = 'rgba(26,115,232,0.08)'
      ctx.fillRect(x, y, width, height)
    } else if (isHovered) {
      ctx.strokeStyle = '#1a73e8'
      ctx.setLineDash([4 / scale, 2 / scale])
      ctx.strokeRect(x + 0.5, y + 0.5, width, height)
      ctx.setLineDash([])
      ctx.fillStyle = 'rgba(26,115,232,0.04)'
      ctx.fillRect(x, y, width, height)
    } else {
      ctx.strokeStyle = 'rgba(0,0,0,0.35)'
      ctx.lineWidth = 1 / scale
      ctx.strokeRect(x + 0.5, y + 0.5, width, height)
    }
  }

  ctx.restore()
}

export function hitTest(
  canvasX: number,
  canvasY: number,
  atlasWidth: number,
  atlasHeight: number,
  placed: PackedSprite[],
  zoom: number,
  panX: number,
  panY: number,
  cw: number,
  ch: number,
): string | null {
  const atlasX = (canvasX - cw / 2 - panX) / zoom + atlasWidth / 2
  const atlasY = (canvasY - ch / 2 - panY) / zoom + atlasHeight / 2

  for (let i = placed.length - 1; i >= 0; i--) {
    const s = placed[i]
    if (atlasX >= s.frame.x && atlasX <= s.frame.x + s.frame.width &&
        atlasY >= s.frame.y && atlasY <= s.frame.y + s.frame.height) {
      return s.id
    }
  }
  return null
}
