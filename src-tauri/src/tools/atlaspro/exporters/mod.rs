// AtlasPacker Pro - exporters shared infrastructure.
//
// Wave 4: format-specific exporters (JSON-Array, TMP Sprite Asset, ...) live
// in sibling modules and all share these helpers for PNG writing, filename
// sanitization, and deterministic Unity GUID generation. Keeping these here
// prevents each exporter from rolling its own slightly-different version.

pub mod asset_meta;
pub mod json_array;
pub mod png_meta;
pub mod tmp_bundle;
pub mod tmp_sprite_asset;

use std::fs;
use std::io::Cursor;
use std::path::{Path, PathBuf};

use image::RgbaImage;
use sha2::{Digest, Sha256};

use crate::error::AppError;

use super::model::EmittedFile;

/// Write an RGBA atlas image as a PNG. Used by every exporter that emits
/// pixel data; centralized so codec choice and IO error mapping are uniform.
pub fn write_atlas_png(image: &RgbaImage, path: &Path) -> Result<(), AppError> {
    let mut bytes = Cursor::new(Vec::new());
    image::DynamicImage::ImageRgba8(image.clone())
        .write_to(&mut bytes, image::ImageFormat::Png)
        .map_err(|err| AppError::new(
            "atlaspro_export_png",
            format!("failed to encode PNG {}: {err}", path.display()),
        ))?;
    atomic_write(path, bytes.get_ref())
}

/// Sanitize a user-supplied atlas name into a filesystem-safe basename. Strips
/// path separators and characters that Windows/macOS reject so users can paste
/// arbitrary input from the UI without breaking the export. Empty/whitespace
/// names fall back to "atlas".
pub fn sanitize_basename(raw: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return "atlas".to_string();
    }
    let cleaned: String = trimmed
        .chars()
        .map(|c| match c {
            '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' | '\0' => '_',
            c if c.is_control() => '_',
            c => c,
        })
        .collect();
    if cleaned.trim_matches('_').is_empty() {
        "atlas".to_string()
    } else {
        cleaned
    }
}

/// Build an output path: `<dir>/<base><suffix>.<ext>`. `suffix` is empty for
/// the @1x atlas and e.g. "@2x" for resolution variants.
pub fn output_path(dir: &Path, base: &str, suffix: &str, ext: &str) -> PathBuf {
    let mut name = String::with_capacity(base.len() + suffix.len() + ext.len() + 1);
    name.push_str(base);
    name.push_str(suffix);
    name.push('.');
    name.push_str(ext);
    dir.join(name)
}

/// Generate a deterministic 32-char lowercase hex Unity GUID from a seed string.
/// Uses MD5 because Unity's own GUIDs are 128 bits expressed as 32 hex chars,
/// so MD5 produces a drop-in compatible value while staying deterministic
/// across runs (important for re-imports - random GUIDs would force Unity to
/// re-link every reference each export).
pub fn deterministic_guid(seed: &str) -> String {
    let digest = md5::compute(seed.as_bytes());
    format!("{:x}", digest)
}

pub fn input_path_hash16(path: &Path) -> Result<String, AppError> {
    let canonical = fs::canonicalize(path).map_err(|err| AppError::new(
        "atlaspro_export_io",
        format!("failed to canonicalize input path {}: {err}", path.display()),
    ))?;
    let digest = Sha256::digest(canonical.to_string_lossy().as_bytes());
    Ok(format!("{:x}", digest)[..16].to_string())
}

pub fn namespaced_guid_seed(prefix: &str, input_path: &Path, base_name: &str) -> Result<String, AppError> {
    Ok(format!(
        "{prefix}:{}:{base_name}",
        input_path_hash16(input_path)?
    ))
}

pub fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), AppError> {
    atomic_write_with(path, bytes, |temp_path, body| fs::write(temp_path, body))
}

fn atomic_write_with<F>(path: &Path, bytes: &[u8], mut writer: F) -> Result<(), AppError>
where
    F: FnMut(&Path, &[u8]) -> std::io::Result<()>,
{
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent).map_err(|err| AppError::new(
                "atlaspro_export_io",
                format!("failed to create directory {}: {err}", parent.display()),
            ))?;
        }
    }

    let temp_path = PathBuf::from(format!("{}.tmp", path.display()));
    if temp_path.exists() {
        fs::remove_file(&temp_path).map_err(|err| AppError::new(
            "atlaspro_export_io",
            format!("failed to clear temp file {}: {err}", temp_path.display()),
        ))?;
    }

    if let Err(err) = writer(&temp_path, bytes) {
        let _ = fs::remove_file(&temp_path);
        return Err(AppError::new(
            "atlaspro_export_io",
            format!("failed to write temp file for {}: {err}", path.display()),
        ));
    }

    if path.exists() {
        fs::remove_file(path).map_err(|err| AppError::new(
            "atlaspro_export_io",
            format!("failed to replace existing file {}: {err}", path.display()),
        ))?;
    }

    fs::rename(&temp_path, path).map_err(|err| {
        let _ = fs::remove_file(&temp_path);
        AppError::new(
            "atlaspro_export_io",
            format!("failed to atomically replace {}: {err}", path.display()),
        )
    })?;

    Ok(())
}

pub fn make_emitted(format: super::model::ExportFormat, path: PathBuf) -> EmittedFile {
    EmittedFile {
        format,
        path: path.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_replaces_path_separators() {
        assert_eq!(sanitize_basename("foo/bar"), "foo_bar");
        assert_eq!(sanitize_basename("a\\b:c*d?e\"f<g>h|i"), "a_b_c_d_e_f_g_h_i");
    }

    #[test]
    fn sanitize_preserves_safe_chars() {
        assert_eq!(sanitize_basename("emoji_pack-01"), "emoji_pack-01");
        assert_eq!(sanitize_basename("中文图集"), "中文图集");
    }

    #[test]
    fn sanitize_falls_back_for_empty_or_only_garbage() {
        assert_eq!(sanitize_basename(""), "atlas");
        assert_eq!(sanitize_basename("   "), "atlas");
        assert_eq!(sanitize_basename("///"), "atlas");
    }

    #[test]
    fn output_path_assembles_correctly() {
        let p = output_path(Path::new("/tmp/out"), "demo", "@2x", "png");
        assert_eq!(p, PathBuf::from("/tmp/out/demo@2x.png"));
        let p = output_path(Path::new("/tmp/out"), "demo", "", "asset");
        assert_eq!(p, PathBuf::from("/tmp/out/demo.asset"));
    }

    #[test]
    fn deterministic_guid_is_stable_and_hex() {
        let g1 = deterministic_guid("foo");
        let g2 = deterministic_guid("foo");
        let g3 = deterministic_guid("bar");
        assert_eq!(g1, g2);
        assert_ne!(g1, g3);
        assert_eq!(g1.len(), 32);
        assert!(g1.chars().all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase()));
    }

    #[test]
    fn namespaced_guid_seed_is_idempotent_and_input_path_scoped() {
        let root = std::env::temp_dir().join(format!(
            "nebulakit_exporter_guid_seed_{}",
            std::process::id()
        ));
        let dir_a = root.join("folder_a");
        let dir_b = root.join("folder_b");
        fs::create_dir_all(&dir_a).unwrap();
        fs::create_dir_all(&dir_b).unwrap();

        let guid_a1 = deterministic_guid(&namespaced_guid_seed(
            "nebulakit:atlaspro:asset",
            &dir_a,
            "atlas",
        ).unwrap());
        let guid_a2 = deterministic_guid(&namespaced_guid_seed(
            "nebulakit:atlaspro:asset",
            &dir_a,
            "atlas",
        ).unwrap());
        let guid_b = deterministic_guid(&namespaced_guid_seed(
            "nebulakit:atlaspro:asset",
            &dir_b,
            "atlas",
        ).unwrap());

        assert_eq!(guid_a1, guid_a2, "same input dir + same atlas name must be idempotent");
        assert_ne!(guid_a1, guid_b, "different input dirs must not collide");

        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn atomic_write_cleans_up_failed_temp_file_without_touching_final_path() {
        let root = std::env::temp_dir().join(format!(
            "nebulakit_exporter_atomic_write_{}",
            std::process::id()
        ));
        fs::create_dir_all(&root).unwrap();
        let final_path = root.join("atlas.asset");
        let temp_path = PathBuf::from(format!("{}.tmp", final_path.display()));

        let result = atomic_write_with(&final_path, b"hello", |temp_path, body| {
            fs::write(temp_path, &body[..2])?;
            Err(std::io::Error::other("simulated failure"))
        });

        let err = result.expect_err("atomic_write should surface simulated write failure");
        assert_eq!(err.code, "atlaspro_export_io");
        assert!(!final_path.exists(), "final path must not appear after failed temp write");
        assert!(!temp_path.exists(), "temp file must be removed after failed temp write");

        let _ = fs::remove_dir_all(&root);
    }
}
