# BOBOTextureV2 Agent Notes

## What This Repo Is

Tauri 2 desktop sprite-atlas packer: Vue 3/TypeScript frontend, Rust backend, Unity TextMeshPro Sprite Asset (.asset + .meta) export. Single-tool UI — `src/App.vue` mounts `src/components/layout/Shell.vue`, no router.

## Commands That Matter

```bash
npm install
npm run typecheck                         # vue-tsc --noEmit
npm run build                             # typecheck + vite build (frontend only)
npm run tauri:dev                         # Tauri dev, Vite on port 1420
npm run tauri:build                       # Full Tauri release build (Windows native)
```

- `npm run build` is **frontend only**. `npm run tauri:build` does both frontend + Rust.
- `build.bat` is the Windows one-click build + deploy script (npm install → tauri:build → git commit → git push). Run on Windows, NOT in WSL.
- **Do NOT cross-compile Windows from WSL** with zigbuild — it OOMs and locks NTFS files. Build natively on Windows.

## Architecture Hotspots

### Rust Backend (`src-tauri/src/tools/atlaspro/`)
- `model.rs` — All IPC request/response types. Fields use `#[serde(rename_all = "camelCase")]`.
- `pipeline.rs` — End-to-end: scan → preprocess → pack → composite → export.
- `exporters/tmp_sprite_asset.rs` — Unity TMP YAML emitter. Critical constants: `TMP_SPRITE_ASSET_SCRIPT_GUID`, `BEARING_Y_RATIO = 0.903125`, `hash_tmp_name()` (DJB2).
- `exporters/tmp_bundle.rs` — 4-file bundle: .png + .png.meta + .asset + .asset.meta.
- `packer.rs` — MaxRects (custom) + Skyline (rectangle-pack crate). `pack_auto_square()` tries POT squares 256→4096.

### Frontend (`src/`)
- `types.ts` — Mirrors Rust `model.rs`. `AtlasProFormat` currently only exposes `png_only | json_array | tmp_sprite_asset` (Rust has `JsonHash` too but not surfaced).
- `src/ipc/` — Tauri `invoke` wrappers. Argument names must match Rust command parameters 1:1.
- `src/stores/` — Pinia setup stores. Key stores:
  - `project.ts` — sources, dirty flag, undo/redo, save/load. **Use `replaceSources()` for loading, not `addSources()` — `addSources` marks dirty.**
  - `pack.ts` — all packing settings + execute/autoPack.
  - `report.ts` — `AtlasProReport` result, atlas preview URL, TMP tag examples.
  - `ui.ts` — selected/hovered sprite, zoom, pan, open sections.
- `src/composables/useProjectFileActions.ts` — **Single source of truth** for save/open/new/close. All three entry points (Toolbar, keyboard shortcuts, close handler) delegate here.
- `src/composables/useFileDrop.ts` — Tauri 2 native drag-drop listener. **HTML5 `dataTransfer.files[].path` does NOT work in Tauri 2.** Must use `currentWindow.onDragDropEvent()` + `webview.onDragDropEvent()`.
- `src/components/layout/WindowChrome.vue` — Custom title bar. **`data-tauri-drag-region` must ONLY be on the title text, NOT on buttons.** Buttons need `@click.stop` and `type="button"`.

## Tauri 2 Gotchas

### Close Button
- `appWindow.destroy()` **requires** `"core:window:allow-destroy"` in `capabilities/default.json`. Without it, `destroy()` fails silently.
- The close handler wraps everything in `try/catch/finally` to reset the `closing` guard. A stuck `closing = true` blocks all future closes.
- Custom X button emits `request-close` event → Shell forwards → App handles directly. `onCloseRequested` is only a safety net for Alt+F4.
- `close()` triggers `onCloseRequested`; `destroy()` bypasses it. Do NOT call `close()` inside the close handler.

### Drag-Drop
- OS-level file drag-drop is intercepted by Tauri. Must use `onDragDropEvent` (not HTML5 drop events). See `useFileDrop.ts`.

### Custom Window Chrome
- `tauri.conf.json`: `decorations: false`, `transparent: false`, `shadow: true`.
- `"csp": null` required for `convertFileSrc()` preview URLs.
- `assetProtocol` enabled with broad scope for local file access.

## IPC Contract

Frontend `invoke()` argument names must match Rust parameter names exactly:
- `invoke('scan_atlaspro_inputs', { request })` → `fn scan_atlaspro_inputs(request: AtlasProScanRequest)`
- `invoke('execute_atlaspro', { request })` → `fn execute_atlaspro(request: AtlasProExecuteRequest)`
- `invoke('save_settings', { settings })` → `fn save_settings(settings: AppSettings)`
- `invoke('read_text_file', { path })` / `invoke('write_text_file', { request: { path, content } })`
- `invoke('path_exists', { path })` → returns `bool`

Do NOT flatten request objects unless the Rust signature changes.

## Project File Format

`.boboproj` — JSON file:
```json
{
  "version": 1,
  "atlasName": "myatlas",
  "outputDir": "/path",
  "settings": { "algorithm", "padding", "trim", "formats", ... },
  "scaleVariants": [],
  "sources": ["/absolute/path/sprite1.png"]
}
```
Save: `project.saveProject(path)` → writes via `writeTextFile` IPC. Load: `project.loadProject(path)` → reads, restores settings, scans sources. Save As checks `pathExists` before overwriting.

## Repo-Specific Constraints

- TMP export does not support rotated sprites. When TMP format selected, `allowRotation` must be false.
- TMP export only emits for @1x atlas, not scale variants.
- `window_width` / `window_height` in `AppSettings` are `f64` in Rust, `number` in TS.
- WSL build artifacts on NTFS (`/mnt/e/`) frequently get file-locked. Build on Windows native.
- The `closing` guard in App.vue must always reset in `finally` block.
