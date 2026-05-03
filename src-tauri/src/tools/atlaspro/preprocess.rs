// AtlasPacker Pro - image preprocessing.
//
// Wave 2 implements:
//   - alpha trimming (removes fully-transparent margin)
//   - sub-rect cropping (Unity sub-sprite extraction post-Y-flip)
//   - extrude (pixel replication around each sprite to prevent bleed)
//
// Padding (gap between sprites) and rotation are *packer concerns*, not pixel
// concerns; they are NOT applied here. This module hands the packer a clean,
// extruded RGBA buffer plus its trimmed bounding box.

use image::{GenericImageView, Rgba, RgbaImage};

use crate::error::AppError;

use super::model::{LoadedSprite, PixelRect, PixelSize, SpriteOrigin, SpriteSource, TrimMode, UnitySpriteMeta};

/// Load one sprite source from disk and run the requested preprocessing.
///
/// `alpha_threshold` matches the request setting: pixels with `alpha <=
/// threshold` are treated as fully transparent for trim purposes.
///
/// `extrude` is applied AFTER trim. The returned `pixels` buffer is `(rect.w +
/// 2*extrude) x (rect.h + 2*extrude)` with the inner region holding the
/// trimmed sprite and the outer ring holding replicated edge pixels.
pub fn load_and_preprocess(
    source: &SpriteSource,
    trim: TrimMode,
    alpha_threshold: u8,
    extrude: u32,
) -> Result<LoadedSprite, AppError> {
    let full = image::open(&source.source_path)
        .map_err(|err| AppError::new("atlaspro_decode", format!("{}: {}", source.source_path, err)))?
        .to_rgba8();
    let (full_w, full_h) = full.dimensions();
    let source_size = PixelSize { width: full_w, height: full_h };

    let sub = clip_sub_rect(&full, source.sub_rect, &source.source_path)?;

    let (trimmed_pixels, trimmed_rect_within_sub, trimmed) = match trim {
        TrimMode::None => {
            let rect = PixelRect::new(0, 0, sub.width(), sub.height());
            (sub, rect, false)
        }
        TrimMode::Alpha => alpha_trim(sub, alpha_threshold),
    };

    // trimmed_rect is expressed in the ORIGINAL source-image coordinate space
    // (top-left); exporters use it verbatim as spriteSourceSize.
    let trimmed_rect = PixelRect::new(
        source.sub_rect.x + trimmed_rect_within_sub.x,
        source.sub_rect.y + trimmed_rect_within_sub.y,
        trimmed_rect_within_sub.width,
        trimmed_rect_within_sub.height,
    );

    let pixels = if extrude > 0 && !trimmed_pixels.is_empty() {
        apply_extrude(&trimmed_pixels, extrude)
    } else {
        trimmed_pixels
    };

    Ok(LoadedSprite {
        id: source.id.clone(),
        name: source.name.clone(),
        origin: source.origin.clone(),
        source_path: source.source_path.clone().into(),
        source_size,
        trimmed_rect,
        sub_rect: source.sub_rect,
        trimmed,
        pixels,
        unity: source.unity.clone(),
    })
}

/// Crop the sub-rectangle out of a full image, validating bounds.
fn clip_sub_rect(full: &RgbaImage, rect: PixelRect, label: &str) -> Result<RgbaImage, AppError> {
    let (full_w, full_h) = full.dimensions();
    if rect.is_empty() {
        return Err(AppError::new("atlaspro_subrect", format!("{label}: sub-rect is empty")));
    }
    if rect.x + rect.width > full_w || rect.y + rect.height > full_h {
        return Err(AppError::new(
            "atlaspro_subrect",
            format!(
                "{label}: sub-rect ({},{},{}x{}) exceeds image bounds {}x{}",
                rect.x, rect.y, rect.width, rect.height, full_w, full_h
            ),
        ));
    }
    Ok(full.view(rect.x, rect.y, rect.width, rect.height).to_image())
}

/// Crop fully-transparent margin around the sprite. Returns the trimmed image,
/// the rectangle inside the input image that survived, and whether anything
/// was actually trimmed.
fn alpha_trim(input: RgbaImage, threshold: u8) -> (RgbaImage, PixelRect, bool) {
    let (w, h) = input.dimensions();
    if w == 0 || h == 0 {
        return (input, PixelRect::new(0, 0, w, h), false);
    }

    let mut min_x = w;
    let mut min_y = h;
    let mut max_x: i64 = -1;
    let mut max_y: i64 = -1;

    for y in 0..h {
        for x in 0..w {
            let alpha = input.get_pixel(x, y).0[3];
            if alpha > threshold {
                if x < min_x { min_x = x; }
                if y < min_y { min_y = y; }
                if (x as i64) > max_x { max_x = x as i64; }
                if (y as i64) > max_y { max_y = y as i64; }
            }
        }
    }

    // Fully transparent image: keep a 1x1 placeholder so the packer doesn't
    // see a zero-area sprite, but flag it as trimmed.
    if max_x < 0 {
        let mut placeholder = RgbaImage::new(1, 1);
        placeholder.put_pixel(0, 0, Rgba([0, 0, 0, 0]));
        return (placeholder, PixelRect::new(0, 0, 1, 1), true);
    }

    let new_w = (max_x as u32) - min_x + 1;
    let new_h = (max_y as u32) - min_y + 1;
    let trimmed = new_w != w || new_h != h;
    if !trimmed {
        return (input, PixelRect::new(0, 0, w, h), false);
    }
    let cropped = input.view(min_x, min_y, new_w, new_h).to_image();
    (cropped, PixelRect::new(min_x, min_y, new_w, new_h), true)
}

/// Replicate edge pixels outward by `extrude` pixels in all four directions.
/// Corners are filled with the corresponding corner pixel of the input.
fn apply_extrude(input: &RgbaImage, extrude: u32) -> RgbaImage {
    let (w, h) = input.dimensions();
    let new_w = w + 2 * extrude;
    let new_h = h + 2 * extrude;
    let mut out = RgbaImage::new(new_w, new_h);

    for y in 0..new_h {
        for x in 0..new_w {
            let src_x = clamp_to_range(x as i64 - extrude as i64, 0, w as i64 - 1) as u32;
            let src_y = clamp_to_range(y as i64 - extrude as i64, 0, h as i64 - 1) as u32;
            out.put_pixel(x, y, *input.get_pixel(src_x, src_y));
        }
    }
    out
}

fn clamp_to_range(v: i64, lo: i64, hi: i64) -> i64 {
    if v < lo { lo } else if v > hi { hi } else { v }
}

/// Convenience: build a SpriteSource representing the whole image. Used by the
/// scanner for standalone files where there is no Unity .meta.
pub fn whole_image_source(
    id: String,
    name: String,
    source_path: String,
    width: u32,
    height: u32,
) -> SpriteSource {
    SpriteSource {
        id,
        name,
        source_path,
        origin: SpriteOrigin::File,
        sub_rect: PixelRect::new(0, 0, width, height),
        unity: None::<UnitySpriteMeta>,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::Rgba;

    fn build_image(w: u32, h: u32, fill: Rgba<u8>) -> RgbaImage {
        let mut img = RgbaImage::new(w, h);
        for y in 0..h {
            for x in 0..w {
                img.put_pixel(x, y, fill);
            }
        }
        img
    }

    #[test]
    fn trim_keeps_opaque_image_unchanged() {
        let img = build_image(4, 4, Rgba([255, 0, 0, 255]));
        let (out, rect, trimmed) = alpha_trim(img.clone(), 0);
        assert!(!trimmed);
        assert_eq!(rect, PixelRect::new(0, 0, 4, 4));
        assert_eq!(out.dimensions(), (4, 4));
    }

    #[test]
    fn trim_removes_transparent_border() {
        let mut img = build_image(4, 4, Rgba([0, 0, 0, 0]));
        img.put_pixel(1, 2, Rgba([255, 255, 255, 255]));
        let (out, rect, trimmed) = alpha_trim(img, 0);
        assert!(trimmed);
        assert_eq!(rect, PixelRect::new(1, 2, 1, 1));
        assert_eq!(out.dimensions(), (1, 1));
    }

    #[test]
    fn trim_fully_transparent_collapses_to_placeholder() {
        let img = build_image(4, 4, Rgba([0, 0, 0, 0]));
        let (out, rect, trimmed) = alpha_trim(img, 0);
        assert!(trimmed);
        assert_eq!(rect, PixelRect::new(0, 0, 1, 1));
        assert_eq!(out.dimensions(), (1, 1));
    }

    #[test]
    fn extrude_replicates_edges() {
        let mut img = RgbaImage::new(2, 2);
        img.put_pixel(0, 0, Rgba([10, 0, 0, 255]));
        img.put_pixel(1, 0, Rgba([20, 0, 0, 255]));
        img.put_pixel(0, 1, Rgba([30, 0, 0, 255]));
        img.put_pixel(1, 1, Rgba([40, 0, 0, 255]));
        let out = apply_extrude(&img, 1);
        assert_eq!(out.dimensions(), (4, 4));
        assert_eq!(out.get_pixel(0, 0).0[0], 10);
        assert_eq!(out.get_pixel(3, 0).0[0], 20);
        assert_eq!(out.get_pixel(0, 3).0[0], 30);
        assert_eq!(out.get_pixel(3, 3).0[0], 40);
        assert_eq!(out.get_pixel(1, 1).0[0], 10);
        assert_eq!(out.get_pixel(2, 2).0[0], 40);
    }

    #[test]
    fn whole_image_source_uses_full_bounds() {
        let s = whole_image_source("id1".into(), "icon".into(), "/tmp/icon.png".into(), 32, 64);
        assert_eq!(s.sub_rect, PixelRect::new(0, 0, 32, 64));
        assert_eq!(s.origin, SpriteOrigin::File);
        assert!(s.unity.is_none());
    }
}
