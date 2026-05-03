// AtlasPacker Pro - Unity TextMeshPro Sprite Asset exporter.
//
// T8 rewrite note (option b): keep the existing pipeline-facing `emit()` on the
// legacy builder for now, and add a separate fixture-locked renderer
// (`render_sprite_asset_doc` / `emit_sprite_asset_doc`) that reproduces
// `tests/fixtures/atlaspro_ground_truth/atlas.asset` byte-for-byte. T11/T12
// will wire the full export pipeline over to the new document emitter.
//
// Legacy pipeline path still emits 4 files:
//   1. <name>.png             the atlas image
//   2. <name>.png.meta        TextureImporter w/ spriteMode=2 + sprites[] +
//                             nameFileIdTable - this is what makes Unity carve
//                             the PNG into named child Sprite assets.
//   3. <name>.asset           TMP_SpriteAsset MonoBehaviour YAML.
//   4. <name>.asset.meta      NativeFormatImporter pointing at fileID 11400000.
//
// Coordinate convention:
//   PackedSprite.frame uses TOP-LEFT origin (atlas-space pixels).
//   Unity glyphRect / sprite.rect use BOTTOM-LEFT origin.
//   So: bottom_left_y = atlas_height - frame.y - frame.height.
//
// Three-way ID consistency (legacy pipeline path):
//   Each sprite gets ONE stable i64 `internal_id` derived from md5 seeds. That
//   same value is written into png.meta `sprites[].internalID`, png.meta
//   `nameFileIdTable[<name>]`, and asset `m_SpriteGlyphTable[].sprite.fileID`.
//   If any of the three drifts, Unity shows the glyph as Missing.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use image::RgbaImage;

use crate::error::AppError;

use super::super::model::{EmittedFile, ExportFormat, PackedSprite, PixelSize};
use super::super::sub_sprite_id::SubSpriteIdentity;
use super::{deterministic_guid, input_path_hash16, make_emitted, output_path, write_atlas_png};

// TMP_SpriteAsset MonoScript GUID. Captured from a Unity 2022.3.62f3 + TMP
// 3.0.6 install in batch mode (see tests/fixtures/unity_ground_truth/).
// This GUID is STABLE across TMP 2.x and 3.x because it is shipped inside the
// `com.unity.textmeshpro` package and Unity uses package-stable GUIDs there.
// If a future Unity release changes it, every TMP_SpriteAsset on the planet
// breaks the same way - so we'd hear about it loud and clear.
const TMP_SPRITE_ASSET_SCRIPT_GUID: &str = "84a92b25f83d49b9bc132d206b370281";

// Unity reserves a small set of well-known fileIDs for built-in object types.
// 11400000 is the "first MonoBehaviour in a .asset file" id; 11500000 is the
// MonoScript reference id; 2800000 is the main Texture2D in a .png.
const TMP_ASSET_MAIN_FILE_ID: i64 = 11400000;
const MONOSCRIPT_FILE_ID: i64 = 11500000;
const TEXTURE2D_MAIN_FILE_ID: i64 = 2800000;
const EMBEDDED_MATERIAL_CLASS_ID: i64 = 21;
const EMBEDDED_MATERIAL_SHADER_FILE_ID: i64 = 4800000;
const EMBEDDED_MATERIAL_SHADER_GUID: &str = "cf81c85f95fe47e1a27f6ae460cf182c";
const FIXTURE_EMBEDDED_MATERIAL_FILE_ID: i64 = 7861952860736239101;

const UNITY_MAX_TEXTURE_SIZE: u32 = 16384;

/// HorizontalBearingY ratio — the fraction of glyph height that sits above the
/// text baseline. TMP uses this to vertically position sprites relative to text.
/// Ratio derived from Unity 2022.3.62f3's TMP 3.0.7 sprite asset examples
/// (115.6 / 128 ≈ 0.903125). Setting it too high makes sprites float above text;
/// too low makes them sink below the baseline.
const BEARING_Y_RATIO: f64 = 0.903125;

/// Format a bearing-Y value the way Unity does in YAML glyph metrics. Integer
/// values are emitted without a decimal point (e.g. "128"); fractional values
/// keep one decimal digit (e.g. "115.6"). Uses f64 for internal precision.
fn bearing_y_str(glyph_height: u32) -> String {
    let v = glyph_height as f64 * BEARING_Y_RATIO;
    let rounded = (v * 10.0).round() / 10.0;
    if (rounded - rounded.trunc()).abs() < 1e-9 {
        format!("{}", rounded as u32)
    } else {
        format!("{:.1}", rounded)
    }
}

pub struct TmpEmitParams<'a> {
    pub atlas_image: &'a RgbaImage,
    pub atlas_size: PixelSize,
    pub placed: &'a [PackedSprite],
    pub output_dir: &'a Path,
    pub input_dir: &'a Path,
    pub base_name: &'a str,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TmpSpriteAssetDoc {
    pub asset_name: String,
    pub root_hash_code: i32,
    pub material_file_id: i64,
    pub material_name: String,
    pub material_hash_code: i32,
    pub version: String,
    pub texture_guid: String,
    pub characters: Vec<SpriteCharacterRow>,
    pub glyphs: Vec<SpriteGlyphRow>,
    pub sprite_infos: Vec<SpriteInfoRow>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteCharacterRow {
    pub element_type: i64,
    pub unicode: i64,
    pub glyph_index: i64,
    pub scale: String,
    pub name: String,
    pub hash_code: i32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteGlyphRow {
    pub index: i64,
    pub metrics: GlyphMetricsText,
    pub rect: GlyphRect,
    pub scale: String,
    pub atlas_index: i64,
    pub class_definition_type: i64,
    pub sprite_file_id: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphMetricsText {
    pub width: String,
    pub height: String,
    pub horizontal_bearing_x: String,
    pub horizontal_bearing_y: String,
    pub horizontal_advance: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GlyphRect {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SpriteInfoRow {
    pub id: i64,
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub x_offset: String,
    pub y_offset: String,
    pub x_advance: String,
    pub scale: String,
    pub name: String,
    pub hash_code: i32,
    pub unicode: i64,
    pub pivot_x: String,
    pub pivot_y: String,
    pub sprite_file_id: i64,
}

pub fn emit(params: TmpEmitParams<'_>) -> Result<Vec<EmittedFile>, AppError> {
    let TmpEmitParams { atlas_image, atlas_size, placed, output_dir, input_dir, base_name } = params;

    let max_dim = atlas_size.width.max(atlas_size.height);
    if max_dim > UNITY_MAX_TEXTURE_SIZE {
        return Err(AppError::new(
            "atlaspro_tmp_texture_size_exceeded",
            format!(
                "TMP atlas {}x{} exceeds Unity's maxTextureSize ({}). Unity would silently downscale the PNG while TMP glyph rects would still reference original coordinates, breaking every sprite. Reduce max width/height to <= {}.",
                atlas_size.width, atlas_size.height, UNITY_MAX_TEXTURE_SIZE, UNITY_MAX_TEXTURE_SIZE
            ),
        ));
    }

    // Build the "sprite identity" table once. Every emitted file references
    // these values, so computing them in one place guarantees consistency.
    let input_hash16 = input_path_hash16(input_dir)?;
    let identities: Vec<SpriteIdentity> = placed
        .iter()
        .map(|p| SpriteIdentity {
            name: p.name.clone(),
            internal_id: stable_internal_id(base_name, &p.name),
            sprite_guid: deterministic_guid(&format!(
                "nebulakit:atlaspro:tmp:sprite:{}:{base_name}:{}",
                input_hash16,
                p.name
            ))
            .chars()
            .take(32)
            .collect(),
        })
        .collect();

    let png_path = output_path(output_dir, base_name, "", "png");
    write_atlas_png(atlas_image, &png_path)?;

    let texture_guid = deterministic_guid(&super::namespaced_guid_seed(
        "nebulakit:atlaspro:tex",
        input_dir,
        base_name,
    )?);
    let asset_guid = deterministic_guid(&super::namespaced_guid_seed(
        "nebulakit:atlaspro:asset",
        input_dir,
        base_name,
    )?);

    let png_meta_path = PathBuf::from(format!("{}.meta", png_path.display()));
    write_text(
        &png_meta_path,
        &build_png_meta(&texture_guid, atlas_size, placed, &identities),
    )?;

    let asset_path = output_path(output_dir, base_name, "", "asset");
    let asset_meta_path = PathBuf::from(format!("{}.meta", asset_path.display()));
    let asset_yaml = build_sprite_asset(base_name, atlas_size, placed, &texture_guid, &identities);
    write_text(&asset_path, &asset_yaml)?;
    write_text(
        &asset_meta_path,
        &build_asset_meta(&asset_guid, TMP_ASSET_MAIN_FILE_ID),
    )?;

    Ok(vec![
        make_emitted(ExportFormat::TmpSpriteAsset, png_path),
        make_emitted(ExportFormat::TmpSpriteAsset, png_meta_path),
        make_emitted(ExportFormat::TmpSpriteAsset, asset_path),
        make_emitted(ExportFormat::TmpSpriteAsset, asset_meta_path),
    ])
}

/// One sprite's stable identity, shared between png.meta and .asset YAML.
struct SpriteIdentity {
    name: String,
    /// 64-bit Unity internal id - the value used by Unity to address sub-assets.
    /// We use a deterministic hash so re-exports preserve references in scenes.
    internal_id: i64,
    /// 32-char hex GUID stored on the carved-out child Sprite (used as
    /// `spriteID` in png.meta sprites[]). Not strictly required for the .asset
    /// to find the sprite (fileID is enough), but Unity stores one and it's
    /// nicer to have something stable.
    sprite_guid: String,
}

fn write_text(path: &Path, body: &str) -> Result<(), AppError> {
    super::atomic_write(path, body.as_bytes())
}

/// TMP_TextUtilities.GetHashCode-equivalent. Verified bit-for-bit against
/// Unity-emitted m_HashCode values for "emoji_blue" (1105941861), "emoji_green"
/// (2141998304), "emoji_red" (1465152040), "emoji_yellow" (822156319) - see
/// the unit test below. Algorithm: DJB2 with XOR (init 0), no case folding,
/// signed 32-bit arithmetic. Note: Unity's TMP source comments claim
/// "case-insensitive" but the actual TMP_TextUtilities.GetSimpleHashCode does
/// NOT uppercase - the case-insensitive variant is GetHashCodeCaseInSensitive.
/// We match the case-sensitive one because that's what TMP_SpriteAsset writes.
pub fn hash_tmp_name(name: &str) -> i32 {
    let mut hash: i32 = 0;
    for c in name.chars() {
        let v = c as i32;
        // ((hash << 5) + hash) ^ c, in 32-bit wrapping arithmetic.
        hash = ((hash << 5).wrapping_add(hash)) ^ v;
    }
    hash
}

/// Fixture-oriented constructor used by the T11 ground-truth packer rewrite.
///
/// `sprite_asset_guid` is accepted for forward compatibility with T12 pipeline
/// wiring; the current TMP SpriteAsset YAML does not embed its own asset GUID,
/// so the fixture-locked T11 document shape does not need it yet.
pub fn build_tmp_sprite_asset_doc(
    name: &str,
    png_guid: &str,
    sprite_asset_guid: &str,
    identities: &[SubSpriteIdentity],
    atlas_dimensions: PixelSize,
    placed_rects: &[PackedSprite],
) -> TmpSpriteAssetDoc {
    let _ = sprite_asset_guid;

    assert_eq!(
        placed_rects.len(),
        identities.len(),
        "placed rect count must match identity count"
    );

    let material_name = format!("{name} Material");

    let characters = placed_rects
        .iter()
        .enumerate()
        .map(|(index, sprite)| SpriteCharacterRow {
            element_type: 2,
            unicode: 65534,
            glyph_index: index as i64,
            scale: "1".to_string(),
            name: sprite.name.clone(),
            hash_code: hash_tmp_name(&sprite.name),
        })
        .collect();

    let glyphs = placed_rects
        .iter()
        .zip(identities.iter())
        .enumerate()
        .map(|(index, (sprite, identity))| {
            let unity_y = atlas_dimensions
                .height
                .saturating_sub(sprite.frame.y)
                .saturating_sub(sprite.frame.height);
            SpriteGlyphRow {
                index: index as i64,
                metrics: GlyphMetricsText {
                    width: sprite.frame.width.to_string(),
                    height: sprite.frame.height.to_string(),
                    horizontal_bearing_x: "0".to_string(),
                    horizontal_bearing_y: bearing_y_str(sprite.frame.height),
                    horizontal_advance: sprite.frame.width.to_string(),
                },
                rect: GlyphRect {
                    x: sprite.frame.x,
                    y: unity_y,
                    width: sprite.frame.width,
                    height: sprite.frame.height,
                },
                scale: "1".to_string(),
                atlas_index: 0,
                class_definition_type: 0,
                sprite_file_id: identity.file_id,
            }
        })
        .collect();

    let sprite_infos = placed_rects
        .iter()
        .zip(identities.iter())
        .enumerate()
        .map(|(index, (sprite, identity))| {
            let unity_y = atlas_dimensions
                .height
                .saturating_sub(sprite.frame.y)
                .saturating_sub(sprite.frame.height);
            SpriteInfoRow {
                id: index as i64,
                x: sprite.frame.x,
                y: unity_y,
                width: sprite.frame.width,
                height: sprite.frame.height,
                x_offset: "0".to_string(),
                y_offset: sprite.frame.height.to_string(),
                x_advance: sprite.frame.width.to_string(),
                scale: "1".to_string(),
                name: sprite.name.clone(),
                hash_code: hash_tmp_name(&sprite.name),
                unicode: 65534,
                pivot_x: "0.5".to_string(),
                pivot_y: "0.5".to_string(),
                sprite_file_id: identity.file_id,
            }
        })
        .collect();

    TmpSpriteAssetDoc {
        asset_name: name.to_string(),
        root_hash_code: hash_tmp_name(name),
        material_file_id: FIXTURE_EMBEDDED_MATERIAL_FILE_ID,
        material_name: material_name.clone(),
        material_hash_code: hash_tmp_name(&material_name),
        version: "1.1.0".to_string(),
        texture_guid: png_guid.to_string(),
        characters,
        glyphs,
        sprite_infos,
    }
}

pub fn render_sprite_asset_doc(doc: &TmpSpriteAssetDoc) -> String {
    let mut out = String::with_capacity(16_384);
    emit_header(&mut out, doc);
    emit_material_pptr(&mut out, doc.material_file_id);
    emit_meta_block(&mut out, doc);
    emit_sprite_sheet_pptr(&mut out, &doc.texture_guid);
    emit_character_table(&mut out, &doc.characters);
    emit_glyph_table(&mut out, &doc.glyphs, &doc.texture_guid);
    emit_sprite_info_list(&mut out, &doc.sprite_infos, &doc.texture_guid);
    emit_footer(&mut out, doc);
    emit_embedded_material(&mut out, doc);
    out
}

pub fn emit_sprite_asset_doc(doc: &TmpSpriteAssetDoc) -> Vec<u8> {
    render_sprite_asset_doc(doc).into_bytes()
}

pub fn emit_tmp_sprite_asset_doc(doc: &TmpSpriteAssetDoc) -> Vec<u8> {
    emit_sprite_asset_doc(doc)
}

fn emit_header(out: &mut String, doc: &TmpSpriteAssetDoc) {
    out.push_str("%YAML 1.1\n");
    out.push_str("%TAG !u! tag:unity3d.com,2011:\n");
    writeln!(out, "--- !u!114 &{TMP_ASSET_MAIN_FILE_ID}").unwrap();
    out.push_str("MonoBehaviour:\n");
    out.push_str("  m_ObjectHideFlags: 0\n");
    out.push_str("  m_CorrespondingSourceObject: {fileID: 0}\n");
    out.push_str("  m_PrefabInstance: {fileID: 0}\n");
    out.push_str("  m_PrefabAsset: {fileID: 0}\n");
    out.push_str("  m_GameObject: {fileID: 0}\n");
    out.push_str("  m_Enabled: 1\n");
    out.push_str("  m_EditorHideFlags: 0\n");
    writeln!(
        out,
        "  m_Script: {{fileID: {MONOSCRIPT_FILE_ID}, guid: {TMP_SPRITE_ASSET_SCRIPT_GUID}, type: 3}}"
    )
    .unwrap();
    writeln!(out, "  m_Name: {}", doc.asset_name).unwrap();
    out.push_str("  m_EditorClassIdentifier: \n");
    writeln!(out, "  hashCode: {}", doc.root_hash_code).unwrap();
}

fn emit_material_pptr(out: &mut String, material_file_id: i64) {
    writeln!(out, "  material: {{fileID: {material_file_id}}}").unwrap();
}

fn emit_meta_block(out: &mut String, doc: &TmpSpriteAssetDoc) {
    writeln!(out, "  materialHashCode: {}", doc.material_hash_code).unwrap();
    writeln!(out, "  m_Version: {}", doc.version).unwrap();
    out.push_str("  m_FaceInfo:\n");
    out.push_str("    m_FaceIndex: 0\n");
    out.push_str("    m_FamilyName: \n");
    out.push_str("    m_StyleName: \n");
    out.push_str("    m_PointSize: 0\n");
    out.push_str("    m_Scale: 0\n");
    out.push_str("    m_UnitsPerEM: 0\n");
    out.push_str("    m_LineHeight: 0\n");
    out.push_str("    m_AscentLine: 0\n");
    out.push_str("    m_CapLine: 0\n");
    out.push_str("    m_MeanLine: 0\n");
    out.push_str("    m_Baseline: 0\n");
    out.push_str("    m_DescentLine: 0\n");
    out.push_str("    m_SuperscriptOffset: 0\n");
    out.push_str("    m_SuperscriptSize: 0\n");
    out.push_str("    m_SubscriptOffset: 0\n");
    out.push_str("    m_SubscriptSize: 0\n");
    out.push_str("    m_UnderlineOffset: 0\n");
    out.push_str("    m_UnderlineThickness: 0\n");
    out.push_str("    m_StrikethroughOffset: 0\n");
    out.push_str("    m_StrikethroughThickness: 0\n");
    out.push_str("    m_TabWidth: 0\n");
}

fn emit_sprite_sheet_pptr(out: &mut String, texture_guid: &str) {
    writeln!(
        out,
        "  spriteSheet: {{fileID: {TEXTURE2D_MAIN_FILE_ID}, guid: {texture_guid}, type: 3}}"
    )
    .unwrap();
}

fn emit_character_table(out: &mut String, rows: &[SpriteCharacterRow]) {
    out.push_str("  m_SpriteCharacterTable:\n");
    for row in rows {
        emit_character_row(out, row);
    }
}

fn emit_character_row(out: &mut String, row: &SpriteCharacterRow) {
    writeln!(out, "  - m_ElementType: {}", row.element_type).unwrap();
    writeln!(out, "    m_Unicode: {}", row.unicode).unwrap();
    writeln!(out, "    m_GlyphIndex: {}", row.glyph_index).unwrap();
    writeln!(out, "    m_Scale: {}", row.scale).unwrap();
    writeln!(out, "    m_Name: {}", row.name).unwrap();
    writeln!(out, "    m_HashCode: {}", row.hash_code).unwrap();
}

fn emit_glyph_table(out: &mut String, rows: &[SpriteGlyphRow], texture_guid: &str) {
    out.push_str("  m_SpriteGlyphTable:\n");
    for row in rows {
        emit_glyph_row(out, row, texture_guid);
    }
}

fn emit_glyph_row(out: &mut String, row: &SpriteGlyphRow, texture_guid: &str) {
    writeln!(out, "  - m_Index: {}", row.index).unwrap();
    out.push_str("    m_Metrics:\n");
    writeln!(out, "      m_Width: {}", row.metrics.width).unwrap();
    writeln!(out, "      m_Height: {}", row.metrics.height).unwrap();
    writeln!(out, "      m_HorizontalBearingX: {}", row.metrics.horizontal_bearing_x).unwrap();
    writeln!(out, "      m_HorizontalBearingY: {}", row.metrics.horizontal_bearing_y).unwrap();
    writeln!(out, "      m_HorizontalAdvance: {}", row.metrics.horizontal_advance).unwrap();
    out.push_str("    m_GlyphRect:\n");
    writeln!(out, "      m_X: {}", row.rect.x).unwrap();
    writeln!(out, "      m_Y: {}", row.rect.y).unwrap();
    writeln!(out, "      m_Width: {}", row.rect.width).unwrap();
    writeln!(out, "      m_Height: {}", row.rect.height).unwrap();
    writeln!(out, "    m_Scale: {}", row.scale).unwrap();
    writeln!(out, "    m_AtlasIndex: {}", row.atlas_index).unwrap();
    writeln!(out, "    m_ClassDefinitionType: {}", row.class_definition_type).unwrap();
    writeln!(
        out,
        "    sprite: {{fileID: {}, guid: {texture_guid}, type: 3}}",
        row.sprite_file_id
    )
    .unwrap();
}

fn emit_sprite_info_list(out: &mut String, rows: &[SpriteInfoRow], texture_guid: &str) {
    out.push_str("  spriteInfoList:\n");
    for row in rows {
        emit_sprite_info_row(out, row, texture_guid);
    }
}

fn emit_sprite_info_row(out: &mut String, row: &SpriteInfoRow, texture_guid: &str) {
    writeln!(out, "  - id: {}", row.id).unwrap();
    writeln!(out, "    x: {}", row.x).unwrap();
    writeln!(out, "    y: {}", row.y).unwrap();
    writeln!(out, "    width: {}", row.width).unwrap();
    writeln!(out, "    height: {}", row.height).unwrap();
    writeln!(out, "    xOffset: {}", row.x_offset).unwrap();
    writeln!(out, "    yOffset: {}", row.y_offset).unwrap();
    writeln!(out, "    xAdvance: {}", row.x_advance).unwrap();
    writeln!(out, "    scale: {}", row.scale).unwrap();
    writeln!(out, "    name: {}", row.name).unwrap();
    writeln!(out, "    hashCode: {}", row.hash_code).unwrap();
    writeln!(out, "    unicode: {}", row.unicode).unwrap();
    writeln!(out, "    pivot: {{x: {}, y: {}}}", row.pivot_x, row.pivot_y).unwrap();
    writeln!(
        out,
        "    sprite: {{fileID: {}, guid: {texture_guid}, type: 3}}",
        row.sprite_file_id
    )
    .unwrap();
}

fn emit_footer(out: &mut String, _doc: &TmpSpriteAssetDoc) {
    out.push_str("  fallbackSpriteAssets: []\n");
}

fn emit_embedded_material(out: &mut String, doc: &TmpSpriteAssetDoc) {
    writeln!(out, "--- !u!{EMBEDDED_MATERIAL_CLASS_ID} &{}", doc.material_file_id).unwrap();
    out.push_str("Material:\n");
    out.push_str("  serializedVersion: 8\n");
    out.push_str("  m_ObjectHideFlags: 1\n");
    out.push_str("  m_CorrespondingSourceObject: {fileID: 0}\n");
    out.push_str("  m_PrefabInstance: {fileID: 0}\n");
    out.push_str("  m_PrefabAsset: {fileID: 0}\n");
    writeln!(out, "  m_Name: {}", doc.material_name).unwrap();
    writeln!(
        out,
        "  m_Shader: {{fileID: {EMBEDDED_MATERIAL_SHADER_FILE_ID}, guid: {EMBEDDED_MATERIAL_SHADER_GUID}, type: 3}}"
    )
    .unwrap();
    out.push_str("  m_Parent: {fileID: 0}\n");
    out.push_str("  m_ModifiedSerializedProperties: 0\n");
    out.push_str("  m_ValidKeywords: []\n");
    out.push_str("  m_InvalidKeywords: []\n");
    out.push_str("  m_LightmapFlags: 4\n");
    out.push_str("  m_EnableInstancingVariants: 0\n");
    out.push_str("  m_DoubleSidedGI: 0\n");
    out.push_str("  m_CustomRenderQueue: -1\n");
    out.push_str("  stringTagMap: {}\n");
    out.push_str("  disabledShaderPasses: []\n");
    out.push_str("  m_LockedProperties: \n");
    out.push_str("  m_SavedProperties:\n");
    out.push_str("    serializedVersion: 3\n");
    out.push_str("    m_TexEnvs:\n");
    out.push_str("    - _MainTex:\n");
    writeln!(
        out,
        "        m_Texture: {{fileID: {TEXTURE2D_MAIN_FILE_ID}, guid: {}, type: 3}}",
        doc.texture_guid
    )
    .unwrap();
    out.push_str("        m_Scale: {x: 1, y: 1}\n");
    out.push_str("        m_Offset: {x: 0, y: 0}\n");
    out.push_str("    m_Ints: []\n");
    out.push_str("    m_Floats:\n");
    out.push_str("    - _ColorMask: 15\n");
    out.push_str("    - _CullMode: 0\n");
    out.push_str("    - _Stencil: 0\n");
    out.push_str("    - _StencilComp: 8\n");
    out.push_str("    - _StencilOp: 0\n");
    out.push_str("    - _StencilReadMask: 255\n");
    out.push_str("    - _StencilWriteMask: 255\n");
    out.push_str("    - _UseUIAlphaClip: 0\n");
    out.push_str("    m_Colors:\n");
    out.push_str("    - _ClipRect: {r: -32767, g: -32767, b: 32767, a: 32767}\n");
    out.push_str("    - _Color: {r: 1, g: 1, b: 1, a: 1}\n");
    out.push_str("  m_BuildTextureStacks: []\n");
}

/// Stable i64 used as Unity sub-asset internalID. Two requirements:
///   1. Deterministic across runs (so scene references survive re-export).
///   2. Distinct per (atlas, sprite) pair (collision -> Unity merges sprites).
/// Implementation: take 8 bytes of MD5(atlas|name) interpreted as little-endian
/// signed i64. Collision probability is ~zero for any real atlas.
fn stable_internal_id(atlas: &str, sprite_name: &str) -> i64 {
    let digest = md5::compute(format!("nebulakit:atlaspro:internal:{atlas}|{sprite_name}").as_bytes());
    let bytes = digest.0;
    i64::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7]])
}

fn build_sprite_asset(
    name: &str,
    atlas_size: PixelSize,
    placed: &[PackedSprite],
    texture_guid: &str,
    identities: &[SpriteIdentity],
) -> String {
    let atlas_h = atlas_size.height;
    let mut yaml = String::with_capacity(4096);

    yaml.push_str("%YAML 1.1\n");
    yaml.push_str("%TAG !u! tag:unity3d.com,2011:\n");
    yaml.push_str(&format!("--- !u!114 &{TMP_ASSET_MAIN_FILE_ID}\n"));
    yaml.push_str("MonoBehaviour:\n");
    yaml.push_str("  m_ObjectHideFlags: 0\n");
    yaml.push_str("  m_CorrespondingSourceObject: {fileID: 0}\n");
    yaml.push_str("  m_PrefabInstance: {fileID: 0}\n");
    yaml.push_str("  m_PrefabAsset: {fileID: 0}\n");
    yaml.push_str("  m_GameObject: {fileID: 0}\n");
    yaml.push_str("  m_Enabled: 1\n");
    yaml.push_str("  m_EditorHideFlags: 0\n");
    yaml.push_str(&format!(
        "  m_Script: {{fileID: {MONOSCRIPT_FILE_ID}, guid: {TMP_SPRITE_ASSET_SCRIPT_GUID}, type: 3}}\n"
    ));
    yaml.push_str(&format!("  m_Name: {name}\n"));
    yaml.push_str("  m_EditorClassIdentifier: \n");
    // Ground truth shows hashCode: 0 / materialHashCode: 0 / m_Version empty.
    // TMP fills these on first edit in the inspector; leaving them zero is the
    // canonical "freshly created" state and avoids drift vs whatever Unity
    // would compute next time the user opens the asset.
    yaml.push_str("  hashCode: 0\n");
    yaml.push_str("  material: {fileID: 0}\n");
    yaml.push_str("  materialHashCode: 0\n");
    yaml.push_str("  m_Version: \n");
    yaml.push_str("  m_FaceInfo:\n");
    yaml.push_str("    m_FaceIndex: 0\n");
    yaml.push_str("    m_FamilyName: \n");
    yaml.push_str("    m_StyleName: \n");
    yaml.push_str("    m_PointSize: 0\n");
    yaml.push_str("    m_Scale: 0\n");
    yaml.push_str("    m_UnitsPerEM: 0\n");
    yaml.push_str("    m_LineHeight: 0\n");
    yaml.push_str("    m_AscentLine: 0\n");
    yaml.push_str("    m_CapLine: 0\n");
    yaml.push_str("    m_MeanLine: 0\n");
    yaml.push_str("    m_Baseline: 0\n");
    yaml.push_str("    m_DescentLine: 0\n");
    yaml.push_str("    m_SuperscriptOffset: 0\n");
    yaml.push_str("    m_SuperscriptSize: 0\n");
    yaml.push_str("    m_SubscriptOffset: 0\n");
    yaml.push_str("    m_SubscriptSize: 0\n");
    yaml.push_str("    m_UnderlineOffset: 0\n");
    yaml.push_str("    m_UnderlineThickness: 0\n");
    yaml.push_str("    m_StrikethroughOffset: 0\n");
    yaml.push_str("    m_StrikethroughThickness: 0\n");
    yaml.push_str("    m_TabWidth: 0\n");
    yaml.push_str(&format!(
        "  spriteSheet: {{fileID: {TEXTURE2D_MAIN_FILE_ID}, guid: {texture_guid}, type: 3}}\n"
    ));

    // SpriteCharacterTable: name -> glyphIndex lookup used by TMP's <sprite name="">
    // tag resolution. Field order is fixed by TMP's serializer; mismatched order
    // would still load but YAML diffs would be noisy on every Unity save.
    yaml.push_str("  m_SpriteCharacterTable:\n");
    for (i, p) in placed.iter().enumerate() {
        let h = hash_tmp_name(&p.name);
        yaml.push_str("  - m_ElementType: 2\n");
        yaml.push_str("    m_Unicode: 0\n");
        yaml.push_str(&format!("    m_GlyphIndex: {i}\n"));
        yaml.push_str("    m_Scale: 1\n");
        yaml.push_str(&format!("    m_Name: {}\n", p.name));
        yaml.push_str(&format!("    m_HashCode: {h}\n"));
    }

    // SpriteGlyphTable: pixel rect + reference back to the carved-out child
    // Sprite asset (by internalID inside the same .png.meta).
    yaml.push_str("  m_SpriteGlyphTable:\n");
    for (i, (p, ident)) in placed.iter().zip(identities.iter()).enumerate() {
        let glyph_y = atlas_h.saturating_sub(p.frame.y).saturating_sub(p.frame.height);
        let w = p.frame.width;
        let h = p.frame.height;
        let bearing_x = 0.0_f32;
        let bearing_y = (h as f32 * BEARING_Y_RATIO as f32).round();
        yaml.push_str(&format!("  - m_Index: {i}\n"));
        yaml.push_str("    m_Metrics:\n");
        yaml.push_str(&format!("      m_Width: {w}\n"));
        yaml.push_str(&format!("      m_Height: {h}\n"));
        yaml.push_str(&format!("      m_HorizontalBearingX: {}\n", format_float(bearing_x)));
        yaml.push_str(&format!("      m_HorizontalBearingY: {}\n", format_float(bearing_y)));
        yaml.push_str(&format!("      m_HorizontalAdvance: {w}\n"));
        yaml.push_str("    m_GlyphRect:\n");
        yaml.push_str(&format!("      m_X: {}\n", p.frame.x));
        yaml.push_str(&format!("      m_Y: {glyph_y}\n"));
        yaml.push_str(&format!("      m_Width: {w}\n"));
        yaml.push_str(&format!("      m_Height: {h}\n"));
        yaml.push_str("    m_Scale: 1\n");
        yaml.push_str("    m_AtlasIndex: 0\n");
        yaml.push_str("    m_ClassDefinitionType: 0\n");
        yaml.push_str(&format!(
            "    sprite: {{fileID: {}, guid: {texture_guid}, type: 3}}\n",
            ident.internal_id
        ));
    }

    yaml.push_str("  spriteInfoList: []\n");
    yaml.push_str("  fallbackSpriteAssets: []\n");

    yaml
}

/// Render a float the way Unity does in YAML: `51.2` not `51.2000` not `51`.
/// Trims trailing zeros but keeps at least one decimal digit if non-integer.
fn format_float(v: f32) -> String {
    if v.fract() == 0.0 {
        format!("{}", v as i64)
    } else {
        // Unity emits up to ~6 sig-figs for these. trim trailing zeros.
        let s = format!("{v}");
        s
    }
}

fn build_png_meta(
    guid: &str,
    atlas_size: PixelSize,
    placed: &[PackedSprite],
    identities: &[SpriteIdentity],
) -> String {
    // Pick the smallest legal Unity maxTextureSize >= max(width, height) so
    // Unity does not downscale our atlas on import.
    let max_dim = atlas_size.width.max(atlas_size.height).max(1);
    let max_texture_size: u32 = [32u32, 64, 128, 256, 512, 1024, 2048, 4096, 8192, 16384]
        .into_iter()
        .find(|&v| v >= max_dim)
        .unwrap_or(16384);

    let atlas_h = atlas_size.height;
    let mut s = String::with_capacity(8192);
    s.push_str("fileFormatVersion: 2\n");
    s.push_str(&format!("guid: {guid}\n"));
    s.push_str("TextureImporter:\n");
    s.push_str("  internalIDToNameTable: []\n");
    s.push_str("  externalObjects: {}\n");
    s.push_str("  serializedVersion: 13\n");
    s.push_str("  mipmaps:\n");
    s.push_str("    mipMapMode: 0\n");
    s.push_str("    enableMipMap: 0\n");
    s.push_str("    sRGBTexture: 1\n");
    s.push_str("    linearTexture: 0\n");
    s.push_str("    fadeOut: 0\n");
    s.push_str("    borderMipMap: 0\n");
    s.push_str("    mipMapsPreserveCoverage: 0\n");
    s.push_str("    alphaTestReferenceValue: 0.5\n");
    s.push_str("    mipMapFadeDistanceStart: 1\n");
    s.push_str("    mipMapFadeDistanceEnd: 3\n");
    s.push_str("  bumpmap:\n");
    s.push_str("    convertToNormalMap: 0\n");
    s.push_str("    externalNormalMap: 0\n");
    s.push_str("    heightScale: 0.25\n");
    s.push_str("    normalMapFilter: 0\n");
    s.push_str("    flipGreenChannel: 0\n");
    s.push_str("  isReadable: 0\n");
    s.push_str("  streamingMipmaps: 0\n");
    s.push_str("  streamingMipmapsPriority: 0\n");
    s.push_str("  vTOnly: 0\n");
    s.push_str("  ignoreMipmapLimit: 0\n");
    s.push_str("  grayScaleToAlpha: 0\n");
    s.push_str("  generateCubemap: 6\n");
    s.push_str("  cubemapConvolution: 0\n");
    s.push_str("  seamlessCubemap: 0\n");
    s.push_str("  textureFormat: 1\n");
    s.push_str(&format!("  maxTextureSize: {max_texture_size}\n"));
    s.push_str("  textureSettings:\n");
    s.push_str("    serializedVersion: 2\n");
    s.push_str("    filterMode: 1\n");
    s.push_str("    aniso: 1\n");
    s.push_str("    mipBias: 0\n");
    s.push_str("    wrapU: 1\n");
    s.push_str("    wrapV: 1\n");
    s.push_str("    wrapW: 1\n");
    s.push_str("  nPOTScale: 0\n");
    s.push_str("  lightmap: 0\n");
    s.push_str("  compressionQuality: 50\n");
    // spriteMode: 2 == Multiple. This is what makes Unity carve the PNG.
    s.push_str("  spriteMode: 2\n");
    s.push_str("  spriteExtrude: 1\n");
    s.push_str("  spriteMeshType: 1\n");
    s.push_str("  alignment: 0\n");
    s.push_str("  spritePivot: {x: 0.5, y: 0.5}\n");
    s.push_str("  spritePixelsToUnits: 100\n");
    s.push_str("  spriteBorder: {x: 0, y: 0, z: 0, w: 0}\n");
    s.push_str("  spriteGenerateFallbackPhysicsShape: 1\n");
    s.push_str("  alphaUsage: 1\n");
    s.push_str("  alphaIsTransparency: 1\n");
    s.push_str("  spriteTessellationDetail: -1\n");
    // textureType: 8 == Sprite (2D and UI).
    s.push_str("  textureType: 8\n");
    s.push_str("  textureShape: 1\n");
    s.push_str("  singleChannelComponent: 0\n");
    s.push_str("  flipbookRows: 1\n");
    s.push_str("  flipbookColumns: 1\n");
    s.push_str("  maxTextureSizeSet: 0\n");
    s.push_str("  compressionQualitySet: 0\n");
    s.push_str("  textureFormatSet: 0\n");
    s.push_str("  ignorePngGamma: 0\n");
    s.push_str("  applyGammaDecoding: 0\n");
    s.push_str("  swizzle: 50462976\n");
    s.push_str("  cookieLightType: 0\n");
    // Single `platformSettings:` key, then 4 list items - matches ground truth
    // exactly (verified against tests/fixtures/unity_ground_truth/test_atlas.png.meta:69).
    // Emitting the header once per loop iteration produces duplicate YAML keys
    // and Unity rejects the .meta on import.
    s.push_str("  platformSettings:\n");
    for build_target in ["DefaultTexturePlatform", "Standalone", "iPhone", "Android"] {
        let texture_compression = if build_target == "DefaultTexturePlatform" { 0 } else { 1 };
        s.push_str("  - serializedVersion: 3\n");
        s.push_str(&format!("    buildTarget: {build_target}\n"));
        s.push_str(&format!("    maxTextureSize: {max_texture_size}\n"));
        s.push_str("    resizeAlgorithm: 0\n");
        s.push_str("    textureFormat: -1\n");
        s.push_str(&format!("    textureCompression: {texture_compression}\n"));
        s.push_str("    compressionQuality: 50\n");
        s.push_str("    crunchedCompression: 0\n");
        s.push_str("    allowsAlphaSplitting: 0\n");
        s.push_str("    overridden: 0\n");
        s.push_str("    ignorePlatformSupport: 0\n");
        s.push_str("    androidETC2FallbackOverride: 0\n");
        s.push_str("    forceMaximumCompressionQuality_BC6H_BC7: 0\n");
    }

    // spriteSheet: the carved sub-sprites. Ground truth uses serializedVersion
    // 2 for both the sheet and each sprite entry.
    s.push_str("  spriteSheet:\n");
    s.push_str("    serializedVersion: 2\n");
    s.push_str("    sprites:\n");
    for (p, ident) in placed.iter().zip(identities.iter()) {
        // Convert top-left atlas coords -> bottom-left Unity coords.
        let rect_y = atlas_h.saturating_sub(p.frame.y).saturating_sub(p.frame.height);
        s.push_str("    - serializedVersion: 2\n");
        s.push_str(&format!("      name: {}\n", p.name));
        s.push_str("      rect:\n");
        s.push_str("        serializedVersion: 2\n");
        s.push_str(&format!("        x: {}\n", p.frame.x));
        s.push_str(&format!("        y: {rect_y}\n"));
        s.push_str(&format!("        width: {}\n", p.frame.width));
        s.push_str(&format!("        height: {}\n", p.frame.height));
        s.push_str("      alignment: 0\n");
        s.push_str("      pivot: {x: 0.5, y: 0.5}\n");
        s.push_str("      border: {x: 0, y: 0, z: 0, w: 0}\n");
        s.push_str("      outline: []\n");
        s.push_str("      physicsShape: []\n");
        s.push_str("      tessellationDetail: 0\n");
        s.push_str("      bones: []\n");
        s.push_str(&format!("      spriteID: {}\n", ident.sprite_guid));
        s.push_str(&format!("      internalID: {}\n", ident.internal_id));
        s.push_str("      vertices: []\n");
        s.push_str("      indices: \n");
        s.push_str("      edges: []\n");
        s.push_str("      weights: []\n");
    }
    // Sheet-level trailers (also from ground truth).
    s.push_str("    outline: []\n");
    s.push_str("    physicsShape: []\n");
    s.push_str("    bones: []\n");
    s.push_str(&format!(
        "    spriteID: {}\n",
        deterministic_guid(&format!("nebulakit:atlaspro:tmp:sheet:{guid}"))
    ));
    s.push_str("    internalID: 0\n");
    s.push_str("    vertices: []\n");
    s.push_str("    indices: \n");
    s.push_str("    edges: []\n");
    s.push_str("    weights: []\n");
    s.push_str("    secondaryTextures: []\n");
    // nameFileIdTable: name -> internalID. Unity uses this to keep references
    // stable when a sprite is renamed (the GUID+internalID lookup outlives the
    // name change).
    s.push_str("    nameFileIdTable:\n");
    // Sort by name for diff-friendly output (matches ground truth ordering).
    let mut sorted: Vec<&SpriteIdentity> = identities.iter().collect();
    sorted.sort_by(|a, b| a.name.cmp(&b.name));
    for ident in sorted {
        s.push_str(&format!("      {}: {}\n", ident.name, ident.internal_id));
    }
    s.push_str("  mipmapLimitGroupName: \n");
    s.push_str("  pSDRemoveMatte: 0\n");
    s.push_str("  userData: \n");
    s.push_str("  assetBundleName: \n");
    s.push_str("  assetBundleVariant: \n");
    s
}

fn build_asset_meta(guid: &str, main_object_file_id: i64) -> String {
    let mut s = String::new();
    s.push_str("fileFormatVersion: 2\n");
    s.push_str(&format!("guid: {guid}\n"));
    s.push_str("NativeFormatImporter:\n");
    s.push_str("  externalObjects: {}\n");
    s.push_str(&format!("  mainObjectFileID: {main_object_file_id}\n"));
    s.push_str("  userData: \n");
    s.push_str("  assetBundleName: \n");
    s.push_str("  assetBundleVariant: \n");
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::atlaspro::model::{PixelRect, PixelSize};

    fn make_sprite(name: &str, x: u32, y: u32, w: u32, h: u32) -> PackedSprite {
        PackedSprite {
            id: name.to_string(),
            name: name.to_string(),
            frame: PixelRect::new(x, y, w, h),
            source_frame: PixelRect::new(0, 0, w, h),
            source_size: PixelSize { width: w, height: h },
            rotated: false,
            trimmed: false,
            unity: None,
        }
    }

    /// THE ground truth check: hash 4 known sprite names and verify the values
    /// match what Unity 2022.3.62f3 + TMP 3.0.6 wrote to disk in
    /// tests/fixtures/unity_ground_truth/test_atlas.asset. If this test ever
    /// fails, the .asset YAML we generate will silently lose <sprite name="">
    /// tag resolution at runtime.
    #[test]
    fn hash_tmp_name_matches_unity_ground_truth() {
        assert_eq!(hash_tmp_name("emoji_blue"),   1105941861);
        assert_eq!(hash_tmp_name("emoji_green"),  2141998304_u32 as i32);
        assert_eq!(hash_tmp_name("emoji_red"),    1465152040);
        assert_eq!(hash_tmp_name("emoji_yellow"), 822156319);
    }

    #[test]
    fn glyph_y_is_flipped_to_bottom_left() {
        // atlas h=256, sprite at top-left (y=10, h=32) => Unity y = 256-10-32 = 214.
        let sprites = vec![make_sprite("smile", 0, 10, 32, 32)];
        let identities = vec![SpriteIdentity {
            name: "smile".into(),
            internal_id: 12345,
            sprite_guid: "abc".into(),
        }];
        let yaml = build_sprite_asset(
            "demo",
            PixelSize { width: 256, height: 256 },
            &sprites,
            "tex_guid",
            &identities,
        );
        assert!(yaml.contains("m_Y: 214"), "missing flipped Y in:\n{yaml}");
        assert!(yaml.contains("m_X: 0"));
        assert!(yaml.contains("m_Width: 32"));
    }

    #[test]
    fn character_table_index_matches_glyph_table() {
        let sprites = vec![make_sprite("a", 0, 0, 8, 8), make_sprite("b", 16, 0, 8, 8)];
        let idents = vec![
            SpriteIdentity { name: "a".into(), internal_id: 1, sprite_guid: "x".into() },
            SpriteIdentity { name: "b".into(), internal_id: 2, sprite_guid: "y".into() },
        ];
        let yaml = build_sprite_asset("demo", PixelSize { width: 64, height: 64 }, &sprites, "t", &idents);
        assert!(yaml.contains("m_GlyphIndex: 0"));
        assert!(yaml.contains("m_GlyphIndex: 1"));
        assert!(yaml.contains("m_Index: 0"));
        assert!(yaml.contains("m_Index: 1"));
        // Each glyph must have a sprite reference - this is THE field that
        // Unity uses to draw the sprite. Missing means glyph is invisible.
        assert!(yaml.contains("    sprite: {fileID: 1, guid: t, type: 3}"));
        assert!(yaml.contains("    sprite: {fileID: 2, guid: t, type: 3}"));
        // m_ClassDefinitionType is required - TMP serializer omits unknowns
        // but keeps the field at 0 for sprite glyphs. Without it Unity logs a
        // benign warning every time the asset is loaded.
        assert!(yaml.contains("m_ClassDefinitionType: 0"));
    }

    #[test]
    fn yaml_uses_correct_monoscript_guid() {
        let yaml = build_sprite_asset("demo", PixelSize { width: 16, height: 16 }, &[], "t", &[]);
        // The wrong GUID here would surface as "The associated script can not
        // be loaded. Please fix any compile errors..." in Unity - the most
        // common AND most cryptic TMP_SpriteAsset failure mode. Pin it.
        assert!(
            yaml.contains(TMP_SPRITE_ASSET_SCRIPT_GUID),
            "MonoScript GUID drifted from ground truth"
        );
        assert!(yaml.contains("m_Script: {fileID: 11500000, guid: 84a92b25"));
    }

    #[test]
    fn png_meta_emits_single_platform_settings_key() {
        let placed: Vec<PackedSprite> = Vec::new();
        let identities: Vec<SpriteIdentity> = Vec::new();
        let meta = build_png_meta(
            "demo", PixelSize { width: 64, height: 64 }, &placed, &identities,
        );
        let header_count = meta.matches("\n  platformSettings:\n").count();
        assert_eq!(
            header_count, 1,
            "platformSettings: must appear exactly ONCE - duplicate keys cause Unity import to fail. Ground truth: tests/fixtures/unity_ground_truth/test_atlas.png.meta line 69."
        );
        for target in ["DefaultTexturePlatform", "Standalone", "iPhone", "Android"] {
            assert!(
                meta.contains(&format!("buildTarget: {target}")),
                "missing platform block: {target}"
            );
        }
    }

    #[test]
    fn sprite_asset_omits_material_reference() {
        // Ground truth shows material:{fileID:0}. Anything else makes Unity
        // try to load a non-existent .mat sub-asset.
        let yaml = build_sprite_asset("demo", PixelSize { width: 16, height: 16 }, &[], "t", &[]);
        assert!(yaml.contains("material: {fileID: 0}"));
        assert!(!yaml.contains("guid: t, type: 2"), "must not write a material reference");
    }

    #[test]
    fn yaml_has_required_tmp_headers() {
        let yaml = build_sprite_asset("demo", PixelSize { width: 16, height: 16 }, &[], "t", &[]);
        assert!(yaml.starts_with("%YAML 1.1\n%TAG !u! tag:unity3d.com,2011:\n"));
        assert!(yaml.contains("MonoBehaviour:"));
        assert!(yaml.contains("m_SpriteCharacterTable:"));
        assert!(yaml.contains("m_SpriteGlyphTable:"));
        assert!(yaml.contains("spriteSheet: {fileID: 2800000"));
        assert!(yaml.contains("spriteInfoList: []"));
        assert!(yaml.contains("fallbackSpriteAssets: []"));
    }

    #[test]
    fn hash_is_case_sensitive_and_deterministic() {
        // TMP_SpriteAsset uses the case-SENSITIVE hash for sprite names.
        // (TMP also has a case-insensitive variant for tag matching, but
        //  that's not what's stored in m_HashCode.)
        assert_ne!(hash_tmp_name("Smile"), hash_tmp_name("smile"));
        assert_eq!(hash_tmp_name("smile"), hash_tmp_name("smile"));
        assert_ne!(hash_tmp_name("smile"), hash_tmp_name("frown"));
    }

    #[test]
    fn png_meta_marks_texture_as_sprite_multiple() {
        let identities = vec![SpriteIdentity {
            name: "a".into(),
            internal_id: 999,
            sprite_guid: "spr_guid_xx".into(),
        }];
        let placed = vec![make_sprite("a", 0, 0, 8, 8)];
        let meta = build_png_meta("xyz", PixelSize { width: 512, height: 256 }, &placed, &identities);
        assert!(meta.contains("guid: xyz"));
        assert!(meta.contains("textureType: 8"));
        assert!(meta.contains("alphaIsTransparency: 1"));
        assert!(meta.contains("maxTextureSize: 512"));
        // spriteMode 2 is what makes Unity carve sub-sprites - without it the
        // entire .asset is broken because there are no children to reference.
        assert!(meta.contains("spriteMode: 2"));
        // sprite block must be present and keyed by name + internalID.
        assert!(meta.contains("name: a"));
        assert!(meta.contains("internalID: 999"));
        assert!(meta.contains("spriteID: spr_guid_xx"));
        // nameFileIdTable: roundtrip the same internalID under the sprite name.
        assert!(meta.contains("a: 999"));
    }

    #[test]
    fn png_meta_picks_next_legal_max_texture_size() {
        let meta = build_png_meta("g", PixelSize { width: 1500, height: 800 }, &[], &[]);
        assert!(meta.contains("maxTextureSize: 2048"));
        let meta2 = build_png_meta("g", PixelSize { width: 4097, height: 4097 }, &[], &[]);
        assert!(meta2.contains("maxTextureSize: 8192"));
        let meta3 = build_png_meta("g", PixelSize { width: 33, height: 1 }, &[], &[]);
        assert!(meta3.contains("maxTextureSize: 64"));
    }

    #[test]
    fn emit_rejects_atlases_larger_than_unity_max_texture_size() {
        use image::RgbaImage;
        let dir = std::env::temp_dir().join(format!(
            "nebulakit_atlaspro_tmp_oversize_{}", std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let img = RgbaImage::new(4, 4);
        let result = emit(TmpEmitParams {
            atlas_image: &img,
            atlas_size: PixelSize { width: 17000, height: 64 },
            placed: &[],
            output_dir: &dir,
            input_dir: &dir,
            base_name: "oversize",
        });
        let err = result.expect_err(
            "TMP must refuse oversize atlases - Unity would silently downscale and break glyph rects",
        );
        assert_eq!(err.code, "atlaspro_tmp_texture_size_exceeded");
        assert!(!dir.join("oversize.png").exists(),
            "no PNG should be written when guard rejects");
    }

    #[test]
    fn emit_accepts_atlases_at_exact_unity_max_texture_size() {
        use image::RgbaImage;
        let dir = std::env::temp_dir().join(format!(
            "nebulakit_atlaspro_tmp_atmax_{}", std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let img = RgbaImage::new(16, 16);
        let result = emit(TmpEmitParams {
            atlas_image: &img,
            atlas_size: PixelSize { width: 16384, height: 16384 },
            placed: &[],
            output_dir: &dir,
            input_dir: &dir,
            base_name: "atmax",
        });
        assert!(result.is_ok(), "exact 16384 must be allowed - boundary is inclusive");
    }

    #[test]
    fn empty_sprite_list_still_produces_valid_skeleton() {
        let yaml = build_sprite_asset("empty", PixelSize { width: 8, height: 8 }, &[], "t", &[]);
        assert!(yaml.contains("m_SpriteCharacterTable:"));
        assert!(yaml.contains("m_SpriteGlyphTable:"));
    }

    #[test]
    fn full_emit_writes_four_files_no_material() {
        use image::RgbaImage;
        let dir = std::env::temp_dir().join(format!("nebulakit_atlaspro_tmp_v9_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let img = RgbaImage::new(16, 16);
        let sprites = vec![make_sprite("a", 0, 0, 8, 8)];
        let emitted = emit(TmpEmitParams {
            atlas_image: &img,
            atlas_size: PixelSize { width: 16, height: 16 },
            placed: &sprites,
            output_dir: &dir,
            input_dir: &dir,
            base_name: "demo",
        }).unwrap();
        // 4 files now, NOT 6 - we no longer ship .mat / .mat.meta because
        // ground truth shows TMP creates the material at runtime.
        assert_eq!(emitted.len(), 4, "expected 4 files (png, png.meta, asset, asset.meta), got {}", emitted.len());
        for f in &emitted {
            assert!(std::path::Path::new(&f.path).exists(), "missing {}", f.path);
            assert_eq!(f.format, ExportFormat::TmpSpriteAsset);
            assert!(!f.path.ends_with(".mat"), "must not emit .mat: {}", f.path);
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn emit_path_hashed_guid_is_idempotent_and_collision_resistant() {
        use image::RgbaImage;

        let root = std::env::temp_dir().join(format!(
            "nebulakit_atlaspro_tmp_guid_scope_{}",
            std::process::id()
        ));
        let output_a = root.join("out_a");
        let output_b = root.join("out_b");
        let input_a = root.join("input_a");
        let input_b = root.join("input_b");
        std::fs::create_dir_all(&output_a).unwrap();
        std::fs::create_dir_all(&output_b).unwrap();
        std::fs::create_dir_all(&input_a).unwrap();
        std::fs::create_dir_all(&input_b).unwrap();

        let img = RgbaImage::new(8, 8);
        let sprites = vec![make_sprite("a", 0, 0, 8, 8)];

        emit(TmpEmitParams {
            atlas_image: &img,
            atlas_size: PixelSize { width: 8, height: 8 },
            placed: &sprites,
            output_dir: &output_a,
            input_dir: &input_a,
            base_name: "atlas",
        }).unwrap();
        let first_guid = std::fs::read_to_string(output_a.join("atlas.asset.meta")).unwrap();
        let first_guid = first_guid
            .lines()
            .find(|line| line.starts_with("guid: "))
            .unwrap()
            .trim_start_matches("guid: ")
            .to_string();

        emit(TmpEmitParams {
            atlas_image: &img,
            atlas_size: PixelSize { width: 8, height: 8 },
            placed: &sprites,
            output_dir: &output_a,
            input_dir: &input_a,
            base_name: "atlas",
        }).unwrap();
        let second_guid = std::fs::read_to_string(output_a.join("atlas.asset.meta")).unwrap();
        let second_guid = second_guid
            .lines()
            .find(|line| line.starts_with("guid: "))
            .unwrap()
            .trim_start_matches("guid: ")
            .to_string();

        emit(TmpEmitParams {
            atlas_image: &img,
            atlas_size: PixelSize { width: 8, height: 8 },
            placed: &sprites,
            output_dir: &output_b,
            input_dir: &input_b,
            base_name: "atlas",
        }).unwrap();
        let third_guid = std::fs::read_to_string(output_b.join("atlas.asset.meta")).unwrap();
        let third_guid = third_guid
            .lines()
            .find(|line| line.starts_with("guid: "))
            .unwrap()
            .trim_start_matches("guid: ")
            .to_string();

        assert_eq!(first_guid, second_guid, "same input folder + atlas name must keep identical GUIDs");
        assert_ne!(first_guid, third_guid, "different input folders must produce different GUIDs");

        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn internal_ids_are_stable_and_distinct() {
        let a1 = stable_internal_id("atlas", "emoji_red");
        let a2 = stable_internal_id("atlas", "emoji_red");
        let b = stable_internal_id("atlas", "emoji_blue");
        let c = stable_internal_id("other_atlas", "emoji_red");
        assert_eq!(a1, a2, "must be deterministic");
        assert_ne!(a1, b, "different sprite name -> different id");
        assert_ne!(a1, c, "different atlas name -> different id");
    }

    #[test]
    fn three_way_id_consistency_png_meta_matches_asset_yaml() {
        // The whole point of the SpriteIdentity table: png.meta and .asset
        // must reference the same internalID for each sprite, three times
        // each (sprites[].internalID, nameFileIdTable[name], glyph.sprite.fileID).
        // If any drift, Unity shows the glyph as Missing.
        use image::RgbaImage;
        let dir = std::env::temp_dir().join(format!("nebulakit_atlaspro_tmp_three_{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let img = RgbaImage::new(64, 64);
        let sprites = vec![
            make_sprite("emoji_red", 0, 0, 16, 16),
            make_sprite("emoji_blue", 16, 0, 16, 16),
        ];
        emit(TmpEmitParams {
            atlas_image: &img,
            atlas_size: PixelSize { width: 64, height: 64 },
            placed: &sprites,
            output_dir: &dir,
            input_dir: &dir,
            base_name: "consist",
        }).unwrap();
        let png_meta = std::fs::read_to_string(dir.join("consist.png.meta")).unwrap();
        let asset = std::fs::read_to_string(dir.join("consist.asset")).unwrap();
        for name in ["emoji_red", "emoji_blue"] {
            let id = stable_internal_id("consist", name);
            assert!(
                png_meta.contains(&format!("internalID: {id}")),
                "png.meta missing internalID {id} for {name}"
            );
            assert!(
                png_meta.contains(&format!("{name}: {id}")),
                "png.meta nameFileIdTable missing {name}: {id}"
            );
            assert!(
                asset.contains(&format!("sprite: {{fileID: {id},")),
                ".asset glyph missing sprite fileID {id} for {name}"
            );
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
