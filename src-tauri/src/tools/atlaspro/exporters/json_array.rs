// AtlasPacker Pro - TexturePacker JSON-Array exporter.
//
// Wave 4: emits the canonical TexturePacker JSON-Array sidecar (Phaser, PixiJS,
// Cocos Creator and most JS engines consume this). The schema is fixed by
// TexturePacker's own documentation; field order matters for byte-for-byte
// compatibility with downstream tooling that diffs the JSON. Top-level keys:
//   - "frames":   array of per-sprite records
//   - "meta":     atlas-level metadata (image filename, format, scale, size)
//
// Coordinate convention: TexturePacker's frame is top-left origin in atlas
// pixels - identical to PackedSprite.frame, so no flipping is required here.
// `rotated` means "rotated 90 deg CW in the atlas" exactly matching
// PackedSprite.rotated; consumers un-rotate when sampling.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::error::AppError;

use super::super::model::{
    AtlasProReport, EmittedFile, ExportFormat, PackedSprite, PixelRect, PixelSize,
};
use super::{make_emitted, output_path};

/// Emit a single JSON-Array sidecar file for one resolution. `image_filename`
/// is what gets written into `meta.image` (e.g. "demo.png" or "demo@2x.png");
/// `scale` is the TexturePacker-style scale string ("1", "0.5", "2").
pub fn emit(
    placed: &[PackedSprite],
    atlas_size: PixelSize,
    image_filename: &str,
    scale: f32,
    output_dir: &Path,
    base_name: &str,
    suffix: &str,
) -> Result<EmittedFile, AppError> {
    let frames: Vec<FrameRecord> = placed.iter().map(FrameRecord::from_sprite).collect();
    let doc = AtlasJson {
        frames,
        meta: MetaSection {
            app: "BOBOTexture AtlasPacker Pro".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            image: image_filename.to_string(),
            format: "RGBA8888".to_string(),
            size: SizeRecord { w: atlas_size.width, h: atlas_size.height },
            scale: format_scale(scale),
        },
    };

    let json = serde_json::to_string_pretty(&doc).map_err(|err| AppError::new(
        "atlaspro_export_json",
        format!("failed to serialize JSON-Array sidecar: {err}"),
    ))?;

    let path: PathBuf = output_path(output_dir, base_name, suffix, "json");
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|err| AppError::new(
                "atlaspro_export_io",
                format!("failed to create directory {}: {err}", parent.display()),
            ))?;
        }
    }
    std::fs::write(&path, json).map_err(|err| AppError::new(
        "atlaspro_export_io",
        format!("failed to write {}: {err}", path.display()),
    ))?;

    Ok(make_emitted(ExportFormat::JsonArray, path))
}

pub fn build_report_outputs(
    placed: &[PackedSprite],
    atlas_size: PixelSize,
    image_filename: &str,
    scale: f32,
) -> Result<String, AppError> {
    let frames: Vec<FrameRecord> = placed.iter().map(FrameRecord::from_sprite).collect();
    let doc = AtlasJson {
        frames,
        meta: MetaSection {
            app: "BOBOTexture AtlasPacker Pro".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            image: image_filename.to_string(),
            format: "RGBA8888".to_string(),
            size: SizeRecord { w: atlas_size.width, h: atlas_size.height },
            scale: format_scale(scale),
        },
    };
    serde_json::to_string_pretty(&doc).map_err(|err| AppError::new(
        "atlaspro_export_json",
        format!("failed to serialize JSON-Array: {err}"),
    ))
}

#[derive(Serialize)]
struct AtlasJson {
    frames: Vec<FrameRecord>,
    meta: MetaSection,
}

#[derive(Serialize)]
struct FrameRecord {
    filename: String,
    frame: RectRecord,
    rotated: bool,
    trimmed: bool,
    #[serde(rename = "spriteSourceSize")]
    sprite_source_size: RectRecord,
    #[serde(rename = "sourceSize")]
    source_size: SizeRecord,
    pivot: PivotRecord,
}

impl FrameRecord {
    fn from_sprite(p: &PackedSprite) -> Self {
        let pivot = if let Some(unity) = p.unity.as_ref() {
            // Unity pivot is normalized to the trimmed sub-rect with bottom-left
            // origin; TexturePacker pivot is normalized to the source size with
            // top-left origin. Approximate by reusing the X axis verbatim and
            // flipping Y; downstream consumers (Phaser etc.) accept this form.
            PivotRecord { x: unity.pivot.x, y: 1.0 - unity.pivot.y }
        } else {
            PivotRecord { x: 0.5, y: 0.5 }
        };
        Self {
            filename: format!("{}.png", p.name),
            frame: RectRecord::from_pixel(p.frame),
            rotated: p.rotated,
            trimmed: p.trimmed,
            sprite_source_size: RectRecord::from_pixel(p.source_frame),
            source_size: SizeRecord {
                w: p.source_size.width,
                h: p.source_size.height,
            },
            pivot,
        }
    }
}

#[derive(Serialize)]
struct RectRecord {
    x: u32,
    y: u32,
    w: u32,
    h: u32,
}

impl RectRecord {
    fn from_pixel(r: PixelRect) -> Self {
        Self { x: r.x, y: r.y, w: r.width, h: r.height }
    }
}

#[derive(Serialize)]
struct SizeRecord { w: u32, h: u32 }

#[derive(Serialize)]
struct PivotRecord { x: f32, y: f32 }

#[derive(Serialize)]
struct MetaSection {
    app: String,
    version: String,
    image: String,
    format: String,
    size: SizeRecord,
    scale: String,
}

fn format_scale(scale: f32) -> String {
    if (scale - scale.trunc()).abs() < f32::EPSILON {
        format!("{}", scale as i32)
    } else {
        // Strip trailing zeros without resorting to a heavy crate.
        let s = format!("{scale}");
        s
    }
}

// Suppress "unused" until pipeline.rs lands; AtlasProReport import is needed for
// docs round-trip via build_report_outputs in subsequent waves.
#[allow(dead_code)]
fn _force_link(_: &AtlasProReport) {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::atlaspro::model::PixelSize;

    fn make_sprite(name: &str, x: u32, y: u32, w: u32, h: u32, rotated: bool) -> PackedSprite {
        PackedSprite {
            id: name.to_string(),
            name: name.to_string(),
            frame: PixelRect::new(x, y, w, h),
            source_frame: PixelRect::new(0, 0, w, h),
            source_size: PixelSize { width: w, height: h },
            rotated,
            trimmed: false,
            unity: None,
        }
    }

    #[test]
    fn emits_valid_json_with_required_fields() {
        let sprites = vec![make_sprite("apple", 0, 0, 32, 32, false)];
        let json = build_report_outputs(
            &sprites,
            PixelSize { width: 64, height: 64 },
            "demo.png",
            1.0,
        ).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["frames"][0]["filename"], "apple.png");
        assert_eq!(parsed["frames"][0]["frame"]["x"], 0);
        assert_eq!(parsed["frames"][0]["frame"]["w"], 32);
        assert_eq!(parsed["frames"][0]["rotated"], false);
        assert_eq!(parsed["meta"]["image"], "demo.png");
        assert_eq!(parsed["meta"]["size"]["w"], 64);
        assert_eq!(parsed["meta"]["scale"], "1");
        assert_eq!(parsed["meta"]["format"], "RGBA8888");
    }

    #[test]
    fn rotated_flag_propagates() {
        let sprites = vec![make_sprite("rot", 10, 10, 8, 16, true)];
        let json = build_report_outputs(
            &sprites,
            PixelSize { width: 32, height: 32 },
            "x.png",
            1.0,
        ).unwrap();
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed["frames"][0]["rotated"], true);
    }

    #[test]
    fn scale_formats_integers_without_decimal() {
        assert_eq!(format_scale(1.0), "1");
        assert_eq!(format_scale(2.0), "2");
        assert_eq!(format_scale(0.5), "0.5");
    }

    #[test]
    fn writes_file_to_disk() {
        let dir = std::env::temp_dir().join(format!("nebulakit_atlaspro_jsonarray_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sprites = vec![make_sprite("a", 0, 0, 8, 8, false)];
        let emitted = emit(
            &sprites,
            PixelSize { width: 16, height: 16 },
            "demo.png",
            1.0,
            &dir,
            "demo",
            "",
        ).unwrap();
        assert_eq!(emitted.format, ExportFormat::JsonArray);
        let body = std::fs::read_to_string(&emitted.path).unwrap();
        assert!(body.contains("\"filename\""));
        assert!(body.contains("\"a.png\""));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
