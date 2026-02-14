//! # Image Filters
//!
//! This module provides various filters for steganography analysis.
//! It includes color space transformations, bit plane extractions,
//! and thresholding utilities.
//!
//! ## Key Features
//! - Bit-plane extraction (0-7 for each RGBA channel)
//! - Color inversions and grayscale conversion
//! - Random color mapping for pattern discovery
//! - Binary thresholding

use image::{DynamicImage, GenericImageView};
use slint::{Rgba8Pixel, SharedPixelBuffer};

/// Supported image analysis filters.
///
/// These modes define how each pixel's color channels are transformed
/// for visual analysis.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterMode {
    /// No transformation applied.
    Normal,
    /// Inverts RGBA values (255 - value).
    Invert,
    /// Converts to grayscale using standard luminance weights.
    Gray,
    /// Applies a deterministic pseudo-random map to colors.
    RandomMap,
    /// Converts to black or white based on average luminance.
    BinaryThreshold,
    /// Displays a specific bit (0-7) of the Red channel.
    RedPlane(u8),
    /// Displays a specific bit (0-7) of the Green channel.
    GreenPlane(u8),
    /// Displays a specific bit (0-7) of the Blue channel.
    BluePlane(u8),
    /// Displays a specific bit (0-7) of the Alpha channel.
    AlphaPlane(u8),
}

impl FilterMode {
    /// Cycles to the next filter mode.
    pub fn next(&self) -> Self {
        match self {
            FilterMode::Normal => FilterMode::Invert,
            FilterMode::Invert => FilterMode::Gray,
            FilterMode::Gray => FilterMode::RandomMap,
            FilterMode::RandomMap => FilterMode::BinaryThreshold,
            FilterMode::BinaryThreshold => FilterMode::RedPlane(7),

            FilterMode::RedPlane(0) => FilterMode::GreenPlane(7),
            FilterMode::RedPlane(i) => FilterMode::RedPlane(i - 1),

            FilterMode::GreenPlane(0) => FilterMode::BluePlane(7),
            FilterMode::GreenPlane(i) => FilterMode::GreenPlane(i - 1),

            FilterMode::BluePlane(0) => FilterMode::AlphaPlane(7),
            FilterMode::BluePlane(i) => FilterMode::BluePlane(i - 1),

            FilterMode::AlphaPlane(0) => FilterMode::Normal,
            FilterMode::AlphaPlane(i) => FilterMode::AlphaPlane(i - 1),
        }
    }

    pub fn prev(&self) -> Self {
        let mut mode = *self;
        loop {
            let next = mode.next();
            if next == *self {
                return mode;
            }
            mode = next;
        }
    }
}

impl std::fmt::Display for FilterMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FilterMode::Normal => write!(f, "Normal Image"),
            FilterMode::Invert => write!(f, "Inverted"),
            FilterMode::Gray => write!(f, "Grayscale"),
            FilterMode::RandomMap => write!(f, "Random Color Map"),
            FilterMode::BinaryThreshold => write!(f, "Binary Threshold"),
            FilterMode::RedPlane(i) => write!(f, "Red Plane {}", i),
            FilterMode::GreenPlane(i) => write!(f, "Green Plane {}", i),
            FilterMode::BluePlane(i) => write!(f, "Blue Plane {}", i),
            FilterMode::AlphaPlane(i) => write!(f, "Alpha Plane {}", i),
        }
    }
}

pub fn apply_filter_pixel(r: u8, g: u8, b: u8, a: u8, mode: FilterMode) -> [u8; 4] {
    match mode {
        FilterMode::Normal => [r, g, b, a],
        FilterMode::Invert => [255 - r, 255 - g, 255 - b, a],
        FilterMode::Gray => {
            let gray = (0.299 * r as f32 + 0.587 * g as f32 + 0.114 * b as f32) as u8;
            [gray, gray, gray, 255]
        }
        FilterMode::RandomMap => {
            let r_mut = (r as u32 * 1234567) % 256;
            let g_mut = (g as u32 * 7654321) % 256;
            let b_mut = (b as u32 * 5555555) % 256;
            [r_mut as u8, g_mut as u8, b_mut as u8, 255]
        }
        FilterMode::BinaryThreshold => {
            let gray = (r as u32 + g as u32 + b as u32) / 3;
            let val = if gray > 127 { 255 } else { 0 };
            [val, val, val, 255]
        }
        FilterMode::RedPlane(bit) => extract_bit_plane(r, bit),
        FilterMode::GreenPlane(bit) => extract_bit_plane(g, bit),
        FilterMode::BluePlane(bit) => extract_bit_plane(b, bit),
        FilterMode::AlphaPlane(bit) => extract_bit_plane(a, bit),
    }
}

pub fn process_image_bytes(img: &DynamicImage, mode: FilterMode) -> (u32, u32, Vec<u8>) {
    let (width, height) = img.dimensions();
    let source_pixels = img.to_rgba8();
    let mut out = Vec::with_capacity((width * height * 4) as usize);

    for pixel in source_pixels.pixels() {
        let [r, g, b, a] = pixel.0;
        let new_pixel = apply_filter_pixel(r, g, b, a, mode);
        out.extend_from_slice(&new_pixel);
    }

    (width, height, out)
}

pub fn process_image(img: &DynamicImage, mode: FilterMode) -> SharedPixelBuffer<Rgba8Pixel> {
    let (width, height, bytes) = process_image_bytes(img, mode);
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    let raw_buffer = buffer.make_mut_slice();

    for (i, pixel) in raw_buffer.iter_mut().enumerate() {
        let base = i * 4;
        *pixel = Rgba8Pixel {
            r: bytes[base],
            g: bytes[base + 1],
            b: bytes[base + 2],
            a: bytes[base + 3],
        };
    }

    buffer
}

pub fn render_mask_overlay_bytes(img: &DynamicImage, mask: u32) -> (u32, u32, Vec<u8>) {
    let (width, height) = img.dimensions();
    let src = img.to_rgba8();
    let mut out = Vec::with_capacity((width * height * 4) as usize);

    let r_mask = (mask & 0xFF) as u8;
    let g_mask = ((mask >> 8) & 0xFF) as u8;
    let b_mask = ((mask >> 16) & 0xFF) as u8;
    let a_mask = ((mask >> 24) & 0xFF) as u8;

    for pixel in src.pixels() {
        let [r, g, b, a] = pixel.0;
        let mut or_r = has_mask_bit(r, r_mask);
        let mut or_g = has_mask_bit(g, g_mask);
        let mut or_b = has_mask_bit(b, b_mask);
        let or_a = has_mask_bit(a, a_mask);

        if or_a {
            or_r = true;
            or_g = true;
            or_b = true;
        }

        let alpha = if or_r || or_g || or_b { 180 } else { 0 };
        out.extend_from_slice(&[
            if or_r { 255 } else { 0 },
            if or_g { 255 } else { 0 },
            if or_b { 255 } else { 0 },
            alpha,
        ]);
    }

    (width, height, out)
}

pub fn blend_overlay_bytes(base: &[u8], overlay: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(base.len());
    let mut i = 0;
    while i + 3 < base.len() && i + 3 < overlay.len() {
        let a = overlay[i + 3] as f32 / 255.0;
        let inv = 1.0 - a;
        let r = (base[i] as f32 * inv + overlay[i] as f32 * a) as u8;
        let g = (base[i + 1] as f32 * inv + overlay[i + 1] as f32 * a) as u8;
        let b = (base[i + 2] as f32 * inv + overlay[i + 2] as f32 * a) as u8;
        out.extend_from_slice(&[r, g, b, 255]);
        i += 4;
    }

    out
}

fn extract_bit_plane(channel_val: u8, bit: u8) -> [u8; 4] {
    let val = if (channel_val >> bit) & 1 == 1 {
        255
    } else {
        0
    };
    [val, val, val, 255]
}

fn has_mask_bit(channel_val: u8, mask: u8) -> bool {
    for bit in 0..8 {
        if (mask >> bit) & 1 == 1 && (channel_val >> bit) & 1 == 1 {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::RgbaImage;

    #[test]
    fn test_filter_normal() {
        assert_eq!(
            apply_filter_pixel(10, 20, 30, 40, FilterMode::Normal),
            [10, 20, 30, 40]
        );
    }

    #[test]
    fn test_filter_invert() {
        assert_eq!(
            apply_filter_pixel(10, 20, 30, 40, FilterMode::Invert),
            [245, 235, 225, 40]
        );
    }

    #[test]
    fn test_filter_gray() {
        let pixel = apply_filter_pixel(100, 100, 100, 255, FilterMode::Gray);
        assert_eq!(pixel[0], pixel[1]);
        assert_eq!(pixel[1], pixel[2]);
        assert_eq!(pixel[3], 255);
    }

    #[test]
    fn test_bit_planes() {
        // Bit 0 of 1 is 1 -> [255, 255, 255, 255]
        assert_eq!(
            apply_filter_pixel(1, 0, 0, 255, FilterMode::RedPlane(0)),
            [255, 255, 255, 255]
        );
        // Bit 0 of 2 is 0 -> [0, 0, 0, 255]
        assert_eq!(
            apply_filter_pixel(2, 0, 0, 255, FilterMode::RedPlane(0)),
            [0, 0, 0, 255]
        );
        // Bit 7 of 128 is 1
        assert_eq!(
            apply_filter_pixel(128, 0, 0, 255, FilterMode::RedPlane(7)),
            [255, 255, 255, 255]
        );
    }

    #[test]
    fn test_filter_cycle() {
        let mode = FilterMode::Normal;
        let next = mode.next();
        assert_eq!(next, FilterMode::Invert);
        let prev = next.prev();
        assert_eq!(prev, FilterMode::Normal);
    }

    #[test]
    fn test_binary_threshold() {
        assert_eq!(
            apply_filter_pixel(200, 200, 200, 255, FilterMode::BinaryThreshold),
            [255, 255, 255, 255]
        );
        assert_eq!(
            apply_filter_pixel(50, 50, 50, 255, FilterMode::BinaryThreshold),
            [0, 0, 0, 255]
        );
    }

    #[test]
    fn test_process_image() {
        let mut img_buf = RgbaImage::new(2, 2);
        img_buf.put_pixel(0, 0, image::Rgba([255, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(img_buf);
        let (w, h, bytes) = process_image_bytes(&img, FilterMode::Invert);
        assert_eq!(w, 2);
        assert_eq!(h, 2);
        // [255,0,0,255] inverted -> [0,255,255,255]
        assert_eq!(bytes[0], 0);
        assert_eq!(bytes[1], 255);
        assert_eq!(bytes[2], 255);
    }

    #[test]
    fn test_blend_overlay() {
        let base = vec![100, 100, 100, 255, 200, 200, 200, 255];
        let overlay = vec![255, 0, 0, 128, 0, 255, 0, 255]; // 50% red, 100% green
        let blended = blend_overlay_bytes(&base, &overlay);
        assert_eq!(blended.len(), 8);
        // Second pixel should be pure green because overlay alpha is 255
        assert_eq!(blended[4], 0);
        assert_eq!(blended[5], 255);
        assert_eq!(blended[6], 0);
    }

    #[test]
    fn test_filter_to_string() {
        assert_eq!(FilterMode::Normal.to_string(), "Normal Image");
        assert_eq!(FilterMode::RedPlane(0).to_string(), "Red Plane 0");
    }
}
