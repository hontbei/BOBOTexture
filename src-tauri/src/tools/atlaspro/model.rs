// AtlasPacker Pro - shared data model.
//
// Wave 2 establishes the IPC-facing request/response shapes plus the internal
// sprite/placement records used by preprocess, packer, compositor and exporters.
// All public structs are serde-friendly so they can flow across the Tauri bridge
// without bespoke wrappers; field naming matches the Vue/TS layer 1:1.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

// --- IPC request: scan inputs ------------------------------------------------

/// Frontend-supplied paths to scan. Accepts files, directories and Unity .meta
/// files; the scanner expands directories and extracts sub-sprites from any
/// .meta whose `spriteMode == 2` (Multiple).
#[derive(Debug, Clone, Deserialize)]
pub struct AtlasProScanRequest {
    pub inputs: Vec<String>,
    pub recursive: bool,
}

// --- IPC response: discovered sprite sources --------------------------------

/// Origin of a sprite source. Unity sub-sprites are virtual: they share a PNG
/// with siblings and carry a sub-rectangle inside that PNG (top-left origin
/// after Y-flip - see `unity_meta` module).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SpriteOrigin {
    /// Standalone image file.
    File,
    /// Sub-sprite carved out of a Unity sprite-sheet PNG via its .meta.
    UnitySubSprite,
}

/// One discovered sprite source. The (path, sub_rect) pair uniquely identifies
/// the sprite even when many sprites share a Unity sprite-sheet PNG.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpriteSource {
    /// Stable, frontend-friendly identifier (uuid v4 hex).
    pub id: String,
    /// User-facing display name (Unity sprite name or PNG filename stem).
    pub name: String,
    /// Absolute path to the source image on disk.
    pub source_path: String,
    /// Origin classification.
    pub origin: SpriteOrigin,
    /// Pixel rectangle inside the source image (top-left origin) describing
    /// which region to use. For `File` origins this is the full image bounds.
    pub sub_rect: PixelRect,
    /// Optional Unity .meta data preserved verbatim for re-export. Only set
    /// when origin == UnitySubSprite.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unity: Option<UnitySpriteMeta>,
}

/// Pixel rectangle, top-left origin, inclusive of (x, y), exclusive of
/// (x + width, y + height). All fields in pixels.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

impl PixelRect {
    pub fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self { x, y, width, height }
    }

    pub fn is_empty(&self) -> bool {
        self.width == 0 || self.height == 0
    }
}

/// Unity-specific sprite metadata preserved verbatim so re-exported atlases
/// keep deterministic GUIDs / pivots / borders.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnitySpriteMeta {
    /// 32-char lowercase hex GUID of the original sprite.
    pub sprite_id: String,
    /// 64-bit internal id (Unity uses i64 even though the YAML emits unquoted).
    pub internal_id: i64,
    /// Alignment enum: 0=Center, 1=TopLeft .. 8=BottomRight, 9=Custom.
    pub alignment: u8,
    /// Pivot in normalized [0,1] coordinates relative to the sub-rect, with
    /// Unity's bottom-left origin convention preserved.
    pub pivot: NormalizedPoint,
    /// Border in pixels: (left, bottom, right, top) i.e. Vector4 (x, y, z, w).
    pub border: Border,
    /// Original GUID of the parent texture asset (32-char lowercase hex).
    pub parent_texture_guid: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NormalizedPoint {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Border {
    pub left: f32,
    pub bottom: f32,
    pub right: f32,
    pub top: f32,
}

// --- IPC request: pack & export ---------------------------------------------

/// Packing strategy. Maps to rectangle-pack heuristics or future custom
/// algorithms. `Skyline` is the Wave 2 default.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PackAlgorithm {
    #[default]
    MaxRects,
    Skyline,
    /// Guillotine. Reserved for Wave 3.
    Guillotine,
}

/// Visual padding strategy applied to each sprite before packing.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaddingOptions {
    /// Pixels of empty space inserted between sprites.
    pub shape_padding: u32,
    /// Pixels of empty space between any sprite and the atlas border.
    pub border_padding: u32,
    /// Pixels of pixel-replication around each sprite (prevents bleeding when
    /// the atlas is sampled with linear filtering).
    pub extrude: u32,
}

/// Trim policy. `Alpha` removes fully-transparent margin; `None` preserves the
/// source bounds. Wave 2 implements both.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrimMode {
    #[default]
    None,
    /// Crop fully transparent pixels (alpha <= threshold) from all four sides.
    Alpha,
}

/// Output target list. A single pack operation may emit several formats.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExportFormat {
    /// Plain PNG only, no metadata sidecar.
    PngOnly,
    /// PNG + JSON-Array (TexturePacker default, used by Phaser etc.).
    JsonArray,
    /// PNG + JSON-Hash (TexturePacker hashed-frames variant).
    JsonHash,
    /// PNG + Unity TextMeshPro Sprite Asset (.asset YAML + .meta).
    TmpSpriteAsset,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasProExecuteRequest {
    /// Sprite sources resolved during the scan phase.
    pub sources: Vec<SpriteSource>,
    /// Output directory (must already be writable).
    pub output_dir: String,
    /// Atlas base name; sanitized before use.
    pub atlas_name: String,
    /// Maximum atlas pixel dimensions. Power-of-two snapping is the
    /// compositor's job, not the packer's.
    pub max_width: u32,
    pub max_height: u32,
    /// Packer choice. Defaults to Skyline.
    #[serde(default)]
    pub algorithm: PackAlgorithm,
    /// Padding/extrude options.
    #[serde(default)]
    pub padding: PaddingOptions,
    /// Trim policy.
    #[serde(default)]
    pub trim: TrimMode,
    /// Allow 90-degree rotation when it improves packing.
    #[serde(default)]
    pub allow_rotation: bool,
    /// Alpha threshold for trim (0..=255). Pixels with alpha <= threshold are
    /// considered transparent. Defaults to 0 (fully transparent only).
    #[serde(default)]
    pub alpha_threshold: u8,
    /// Export targets to emit.
    pub formats: Vec<ExportFormat>,
    /// Resolution variants to emit alongside the @1x atlas. Empty list means
    /// only @1x is produced. Each variant downscales/upscales the final atlas
    /// using a high-quality filter; placement coordinates are scaled in lock-
    /// step so exporter sidecars stay consistent per variant.
    #[serde(default)]
    pub scale_variants: Vec<ScaleVariant>,
    /// When true, the packer tries progressively larger power-of-two squares
    /// (256→512→1024→2048→4096) up to max_width.min(max_height) and picks the
    /// smallest one that fits all sprites. Disabled by default for backward
    /// compatibility with explicit max_width/max_height requests.
    #[serde(default)]
    pub auto_size: bool,
}

/// One output resolution. `suffix` is appended to the atlas filename (TexturePacker
/// convention: `atlas@2x.png`); `scale` multiplies all pixel dimensions and
/// placements (1.0 == passthrough; 0.5 == half size; 2.0 == double size).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ScaleVariant {
    pub suffix: String,
    pub scale: f32,
}

// --- IPC response: pack report ----------------------------------------------

/// One placed sprite as it appears in the final atlas. All coordinates are in
/// top-left-origin pixels relative to the atlas image; downstream exporters
/// (e.g. TMP) flip Y as needed.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PackedSprite {
    pub id: String,
    pub name: String,
    /// Final placement rectangle inside the atlas (post-padding).
    pub frame: PixelRect,
    /// Source rectangle within the original image (post-trim, pre-padding).
    pub source_frame: PixelRect,
    /// Original image bounds before trimming.
    pub source_size: PixelSize,
    /// True when the sprite was rotated 90 degrees clockwise during packing.
    pub rotated: bool,
    /// True when the sprite was alpha-trimmed.
    pub trimmed: bool,
    /// Unity metadata carried over (only present for Unity sub-sprites).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unity: Option<UnitySpriteMeta>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PixelSize {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AtlasProReport {
    /// Final atlas dimensions.
    pub atlas_size: PixelSize,
    /// All successfully placed sprites.
    pub placed: Vec<PackedSprite>,
    /// Sprites that did not fit; the user must shrink them or enlarge the atlas.
    pub skipped: Vec<SkippedSprite>,
    /// Output files generated, keyed by export format name.
    pub outputs: Vec<EmittedFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SkippedSprite {
    pub id: String,
    pub name: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EmittedFile {
    pub format: ExportFormat,
    pub path: String,
}

// --- Internal staging types (NOT crossing IPC) ------------------------------

/// In-memory loaded sprite ready for packing. Lives only in the Rust process.
#[derive(Debug)]
pub struct LoadedSprite {
    pub id: String,
    pub name: String,
    pub origin: SpriteOrigin,
    pub source_path: PathBuf,
    /// Original image bounds (pre-trim).
    pub source_size: PixelSize,
    /// Trimmed bounds within the source image (post-trim, top-left origin).
    pub trimmed_rect: PixelRect,
    /// Sub-rect of the source image that this sprite represents. Equals the
    /// full image rect for File-origin sprites, and the Unity sprite rect
    /// (already converted to top-left coordinates) for UnitySubSprite-origin
    /// sprites. Used by exporters to compute sub-sprite-local coordinates.
    pub sub_rect: PixelRect,
    /// True when trim actually shrunk the rectangle.
    pub trimmed: bool,
    /// Pixel buffer of the trimmed region (no padding/extrude yet).
    pub pixels: image::RgbaImage,
    /// Unity metadata to carry through to exporters.
    pub unity: Option<UnitySpriteMeta>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pixel_rect_emptiness() {
        assert!(PixelRect::new(0, 0, 0, 10).is_empty());
        assert!(PixelRect::new(0, 0, 10, 0).is_empty());
        assert!(!PixelRect::new(0, 0, 1, 1).is_empty());
    }

    #[test]
    fn pack_algorithm_default_is_maxrects() {
        assert_eq!(PackAlgorithm::default(), PackAlgorithm::MaxRects);
    }

    #[test]
    fn execute_request_round_trip() {
        // Confirms the camelCase contract the Vue layer relies on.
        let json = r#"{
            "sources": [],
            "outputDir": "/tmp/out",
            "atlasName": "demo",
            "maxWidth": 2048,
            "maxHeight": 2048,
            "formats": ["json_array"]
        }"#;
        let request: AtlasProExecuteRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.atlas_name, "demo");
        assert_eq!(request.max_width, 2048);
        assert_eq!(request.algorithm, PackAlgorithm::MaxRects);
        assert_eq!(request.formats, vec![ExportFormat::JsonArray]);
        assert!(!request.allow_rotation);
    }

    #[test]
    fn execute_request_accepts_full_camelcase_payload_from_frontend() {
        let json = r#"{
            "sources": [],
            "outputDir": "/tmp/out",
            "atlasName": "demo",
            "maxWidth": 4096,
            "maxHeight": 4096,
            "algorithm": "skyline",
            "padding": { "borderPadding": 4, "shapePadding": 2, "extrude": 1 },
            "trim": "alpha",
            "allowRotation": true,
            "alphaThreshold": 8,
            "formats": ["png_only", "json_array", "tmp_sprite_asset"],
            "scaleVariants": [{ "suffix": "@2x", "scale": 2.0 }]
        }"#;
        let request: AtlasProExecuteRequest = serde_json::from_str(json).expect(
            "frontend camelCase payload must deserialize without manual key juggling",
        );
        assert_eq!(request.padding.border_padding, 4);
        assert_eq!(request.padding.shape_padding, 2);
        assert_eq!(request.padding.extrude, 1);
        assert_eq!(request.trim, TrimMode::Alpha);
        assert!(request.allow_rotation);
        assert_eq!(request.alpha_threshold, 8);
        assert_eq!(request.scale_variants.len(), 1);
        assert_eq!(request.scale_variants[0].suffix, "@2x");
    }

    #[test]
    fn execute_request_rejects_legacy_padding_keys() {
        let json = r#"{
            "sources": [], "outputDir": "/x", "atlasName": "y",
            "maxWidth": 64, "maxHeight": 64,
            "padding": { "border": 1, "shape": 1, "extrude": 0 },
            "formats": ["png_only"]
        }"#;
        let err = serde_json::from_str::<AtlasProExecuteRequest>(json)
            .expect_err("legacy 'border'/'shape' keys must be rejected, not silently accepted");
        let msg = err.to_string();
        assert!(msg.contains("shapePadding") || msg.contains("borderPadding"),
            "error must point at the camelCase contract violation, got: {msg}");
    }
}
