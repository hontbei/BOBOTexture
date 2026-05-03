// AtlasPacker Pro - 2D rectangle packer (MaxRects + Skyline).

use std::collections::BTreeMap;

use rectangle_pack::{
    contains_smallest_box, pack_rects, volume_heuristic, GroupedRectsToPlace, RectToInsert,
    RectanglePackError, TargetBin,
};

use crate::error::AppError;

use super::model::{LoadedSprite, PaddingOptions, PackedSprite, PixelRect, PixelSize, SpriteOrigin};

const SINGLE_BIN: &str = "atlas-0";

#[derive(Debug)]
pub struct PackOutcome {
    pub atlas_size: PixelSize,
    pub placed: Vec<PackedSprite>,
}

/// Legacy rectangle-pack implementation kept for `pipeline.rs`.
///
/// Pack a list of preprocessed sprites into a single bin of `max_width x
/// max_height`. Returns final placements (top-left origin, in the atlas
/// coordinate space) plus the tight atlas bounds.
///
/// `extrude` must match the value passed to `preprocess::load_and_preprocess`
/// because the packer needs to strip it from the reported `frame`.
///
/// When `allow_rotation` is true, every non-square sprite is rotated to
/// portrait orientation (height >= width) before being handed to rectangle-pack.
/// Rotated placements are flagged via `PackedSprite.rotated` so the compositor
/// applies the same 90 deg CW rotation when blitting. This is a single-pass
/// heuristic - cheaper than try-both per sprite and empirically gives a meaningful
/// density gain on mixed orientation atlases without the combinatorial cost.
pub fn pack(
    sprites: &[LoadedSprite],
    max_width: u32,
    max_height: u32,
    padding: PaddingOptions,
    extrude: u32,
    allow_rotation: bool,
) -> Result<PackOutcome, AppError> {
    if sprites.is_empty() {
        return Ok(PackOutcome {
            atlas_size: PixelSize { width: 1, height: 1 },
            placed: Vec::new(),
        });
    }
    if max_width == 0 || max_height == 0 {
        return Err(AppError::new("atlaspro_pack_size", "atlas size must be > 0"));
    }

    let usable_w = max_width.checked_sub(padding.border_padding.saturating_mul(2)).ok_or_else(|| {
        AppError::new("atlaspro_pack_size", "border padding exceeds atlas width")
    })?;
    let usable_h = max_height.checked_sub(padding.border_padding.saturating_mul(2)).ok_or_else(|| {
        AppError::new("atlaspro_pack_size", "border padding exceeds atlas height")
    })?;
    if usable_w == 0 || usable_h == 0 {
        return Err(AppError::new("atlaspro_pack_size", "border padding consumes the entire atlas"));
    }

    let mut rotated_flags: Vec<bool> = Vec::with_capacity(sprites.len());

    let mut grouped = GroupedRectsToPlace::<usize, ()>::new();
    for (idx, sprite) in sprites.iter().enumerate() {
        let (buf_w, buf_h) = sprite.pixels.dimensions();
        let rotate = allow_rotation && buf_w > buf_h;
        rotated_flags.push(rotate);
        let (oriented_w, oriented_h) = if rotate { (buf_h, buf_w) } else { (buf_w, buf_h) };

        let inflated_w = oriented_w.checked_add(padding.shape_padding).ok_or_else(|| {
            AppError::new("atlaspro_pack_overflow", format!("{}: width overflow", sprite.name))
        })?;
        let inflated_h = oriented_h.checked_add(padding.shape_padding).ok_or_else(|| {
            AppError::new("atlaspro_pack_overflow", format!("{}: height overflow", sprite.name))
        })?;
        if inflated_w > usable_w || inflated_h > usable_h {
            return Err(AppError::new(
                "atlaspro_pack_fit",
                format!(
                    "{} ({}x{}, padded {}x{}) is larger than the atlas usable area {}x{}",
                    sprite.name, buf_w, buf_h, inflated_w, inflated_h, usable_w, usable_h
                ),
            ));
        }
        grouped.push_rect(idx, None, RectToInsert::new(inflated_w, inflated_h, 1));
    }

    let mut bins = BTreeMap::new();
    bins.insert(SINGLE_BIN, TargetBin::new(usable_w, usable_h, 1));

    let result = pack_rects(&grouped, &mut bins, &volume_heuristic, &contains_smallest_box).map_err(|err| {
        match err {
            RectanglePackError::NotEnoughBinSpace => AppError::new(
                "atlaspro_pack_fit",
                format!("sprites do not fit into a single {max_width}x{max_height} atlas"),
            ),
        }
    })?;

    let mut placed: Vec<PackedSprite> = Vec::with_capacity(sprites.len());
    let mut atlas_w = 1u32;
    let mut atlas_h = 1u32;

    let mut entries: Vec<(usize, &rectangle_pack::PackedLocation)> =
        result.packed_locations().iter().map(|(id, (_, loc))| (*id, loc)).collect();
    entries.sort_by_key(|(id, _)| *id);

    for (idx, loc) in entries {
        let sprite = &sprites[idx];
        let (buf_w, buf_h) = sprite.pixels.dimensions();
        let rotated = rotated_flags[idx];
        let (atlas_buf_w, atlas_buf_h) = if rotated { (buf_h, buf_w) } else { (buf_w, buf_h) };

        let extruded_x = loc.x() + padding.border_padding;
        let extruded_y = loc.y() + padding.border_padding;

        let frame_x = extruded_x + extrude;
        let frame_y = extruded_y + extrude;
        let frame_w = atlas_buf_w.saturating_sub(extrude.saturating_mul(2));
        let frame_h = atlas_buf_h.saturating_sub(extrude.saturating_mul(2));

        let edge_x = extruded_x + atlas_buf_w;
        let edge_y = extruded_y + atlas_buf_h;
        if edge_x > atlas_w {
            atlas_w = edge_x;
        }
        if edge_y > atlas_h {
            atlas_h = edge_y;
        }

        let (reported_source_size, reported_source_frame) = match sprite.origin {
            SpriteOrigin::UnitySubSprite => (
                PixelSize { width: sprite.sub_rect.width, height: sprite.sub_rect.height },
                PixelRect::new(
                    sprite.trimmed_rect.x.saturating_sub(sprite.sub_rect.x),
                    sprite.trimmed_rect.y.saturating_sub(sprite.sub_rect.y),
                    sprite.trimmed_rect.width,
                    sprite.trimmed_rect.height,
                ),
            ),
            SpriteOrigin::File => (sprite.source_size, sprite.trimmed_rect),
        };

        placed.push(PackedSprite {
            id: sprite.id.clone(),
            name: sprite.name.clone(),
            frame: PixelRect::new(frame_x, frame_y, frame_w.max(1), frame_h.max(1)),
            source_frame: reported_source_frame,
            source_size: reported_source_size,
            rotated,
            trimmed: sprite.trimmed,
            unity: sprite.unity.clone(),
        });
    }

    let tight_w = (atlas_w + padding.border_padding).max(1);
    let tight_h = (atlas_h + padding.border_padding).max(1);
    let pot_w = next_power_of_two(tight_w).min(max_width);
    let pot_h = next_power_of_two(tight_h).min(max_height);
    let atlas_size = PixelSize { width: pot_w, height: pot_h };
    Ok(PackOutcome { atlas_size, placed })
}

#[derive(Clone, Copy)]
struct FreeRect { x: u32, y: u32, w: u32, h: u32 }

/// 4-way split of a free rectangle against a placed rectangle.
/// Returns Some(vec of new free rects) if they overlap, None if no overlap.
fn split_free_node(free_rect: FreeRect, used: FreeRect) -> Option<Vec<FreeRect>> {
    if used.x >= free_rect.x + free_rect.w
        || used.x + used.w <= free_rect.x
        || used.y >= free_rect.y + free_rect.h
        || used.y + used.h <= free_rect.y
    {
        return None;
    }
    let mut result = Vec::with_capacity(4);
    // Left
    if used.x > free_rect.x {
        result.push(FreeRect { x: free_rect.x, y: free_rect.y, w: used.x - free_rect.x, h: free_rect.h });
    }
    // Right
    if used.x + used.w < free_rect.x + free_rect.w {
        result.push(FreeRect { x: used.x + used.w, y: free_rect.y, w: free_rect.x + free_rect.w - used.x - used.w, h: free_rect.h });
    }
    // Top
    if used.y > free_rect.y {
        result.push(FreeRect { x: free_rect.x, y: free_rect.y, w: free_rect.w, h: used.y - free_rect.y });
    }
    // Bottom
    if used.y + used.h < free_rect.y + free_rect.h {
        result.push(FreeRect { x: free_rect.x, y: used.y + used.h, w: free_rect.w, h: free_rect.y + free_rect.h - used.y - used.h });
    }
    Some(result)
}

fn pack_max_rects(
    sprites: &[LoadedSprite],
    max_width: u32,
    max_height: u32,
    padding: PaddingOptions,
    extrude: u32,
    allow_rotation: bool,
) -> Result<PackOutcome, AppError> {
    if sprites.is_empty() {
        return Ok(PackOutcome {
            atlas_size: PixelSize { width: 1, height: 1 },
            placed: Vec::new(),
        });
    }
    if max_width == 0 || max_height == 0 {
        return Err(AppError::new("atlaspro_pack_size", "atlas size must be > 0"));
    }

    let usable_w = max_width.checked_sub(padding.border_padding.saturating_mul(2)).ok_or_else(|| {
        AppError::new("atlaspro_pack_size", "border padding exceeds atlas width")
    })?;
    let usable_h = max_height.checked_sub(padding.border_padding.saturating_mul(2)).ok_or_else(|| {
        AppError::new("atlaspro_pack_size", "border padding exceeds atlas height")
    })?;
    if usable_w == 0 || usable_h == 0 {
        return Err(AppError::new("atlaspro_pack_size", "border padding consumes the entire atlas"));
    }

    // Build entries sorted by padded area descending for best packing density.
    struct Item {
        idx: usize,
        pw: u32,
        ph: u32,
        orig_w: u32,
        orig_h: u32,
    }
    let mut items: Vec<Item> = sprites
        .iter()
        .enumerate()
        .map(|(i, sprite)| {
            let (ow, oh) = sprite.pixels.dimensions();
            let pw = ow.checked_add(padding.shape_padding).unwrap_or(u32::MAX);
            let ph = oh.checked_add(padding.shape_padding).unwrap_or(u32::MAX);
            if pw > usable_w || ph > usable_h {
                return Err(AppError::new(
                    "atlaspro_pack_fit",
                    format!("{} ({}x{}, padded {}x{}) exceeds usable area", sprite.name, ow, oh, pw, ph),
                ));
            }
            Ok(Item { idx: i, pw, ph, orig_w: ow, orig_h: oh })
        })
        .collect::<Result<Vec<_>, _>>()?;

    items.sort_by(|a, b| (b.pw as u64 * b.ph as u64).cmp(&(a.pw as u64 * a.ph as u64)));

    let mut free_rects: Vec<FreeRect> = vec![FreeRect { x: 0, y: 0, w: usable_w, h: usable_h }];
    let mut placements: Vec<(usize, u32, u32, bool)> = Vec::with_capacity(items.len()); // (idx, x, y, rotated)

    for item in &items {
        let mut best_score1 = u32::MAX;
        let mut best_score2 = u32::MAX;
        let mut best: Option<(usize, FreeRect, bool)> = None; // (free_idx, placed_rect, rotated)

        // Try upright, then rotated if square != and rotation allowed.
        let tries: [(u32, u32, bool); 2] = [
            (item.pw, item.ph, false),
            (if allow_rotation && item.orig_w != item.orig_h { item.ph } else { 0 }, if allow_rotation && item.orig_w != item.orig_h { item.pw } else { 0 }, true),
        ];

        for (pw, ph, rot) in &tries {
            if *pw == 0 || *ph == 0 || *pw > usable_w || *ph > usable_h {
                continue;
            }
            for (fi, fr) in free_rects.iter().enumerate() {
                if fr.w < *pw || fr.h < *ph {
                    continue;
                }
                let leftover_w = fr.w - *pw;
                let leftover_h = fr.h - *ph;
                let short = leftover_w.min(leftover_h);
                let long = leftover_w.max(leftover_h);
                if short < best_score1 || (short == best_score1 && long < best_score2) {
                    best_score1 = short;
                    best_score2 = long;
                    best = Some((fi, FreeRect { x: fr.x, y: fr.y, w: *pw, h: *ph }, *rot));
                }
            }
        }

        let (_, placed, rotated) = best.ok_or_else(|| {
            AppError::new("atlaspro_pack_fit", format!("{} cannot be placed in {}x{} atlas", sprites[item.idx].name, usable_w, usable_h))
        })?;

        // Place the sprite by splitting ALL overlapping free rects, not just
        // the one selected. This matches HeituTexturePacker's MaxRects behaviour
        // and prevents overlapping placements when free rects intersect.
        let mut new_free = Vec::with_capacity(free_rects.len() + 4);
        for fr in &free_rects {
            let splits = split_free_node(*fr, placed);
            match splits {
                Some(rects) => new_free.extend(rects),
                None => new_free.push(*fr),
            }
        }
        free_rects = new_free;

        // Prune: remove any free rect fully contained within another.
        let mut i = 0;
        while i < free_rects.len() {
            let a = free_rects[i];
            let mut contained = false;
            let mut j = 0;
            while j < free_rects.len() {
                if i == j { j += 1; continue; }
                let b = free_rects[j];
                if a.x >= b.x && a.y >= b.y
                    && a.x + a.w <= b.x + b.w
                    && a.y + a.h <= b.y + b.h
                {
                    contained = true;
                    break;
                }
                j += 1;
            }
            if contained {
                free_rects.remove(i);
            } else {
                i += 1;
            }
        }

        placements.push((item.idx, placed.x, placed.y, rotated));
    }

    // Convert to PackedSprite outputs (same logic as legacy `pack`)
    let mut rotated_flags: Vec<bool> = vec![false; sprites.len()];
    let mut placed: Vec<PackedSprite> = Vec::with_capacity(sprites.len());
    let mut atlas_w = 1u32;
    let mut atlas_h = 1u32;

    // Sort placements back to original sprite index order for deterministic output
    placements.sort_by_key(|(idx, _, _, _)| *idx);

    for (idx, px, py, rotated) in &placements {
        let sprite = &sprites[*idx];
        let (buf_w, buf_h) = sprite.pixels.dimensions();
        rotated_flags[*idx] = *rotated;
        let (atlas_buf_w, atlas_buf_h) = if *rotated { (buf_h, buf_w) } else { (buf_w, buf_h) };

        let extruded_x = px + padding.border_padding;
        let extruded_y = py + padding.border_padding;

        let frame_x = extruded_x + extrude;
        let frame_y = extruded_y + extrude;
        let frame_w = atlas_buf_w.saturating_sub(extrude.saturating_mul(2));
        let frame_h = atlas_buf_h.saturating_sub(extrude.saturating_mul(2));

        let edge_x = extruded_x + atlas_buf_w;
        let edge_y = extruded_y + atlas_buf_h;
        if edge_x > atlas_w { atlas_w = edge_x; }
        if edge_y > atlas_h { atlas_h = edge_y; }

        let (reported_source_size, reported_source_frame) = match sprite.origin {
            SpriteOrigin::UnitySubSprite => (
                PixelSize { width: sprite.sub_rect.width, height: sprite.sub_rect.height },
                PixelRect::new(
                    sprite.trimmed_rect.x.saturating_sub(sprite.sub_rect.x),
                    sprite.trimmed_rect.y.saturating_sub(sprite.sub_rect.y),
                    sprite.trimmed_rect.width,
                    sprite.trimmed_rect.height,
                ),
            ),
            SpriteOrigin::File => (sprite.source_size, sprite.trimmed_rect),
        };

        placed.push(PackedSprite {
            id: sprite.id.clone(),
            name: sprite.name.clone(),
            frame: PixelRect::new(frame_x, frame_y, frame_w.max(1), frame_h.max(1)),
            source_frame: reported_source_frame,
            source_size: reported_source_size,
            rotated: *rotated,
            trimmed: sprite.trimmed,
            unity: sprite.unity.clone(),
        });
    }

    let tight_w = (atlas_w + padding.border_padding).max(1);
    let tight_h = (atlas_h + padding.border_padding).max(1);
    let pot_w = next_power_of_two(tight_w).min(max_width);
    let pot_h = next_power_of_two(tight_h).min(max_height);
    let atlas_size = PixelSize { width: pot_w, height: pot_h };
    Ok(PackOutcome { atlas_size, placed })
}

/// Smallest power-of-two >= `n`. Returns 1 for n=0, n itself if already POT.
fn next_power_of_two(n: u32) -> u32 {
    if n <= 1 {
        return 1;
    }
    let mut p = 1u32;
    while p < n {
        p <<= 1;
    }
    p
}
/// Try progressively larger power-of-two squares (256, 512, 1024, 2048, 4096)
/// up to `max_dim`. Returns the first square that fits all sprites, or an
/// error if none can fit even at the largest size <= max_dim.
pub fn pack_auto_square(
    sprites: &[LoadedSprite],
    max_dim: u32,
    algorithm: super::model::PackAlgorithm,
    padding: PaddingOptions,
    extrude: u32,
    allow_rotation: bool,
) -> Result<PackOutcome, AppError> {
    const POT_SIZES: [u32; 5] = [256, 512, 1024, 2048, 4096];

    let mut last_err: Option<AppError> = None;
    for &size in &POT_SIZES {
        if size > max_dim {
            break;
        }
        match pack_with_algorithm(sprites, size, size, algorithm, padding, extrude, allow_rotation) {
            Ok(outcome) => return Ok(outcome),
            Err(e) => last_err = Some(e),
        }
    }

    Err(last_err.unwrap_or_else(|| AppError::new(
        "atlaspro_auto_size",
        format!("sprites cannot fit in any square atlas up to {max_dim}x{max_dim}"),
    )))
}

pub fn pack_with_algorithm(
    sprites: &[LoadedSprite],
    max_width: u32,
    max_height: u32,
    algorithm: super::model::PackAlgorithm,
    padding: PaddingOptions,
    extrude: u32,
    allow_rotation: bool,
) -> Result<PackOutcome, AppError> {
    use super::model::PackAlgorithm;

    match algorithm {
        PackAlgorithm::Skyline => pack(
            sprites,
            max_width,
            max_height,
            padding,
            extrude,
            allow_rotation,
        ),
        PackAlgorithm::MaxRects => pack_max_rects(
            sprites,
            max_width,
            max_height,
            padding,
            extrude,
            allow_rotation,
        ),
        other => Err(AppError::new(
            "atlaspro_pack_algorithm_unsupported",
            format!(
                "packing algorithm {other:?} is not implemented yet; use Skyline or MaxRects"
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::atlaspro::model::{PixelRect, PixelSize, SpriteOrigin};
    use image::{Rgba, RgbaImage};

    fn fake(id: &str, w: u32, h: u32) -> LoadedSprite {
        LoadedSprite {
            id: id.to_string(),
            name: id.to_string(),
            origin: SpriteOrigin::File,
            source_path: std::path::PathBuf::from(format!("/tmp/{id}.png")),
            source_size: PixelSize { width: w, height: h },
            trimmed_rect: PixelRect::new(0, 0, w, h),
            sub_rect: PixelRect::new(0, 0, w, h),
            trimmed: false,
            pixels: RgbaImage::from_pixel(w, h, Rgba([255, 0, 0, 255])),
            unity: None,
        }
    }

    #[test]
    fn empty_input_returns_empty_atlas() {
        let outcome = pack(&[], 256, 256, PaddingOptions::default(), 0, false).unwrap();
        assert_eq!(outcome.placed.len(), 0);
        assert_eq!(outcome.atlas_size, PixelSize { width: 1, height: 1 });
    }

    #[test]
    fn packs_two_squares_without_padding() {
        let sprites = vec![fake("a", 32, 32), fake("b", 32, 32)];
        let outcome = pack(&sprites, 64, 64, PaddingOptions::default(), 0, false).unwrap();
        assert_eq!(outcome.placed.len(), 2);
        for p in &outcome.placed {
            assert_eq!(p.frame.width, 32);
            assert_eq!(p.frame.height, 32);
            assert!(!p.rotated);
            assert!(!p.trimmed);
        }
    }

    #[test]
    fn shape_padding_creates_gap_between_sprites() {
        let sprites = vec![fake("a", 32, 32), fake("b", 32, 32)];
        let padding = PaddingOptions { shape_padding: 4, border_padding: 0, extrude: 0 };
        let outcome = pack(&sprites, 128, 128, padding, 0, false).unwrap();
        assert_eq!(outcome.placed.len(), 2);
        let a = outcome.placed.iter().find(|p| p.name == "a").unwrap();
        let b = outcome.placed.iter().find(|p| p.name == "b").unwrap();
        let horizontal_gap = (a.frame.x as i64 - b.frame.x as i64).unsigned_abs() as u32;
        let vertical_gap = (a.frame.y as i64 - b.frame.y as i64).unsigned_abs() as u32;
        assert!(horizontal_gap >= 32 + 4 || vertical_gap >= 32 + 4,
            "expected at least one axis to leave the padded distance, got h={horizontal_gap} v={vertical_gap}");
    }

    #[test]
    fn border_padding_shifts_placement_inward() {
        let sprites = vec![fake("only", 16, 16)];
        let padding = PaddingOptions { shape_padding: 0, border_padding: 8, extrude: 0 };
        let outcome = pack(&sprites, 64, 64, padding, 0, false).unwrap();
        assert_eq!(outcome.placed[0].frame.x, 8);
        assert_eq!(outcome.placed[0].frame.y, 8);
    }

    #[test]
    fn returns_pack_fit_error_when_oversized() {
        let sprites = vec![fake("big", 100, 100)];
        let err = pack(&sprites, 64, 64, PaddingOptions::default(), 0, false).unwrap_err();
        assert_eq!(err.code, "atlaspro_pack_fit");
    }

    #[test]
    fn returns_pack_fit_error_when_total_exceeds() {
        let sprites = vec![fake("a", 50, 50), fake("b", 50, 50), fake("c", 50, 50)];
        let err = pack(&sprites, 64, 64, PaddingOptions::default(), 0, false).unwrap_err();
        assert_eq!(err.code, "atlaspro_pack_fit");
    }

    #[test]
    fn rotation_flag_set_for_wide_sprites_when_allowed() {
        let sprites = vec![fake("wide", 32, 8), fake("square", 16, 16)];
        let outcome = pack(&sprites, 64, 64, PaddingOptions::default(), 0, true).unwrap();
        let wide = outcome.placed.iter().find(|p| p.name == "wide").unwrap();
        let square = outcome.placed.iter().find(|p| p.name == "square").unwrap();
        assert!(wide.rotated, "wide sprite should rotate to portrait");
        assert!(!square.rotated, "square sprite should never rotate");
        assert_eq!(wide.frame.width, 8);
        assert_eq!(wide.frame.height, 32);
    }

    #[test]
    fn rotation_disabled_keeps_orientation() {
        let sprites = vec![fake("wide", 32, 8)];
        let outcome = pack(&sprites, 64, 64, PaddingOptions::default(), 0, false).unwrap();
        let wide = &outcome.placed[0];
        assert!(!wide.rotated);
        assert_eq!(wide.frame.width, 32);
        assert_eq!(wide.frame.height, 8);
    }

    #[test]
    fn rotation_enables_fit_that_would_otherwise_fail() {
        let sprites = vec![fake("strip", 60, 6)];
        let outcome = pack(&sprites, 16, 64, PaddingOptions::default(), 0, true).unwrap();
        assert!(outcome.placed[0].rotated);
        assert_eq!(outcome.placed[0].frame.width, 6);
        assert_eq!(outcome.placed[0].frame.height, 60);
    }

    #[test]
    fn unity_sub_sprite_reports_local_coordinates() {
        let parent_size = PixelSize { width: 256, height: 256 };
        let sub_rect = PixelRect::new(40, 60, 32, 32);
        let trimmed_rect = PixelRect::new(44, 64, 24, 24);

        let sprite = LoadedSprite {
            id: "u".into(),
            name: "u".into(),
            origin: SpriteOrigin::UnitySubSprite,
            source_path: std::path::PathBuf::from("/tmp/parent.png"),
            source_size: parent_size,
            trimmed_rect,
            sub_rect,
            trimmed: true,
            pixels: RgbaImage::from_pixel(24, 24, Rgba([0, 0, 0, 255])),
            unity: None,
        };
        let outcome = pack(&[sprite], 128, 128, PaddingOptions::default(), 0, false).unwrap();
        let p = &outcome.placed[0];
        assert_eq!(p.source_size, PixelSize { width: 32, height: 32 },
            "source_size must be sub-sprite W/H, not parent PNG size");
        assert_eq!(p.source_frame, PixelRect::new(4, 4, 24, 24),
            "source_frame must be relative to sub-sprite top-left");
    }

    #[test]
    fn file_origin_keeps_full_image_coordinates() {
        let sprite = fake("f", 64, 64);
        let outcome = pack(&[sprite], 128, 128, PaddingOptions::default(), 0, false).unwrap();
        let p = &outcome.placed[0];
        assert_eq!(p.source_size, PixelSize { width: 64, height: 64 });
        assert_eq!(p.source_frame, PixelRect::new(0, 0, 64, 64));
    }

    #[test]
    fn file_origin_with_nonzero_trimmed_rect_stays_full_image_relative() {
        let mut sprite = fake("trimmed", 64, 64);
        sprite.trimmed = true;
        sprite.trimmed_rect = PixelRect::new(10, 12, 24, 20);
        sprite.pixels = image::RgbaImage::from_pixel(24, 20, image::Rgba([0, 0, 0, 255]));

        let outcome = pack(&[sprite], 128, 128, PaddingOptions::default(), 0, false).unwrap();
        let p = &outcome.placed[0];
        assert_eq!(p.source_size, PixelSize { width: 64, height: 64 },
            "File origin must report ORIGINAL PNG size, not trimmed size");
        assert_eq!(p.source_frame, PixelRect::new(10, 12, 24, 20),
            "File origin trimmed_rect must remain full-image-relative");
        assert!(p.trimmed);
    }

    #[test]
    fn next_pot_handles_edge_cases() {
        assert_eq!(next_power_of_two(0), 1);
        assert_eq!(next_power_of_two(1), 1);
        assert_eq!(next_power_of_two(2), 2);
        assert_eq!(next_power_of_two(3), 4);
        assert_eq!(next_power_of_two(64), 64);
        assert_eq!(next_power_of_two(65), 128);
        assert_eq!(next_power_of_two(1023), 1024);
    }

    #[test]
    fn atlas_size_is_rounded_up_to_power_of_two_independently_per_axis() {
        let sprites = vec![fake("a", 33, 17)];
        let outcome = pack(&sprites, 256, 256, PaddingOptions::default(), 0, false).unwrap();
        assert_eq!(outcome.atlas_size, PixelSize { width: 64, height: 32 },
            "tight 33x17 must round up to POT 64x32, not square");
    }

    #[test]
    fn pot_rounding_capped_at_max_dimensions() {
        let sprites = vec![fake("a", 100, 100)];
        let outcome = pack(&sprites, 100, 100, PaddingOptions::default(), 0, false).unwrap();
        assert_eq!(outcome.atlas_size.width, 100,
            "POT cap must respect user max_width even when not POT itself");
        assert_eq!(outcome.atlas_size.height, 100);
    }

}
