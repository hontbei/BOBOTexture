#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DIST_EXE="$ROOT_DIR/dist/BOBOTextureV2-win11-x64.exe"
DIST_WEBVIEW2_DLL="$ROOT_DIR/dist/WebView2Loader.dll"
RC_WRAPPER="$ROOT_DIR/scripts/zig-windres"
WIN_RELEASE_DIR="$ROOT_DIR/src-tauri/target/x86_64-pc-windows-gnu/release"

cd "$ROOT_DIR"
npm run build

cd "$ROOT_DIR/src-tauri"
RC_x86_64_pc_windows_gnu="$RC_WRAPPER" cargo zigbuild --release --target x86_64-pc-windows-gnu --features custom-protocol

cd "$ROOT_DIR"
node ./scripts/sync-win11-dist.mjs --require-all
printf 'Win11 executable ready: %s\n' "$DIST_EXE"
printf 'Win11 WebView2 loader ready: %s\n' "$DIST_WEBVIEW2_DLL"
