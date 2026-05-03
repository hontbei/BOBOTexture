// AtlasPacker Pro - compositor.
//
// Wave 3: blits each preprocessed sprite buffer onto a single transparent RGBA
// atlas image at the location chosen by `packer::pack`. Rotated placements
// (rotated == true, set by the packer when 90 deg CW rotation gives a better
// fit) are rotated in-buffer here before blit so the atlas pixels match what
// the exporter will reference via PackedSprite.frame.
//
// Coordinate convention reminder: PackedSprite.frame is ALWAYS top-left origin
// in atlas pixels and ALREADY accounts for the extrude border baked in by
// preprocess. The compositor therefore subtracts `extrude` to find the true
// blit origin (i.e. where the extruded buffer's top-left corner goes) and
// blits the FULL extruded buffer.

use image::{imageops, ImageBuffer, Rgba, RgbaImage};

use crate::error::AppError;

use super::model::{LoadedSprite, PackedSprite, PixelSize};

pub struct CompositeInputs<'a> {
    pub atlas_size: PixelSize,
    pub placed: &'a [PackedSprite],
    pub sources: &'a [LoadedSprite],
    pub extrude: u32,
}

/// Build the final atlas image. Returns a fully transparent canvas with each
/// placed sprite blitted in. Caller is responsible for writing it to disk.
pub fn composite(inputs: CompositeInputs<'_>) -> Result<RgbaImage, AppError> {
    let CompositeInputs { atlas_size, placed, sources, extrude } = inputs;

    if atlas_size.width == 0 || atlas_size.height == 0 {
        return Err(AppError::new(
            "atlaspro_composite_size",
            "atlas dimensions must be > 0",
        ));
    }

    let mut atlas: RgbaImage = ImageBuffer::from_pixel(
        atlas_size.width,
        atlas_size.height,
        Rgba([0, 0, 0, 0]),
    );

    for spec in placed {
        let source = sources
            .iter()
            .find(|s| s.id == spec.id)
            .ok_or_else(|| AppError::new(
                "atlaspro_composite_missing",
                format!("composite: source for sprite '{}' not found", spec.name),
            ))?;

        let buffer: RgbaImage = if spec.rotated {
            imageops::rotate90(&source.pixels)
        } else {
            source.pixels.clone()
        };

        let (buf_w, buf_h) = buffer.dimensions();

        let blit_x = spec.frame.x.checked_sub(extrude).ok_or_else(|| AppError::new(
            "atlaspro_composite_underflow",
            format!("composite: frame.x < extrude for '{}'", spec.name),
        ))?;
        let blit_y = spec.frame.y.checked_sub(extrude).ok_or_else(|| AppError::new(
            "atlaspro_composite_underflow",
            format!("composite: frame.y < extrude for '{}'", spec.name),
        ))?;

        if blit_x.saturating_add(buf_w) > atlas_size.width
            || blit_y.saturating_add(buf_h) > atlas_size.height
        {
            return Err(AppError::new(
                "atlaspro_composite_overflow",
                format!(
                    "composite: '{}' would overflow atlas ({},{}) + ({}x{}) > ({}x{})",
                    spec.name, blit_x, blit_y, buf_w, buf_h, atlas_size.width, atlas_size.height
                ),
            ));
        }

        imageops::overlay(&mut atlas, &buffer, blit_x as i64, blit_y as i64);
    }

    Ok(atlas)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tools::atlaspro::model::{PixelRect, PixelSize, SpriteOrigin};
    use image::Rgba;

    fn fake_solid(id: &str, w: u32, h: u32, color: [u8; 4]) -> LoadedSprite {
        LoadedSprite {
            id: id.to_string(),
            name: id.to_string(),
            origin: SpriteOrigin::File,
            source_path: std::path::PathBuf::from(format!("/tmp/{id}.png")),
            source_size: PixelSize { width: w, height: h },
            trimmed_rect: PixelRect::new(0, 0, w, h),
            sub_rect: PixelRect::new(0, 0, w, h),
            trimmed: false,
            pixels: ImageBuffer::from_pixel(w, h, Rgba(color)),
            unity: None,
        }
    }

    fn make_placed(id: &str, frame: PixelRect, rotated: bool) -> PackedSprite {
        PackedSprite {
            id: id.to_string(),
            name: id.to_string(),
            frame,
            source_frame: PixelRect::new(0, 0, frame.width, frame.height),
            source_size: PixelSize { width: frame.width, height: frame.height },
            rotated,
            trimmed: false,
            unity: None,
        }
    }

    #[test]
    fn composite_places_two_sprites_at_correct_coords() {
        let sources = vec![
            fake_solid("a", 16, 16, [255, 0, 0, 255]),
            fake_solid("b", 16, 16, [0, 255, 0, 255]),
        ];
        let placed = vec![
            make_placed("a", PixelRect::new(0, 0, 16, 16), false),
            make_placed("b", PixelRect::new(16, 0, 16, 16), false),
        ];
        let atlas = composite(CompositeInputs {
            atlas_size: PixelSize { width: 32, height: 16 },
            placed: &placed,
            sources: &sources,
            extrude: 0,
        }).unwrap();
        assert_eq!(atlas.get_pixel(0, 0).0, [255, 0, 0, 255]);
        assert_eq!(atlas.get_pixel(15, 15).0, [255, 0, 0, 255]);
        assert_eq!(atlas.get_pixel(16, 0).0, [0, 255, 0, 255]);
        assert_eq!(atlas.get_pixel(31, 15).0, [0, 255, 0, 255]);
    }

    #[test]
    fn composite_rotates_when_flag_set() {
        let mut buf: RgbaImage = ImageBuffer::from_pixel(4, 2, Rgba([0, 0, 0, 0]));
        for x in 0..4 { buf.put_pixel(x, 0, Rgba([255, 0, 0, 255])); }
        let mut sprite = fake_solid("rot", 4, 2, [0, 0, 0, 0]);
        sprite.pixels = buf;

        let placed = vec![make_placed("rot", PixelRect::new(0, 0, 2, 4), true)];
        let atlas = composite(CompositeInputs {
            atlas_size: PixelSize { width: 2, height: 4 },
            placed: &placed,
            sources: &[sprite],
            extrude: 0,
        }).unwrap();
        // After 90 deg CW rotation, the original top row (red) becomes the
        // right column at x=1, y=0..3.
        assert_eq!(atlas.get_pixel(1, 0).0, [255, 0, 0, 255]);
        assert_eq!(atlas.get_pixel(1, 3).0, [255, 0, 0, 255]);
        assert_eq!(atlas.get_pixel(0, 0).0, [0, 0, 0, 0]);
    }

    #[test]
    fn composite_strips_extrude_from_blit_origin() {
        let sprite = fake_solid("ex", 6, 6, [10, 20, 30, 255]);
        let placed = vec![make_placed("ex", PixelRect::new(2, 2, 4, 4), false)];
        let atlas = composite(CompositeInputs {
            atlas_size: PixelSize { width: 8, height: 8 },
            placed: &placed,
            sources: &[sprite],
            extrude: 1,
        }).unwrap();
        // extrude=1 means blit origin = (2-1, 2-1) = (1,1); buffer is 6x6.
        assert_eq!(atlas.get_pixel(1, 1).0, [10, 20, 30, 255]);
        assert_eq!(atlas.get_pixel(6, 6).0, [10, 20, 30, 255]);
        assert_eq!(atlas.get_pixel(0, 0).0, [0, 0, 0, 0]);
    }

    #[test]
    fn composite_fails_when_source_missing() {
        let placed = vec![make_placed("ghost", PixelRect::new(0, 0, 4, 4), false)];
        let err = composite(CompositeInputs {
            atlas_size: PixelSize { width: 8, height: 8 },
            placed: &placed,
            sources: &[],
            extrude: 0,
        }).unwrap_err();
        assert_eq!(err.code, "atlaspro_composite_missing");
    }

    #[test]
    fn composite_fails_on_overflow() {
        let sprite = fake_solid("big", 16, 16, [1, 2, 3, 255]);
        let placed = vec![make_placed("big", PixelRect::new(8, 8, 16, 16), false)];
        let err = composite(CompositeInputs {
            atlas_size: PixelSize { width: 16, height: 16 },
            placed: &placed,
            sources: &[sprite],
            extrude: 0,
        }).unwrap_err();
        assert_eq!(err.code, "atlaspro_composite_overflow");
    }
}
