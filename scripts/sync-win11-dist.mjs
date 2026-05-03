import { copyFileSync, existsSync, mkdirSync } from 'node:fs'
import path from 'node:path'
import { fileURLToPath } from 'node:url'
import process from 'node:process'

const scriptDir = path.dirname(fileURLToPath(import.meta.url))
const normalizedRootDir = path.resolve(scriptDir, '..')

const distDir = path.join(normalizedRootDir, 'dist')
const releaseDir = path.join(normalizedRootDir, 'src-tauri', 'target', 'x86_64-pc-windows-gnu', 'release')
const requireAll = process.argv.includes('--require-all')

const artifacts = [
  {
    source: path.join(releaseDir, 'bobotexture-v2.exe'),
    target: path.join(distDir, 'BOBOTextureV2-win11-x64.exe'),
    label: 'Win11 executable',
  },
  {
    source: path.join(releaseDir, 'WebView2Loader.dll'),
    target: path.join(distDir, 'WebView2Loader.dll'),
    label: 'WebView2 loader',
  },
]

mkdirSync(distDir, { recursive: true })

let missingCount = 0

for (const artifact of artifacts) {
  if (!existsSync(artifact.source)) {
    missingCount += 1
    console.warn(`[sync-win11-dist] Missing ${artifact.label}: ${artifact.source}`)
    continue
  }

  copyFileSync(artifact.source, artifact.target)
  console.log(`[sync-win11-dist] Copied ${artifact.label}: ${artifact.target}`)
}

if (requireAll && missingCount > 0) {
  process.exitCode = 1
}
