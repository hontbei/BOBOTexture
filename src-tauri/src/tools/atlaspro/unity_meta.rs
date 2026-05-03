// AtlasPacker Pro - Unity .meta sprite-sheet parser.
//
// Parses Unity TextureImporter .meta YAML files where `spriteMode: 2` (Multiple)
// and extracts each sub-sprite as a virtual SpriteSource so the rest of the
// pipeline can treat it identically to a standalone PNG.
//
// CRITICAL coordinate convention:
//   Unity stores `spriteSheet.sprites[].rect` with BOTTOM-LEFT origin
//   (rect.y == 0 means the sprite touches the BOTTOM of the texture).
//   Our pipeline uses TOP-LEFT origin everywhere (matches the `image` crate).
//   The conversion is done exactly once, here, in `flip_y_to_top_left`:
//       top_left_y = texture_height - rect.y - rect.height

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::AppError;

use super::model::{Border, NormalizedPoint, PixelRect, SpriteOrigin, SpriteSource, UnitySpriteMeta};

const SPRITE_MODE_MULTIPLE: i64 = 2;

#[derive(Debug, Deserialize)]
struct MetaRoot {
    #[serde(rename = "TextureImporter")]
    texture_importer: TextureImporter,
    #[serde(default)]
    guid: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TextureImporter {
    #[serde(default, rename = "spriteMode")]
    sprite_mode: i64,
    #[serde(default, rename = "spriteSheet")]
    sprite_sheet: Option<SpriteSheetSection>,
}

#[derive(Debug, Deserialize)]
struct SpriteSheetSection {
    #[serde(default)]
    sprites: Vec<SpriteEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SpriteEntry {
    #[serde(default)]
    name: String,
    rect: SpriteRect,
    #[serde(default)]
    alignment: u8,
    #[serde(default)]
    pivot: PointXY,
    #[serde(default)]
    border: Vector4,
    #[serde(default, rename = "spriteID")]
    sprite_id: String,
    #[serde(default, rename = "internalID")]
    internal_id: i64,
}

#[derive(Debug, Deserialize)]
struct SpriteRect {
    #[serde(default)]
    x: i64,
    #[serde(default)]
    y: i64,
    #[serde(default)]
    width: i64,
    #[serde(default)]
    height: i64,
}

#[derive(Debug, Default, Deserialize)]
struct PointXY {
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
}

#[derive(Debug, Default, Deserialize)]
struct Vector4 {
    #[serde(default)]
    x: f32,
    #[serde(default)]
    y: f32,
    #[serde(default)]
    z: f32,
    #[serde(default)]
    w: f32,
}

/// Outcome of parsing a .meta file.
#[derive(Debug)]
pub enum MetaParseOutcome {
    /// `spriteMode == 2` and sprites were extracted.
    Multiple { texture_path: PathBuf, sprites: Vec<SpriteSource> },
    /// `spriteMode != 2`, caller should treat the sibling PNG as a single image.
    NotMultiple,
}

/// Parse a Unity .meta file and, when it describes a Multiple-mode sprite-sheet,
/// emit one SpriteSource per sub-sprite. The sibling PNG is located by
/// stripping the `.meta` suffix from `meta_path`.
pub fn parse_meta_file(meta_path: &Path) -> Result<MetaParseOutcome, AppError> {
    let body = fs::read_to_string(meta_path)
        .map_err(|err| AppError::new("atlaspro_meta_read", format!("{}: {}", meta_path.display(), err)))?;
    let texture_path = sibling_texture_path(meta_path)?;
    parse_meta_string(&body, &texture_path)
}

/// Same as `parse_meta_file` but operates on an in-memory YAML string with the
/// sibling texture path provided externally. Used by tests and by the scanner
/// when it has already cached the file body.
pub fn parse_meta_string(body: &str, texture_path: &Path) -> Result<MetaParseOutcome, AppError> {
    let root: MetaRoot = serde_yaml::from_str(body)
        .map_err(|err| AppError::new("atlaspro_meta_parse", format!("{}: {}", texture_path.display(), err)))?;

    if root.texture_importer.sprite_mode != SPRITE_MODE_MULTIPLE {
        return Ok(MetaParseOutcome::NotMultiple);
    }

    let Some(sheet) = root.texture_importer.sprite_sheet else {
        return Ok(MetaParseOutcome::Multiple { texture_path: texture_path.to_path_buf(), sprites: Vec::new() });
    };

    let (tex_w, tex_h) = read_image_dimensions(texture_path)?;

    let mut out = Vec::with_capacity(sheet.sprites.len());
    for entry in sheet.sprites {
        let sprite = build_sprite_source(entry, texture_path, &root.guid, tex_w, tex_h)?;
        out.push(sprite);
    }

    Ok(MetaParseOutcome::Multiple { texture_path: texture_path.to_path_buf(), sprites: out })
}

fn build_sprite_source(
    entry: SpriteEntry,
    texture_path: &Path,
    parent_guid: &str,
    tex_w: u32,
    tex_h: u32,
) -> Result<SpriteSource, AppError> {
    if entry.rect.width <= 0 || entry.rect.height <= 0 {
        return Err(AppError::new(
            "atlaspro_meta_rect",
            format!("sprite '{}' has non-positive size {}x{}", entry.name, entry.rect.width, entry.rect.height),
        ));
    }
    if entry.rect.x < 0 || entry.rect.y < 0 {
        return Err(AppError::new(
            "atlaspro_meta_rect",
            format!("sprite '{}' has negative origin ({}, {})", entry.name, entry.rect.x, entry.rect.y),
        ));
    }

    let unity_x = entry.rect.x as u64;
    let unity_y = entry.rect.y as u64;
    let w = entry.rect.width as u64;
    let h = entry.rect.height as u64;

    if unity_x + w > tex_w as u64 || unity_y + h > tex_h as u64 {
        return Err(AppError::new(
            "atlaspro_meta_rect",
            format!(
                "sprite '{}' rect ({},{},{}x{}) exceeds texture {}x{}",
                entry.name, unity_x, unity_y, w, h, tex_w, tex_h
            ),
        ));
    }

    let top_left_y = flip_y_to_top_left(unity_y, h, tex_h as u64);
    let sub_rect = PixelRect::new(unity_x as u32, top_left_y as u32, w as u32, h as u32);

    let unity_meta = UnitySpriteMeta {
        sprite_id: entry.sprite_id,
        internal_id: entry.internal_id,
        alignment: entry.alignment,
        pivot: NormalizedPoint { x: entry.pivot.x, y: entry.pivot.y },
        border: Border {
            left: entry.border.x,
            bottom: entry.border.y,
            right: entry.border.z,
            top: entry.border.w,
        },
        parent_texture_guid: parent_guid.to_string(),
    };

    Ok(SpriteSource {
        id: uuid::Uuid::new_v4().simple().to_string(),
        name: if entry.name.is_empty() { "sprite".to_string() } else { entry.name },
        source_path: texture_path.display().to_string(),
        origin: SpriteOrigin::UnitySubSprite,
        sub_rect,
        unity: Some(unity_meta),
    })
}

/// Convert a Unity bottom-left rect Y to a top-left image Y.
/// All values in pixels; texture_height must be > 0 and rect must fit.
#[inline]
fn flip_y_to_top_left(unity_y: u64, rect_height: u64, texture_height: u64) -> u64 {
    texture_height - unity_y - rect_height
}

fn sibling_texture_path(meta_path: &Path) -> Result<PathBuf, AppError> {
    let s = meta_path.to_string_lossy();
    if !s.ends_with(".meta") {
        return Err(AppError::new(
            "atlaspro_meta_path",
            format!("{}: not a .meta file", meta_path.display()),
        ));
    }
    let trimmed = &s[..s.len() - ".meta".len()];
    let texture = PathBuf::from(trimmed);
    if !texture.is_file() {
        return Err(AppError::new(
            "atlaspro_meta_path",
            format!("{}: sibling texture not found", texture.display()),
        ));
    }
    Ok(texture)
}

fn read_image_dimensions(path: &Path) -> Result<(u32, u32), AppError> {
    let reader = image::ImageReader::open(path)
        .map_err(|err| AppError::new("atlaspro_meta_image", format!("{}: {}", path.display(), err)))?;
    reader
        .into_dimensions()
        .map_err(|err| AppError::new("atlaspro_meta_image", format!("{}: {}", path.display(), err)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_nanos();
        let dir = std::env::temp_dir().join(format!("nebulakit_atlaspro_{label}_{stamp}"));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn flip_y_round_trip() {
        // Texture 100 tall, sprite at unity_y=10 height=20 => top_left_y = 70.
        assert_eq!(flip_y_to_top_left(10, 20, 100), 70);
        // Sprite touching the top: unity_y = 80, height = 20 => top_left_y = 0.
        assert_eq!(flip_y_to_top_left(80, 20, 100), 0);
        // Sprite touching the bottom: unity_y = 0, height = 20 => top_left_y = 80.
        assert_eq!(flip_y_to_top_left(0, 20, 100), 80);
    }

    #[test]
    fn rejects_non_meta_paths() {
        let err = sibling_texture_path(Path::new("/tmp/foo.png")).unwrap_err();
        assert_eq!(err.code, "atlaspro_meta_path");
    }

    #[test]
    fn parses_real_unity_meta_with_two_sprites() {
        let dir = temp_dir("metaparse");
        let png_path = dir.join("sheet.png");
        // 64x32 fully transparent PNG; only dimensions matter for the parser.
        RgbaImage::from_pixel(64, 32, Rgba([0, 0, 0, 0])).save(&png_path).unwrap();

        let yaml = r#"
fileFormatVersion: 2
guid: aaaabbbbccccddddeeeeffff00001111
TextureImporter:
  serializedVersion: 12
  spriteMode: 2
  spriteSheet:
    serializedVersion: 2
    sprites:
    - serializedVersion: 2
      name: Block_0
      rect:
        serializedVersion: 2
        x: 0
        y: 0
        width: 28
        height: 26
      alignment: 0
      pivot: {x: 0.5, y: 0.5}
      border: {x: 1, y: 2, z: 3, w: 4}
      spriteID: 81d2b95829e583320800000000000000
      internalID: 2537882373423050008
    - serializedVersion: 2
      name: Block_1
      rect:
        serializedVersion: 2
        x: 30
        y: 6
        width: 28
        height: 26
      alignment: 9
      pivot: {x: 0.25, y: 0.75}
      border: {x: 0, y: 0, z: 0, w: 0}
      spriteID: 11112222333344445555666677778888
      internalID: 1234567890123456789
"#;

        let outcome = parse_meta_string(yaml, &png_path).unwrap();
        let sprites = match outcome {
            MetaParseOutcome::Multiple { sprites, .. } => sprites,
            MetaParseOutcome::NotMultiple => panic!("expected Multiple"),
        };
        assert_eq!(sprites.len(), 2);

        // Block_0: unity (0, 0, 28x26) on a 32-tall texture => top_left y = 32 - 0 - 26 = 6.
        let s0 = &sprites[0];
        assert_eq!(s0.name, "Block_0");
        assert_eq!(s0.origin, SpriteOrigin::UnitySubSprite);
        assert_eq!(s0.sub_rect, PixelRect::new(0, 6, 28, 26));
        let u0 = s0.unity.as_ref().unwrap();
        assert_eq!(u0.sprite_id, "81d2b95829e583320800000000000000");
        assert_eq!(u0.internal_id, 2537882373423050008);
        assert_eq!(u0.alignment, 0);
        assert_eq!(u0.pivot.x, 0.5);
        assert_eq!(u0.pivot.y, 0.5);
        assert_eq!(u0.border.left, 1.0);
        assert_eq!(u0.border.bottom, 2.0);
        assert_eq!(u0.border.right, 3.0);
        assert_eq!(u0.border.top, 4.0);
        assert_eq!(u0.parent_texture_guid, "aaaabbbbccccddddeeeeffff00001111");

        // Block_1: unity (30, 6, 28x26) => top_left y = 32 - 6 - 26 = 0.
        let s1 = &sprites[1];
        assert_eq!(s1.sub_rect, PixelRect::new(30, 0, 28, 26));
        assert_eq!(s1.unity.as_ref().unwrap().alignment, 9);
    }

    #[test]
    fn returns_not_multiple_for_single_mode() {
        let dir = temp_dir("single");
        let png_path = dir.join("img.png");
        RgbaImage::from_pixel(8, 8, Rgba([255, 0, 0, 255])).save(&png_path).unwrap();
        let yaml = "fileFormatVersion: 2\nguid: deadbeef\nTextureImporter:\n  spriteMode: 1\n";
        let outcome = parse_meta_string(yaml, &png_path).unwrap();
        assert!(matches!(outcome, MetaParseOutcome::NotMultiple));
    }

    #[test]
    fn rejects_rect_outside_texture() {
        let dir = temp_dir("oob");
        let png_path = dir.join("tiny.png");
        RgbaImage::from_pixel(10, 10, Rgba([0, 0, 0, 0])).save(&png_path).unwrap();
        let yaml = r#"
fileFormatVersion: 2
guid: ffffffffffffffffffffffffffffffff
TextureImporter:
  spriteMode: 2
  spriteSheet:
    sprites:
    - name: TooBig
      rect: {x: 5, y: 5, width: 20, height: 20}
      alignment: 0
      pivot: {x: 0, y: 0}
      border: {x: 0, y: 0, z: 0, w: 0}
      spriteID: ""
      internalID: 0
"#;
        let err = parse_meta_string(yaml, &png_path).unwrap_err();
        assert_eq!(err.code, "atlaspro_meta_rect");
    }
}
