//! # Stereoscopic Analysis
//!
//! Provides utilities for rendering stereoscopic images by shifting color channels.
//! This technique is useful for identifying depth-based steganography or patterns
//! hidden in different color planes with spatial offsets.

use image::{DynamicImage, GenericImageView};

/// Renders a stereo view by shifting the red channel.
///
/// This function creates a composite image where pixels from the original image
/// are blended with pixels from the same image offset horizontally.
///
/// # Arguments
///
/// * `img` - The source image.
/// * `offset` - The horizontal pixel offset for the stereo effect.
///
/// # Returns
///
/// A new `DynamicImage` containing the stereo-rendered view.
pub fn render_stereo(img: &DynamicImage, offset: i32) -> DynamicImage {
    let (width, height) = img.dimensions();
    let src = img.to_rgba8();
    let mut out = image::RgbaImage::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let x2 = x as i32 + offset;
            let p1 = src.get_pixel(x, y).0;
            let p2 = if x2 >= 0 && x2 < width as i32 {
                src.get_pixel(x2 as u32, y).0
            } else {
                [0, 0, 0, 255]
            };

            let mixed = [
                ((p1[0] as u16 + p2[0] as u16) / 2) as u8,
                ((p1[1] as u16 + p2[1] as u16) / 2) as u8,
                ((p1[2] as u16 + p2[2] as u16) / 2) as u8,
                255,
            ];
            out.put_pixel(x, y, image::Rgba(mixed));
        }
    }

    DynamicImage::ImageRgba8(out)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    #[test]
    fn test_stereo_offset_zero() {
        let mut img_buf = RgbaImage::new(2, 2);
        img_buf.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(img_buf);
        let stereo = render_stereo(&img, 0);
        assert_eq!(stereo.to_rgba8().get_pixel(0, 0).0, [255, 0, 0, 255]);
    }

    #[test]
    fn test_stereo_offset_positive() {
        let mut img_buf = RgbaImage::new(2, 1);
        img_buf.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
        img_buf.put_pixel(1, 0, Rgba([0, 255, 0, 255]));
        let img = DynamicImage::ImageRgba8(img_buf);
        // Offset 1: blends (0,0) with (1,0)
        let stereo = render_stereo(&img, 1);
        let p = stereo.to_rgba8().get_pixel(0, 0).0;
        // R: (255+0)/2 = 127, G: (0+255)/2 = 127
        assert_eq!(p[0], 127);
        assert_eq!(p[1], 127);
    }
}
