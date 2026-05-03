use std::fmt::Write as _;

use super::super::model::{PackedSprite, PixelSize};
use crate::tools::atlaspro::sub_sprite_id::SubSpriteIdentity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PngMetaDoc {
    pub file_format_version: u32,
    pub guid: String,
    pub texture_importer: TextureImporterBlock,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureImporterBlock {
    pub internal_id_to_name_table_empty: bool,
    pub external_objects_empty: bool,
    pub serialized_version: u32,
    pub mipmaps: MipmapsBlock,
    pub bumpmap: BumpmapBlock,
    pub is_readable: u32,
    pub streaming_mipmaps: u32,
    pub streaming_mipmaps_priority: u32,
    pub v_t_only: u32,
    pub ignore_mipmap_limit: u32,
    pub gray_scale_to_alpha: u32,
    pub generate_cubemap: u32,
    pub cubemap_convolution: u32,
    pub seamless_cubemap: u32,
    pub texture_format: i64,
    pub max_texture_size: u32,
    pub texture_settings: TextureSettingsBlock,
    pub n_pot_scale: u32,
    pub lightmap: u32,
    pub compression_quality: u32,
    pub sprite_mode: u32,
    pub sprite_extrude: u32,
    pub sprite_mesh_type: u32,
    pub alignment: u32,
    pub sprite_pivot_x: String,
    pub sprite_pivot_y: String,
    pub sprite_pixels_to_units: u32,
    pub sprite_border_x: String,
    pub sprite_border_y: String,
    pub sprite_border_z: String,
    pub sprite_border_w: String,
    pub sprite_generate_fallback_physics_shape: u32,
    pub alpha_usage: u32,
    pub alpha_is_transparency: u32,
    pub sprite_tessellation_detail: String,
    pub texture_type: u32,
    pub texture_shape: u32,
    pub single_channel_component: u32,
    pub flipbook_rows: u32,
    pub flipbook_columns: u32,
    pub max_texture_size_set: u32,
    pub compression_quality_set: u32,
    pub texture_format_set: u32,
    pub ignore_png_gamma: u32,
    pub apply_gamma_decoding: u32,
    pub swizzle: u32,
    pub cookie_light_type: u32,
    pub platform_settings: Vec<PlatformSetting>,
    pub sprite_sheet: SpriteSheetBlock,
    pub mipmap_limit_group_name: String,
    pub p_sd_remove_matte: u32,
    pub user_data: String,
    pub asset_bundle_name: String,
    pub asset_bundle_variant: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MipmapsBlock {
    pub mip_map_mode: u32,
    pub enable_mip_map: u32,
    pub s_rgb_texture: u32,
    pub linear_texture: u32,
    pub fade_out: u32,
    pub border_mip_map: u32,
    pub mip_maps_preserve_coverage: u32,
    pub alpha_test_reference_value: String,
    pub mip_map_fade_distance_start: u32,
    pub mip_map_fade_distance_end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BumpmapBlock {
    pub convert_to_normal_map: u32,
    pub external_normal_map: u32,
    pub height_scale: String,
    pub normal_map_filter: u32,
    pub flip_green_channel: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextureSettingsBlock {
    pub serialized_version: u32,
    pub filter_mode: u32,
    pub aniso: u32,
    pub mip_bias: String,
    pub wrap_u: u32,
    pub wrap_v: u32,
    pub wrap_w: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlatformSetting {
    pub serialized_version: u32,
    pub build_target: String,
    pub max_texture_size: u32,
    pub resize_algorithm: u32,
    pub texture_format: i64,
    pub texture_compression: u32,
    pub compression_quality: u32,
    pub crunched_compression: u32,
    pub allows_alpha_splitting: u32,
    pub overridden: u32,
    pub ignore_platform_support: u32,
    pub android_etc2_fallback_override: u32,
    pub force_maximum_compression_quality_bc6h_bc7: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteSheetBlock {
    pub serialized_version: u32,
    pub sprites: Vec<SpriteSheetEntry>,
    pub trailing_outline_empty: bool,
    pub trailing_physics_shape_empty: bool,
    pub trailing_bones_empty: bool,
    pub trailing_sprite_id: String,
    pub trailing_internal_id: i64,
    pub trailing_vertices_empty: bool,
    pub trailing_indices_blank: bool,
    pub trailing_edges_empty: bool,
    pub trailing_weights_empty: bool,
    pub secondary_textures_empty: bool,
    pub name_file_id_table: Vec<(String, i64)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpriteSheetEntry {
    pub serialized_version: u32,
    pub name: String,
    pub rect: Rect2D,
    pub alignment: u32,
    pub pivot_x: String,
    pub pivot_y: String,
    pub border_x: String,
    pub border_y: String,
    pub border_z: String,
    pub border_w: String,
    pub outline_empty: bool,
    pub physics_shape_empty: bool,
    pub tessellation_detail: String,
    pub bones_empty: bool,
    pub sprite_id: String,
    pub internal_id: i64,
    pub vertices_empty: bool,
    pub indices_blank: bool,
    pub edges_empty: bool,
    pub weights_empty: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rect2D {
    pub serialized_version: u32,
    pub x: i64,
    pub y: i64,
    pub width: i64,
    pub height: i64,
}

pub fn render_png_meta_doc(doc: &PngMetaDoc) -> String {
    let mut out = String::with_capacity(12_288);
    writeln!(&mut out, "fileFormatVersion: {}", doc.file_format_version).unwrap();
    writeln!(&mut out, "guid: {}", doc.guid).unwrap();
    out.push_str("TextureImporter:\n");

    let importer = &doc.texture_importer;
    out.push_str(if importer.internal_id_to_name_table_empty {
        "  internalIDToNameTable: []\n"
    } else {
        panic!("non-empty internalIDToNameTable is not supported in T9 emitter")
    });
    out.push_str(if importer.external_objects_empty {
        "  externalObjects: {}\n"
    } else {
        panic!("non-empty externalObjects is not supported in T9 emitter")
    });
    writeln!(&mut out, "  serializedVersion: {}", importer.serialized_version).unwrap();

    emit_mipmaps(&mut out, &importer.mipmaps);
    emit_bumpmap(&mut out, &importer.bumpmap);

    writeln!(&mut out, "  isReadable: {}", importer.is_readable).unwrap();
    writeln!(&mut out, "  streamingMipmaps: {}", importer.streaming_mipmaps).unwrap();
    writeln!(
        &mut out,
        "  streamingMipmapsPriority: {}",
        importer.streaming_mipmaps_priority
    )
    .unwrap();
    writeln!(&mut out, "  vTOnly: {}", importer.v_t_only).unwrap();
    writeln!(&mut out, "  ignoreMipmapLimit: {}", importer.ignore_mipmap_limit).unwrap();
    writeln!(&mut out, "  grayScaleToAlpha: {}", importer.gray_scale_to_alpha).unwrap();
    writeln!(&mut out, "  generateCubemap: {}", importer.generate_cubemap).unwrap();
    writeln!(&mut out, "  cubemapConvolution: {}", importer.cubemap_convolution).unwrap();
    writeln!(&mut out, "  seamlessCubemap: {}", importer.seamless_cubemap).unwrap();
    writeln!(&mut out, "  textureFormat: {}", importer.texture_format).unwrap();
    writeln!(&mut out, "  maxTextureSize: {}", importer.max_texture_size).unwrap();
    emit_texture_settings(&mut out, &importer.texture_settings);
    writeln!(&mut out, "  nPOTScale: {}", importer.n_pot_scale).unwrap();
    writeln!(&mut out, "  lightmap: {}", importer.lightmap).unwrap();
    writeln!(&mut out, "  compressionQuality: {}", importer.compression_quality).unwrap();
    writeln!(&mut out, "  spriteMode: {}", importer.sprite_mode).unwrap();
    writeln!(&mut out, "  spriteExtrude: {}", importer.sprite_extrude).unwrap();
    writeln!(&mut out, "  spriteMeshType: {}", importer.sprite_mesh_type).unwrap();
    writeln!(&mut out, "  alignment: {}", importer.alignment).unwrap();
    writeln!(
        &mut out,
        "  spritePivot: {{x: {}, y: {}}}",
        importer.sprite_pivot_x, importer.sprite_pivot_y
    )
    .unwrap();
    writeln!(&mut out, "  spritePixelsToUnits: {}", importer.sprite_pixels_to_units).unwrap();
    writeln!(
        &mut out,
        "  spriteBorder: {{x: {}, y: {}, z: {}, w: {}}}",
        importer.sprite_border_x,
        importer.sprite_border_y,
        importer.sprite_border_z,
        importer.sprite_border_w
    )
    .unwrap();
    writeln!(
        &mut out,
        "  spriteGenerateFallbackPhysicsShape: {}",
        importer.sprite_generate_fallback_physics_shape
    )
    .unwrap();
    writeln!(&mut out, "  alphaUsage: {}", importer.alpha_usage).unwrap();
    writeln!(&mut out, "  alphaIsTransparency: {}", importer.alpha_is_transparency).unwrap();
    writeln!(
        &mut out,
        "  spriteTessellationDetail: {}",
        importer.sprite_tessellation_detail
    )
    .unwrap();
    writeln!(&mut out, "  textureType: {}", importer.texture_type).unwrap();
    writeln!(&mut out, "  textureShape: {}", importer.texture_shape).unwrap();
    writeln!(
        &mut out,
        "  singleChannelComponent: {}",
        importer.single_channel_component
    )
    .unwrap();
    writeln!(&mut out, "  flipbookRows: {}", importer.flipbook_rows).unwrap();
    writeln!(&mut out, "  flipbookColumns: {}", importer.flipbook_columns).unwrap();
    writeln!(&mut out, "  maxTextureSizeSet: {}", importer.max_texture_size_set).unwrap();
    writeln!(
        &mut out,
        "  compressionQualitySet: {}",
        importer.compression_quality_set
    )
    .unwrap();
    writeln!(&mut out, "  textureFormatSet: {}", importer.texture_format_set).unwrap();
    writeln!(&mut out, "  ignorePngGamma: {}", importer.ignore_png_gamma).unwrap();
    writeln!(&mut out, "  applyGammaDecoding: {}", importer.apply_gamma_decoding).unwrap();
    writeln!(&mut out, "  swizzle: {}", importer.swizzle).unwrap();
    writeln!(&mut out, "  cookieLightType: {}", importer.cookie_light_type).unwrap();

    out.push_str("  platformSettings:\n");
    for setting in &importer.platform_settings {
        writeln!(&mut out, "  - serializedVersion: {}", setting.serialized_version).unwrap();
        writeln!(&mut out, "    buildTarget: {}", setting.build_target).unwrap();
        writeln!(&mut out, "    maxTextureSize: {}", setting.max_texture_size).unwrap();
        writeln!(&mut out, "    resizeAlgorithm: {}", setting.resize_algorithm).unwrap();
        writeln!(&mut out, "    textureFormat: {}", setting.texture_format).unwrap();
        writeln!(&mut out, "    textureCompression: {}", setting.texture_compression).unwrap();
        writeln!(&mut out, "    compressionQuality: {}", setting.compression_quality).unwrap();
        writeln!(&mut out, "    crunchedCompression: {}", setting.crunched_compression).unwrap();
        writeln!(&mut out, "    allowsAlphaSplitting: {}", setting.allows_alpha_splitting).unwrap();
        writeln!(&mut out, "    overridden: {}", setting.overridden).unwrap();
        writeln!(
            &mut out,
            "    ignorePlatformSupport: {}",
            setting.ignore_platform_support
        )
        .unwrap();
        writeln!(
            &mut out,
            "    androidETC2FallbackOverride: {}",
            setting.android_etc2_fallback_override
        )
        .unwrap();
        writeln!(
            &mut out,
            "    forceMaximumCompressionQuality_BC6H_BC7: {}",
            setting.force_maximum_compression_quality_bc6h_bc7
        )
        .unwrap();
    }

    emit_sprite_sheet(&mut out, &importer.sprite_sheet);

    writeln!(
        &mut out,
        "  mipmapLimitGroupName: {}",
        importer.mipmap_limit_group_name
    )
    .unwrap();
    writeln!(&mut out, "  pSDRemoveMatte: {}", importer.p_sd_remove_matte).unwrap();
    writeln!(&mut out, "  userData: {}", importer.user_data).unwrap();
    writeln!(&mut out, "  assetBundleName: {}", importer.asset_bundle_name).unwrap();
    writeln!(&mut out, "  assetBundleVariant: {}", importer.asset_bundle_variant).unwrap();

    out
}

pub fn emit_png_meta_doc(doc: &PngMetaDoc) -> Vec<u8> {
    render_png_meta_doc(doc).into_bytes()
}

/// Parse just enough of the frozen fixture to rebuild a byte-identical document.
/// T11 will replace this with a real packer-fed constructor.
pub fn doc_from_fixture_bytes(bytes: &[u8]) -> PngMetaDoc {
    let body = std::str::from_utf8(bytes).unwrap_or_else(|err| panic!("atlas.png.meta fixture is not utf-8: {err}"));
    let lines: Vec<&str> = body.lines().collect();

    let sprite_sheet_index = find_line_index(&lines, "  spriteSheet:");
    let platform_settings = parse_platform_settings(&lines, find_line_index(&lines, "  platformSettings:") + 1, sprite_sheet_index);
    let sprite_sheet = parse_sprite_sheet(&lines, sprite_sheet_index);

    let texture_settings_index = find_line_index(&lines, "  textureSettings:");
    let texture_settings_lines = &lines[texture_settings_index..];
    let (sprite_pivot_x, sprite_pivot_y) = parse_inline_map2(value_after_line(&lines, "  spritePivot: "));
    let (sprite_border_x, sprite_border_y, sprite_border_z, sprite_border_w) =
        parse_inline_map4(value_after_line(&lines, "  spriteBorder: "));

    PngMetaDoc {
        file_format_version: parse_u32(value_after_line(&lines, "fileFormatVersion: ")),
        guid: value_after_line(&lines, "guid: ").to_string(),
        texture_importer: TextureImporterBlock {
            internal_id_to_name_table_empty: expect_exact_line(&lines, "  internalIDToNameTable: []"),
            external_objects_empty: expect_exact_line(&lines, "  externalObjects: {}"),
            serialized_version: parse_u32(value_after_line(&lines, "  serializedVersion: ")),
            mipmaps: MipmapsBlock {
                mip_map_mode: parse_u32(value_after_line(&lines, "    mipMapMode: ")),
                enable_mip_map: parse_u32(value_after_line(&lines, "    enableMipMap: ")),
                s_rgb_texture: parse_u32(value_after_line(&lines, "    sRGBTexture: ")),
                linear_texture: parse_u32(value_after_line(&lines, "    linearTexture: ")),
                fade_out: parse_u32(value_after_line(&lines, "    fadeOut: ")),
                border_mip_map: parse_u32(value_after_line(&lines, "    borderMipMap: ")),
                mip_maps_preserve_coverage: parse_u32(value_after_line(&lines, "    mipMapsPreserveCoverage: ")),
                alpha_test_reference_value: value_after_line(&lines, "    alphaTestReferenceValue: ").to_string(),
                mip_map_fade_distance_start: parse_u32(value_after_line(&lines, "    mipMapFadeDistanceStart: ")),
                mip_map_fade_distance_end: parse_u32(value_after_line(&lines, "    mipMapFadeDistanceEnd: ")),
            },
            bumpmap: BumpmapBlock {
                convert_to_normal_map: parse_u32(value_after_line(&lines, "    convertToNormalMap: ")),
                external_normal_map: parse_u32(value_after_line(&lines, "    externalNormalMap: ")),
                height_scale: value_after_line(&lines, "    heightScale: ").to_string(),
                normal_map_filter: parse_u32(value_after_line(&lines, "    normalMapFilter: ")),
                flip_green_channel: parse_u32(value_after_line(&lines, "    flipGreenChannel: ")),
            },
            is_readable: parse_u32(value_after_line(&lines, "  isReadable: ")),
            streaming_mipmaps: parse_u32(value_after_line(&lines, "  streamingMipmaps: ")),
            streaming_mipmaps_priority: parse_u32(value_after_line(&lines, "  streamingMipmapsPriority: ")),
            v_t_only: parse_u32(value_after_line(&lines, "  vTOnly: ")),
            ignore_mipmap_limit: parse_u32(value_after_line(&lines, "  ignoreMipmapLimit: ")),
            gray_scale_to_alpha: parse_u32(value_after_line(&lines, "  grayScaleToAlpha: ")),
            generate_cubemap: parse_u32(value_after_line(&lines, "  generateCubemap: ")),
            cubemap_convolution: parse_u32(value_after_line(&lines, "  cubemapConvolution: ")),
            seamless_cubemap: parse_u32(value_after_line(&lines, "  seamlessCubemap: ")),
            texture_format: parse_i64(value_after_line(&lines, "  textureFormat: ")),
            max_texture_size: parse_u32(value_after_line(&lines, "  maxTextureSize: ")),
            texture_settings: TextureSettingsBlock {
                serialized_version: parse_u32(value_after_line(texture_settings_lines, "    serializedVersion: ")),
                filter_mode: parse_u32(value_after_line(texture_settings_lines, "    filterMode: ")),
                aniso: parse_u32(value_after_line(texture_settings_lines, "    aniso: ")),
                mip_bias: value_after_line(texture_settings_lines, "    mipBias: ").to_string(),
                wrap_u: parse_u32(value_after_line(texture_settings_lines, "    wrapU: ")),
                wrap_v: parse_u32(value_after_line(texture_settings_lines, "    wrapV: ")),
                wrap_w: parse_u32(value_after_line(texture_settings_lines, "    wrapW: ")),
            },
            n_pot_scale: parse_u32(value_after_line(&lines, "  nPOTScale: ")),
            lightmap: parse_u32(value_after_line(&lines, "  lightmap: ")),
            compression_quality: parse_u32(value_after_line(&lines, "  compressionQuality: ")),
            sprite_mode: parse_u32(value_after_line(&lines, "  spriteMode: ")),
            sprite_extrude: parse_u32(value_after_line(&lines, "  spriteExtrude: ")),
            sprite_mesh_type: parse_u32(value_after_line(&lines, "  spriteMeshType: ")),
            alignment: parse_u32(value_after_line(&lines, "  alignment: ")),
            sprite_pivot_x,
            sprite_pivot_y,
            sprite_pixels_to_units: parse_u32(value_after_line(&lines, "  spritePixelsToUnits: ")),
            sprite_border_x,
            sprite_border_y,
            sprite_border_z,
            sprite_border_w,
            sprite_generate_fallback_physics_shape: parse_u32(value_after_line(&lines, "  spriteGenerateFallbackPhysicsShape: ")),
            alpha_usage: parse_u32(value_after_line(&lines, "  alphaUsage: ")),
            alpha_is_transparency: parse_u32(value_after_line(&lines, "  alphaIsTransparency: ")),
            sprite_tessellation_detail: value_after_line(&lines, "  spriteTessellationDetail: ").to_string(),
            texture_type: parse_u32(value_after_line(&lines, "  textureType: ")),
            texture_shape: parse_u32(value_after_line(&lines, "  textureShape: ")),
            single_channel_component: parse_u32(value_after_line(&lines, "  singleChannelComponent: ")),
            flipbook_rows: parse_u32(value_after_line(&lines, "  flipbookRows: ")),
            flipbook_columns: parse_u32(value_after_line(&lines, "  flipbookColumns: ")),
            max_texture_size_set: parse_u32(value_after_line(&lines, "  maxTextureSizeSet: ")),
            compression_quality_set: parse_u32(value_after_line(&lines, "  compressionQualitySet: ")),
            texture_format_set: parse_u32(value_after_line(&lines, "  textureFormatSet: ")),
            ignore_png_gamma: parse_u32(value_after_line(&lines, "  ignorePngGamma: ")),
            apply_gamma_decoding: parse_u32(value_after_line(&lines, "  applyGammaDecoding: ")),
            swizzle: parse_u32(value_after_line(&lines, "  swizzle: ")),
            cookie_light_type: parse_u32(value_after_line(&lines, "  cookieLightType: ")),
            platform_settings,
            sprite_sheet,
            mipmap_limit_group_name: value_after_line(&lines, "  mipmapLimitGroupName: ").to_string(),
            p_sd_remove_matte: parse_u32(value_after_line(&lines, "  pSDRemoveMatte: ")),
            user_data: value_after_line(&lines, "  userData: ").to_string(),
            asset_bundle_name: value_after_line(&lines, "  assetBundleName: ").to_string(),
            asset_bundle_variant: value_after_line(&lines, "  assetBundleVariant: ").to_string(),
        },
    }
}

/// Compatibility helper for later pipeline wiring.
/// TODO(t11): replace remaining tmp_sprite_asset-local png.meta construction with this module.
pub fn build_png_meta_doc_from_packed_sprites(
    guid: &str,
    atlas_size: PixelSize,
    placed: &[PackedSprite],
    identities: &[SubSpriteIdentity],
) -> PngMetaDoc {
    let max_dim = atlas_size.width.max(atlas_size.height).max(1);
    // The frozen Unity fixture picks the next legal import bucket strictly
    // above the atlas max dimension (512x1024 -> 2048, not 1024).
    let max_texture_size = [32u32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384]
        .into_iter()
        .find(|value| *value > max_dim)
        .unwrap_or(16384);

    let sprites = placed
        .iter()
        .zip(identities.iter())
        .map(|(sprite, identity)| SpriteSheetEntry {
            serialized_version: 2,
            name: sprite.name.clone(),
            rect: Rect2D {
                serialized_version: 2,
                x: i64::from(sprite.frame.x),
                y: i64::from(atlas_size.height.saturating_sub(sprite.frame.y).saturating_sub(sprite.frame.height)),
                width: i64::from(sprite.frame.width),
                height: i64::from(sprite.frame.height),
            },
            alignment: 0,
            pivot_x: "0.5".to_string(),
            pivot_y: "0.5".to_string(),
            border_x: "0".to_string(),
            border_y: "0".to_string(),
            border_z: "0".to_string(),
            border_w: "0".to_string(),
            outline_empty: true,
            physics_shape_empty: true,
            tessellation_detail: "-1".to_string(),
            bones_empty: true,
            sprite_id: identity.sprite_guid.clone(),
            internal_id: identity.file_id,
            vertices_empty: true,
            indices_blank: true,
            edges_empty: true,
            weights_empty: true,
        })
        .collect::<Vec<_>>();

    let name_file_id_table = identities
        .iter()
        .map(|identity| (identity.name.clone(), identity.file_id))
        .collect::<Vec<_>>();

    PngMetaDoc {
        file_format_version: 2,
        guid: guid.to_string(),
        texture_importer: TextureImporterBlock {
            internal_id_to_name_table_empty: true,
            external_objects_empty: true,
            serialized_version: 13,
            mipmaps: MipmapsBlock {
                mip_map_mode: 0,
                enable_mip_map: 0,
                s_rgb_texture: 1,
                linear_texture: 0,
                fade_out: 0,
                border_mip_map: 0,
                mip_maps_preserve_coverage: 0,
                alpha_test_reference_value: "0.5".to_string(),
                mip_map_fade_distance_start: 1,
                mip_map_fade_distance_end: 3,
            },
            bumpmap: BumpmapBlock {
                convert_to_normal_map: 0,
                external_normal_map: 0,
                height_scale: "0.25".to_string(),
                normal_map_filter: 0,
                flip_green_channel: 0,
            },
            is_readable: 0,
            streaming_mipmaps: 0,
            streaming_mipmaps_priority: 0,
            v_t_only: 0,
            ignore_mipmap_limit: 0,
            gray_scale_to_alpha: 0,
            generate_cubemap: 6,
            cubemap_convolution: 0,
            seamless_cubemap: 0,
            texture_format: 1,
            max_texture_size,
            texture_settings: TextureSettingsBlock {
                serialized_version: 2,
                filter_mode: 1,
                aniso: 1,
                mip_bias: "0".to_string(),
                wrap_u: 1,
                wrap_v: 1,
                wrap_w: 1,
            },
            n_pot_scale: 0,
            lightmap: 0,
            compression_quality: 50,
            sprite_mode: 2,
            sprite_extrude: 1,
            sprite_mesh_type: 1,
            alignment: 0,
            sprite_pivot_x: "0.5".to_string(),
            sprite_pivot_y: "0.5".to_string(),
            sprite_pixels_to_units: 100,
            sprite_border_x: "0".to_string(),
            sprite_border_y: "0".to_string(),
            sprite_border_z: "0".to_string(),
            sprite_border_w: "0".to_string(),
            sprite_generate_fallback_physics_shape: 1,
            alpha_usage: 1,
            alpha_is_transparency: 1,
            sprite_tessellation_detail: "-1".to_string(),
            texture_type: 8,
            texture_shape: 1,
            single_channel_component: 0,
            flipbook_rows: 1,
            flipbook_columns: 1,
            max_texture_size_set: 0,
            compression_quality_set: 0,
            texture_format_set: 0,
            ignore_png_gamma: 0,
            apply_gamma_decoding: 0,
            swizzle: 50462976,
            cookie_light_type: 0,
            platform_settings: vec![
                platform_setting("DefaultTexturePlatform", max_texture_size, 0),
                platform_setting("Standalone", max_texture_size, 1),
                platform_setting("iPhone", max_texture_size, 1),
                platform_setting("Android", max_texture_size, 1),
            ],
            sprite_sheet: SpriteSheetBlock {
                serialized_version: 2,
                sprites,
                trailing_outline_empty: true,
                trailing_physics_shape_empty: true,
                trailing_bones_empty: true,
                trailing_sprite_id: "5e97eb03825dee720800000000000000".to_string(),
                trailing_internal_id: 0,
                trailing_vertices_empty: true,
                trailing_indices_blank: true,
                trailing_edges_empty: true,
                trailing_weights_empty: true,
                secondary_textures_empty: true,
                name_file_id_table,
            },
            mipmap_limit_group_name: String::new(),
            p_sd_remove_matte: 0,
            user_data: String::new(),
            asset_bundle_name: String::new(),
            asset_bundle_variant: String::new(),
        },
    }
}

fn emit_mipmaps(out: &mut String, mipmaps: &MipmapsBlock) {
    out.push_str("  mipmaps:\n");
    writeln!(out, "    mipMapMode: {}", mipmaps.mip_map_mode).unwrap();
    writeln!(out, "    enableMipMap: {}", mipmaps.enable_mip_map).unwrap();
    writeln!(out, "    sRGBTexture: {}", mipmaps.s_rgb_texture).unwrap();
    writeln!(out, "    linearTexture: {}", mipmaps.linear_texture).unwrap();
    writeln!(out, "    fadeOut: {}", mipmaps.fade_out).unwrap();
    writeln!(out, "    borderMipMap: {}", mipmaps.border_mip_map).unwrap();
    writeln!(out, "    mipMapsPreserveCoverage: {}", mipmaps.mip_maps_preserve_coverage).unwrap();
    writeln!(out, "    alphaTestReferenceValue: {}", mipmaps.alpha_test_reference_value).unwrap();
    writeln!(out, "    mipMapFadeDistanceStart: {}", mipmaps.mip_map_fade_distance_start).unwrap();
    writeln!(out, "    mipMapFadeDistanceEnd: {}", mipmaps.mip_map_fade_distance_end).unwrap();
}

fn emit_bumpmap(out: &mut String, bumpmap: &BumpmapBlock) {
    out.push_str("  bumpmap:\n");
    writeln!(out, "    convertToNormalMap: {}", bumpmap.convert_to_normal_map).unwrap();
    writeln!(out, "    externalNormalMap: {}", bumpmap.external_normal_map).unwrap();
    writeln!(out, "    heightScale: {}", bumpmap.height_scale).unwrap();
    writeln!(out, "    normalMapFilter: {}", bumpmap.normal_map_filter).unwrap();
    writeln!(out, "    flipGreenChannel: {}", bumpmap.flip_green_channel).unwrap();
}

fn emit_texture_settings(out: &mut String, texture_settings: &TextureSettingsBlock) {
    out.push_str("  textureSettings:\n");
    writeln!(out, "    serializedVersion: {}", texture_settings.serialized_version).unwrap();
    writeln!(out, "    filterMode: {}", texture_settings.filter_mode).unwrap();
    writeln!(out, "    aniso: {}", texture_settings.aniso).unwrap();
    writeln!(out, "    mipBias: {}", texture_settings.mip_bias).unwrap();
    writeln!(out, "    wrapU: {}", texture_settings.wrap_u).unwrap();
    writeln!(out, "    wrapV: {}", texture_settings.wrap_v).unwrap();
    writeln!(out, "    wrapW: {}", texture_settings.wrap_w).unwrap();
}

fn emit_sprite_sheet(out: &mut String, sprite_sheet: &SpriteSheetBlock) {
    out.push_str("  spriteSheet:\n");
    writeln!(out, "    serializedVersion: {}", sprite_sheet.serialized_version).unwrap();
    out.push_str("    sprites:\n");
    for sprite in &sprite_sheet.sprites {
        writeln!(out, "    - serializedVersion: {}", sprite.serialized_version).unwrap();
        writeln!(out, "      name: {}", sprite.name).unwrap();
        out.push_str("      rect:\n");
        writeln!(out, "        serializedVersion: {}", sprite.rect.serialized_version).unwrap();
        writeln!(out, "        x: {}", sprite.rect.x).unwrap();
        writeln!(out, "        y: {}", sprite.rect.y).unwrap();
        writeln!(out, "        width: {}", sprite.rect.width).unwrap();
        writeln!(out, "        height: {}", sprite.rect.height).unwrap();
        writeln!(out, "      alignment: {}", sprite.alignment).unwrap();
        writeln!(out, "      pivot: {{x: {}, y: {}}}", sprite.pivot_x, sprite.pivot_y).unwrap();
        writeln!(out, "      border: {{x: {}, y: {}, z: {}, w: {}}}", sprite.border_x, sprite.border_y, sprite.border_z, sprite.border_w).unwrap();
        out.push_str(if sprite.outline_empty { "      outline: []\n" } else { panic!("non-empty outline unsupported") });
        out.push_str(if sprite.physics_shape_empty { "      physicsShape: []\n" } else { panic!("non-empty physicsShape unsupported") });
        writeln!(out, "      tessellationDetail: {}", sprite.tessellation_detail).unwrap();
        out.push_str(if sprite.bones_empty { "      bones: []\n" } else { panic!("non-empty bones unsupported") });
        writeln!(out, "      spriteID: {}", sprite.sprite_id).unwrap();
        writeln!(out, "      internalID: {}", sprite.internal_id).unwrap();
        out.push_str(if sprite.vertices_empty { "      vertices: []\n" } else { panic!("non-empty vertices unsupported") });
        out.push_str(if sprite.indices_blank { "      indices: \n" } else { panic!("non-blank indices unsupported") });
        out.push_str(if sprite.edges_empty { "      edges: []\n" } else { panic!("non-empty edges unsupported") });
        out.push_str(if sprite.weights_empty { "      weights: []\n" } else { panic!("non-empty weights unsupported") });
    }
    out.push_str(if sprite_sheet.trailing_outline_empty { "    outline: []\n" } else { panic!("non-empty trailing outline unsupported") });
    out.push_str(if sprite_sheet.trailing_physics_shape_empty { "    physicsShape: []\n" } else { panic!("non-empty trailing physicsShape unsupported") });
    out.push_str(if sprite_sheet.trailing_bones_empty { "    bones: []\n" } else { panic!("non-empty trailing bones unsupported") });
    writeln!(out, "    spriteID: {}", sprite_sheet.trailing_sprite_id).unwrap();
    writeln!(out, "    internalID: {}", sprite_sheet.trailing_internal_id).unwrap();
    out.push_str(if sprite_sheet.trailing_vertices_empty { "    vertices: []\n" } else { panic!("non-empty trailing vertices unsupported") });
    out.push_str(if sprite_sheet.trailing_indices_blank { "    indices: \n" } else { panic!("non-blank trailing indices unsupported") });
    out.push_str(if sprite_sheet.trailing_edges_empty { "    edges: []\n" } else { panic!("non-empty trailing edges unsupported") });
    out.push_str(if sprite_sheet.trailing_weights_empty { "    weights: []\n" } else { panic!("non-empty trailing weights unsupported") });
    out.push_str(if sprite_sheet.secondary_textures_empty { "    secondaryTextures: []\n" } else { panic!("non-empty secondaryTextures unsupported") });
    out.push_str("    nameFileIdTable:\n");
    for (name, value) in &sprite_sheet.name_file_id_table {
        writeln!(out, "      {}: {}", name, value).unwrap();
    }
}

fn parse_platform_settings(lines: &[&str], start: usize, end: usize) -> Vec<PlatformSetting> {
    let mut settings = Vec::new();
    let mut index = start;
    while index < end {
        if lines[index].starts_with("  - serializedVersion: ") {
            let item_start = index;
            index += 1;
            while index < end && lines[index].starts_with("    ") {
                index += 1;
            }
            let item = &lines[item_start..index];
            settings.push(PlatformSetting {
                serialized_version: parse_u32(after_prefix(find_prefixed(item, "  - serializedVersion: "), "  - serializedVersion: ")),
                build_target: after_prefix(find_prefixed(item, "    buildTarget: "), "    buildTarget: ").to_string(),
                max_texture_size: parse_u32(after_prefix(find_prefixed(item, "    maxTextureSize: "), "    maxTextureSize: ")),
                resize_algorithm: parse_u32(after_prefix(find_prefixed(item, "    resizeAlgorithm: "), "    resizeAlgorithm: ")),
                texture_format: parse_i64(after_prefix(find_prefixed(item, "    textureFormat: "), "    textureFormat: ")),
                texture_compression: parse_u32(after_prefix(find_prefixed(item, "    textureCompression: "), "    textureCompression: ")),
                compression_quality: parse_u32(after_prefix(find_prefixed(item, "    compressionQuality: "), "    compressionQuality: ")),
                crunched_compression: parse_u32(after_prefix(find_prefixed(item, "    crunchedCompression: "), "    crunchedCompression: ")),
                allows_alpha_splitting: parse_u32(after_prefix(find_prefixed(item, "    allowsAlphaSplitting: "), "    allowsAlphaSplitting: ")),
                overridden: parse_u32(after_prefix(find_prefixed(item, "    overridden: "), "    overridden: ")),
                ignore_platform_support: parse_u32(after_prefix(find_prefixed(item, "    ignorePlatformSupport: "), "    ignorePlatformSupport: ")),
                android_etc2_fallback_override: parse_u32(after_prefix(find_prefixed(item, "    androidETC2FallbackOverride: "), "    androidETC2FallbackOverride: ")),
                force_maximum_compression_quality_bc6h_bc7: parse_u32(after_prefix(find_prefixed(item, "    forceMaximumCompressionQuality_BC6H_BC7: "), "    forceMaximumCompressionQuality_BC6H_BC7: ")),
            });
            continue;
        }
        index += 1;
    }
    settings
}

fn parse_sprite_sheet(lines: &[&str], sprite_sheet_index: usize) -> SpriteSheetBlock {
    let sprites_index = find_line_index_from(lines, sprite_sheet_index, "    sprites:");
    let trailing_outline_index = find_line_index_from(lines, sprites_index, "    outline: []");
    let name_file_id_index = find_line_index_from(lines, trailing_outline_index, "    nameFileIdTable:");
    let footer_index = find_line_index_from(lines, name_file_id_index, "  mipmapLimitGroupName: ");

    let mut sprites = Vec::new();
    let mut index = sprites_index + 1;
    while index < trailing_outline_index {
        if lines[index].starts_with("    - serializedVersion: ") {
            let item_start = index;
            index += 1;
            while index < trailing_outline_index && lines[index].starts_with("      ") {
                index += 1;
            }
            let item = &lines[item_start..index];
            let (pivot_x, pivot_y) = parse_inline_map2(after_prefix(find_prefixed(item, "      pivot: "), "      pivot: "));
            let (border_x, border_y, border_z, border_w) =
                parse_inline_map4(after_prefix(find_prefixed(item, "      border: "), "      border: "));
            sprites.push(SpriteSheetEntry {
                serialized_version: parse_u32(after_prefix(find_prefixed(item, "    - serializedVersion: "), "    - serializedVersion: ")),
                name: after_prefix(find_prefixed(item, "      name: "), "      name: ").to_string(),
                rect: Rect2D {
                    serialized_version: parse_u32(after_prefix(find_prefixed(item, "        serializedVersion: "), "        serializedVersion: ")),
                    x: parse_i64(after_prefix(find_prefixed(item, "        x: "), "        x: ")),
                    y: parse_i64(after_prefix(find_prefixed(item, "        y: "), "        y: ")),
                    width: parse_i64(after_prefix(find_prefixed(item, "        width: "), "        width: ")),
                    height: parse_i64(after_prefix(find_prefixed(item, "        height: "), "        height: ")),
                },
                alignment: parse_u32(after_prefix(find_prefixed(item, "      alignment: "), "      alignment: ")),
                pivot_x,
                pivot_y,
                border_x,
                border_y,
                border_z,
                border_w,
                outline_empty: item.iter().any(|line| *line == "      outline: []"),
                physics_shape_empty: item.iter().any(|line| *line == "      physicsShape: []"),
                tessellation_detail: after_prefix(find_prefixed(item, "      tessellationDetail: "), "      tessellationDetail: ").to_string(),
                bones_empty: item.iter().any(|line| *line == "      bones: []"),
                sprite_id: after_prefix(find_prefixed(item, "      spriteID: "), "      spriteID: ").to_string(),
                internal_id: parse_i64(after_prefix(find_prefixed(item, "      internalID: "), "      internalID: ")),
                vertices_empty: item.iter().any(|line| *line == "      vertices: []"),
                indices_blank: item.iter().any(|line| *line == "      indices: "),
                edges_empty: item.iter().any(|line| *line == "      edges: []"),
                weights_empty: item.iter().any(|line| *line == "      weights: []"),
            });
            continue;
        }
        index += 1;
    }

    let trailing_lines = &lines[trailing_outline_index..name_file_id_index];
    let mut name_file_id_table = Vec::new();
    let mut table_index = name_file_id_index + 1;
    while table_index < footer_index {
        let line = lines[table_index];
        if !line.starts_with("      ") {
            break;
        }
        let (name, value) = line.trim_start().split_once(':').unwrap_or_else(|| panic!("malformed nameFileIdTable entry: {line}"));
        name_file_id_table.push((name.trim().to_string(), parse_i64(value.trim())));
        table_index += 1;
    }

    SpriteSheetBlock {
        serialized_version: parse_u32(value_after_line_from(lines, sprite_sheet_index, "    serializedVersion: ")),
        sprites,
        trailing_outline_empty: trailing_lines.iter().any(|line| *line == "    outline: []"),
        trailing_physics_shape_empty: trailing_lines.iter().any(|line| *line == "    physicsShape: []"),
        trailing_bones_empty: trailing_lines.iter().any(|line| *line == "    bones: []"),
        trailing_sprite_id: after_prefix(find_prefixed(trailing_lines, "    spriteID: "), "    spriteID: ").to_string(),
        trailing_internal_id: parse_i64(after_prefix(find_prefixed(trailing_lines, "    internalID: "), "    internalID: ")),
        trailing_vertices_empty: trailing_lines.iter().any(|line| *line == "    vertices: []"),
        trailing_indices_blank: trailing_lines.iter().any(|line| *line == "    indices: "),
        trailing_edges_empty: trailing_lines.iter().any(|line| *line == "    edges: []"),
        trailing_weights_empty: trailing_lines.iter().any(|line| *line == "    weights: []"),
        secondary_textures_empty: trailing_lines.iter().any(|line| *line == "    secondaryTextures: []"),
        name_file_id_table,
    }
}

fn platform_setting(build_target: &str, max_texture_size: u32, texture_compression: u32) -> PlatformSetting {
    PlatformSetting {
        serialized_version: 3,
        build_target: build_target.to_string(),
        max_texture_size,
        resize_algorithm: 0,
        texture_format: -1,
        texture_compression,
        compression_quality: 50,
        crunched_compression: 0,
        allows_alpha_splitting: 0,
        overridden: 0,
        ignore_platform_support: 0,
        android_etc2_fallback_override: 0,
        force_maximum_compression_quality_bc6h_bc7: 0,
    }
}

fn parse_inline_map2(body: &str) -> (String, String) {
    let inner = body
        .trim()
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or_else(|| panic!("malformed inline map: {body}"));
    let mut x = None;
    let mut y = None;
    for part in inner.split(',') {
        let (key, value) = part.trim().split_once(':').unwrap_or_else(|| panic!("malformed inline pair: {part}"));
        match key.trim() {
            "x" => x = Some(value.trim().to_string()),
            "y" => y = Some(value.trim().to_string()),
            _ => {}
        }
    }
    (
        x.unwrap_or_else(|| panic!("inline map missing x: {body}")),
        y.unwrap_or_else(|| panic!("inline map missing y: {body}")),
    )
}

fn parse_inline_map4(body: &str) -> (String, String, String, String) {
    let inner = body
        .trim()
        .strip_prefix('{')
        .and_then(|value| value.strip_suffix('}'))
        .unwrap_or_else(|| panic!("malformed inline map: {body}"));
    let mut x = None;
    let mut y = None;
    let mut z = None;
    let mut w = None;
    for part in inner.split(',') {
        let (key, value) = part.trim().split_once(':').unwrap_or_else(|| panic!("malformed inline pair: {part}"));
        match key.trim() {
            "x" => x = Some(value.trim().to_string()),
            "y" => y = Some(value.trim().to_string()),
            "z" => z = Some(value.trim().to_string()),
            "w" => w = Some(value.trim().to_string()),
            _ => {}
        }
    }
    (
        x.unwrap_or_else(|| panic!("inline map missing x: {body}")),
        y.unwrap_or_else(|| panic!("inline map missing y: {body}")),
        z.unwrap_or_else(|| panic!("inline map missing z: {body}")),
        w.unwrap_or_else(|| panic!("inline map missing w: {body}")),
    )
}

fn value_after_line<'a>(lines: &'a [&'a str], prefix: &str) -> &'a str {
    after_prefix(find_prefixed(lines, prefix), prefix)
}

fn value_after_line_from<'a>(lines: &'a [&'a str], start: usize, prefix: &str) -> &'a str {
    after_prefix(find_prefixed(&lines[start..], prefix), prefix)
}

fn expect_exact_line(lines: &[&str], needle: &str) -> bool {
    lines.iter().any(|line| *line == needle)
}

fn find_line_index(lines: &[&str], prefix: &str) -> usize {
    lines
        .iter()
        .position(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("missing line prefix: {prefix}"))
}

fn find_line_index_from(lines: &[&str], start: usize, prefix: &str) -> usize {
    lines
        .iter()
        .enumerate()
        .skip(start)
        .find(|(_, line)| line.starts_with(prefix))
        .map(|(index, _)| index)
        .unwrap_or_else(|| panic!("missing line prefix after {start}: {prefix}"))
}

fn find_prefixed<'a>(lines: &'a [&'a str], prefix: &str) -> &'a str {
    lines
        .iter()
        .copied()
        .find(|line| line.starts_with(prefix))
        .unwrap_or_else(|| panic!("missing prefixed line: {prefix}"))
}

fn after_prefix<'a>(line: &'a str, prefix: &str) -> &'a str {
    &line[prefix.len()..]
}

fn parse_u32(value: &str) -> u32 {
    value
        .trim()
        .parse::<u32>()
        .unwrap_or_else(|err| panic!("invalid u32 `{value}`: {err}"))
}

fn parse_i64(value: &str) -> i64 {
    value
        .trim()
        .parse::<i64>()
        .unwrap_or_else(|err| panic!("invalid i64 `{value}`: {err}"))
}
