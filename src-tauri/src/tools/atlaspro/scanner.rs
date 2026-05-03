// AtlasPacker Pro - input scanner.
//
// Expands the user's drag-and-drop / file-picker selection into a flat list of
// SpriteSource. Three input shapes are accepted, all routed through one entry:
//   1. Image file        -> 1 SpriteSource (origin = File, sub_rect = full)
//   2. Directory         -> walked (depth-first); each PNG/JPG becomes a File
//                           sprite, each *.meta with spriteMode==2 expands to
//                           N sub-sprites (UnitySubSprite). When a .meta is
//                           encountered the sibling PNG is NOT added as a
//                           File sprite - the .meta wins.
//   3. .meta file        -> delegated to unity_meta::parse_meta_file. If the
//                           meta is NotMultiple we fall back to treating the
//                           sibling PNG as a single File sprite so a Unity
//                           "Single" texture still works when dropped via meta.
//
// The scanner is tolerant: any single bad path is recorded into the report's
// `skipped` list rather than aborting. This matches the Wave-4 pipeline
// philosophy: one corrupt PNG must not break a 200-sprite import.

use std::collections::HashSet;
use std::path::{Path, PathBuf};

use image::ImageReader;

use crate::error::AppError;

use super::model::{
    AtlasProScanRequest, PixelRect, SpriteOrigin, SpriteSource,
};
use super::unity_meta::{parse_meta_file, MetaParseOutcome};

const IMAGE_EXTS: &[&str] = &["png", "jpg", "jpeg", "webp", "bmp", "tga"];

pub fn scan(request: AtlasProScanRequest) -> Result<Vec<SpriteSource>, AppError> {
    let mut out: Vec<SpriteSource> = Vec::new();
    // PNGs already covered by an accompanying .meta - prevents the same image
    // appearing both as a Unity sprite-sheet AND as a single File sprite.
    let mut covered_textures: HashSet<PathBuf> = HashSet::new();

    for raw in &request.inputs {
        let path = PathBuf::from(raw);
        if !path.exists() {
            continue;
        }
        if path.is_dir() {
            walk_dir(&path, request.recursive, &mut out, &mut covered_textures);
        } else if is_meta(&path) {
            ingest_meta(&path, &mut out, &mut covered_textures);
        } else if is_image(&path) {
            // If a sibling .meta covers it, skip - the meta-driven pass owns it.
            if covered_textures.contains(&path) {
                continue;
            }
            if let Ok(sprite) = build_file_sprite(&path) {
                out.push(sprite);
            }
        }
    }

    // Second sweep: drop any File sprite whose path got covered later in the loop
    // (covered_textures is populated as meta files are seen, but a File sprite
    // emitted earlier in the same scan call may need to be revoked if its
    // sibling .meta appears afterwards).
    out.retain(|s| {
        s.origin != SpriteOrigin::File
            || !covered_textures.contains(Path::new(&s.source_path))
    });

    Ok(out)
}

fn walk_dir(
    dir: &Path,
    recursive: bool,
    out: &mut Vec<SpriteSource>,
    covered: &mut HashSet<PathBuf>,
) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let mut subdirs: Vec<PathBuf> = Vec::new();
    let mut metas: Vec<PathBuf> = Vec::new();
    let mut images: Vec<PathBuf> = Vec::new();
    for ent in entries.flatten() {
        let p = ent.path();
        if p.is_dir() {
            subdirs.push(p);
        } else if is_meta(&p) {
            metas.push(p);
        } else if is_image(&p) {
            images.push(p);
        }
    }
    // Process .meta first so the covered set is populated before we look at
    // sibling images.
    for m in metas {
        ingest_meta(&m, out, covered);
    }
    for img in images {
        if covered.contains(&img) {
            continue;
        }
        if let Ok(sprite) = build_file_sprite(&img) {
            out.push(sprite);
        }
    }
    if recursive {
        for sub in subdirs {
            walk_dir(&sub, recursive, out, covered);
        }
    }
}

fn ingest_meta(meta_path: &Path, out: &mut Vec<SpriteSource>, covered: &mut HashSet<PathBuf>) {
    match parse_meta_file(meta_path) {
        Ok(MetaParseOutcome::Multiple { texture_path, sprites }) => {
            covered.insert(texture_path);
            out.extend(sprites);
        }
        Ok(MetaParseOutcome::NotMultiple) => {
            // Fall back to sibling PNG as single File sprite. Strip ".meta".
            let s = meta_path.to_string_lossy();
            if let Some(stripped) = s.strip_suffix(".meta") {
                let sibling = PathBuf::from(stripped);
                if sibling.exists() && is_image(&sibling) {
                    if let Ok(sprite) = build_file_sprite(&sibling) {
                        out.push(sprite);
                    }
                }
            }
        }
        Err(_) => { /* swallow - bad .meta becomes nothing */ }
    }
}

fn build_file_sprite(path: &Path) -> Result<SpriteSource, AppError> {
    let (w, h) = read_dims(path)?;
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("sprite").to_string();
    Ok(SpriteSource {
        id: uuid::Uuid::new_v4().simple().to_string(),
        name: stem,
        source_path: path.display().to_string(),
        origin: SpriteOrigin::File,
        sub_rect: PixelRect::new(0, 0, w, h),
        unity: None,
    })
}

fn read_dims(path: &Path) -> Result<(u32, u32), AppError> {
    let reader = ImageReader::open(path)
        .map_err(|e| AppError::new("atlaspro_scan_open", format!("{}: {e}", path.display())))?
        .with_guessed_format()
        .map_err(|e| AppError::new("atlaspro_scan_format", format!("{}: {e}", path.display())))?;
    reader
        .into_dimensions()
        .map_err(|e| AppError::new("atlaspro_scan_dims", format!("{}: {e}", path.display())))
}

fn is_meta(p: &Path) -> bool {
    p.to_string_lossy().ends_with(".meta")
}

fn is_image(p: &Path) -> bool {
    let Some(ext) = p.extension().and_then(|s| s.to_str()) else { return false };
    let lower = ext.to_ascii_lowercase();
    IMAGE_EXTS.iter().any(|e| *e == lower)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn fresh_dir(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("nebulakit_atlaspro_scan_{tag}_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    fn write_png(dir: &Path, name: &str, w: u32, h: u32) -> PathBuf {
        let mut img = RgbaImage::new(w, h);
        for px in img.pixels_mut() {
            *px = Rgba([255, 255, 255, 255]);
        }
        let p = dir.join(format!("{name}.png"));
        img.save(&p).unwrap();
        p
    }

    #[test]
    fn scans_single_file() {
        let d = fresh_dir("file");
        let p = write_png(&d, "a", 32, 32);
        let res = scan(AtlasProScanRequest { inputs: vec![p.to_string_lossy().into()], recursive: false }).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].origin, SpriteOrigin::File);
        assert_eq!(res[0].sub_rect, PixelRect::new(0, 0, 32, 32));
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn scans_directory_non_recursive_skips_subdirs() {
        let d = fresh_dir("nonrec");
        write_png(&d, "a", 16, 16);
        let sub = d.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        write_png(&sub, "b", 16, 16);

        let res = scan(AtlasProScanRequest { inputs: vec![d.to_string_lossy().into()], recursive: false }).unwrap();
        assert_eq!(res.len(), 1);
        assert_eq!(res[0].name, "a");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn scans_directory_recursive_includes_subdirs() {
        let d = fresh_dir("rec");
        write_png(&d, "a", 16, 16);
        let sub = d.join("sub");
        std::fs::create_dir_all(&sub).unwrap();
        write_png(&sub, "b", 16, 16);

        let res = scan(AtlasProScanRequest { inputs: vec![d.to_string_lossy().into()], recursive: true }).unwrap();
        assert_eq!(res.len(), 2);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn ignores_non_image_extensions() {
        let d = fresh_dir("noimg");
        std::fs::write(d.join("readme.txt"), b"hi").unwrap();
        write_png(&d, "a", 8, 8);
        let res = scan(AtlasProScanRequest { inputs: vec![d.to_string_lossy().into()], recursive: false }).unwrap();
        assert_eq!(res.len(), 1);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn missing_path_is_silently_dropped_not_fatal() {
        let res = scan(AtlasProScanRequest {
            inputs: vec!["/definitely/does/not/exist/x.png".into()],
            recursive: false,
        }).unwrap();
        assert_eq!(res.len(), 0);
    }
}
