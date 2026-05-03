use std::path::Path;

use image::RgbaImage;

use crate::error::AppError;
use crate::tools::atlaspro::model::{EmittedFile, ExportFormat, PackedSprite, PixelSize};

const UNITY_MAX_TEXTURE_SIZE: u32 = 16384;

pub struct TmpBundleParams<'a> {
    pub atlas_image: &'a RgbaImage,
    pub atlas_size: PixelSize,
    pub placed: &'a [PackedSprite],
    pub output_dir: &'a Path,
    pub input_dir: &'a Path,
    pub base_name: &'a str,
}

pub fn emit_tmp_bundle(params: TmpBundleParams<'_>) -> Result<Vec<EmittedFile>, AppError> {
    use super::{asset_meta, png_meta, tmp_sprite_asset};
    use super::{
        atomic_write, deterministic_guid, make_emitted, namespaced_guid_seed, output_path,
        write_atlas_png,
    };
    use crate::tools::atlaspro::sub_sprite_id;

    let max_dim = params.atlas_size.width.max(params.atlas_size.height);
    if max_dim > UNITY_MAX_TEXTURE_SIZE {
        return Err(AppError::new(
            "atlaspro_tmp_texture_size_exceeded",
            format!(
                "TMP atlas {}x{} exceeds Unity's maxTextureSize ({}). Unity would silently downscale the PNG while TMP glyph rects would still reference original coordinates, breaking every sprite. Reduce max width/height to <= {}.",
                params.atlas_size.width,
                params.atlas_size.height,
                UNITY_MAX_TEXTURE_SIZE,
                UNITY_MAX_TEXTURE_SIZE
            ),
        ));
    }

    let identities = sub_sprite_id::identities_for_names(
        params.placed.iter().map(|packed| packed.name.as_str()),
    );

    let png_guid = deterministic_guid(&namespaced_guid_seed(
        "nebulakit:atlaspro:tmp:png",
        params.input_dir,
        params.base_name,
    )?);
    let asset_guid = deterministic_guid(&namespaced_guid_seed(
        "nebulakit:atlaspro:asset",
        params.input_dir,
        params.base_name,
    )?);

    let png_path = output_path(params.output_dir, params.base_name, "", "png");
    let png_meta_path = output_path(params.output_dir, params.base_name, "", "png.meta");
    let asset_path = output_path(params.output_dir, params.base_name, "", "asset");
    let asset_meta_path = output_path(params.output_dir, params.base_name, "", "asset.meta");

    write_atlas_png(params.atlas_image, &png_path)?;

    let png_meta_doc = png_meta::build_png_meta_doc_from_packed_sprites(
        &png_guid,
        params.atlas_size,
        params.placed,
        &identities,
    );
    let png_meta_bytes = png_meta::emit_png_meta_doc(&png_meta_doc);
    atomic_write(&png_meta_path, &png_meta_bytes)?;

    let asset_doc = tmp_sprite_asset::build_tmp_sprite_asset_doc(
        params.base_name,
        &png_guid,
        &asset_guid,
        &identities,
        params.atlas_size,
        params.placed,
    );
    let asset_bytes = tmp_sprite_asset::emit_tmp_sprite_asset_doc(&asset_doc);
    atomic_write(&asset_path, &asset_bytes)?;

    let asset_meta_doc = asset_meta::build_asset_meta_doc(&asset_guid);
    let asset_meta_bytes = asset_meta::emit_asset_meta_doc(&asset_meta_doc);
    atomic_write(&asset_meta_path, &asset_meta_bytes)?;

    Ok(vec![
        make_emitted(ExportFormat::TmpSpriteAsset, png_path),
        make_emitted(ExportFormat::TmpSpriteAsset, png_meta_path),
        make_emitted(ExportFormat::TmpSpriteAsset, asset_path),
        make_emitted(ExportFormat::TmpSpriteAsset, asset_meta_path),
    ])
}
