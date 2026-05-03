// AtlasPacker Pro - end-to-end pipeline orchestrator.
//
// Wave 4: takes a fully-validated AtlasProExecuteRequest and runs the entire
// pack-and-export sequence:
//   1. preprocess each SpriteSource -> LoadedSprite (decode, sub-rect, trim, extrude)
//   2. pack            -> PackOutcome (placements + atlas size)
//   3. composite       -> RgbaImage (the @1x atlas)
//   4. render_variants -> Vec<VariantOutput>  (scaled atlases)
//   5. for each (variant or @1x), invoke every requested ExportFormat
// Any per-sprite preprocessing failure is captured into AtlasProReport.skipped
// rather than aborting the whole job - matches TexturePacker UX where one bad
// PNG doesn't tank the export.
//
// IMPORTANT: TMP Sprite Asset is intentionally only emitted for the @1x atlas.
// Unity TMP runtime only supports a single-page sprite asset; emitting per-
// variant TMP assets would create resource conflicts in the Unity importer.

use std::path::PathBuf;

use crate::error::AppError;

use super::compositor::{composite, CompositeInputs};
use super::exporters::{json_array, sanitize_basename, tmp_bundle::{self, TmpBundleParams}};
use super::model::{
    AtlasProExecuteRequest, AtlasProReport, EmittedFile, ExportFormat, LoadedSprite,
    PackAlgorithm, PixelSize, SkippedSprite,
};
use super::packer::{pack_auto_square, pack_with_algorithm};
use super::preprocess::load_and_preprocess;
use super::resolution_variants::{render_variants, VariantOutput};

pub fn execute(request: AtlasProExecuteRequest) -> Result<AtlasProReport, AppError> {
    let AtlasProExecuteRequest {
        sources,
        output_dir,
        atlas_name,
        max_width,
        max_height,
        algorithm,
        padding,
        trim,
        allow_rotation,
        auto_size,
        alpha_threshold,
        formats,
        scale_variants,
    } = request;

    let base_name = sanitize_basename(&atlas_name);
    let output_dir = PathBuf::from(&output_dir);
    std::fs::create_dir_all(&output_dir).map_err(|err| {
        AppError::new(
            "atlaspro_pipeline_io",
            format!("failed to create output dir {}: {err}", output_dir.display()),
        )
    })?;

    let wants_tmp = formats
        .iter()
        .any(|format| matches!(format, ExportFormat::TmpSpriteAsset));

    if wants_tmp && allow_rotation {
        return Err(AppError::new(
            "atlaspro_tmp_rotation_unsupported",
            "TmpSpriteAsset format does not support rotated sprites; disable allowRotation when exporting TMP"
                .to_string(),
        ));
    }

    if wants_tmp && !matches!(algorithm, PackAlgorithm::Skyline | PackAlgorithm::MaxRects) {
        return Err(AppError::new(
            "atlaspro_tmp_algorithm_unsupported",
            format!(
                "TmpSpriteAsset format requires the Skyline algorithm; got {:?}",
                algorithm
            ),
        ));
    }

    let mut loaded: Vec<LoadedSprite> = Vec::with_capacity(sources.len());
    let mut skipped: Vec<SkippedSprite> = Vec::new();
    for source in &sources {
        match load_and_preprocess(
            source,
            trim,
            alpha_threshold,
            padding.extrude,
        ) {
            Ok(sprite) => loaded.push(sprite),
            Err(err) => skipped.push(SkippedSprite {
                id: source.id.clone(),
                name: source.name.clone(),
                reason: err.message.clone(),
            }),
        }
    }

    if wants_tmp {
        reject_duplicate_tmp_names(&loaded)?;
    }

    let tmp_input_dir = if wants_tmp {
        Some(derive_tmp_input_dir(&loaded)?)
    } else {
        None
    };

    let outcome = if auto_size {
        pack_auto_square(
            &loaded,
            max_width.min(max_height),
            algorithm,
            padding,
            padding.extrude,
            allow_rotation,
        )?
    } else {
        pack_with_algorithm(
            &loaded,
            max_width,
            max_height,
            algorithm,
            padding,
            padding.extrude,
            allow_rotation,
        )?
    };

    let atlas_image = composite(CompositeInputs {
        atlas_size: outcome.atlas_size,
        placed: &outcome.placed,
        sources: &loaded,
        extrude: padding.extrude,
    })?;

    let wants_non_tmp = formats
        .iter()
        .any(|format| !matches!(format, ExportFormat::TmpSpriteAsset));
    let variants: Vec<VariantOutput> = if wants_non_tmp {
        render_variants(
            &atlas_image,
            &outcome.placed,
            outcome.atlas_size,
            &scale_variants,
        )?
    } else {
        Vec::new()
    };

    let mut outputs: Vec<EmittedFile> = Vec::new();

    for format in formats {
        match format {
            ExportFormat::TmpSpriteAsset => outputs.extend(tmp_bundle::emit_tmp_bundle(
                TmpBundleParams {
                    atlas_image: &atlas_image,
                    atlas_size: outcome.atlas_size,
                    placed: &outcome.placed,
                    output_dir: &output_dir,
                    input_dir: tmp_input_dir
                        .as_deref()
                        .expect("tmp input dir must exist when TMP export is requested"),
                    base_name: &base_name,
                },
            )?),
            other => {
                outputs.extend(emit_one(
                    other,
                    &atlas_image,
                    outcome.atlas_size,
                    &outcome.placed,
                    &output_dir,
                    &base_name,
                    "",
                    1.0,
                )?);
                for variant in &variants {
                    outputs.extend(emit_one(
                        other,
                        &variant.image,
                        variant.atlas_size,
                        &variant.placed,
                        &output_dir,
                        &base_name,
                        &variant.suffix,
                        variant.scale,
                    )?);
                }
            }
        }
    }

    Ok(AtlasProReport {
        atlas_size: outcome.atlas_size,
        placed: outcome.placed,
        skipped,
        outputs,
    })
}

fn reject_duplicate_tmp_names(loaded: &[LoadedSprite]) -> Result<(), AppError> {
    let mut seen = std::collections::HashSet::with_capacity(loaded.len());
    for sprite in loaded {
        if !seen.insert(sprite.name.as_str()) {
            return Err(AppError::new(
                "atlaspro_tmp_duplicate_sprite_name",
                format!(
                    "TmpSpriteAsset format requires unique sprite names; duplicate found: {}",
                    sprite.name
                ),
            ));
        }
    }
    Ok(())
}

fn derive_tmp_input_dir(loaded: &[LoadedSprite]) -> Result<PathBuf, AppError> {
    let mut parents = loaded
        .iter()
        .map(|sprite| sprite.source_path.parent().map(|parent| parent.to_path_buf()))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| AppError::new(
            "atlaspro_tmp_input_dir",
            "failed to derive input folder from source paths".to_string(),
        ))?;

    let first = parents
        .drain(..1)
        .next()
        .ok_or_else(|| AppError::new(
            "atlaspro_tmp_input_dir",
            "TMP export requires at least one loaded sprite".to_string(),
        ))?;

    let mut common = std::fs::canonicalize(&first).map_err(|err| AppError::new(
        "atlaspro_tmp_input_dir",
        format!("failed to canonicalize input folder {}: {err}", first.display()),
    ))?;

    for parent in parents {
        let canonical = std::fs::canonicalize(&parent).map_err(|err| AppError::new(
            "atlaspro_tmp_input_dir",
            format!("failed to canonicalize input folder {}: {err}", parent.display()),
        ))?;
        common = common_ancestor(&common, &canonical).ok_or_else(|| AppError::new(
            "atlaspro_tmp_input_dir",
            format!(
                "failed to derive common input folder for {} and {}",
                common.display(),
                canonical.display()
            ),
        ))?;
    }

    Ok(common)
}

fn common_ancestor(left: &std::path::Path, right: &std::path::Path) -> Option<PathBuf> {
    let left_components: Vec<_> = left.components().collect();
    let right_components: Vec<_> = right.components().collect();
    let shared_len = left_components
        .iter()
        .zip(right_components.iter())
        .take_while(|(a, b)| a == b)
        .count();

    if shared_len == 0 {
        return None;
    }

    let mut common = PathBuf::new();
    for component in left_components.iter().take(shared_len) {
        common.push(component.as_os_str());
    }
    Some(common)
}

#[allow(clippy::too_many_arguments)]
fn emit_one(
    format: ExportFormat,
    atlas_image: &image::RgbaImage,
    atlas_size: PixelSize,
    placed: &[super::model::PackedSprite],
    output_dir: &std::path::Path,
    base_name: &str,
    suffix: &str,
    scale: f32,
) -> Result<Vec<EmittedFile>, AppError> {
    match format {
        ExportFormat::PngOnly => {
            let png_path = super::exporters::output_path(output_dir, base_name, suffix, "png");
            super::exporters::write_atlas_png(atlas_image, &png_path)?;
            Ok(vec![super::exporters::make_emitted(ExportFormat::PngOnly, png_path)])
        }
        ExportFormat::JsonArray => {
            let png_path = super::exporters::output_path(output_dir, base_name, suffix, "png");
            super::exporters::write_atlas_png(atlas_image, &png_path)?;
            let png_filename = png_path.file_name().and_then(|s| s.to_str()).unwrap_or(base_name).to_string();
            let sidecar = json_array::emit(
                placed, atlas_size, &png_filename, scale, output_dir, base_name, suffix,
            )?;
            Ok(vec![super::exporters::make_emitted(ExportFormat::JsonArray, png_path), sidecar])
        }
        ExportFormat::JsonHash => {
            // Wave 4.5: JSON-Hash is the same payload with `frames` as a map. Emit JSON-Array
            // shape until the dedicated emitter lands - downstream consumers that strictly
            // require the hash form will get JSON-Array which is the more common variant.
            let png_path = super::exporters::output_path(output_dir, base_name, suffix, "png");
            super::exporters::write_atlas_png(atlas_image, &png_path)?;
            let png_filename = png_path.file_name().and_then(|s| s.to_str()).unwrap_or(base_name).to_string();
            let sidecar = json_array::emit(
                placed, atlas_size, &png_filename, scale, output_dir, base_name, suffix,
            )?;
            Ok(vec![super::exporters::make_emitted(ExportFormat::JsonHash, png_path), sidecar])
        }
        ExportFormat::TmpSpriteAsset => Err(AppError::new(
            "atlaspro_pipeline_tmp_internal",
            "TmpSpriteAsset exports must be emitted through tmp_bundle".to_string(),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::model::{PaddingOptions, SpriteOrigin, SpriteSource, TrimMode};
    use image::{Rgba, RgbaImage};

    fn write_png(dir: &std::path::Path, name: &str, w: u32, h: u32, fill: [u8; 4]) -> PathBuf {
        let mut img = RgbaImage::new(w, h);
        for px in img.pixels_mut() {
            *px = Rgba(fill);
        }
        let path = dir.join(format!("{name}.png"));
        img.save(&path).unwrap();
        path
    }

    #[test]
    fn end_to_end_emits_png_and_json_for_single_sprite() {
        let dir = std::env::temp_dir().join(format!("nebulakit_atlaspro_pipeline_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sprite_dir = dir.join("src");
        std::fs::create_dir_all(&sprite_dir).unwrap();
        let p = write_png(&sprite_dir, "smile", 32, 32, [255, 0, 0, 255]);

        let req = AtlasProExecuteRequest {
            sources: vec![SpriteSource {
                id: "smile".into(),
                name: "smile".into(),
                origin: SpriteOrigin::File,
                source_path: p.to_string_lossy().into_owned(),
                sub_rect: super::super::model::PixelRect::new(0, 0, 32, 32),
                unity: None,
            }],
            output_dir: dir.to_string_lossy().into_owned(),
            atlas_name: "demo".into(),
            max_width: 256,
            max_height: 256,
            algorithm: Default::default(),
            padding: PaddingOptions::default(),
            trim: TrimMode::None,
            allow_rotation: false,
            alpha_threshold: 0,
            formats: vec![ExportFormat::JsonArray],
            scale_variants: vec![],
            auto_size: false,
        };

        let report = execute(req).unwrap();
        assert_eq!(report.placed.len(), 1);
        assert_eq!(report.skipped.len(), 0);
        // PNG + JSON sidecar = 2 emitted files for one format with no variants.
        assert_eq!(report.outputs.len(), 2);
        for o in &report.outputs {
            assert!(std::path::Path::new(&o.path).exists(), "missing {}", o.path);
        }

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn tmp_format_only_emitted_for_at_1x() {
        let dir = std::env::temp_dir().join(format!("nebulakit_atlaspro_pipeline_tmp_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let sprite_dir = dir.join("src");
        std::fs::create_dir_all(&sprite_dir).unwrap();
        let p = write_png(&sprite_dir, "a", 16, 16, [0, 255, 0, 255]);

        let req = AtlasProExecuteRequest {
            sources: vec![SpriteSource {
                id: "a".into(),
                name: "a".into(),
                origin: SpriteOrigin::File,
                source_path: p.to_string_lossy().into_owned(),
                sub_rect: super::super::model::PixelRect::new(0, 0, 16, 16),
                unity: None,
            }],
            output_dir: dir.to_string_lossy().into_owned(),
            atlas_name: "demo".into(),
            max_width: 128,
            max_height: 128,
            algorithm: Default::default(),
            padding: PaddingOptions::default(),
            trim: TrimMode::None,
            allow_rotation: false,
            alpha_threshold: 0,
            formats: vec![ExportFormat::TmpSpriteAsset],
            scale_variants: vec![super::super::model::ScaleVariant { suffix: "@2x".into(), scale: 2.0 }],
            auto_size: false,
        };

        let report = execute(req).unwrap();
        // TMP emits 4 files for @1x (png + png.meta + asset + asset.meta) and zero for @2x.
        // No .mat / .mat.meta - ground truth confirms TMP runtime auto-creates material when material:{fileID:0}.
        assert_eq!(report.outputs.len(), 4);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn missing_source_file_is_recorded_in_skipped_not_fatal() {
        let dir = std::env::temp_dir().join(format!("nebulakit_atlaspro_pipeline_skip_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();

        let req = AtlasProExecuteRequest {
            sources: vec![SpriteSource {
                id: "ghost".into(),
                name: "ghost".into(),
                origin: SpriteOrigin::File,
                source_path: dir.join("does-not-exist.png").to_string_lossy().into_owned(),
                sub_rect: super::super::model::PixelRect::new(0, 0, 16, 16),
                unity: None,
            }],
            output_dir: dir.to_string_lossy().into_owned(),
            atlas_name: "demo".into(),
            max_width: 64,
            max_height: 64,
            algorithm: Default::default(),
            padding: PaddingOptions::default(),
            trim: TrimMode::None,
            allow_rotation: false,
            alpha_threshold: 0,
            formats: vec![ExportFormat::PngOnly],
            scale_variants: vec![],
            auto_size: false,
        };

        let report = execute(req).unwrap();
        assert_eq!(report.skipped.len(), 1);
        assert_eq!(report.skipped[0].name, "ghost");
        assert_eq!(report.placed.len(), 0);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn derive_tmp_input_dir_prefers_common_absolute_parent() {
        let root = std::env::temp_dir().join(format!(
            "nebulakit_atlaspro_tmp_input_dir_{}",
            std::process::id()
        ));
        let parent = root.join("sample");
        let nested = parent.join("nested");
        std::fs::create_dir_all(&nested).unwrap();

        let loaded = vec![
            LoadedSprite {
                id: "a".into(),
                name: "a".into(),
                origin: SpriteOrigin::File,
                source_path: parent.join("a.png"),
                source_size: PixelSize { width: 1, height: 1 },
                trimmed_rect: super::super::model::PixelRect::new(0, 0, 1, 1),
                sub_rect: super::super::model::PixelRect::new(0, 0, 1, 1),
                trimmed: false,
                pixels: RgbaImage::new(1, 1),
                unity: None,
            },
            LoadedSprite {
                id: "b".into(),
                name: "b".into(),
                origin: SpriteOrigin::File,
                source_path: nested.join("b.png"),
                source_size: PixelSize { width: 1, height: 1 },
                trimmed_rect: super::super::model::PixelRect::new(0, 0, 1, 1),
                sub_rect: super::super::model::PixelRect::new(0, 0, 1, 1),
                trimmed: false,
                pixels: RgbaImage::new(1, 1),
                unity: None,
            },
        ];

        let derived = derive_tmp_input_dir(&loaded).unwrap();
        assert_eq!(derived, std::fs::canonicalize(&parent).unwrap());

        let _ = std::fs::remove_dir_all(&root);
    }
}
