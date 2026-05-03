// AtlasPacker Pro - resolution variants.
//
// Wave 3: produce multiple-resolution copies of a finished atlas using high
// quality (Lanczos3) resampling. Each variant carries a parallel PackedSprite
// list whose pixel coordinates are scaled in lock-step so exporters can emit
// per-resolution sidecars (TexturePacker @0.5x convention).
//
// Floating point note: scale * pixel coordinates are rounded with .round() to
// the nearest integer because truncation produces visible 1-pixel seams in
// downscaled atlases (sprite bottom-edges drift up by one pixel and bleed
// transparent rows into linear sampling). max(1) ensures degenerate widths
// stay non-zero so exporters do not trip on zero-area frames.

use image::{imageops::FilterType, RgbaImage};

use crate::error::AppError;

use super::model::{PackedSprite, PixelRect, PixelSize, ScaleVariant};

#[derive(Debug)]
pub struct VariantOutput {
    pub suffix: String,
    pub scale: f32,
    pub image: RgbaImage,
    pub placed: Vec<PackedSprite>,
    pub atlas_size: PixelSize,
}

/// Render every requested scale variant from the @1x atlas + placements. The
/// @1x atlas is NOT included in the returned list - callers always emit it
/// separately as the canonical output.
pub fn render_variants(
    base_atlas: &RgbaImage,
    base_placed: &[PackedSprite],
    base_size: PixelSize,
    variants: &[ScaleVariant],
) -> Result<Vec<VariantOutput>, AppError> {
    let mut out = Vec::with_capacity(variants.len());
    for v in variants {
        if !(v.scale > 0.0) || !v.scale.is_finite() {
            return Err(AppError::new(
                "atlaspro_variant_scale",
                format!("variant '{}' has invalid scale {}", v.suffix, v.scale),
            ));
        }
        let new_w = ((base_size.width as f32) * v.scale).round().max(1.0) as u32;
        let new_h = ((base_size.height as f32) * v.scale).round().max(1.0) as u32;
        let scaled = if (v.scale - 1.0).abs() < f32::EPSILON {
            base_atlas.clone()
        } else {
            image::imageops::resize(base_atlas, new_w, new_h, FilterType::Lanczos3)
        };

        let placed = base_placed
            .iter()
            .map(|p| scale_placement(p, v.scale))
            .collect();

        out.push(VariantOutput {
            suffix: v.suffix.clone(),
            scale: v.scale,
            image: scaled,
            placed,
            atlas_size: PixelSize { width: new_w, height: new_h },
        });
    }
    Ok(out)
}

fn scale_placement(p: &PackedSprite, scale: f32) -> PackedSprite {
    PackedSprite {
        id: p.id.clone(),
        name: p.name.clone(),
        frame: scale_rect(p.frame, scale),
        source_frame: scale_rect(p.source_frame, scale),
        source_size: scale_size(p.source_size, scale),
        rotated: p.rotated,
        trimmed: p.trimmed,
        unity: p.unity.clone(),
    }
}

fn scale_rect(r: PixelRect, scale: f32) -> PixelRect {
    PixelRect::new(
        ((r.x as f32) * scale).round() as u32,
        ((r.y as f32) * scale).round() as u32,
        (((r.width as f32) * scale).round() as u32).max(1),
        (((r.height as f32) * scale).round() as u32).max(1),
    )
}

fn scale_size(s: PixelSize, scale: f32) -> PixelSize {
    PixelSize {
        width: (((s.width as f32) * scale).round() as u32).max(1),
        height: (((s.height as f32) * scale).round() as u32).max(1),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Rgba};

    fn red_atlas(w: u32, h: u32) -> RgbaImage {
        ImageBuffer::from_pixel(w, h, Rgba([255, 0, 0, 255]))
    }

    fn placed(name: &str, x: u32, y: u32, w: u32, h: u32) -> PackedSprite {
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

    #[test]
    fn no_variants_returns_empty() {
        let outs = render_variants(
            &red_atlas(64, 64),
            &[placed("a", 0, 0, 32, 32)],
            PixelSize { width: 64, height: 64 },
            &[],
        ).unwrap();
        assert_eq!(outs.len(), 0);
    }

    #[test]
    fn half_scale_halves_dimensions_and_coords() {
        let placements = vec![placed("a", 32, 32, 32, 32)];
        let variants = vec![ScaleVariant { suffix: "@0.5x".into(), scale: 0.5 }];
        let outs = render_variants(
            &red_atlas(64, 64),
            &placements,
            PixelSize { width: 64, height: 64 },
            &variants,
        ).unwrap();
        assert_eq!(outs.len(), 1);
        assert_eq!(outs[0].atlas_size, PixelSize { width: 32, height: 32 });
        assert_eq!(outs[0].image.dimensions(), (32, 32));
        assert_eq!(outs[0].placed[0].frame, PixelRect::new(16, 16, 16, 16));
    }

    #[test]
    fn double_scale_doubles_dimensions_and_coords() {
        let placements = vec![placed("a", 4, 8, 16, 16)];
        let variants = vec![ScaleVariant { suffix: "@2x".into(), scale: 2.0 }];
        let outs = render_variants(
            &red_atlas(64, 64),
            &placements,
            PixelSize { width: 64, height: 64 },
            &variants,
        ).unwrap();
        assert_eq!(outs[0].atlas_size, PixelSize { width: 128, height: 128 });
        assert_eq!(outs[0].placed[0].frame, PixelRect::new(8, 16, 32, 32));
    }

    #[test]
    fn unity_scale_passthrough_clones_atlas() {
        let variants = vec![ScaleVariant { suffix: "@1x".into(), scale: 1.0 }];
        let outs = render_variants(
            &red_atlas(8, 8),
            &[],
            PixelSize { width: 8, height: 8 },
            &variants,
        ).unwrap();
        assert_eq!(outs[0].atlas_size, PixelSize { width: 8, height: 8 });
        assert_eq!(outs[0].image.dimensions(), (8, 8));
    }

    #[test]
    fn rejects_non_positive_scale() {
        let variants = vec![ScaleVariant { suffix: "bad".into(), scale: 0.0 }];
        let err = render_variants(
            &red_atlas(8, 8),
            &[],
            PixelSize { width: 8, height: 8 },
            &variants,
        ).unwrap_err();
        assert_eq!(err.code, "atlaspro_variant_scale");
    }

    #[test]
    fn rejects_nan_scale() {
        let variants = vec![ScaleVariant { suffix: "nan".into(), scale: f32::NAN }];
        let err = render_variants(
            &red_atlas(8, 8),
            &[],
            PixelSize { width: 8, height: 8 },
            &variants,
        ).unwrap_err();
        assert_eq!(err.code, "atlaspro_variant_scale");
    }
}
