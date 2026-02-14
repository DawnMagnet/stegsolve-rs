//! # Image Loading
//!
//! Utilities for loading images from various formats into a unified representation.
//! This module handles both static images and multi-frame animations (like GIFs).
//!
//! ## Supported Formats
//! - PNG, JPEG, BMP, GIF
//! - WebP, TIFF, TGA, PNM
//! - Fallback mechanism for robust loading

use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, ImageFormat};
use std::io::Cursor;

/// Loads all frames from an image byte slice.
///
/// This function attempts to decode the image bytes into one or more frames.
/// For animated formats like GIF, it returns all frames. For static formats,
/// it returns a single-element vector.
///
/// # Arguments
///
/// * `bytes` - The raw image data as a byte slice.
///
/// # Returns
///
/// Returns a `Result` containing a `Vec<DynamicImage>` on success,
/// or an error if decoding fails.
///
/// # Examples
///
/// ```rust
/// use stegsolve_rs::core::loader::load_frames_from_bytes;
/// let bytes = include_bytes!("../../assets/icons/icon.png");
/// let frames = load_frames_from_bytes(bytes).unwrap();
/// assert_eq!(frames.len(), 1);
/// ```
pub fn load_frames_from_bytes(bytes: &[u8]) -> Result<Vec<DynamicImage>, anyhow::Error> {
    let mut frames: Vec<DynamicImage> = Vec::new();

    if let Ok(format) = image::guess_format(bytes)
        && format == ImageFormat::Gif
        && let Ok(decoder) = GifDecoder::new(Cursor::new(bytes))
    {
        for frame in decoder.into_frames().flatten() {
            frames.push(DynamicImage::ImageRgba8(frame.into_buffer()));
        }
    }
    if frames.is_empty() {
        frames.push(load_image_robust_from_bytes(bytes)?);
    }

    Ok(frames)
}

fn load_image_robust_from_bytes(bytes: &[u8]) -> Result<DynamicImage, anyhow::Error> {
    if let Ok(img) = image::load_from_memory(bytes) {
        return Ok(img);
    }

    let formats = [
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Gif,
        ImageFormat::Bmp,
        ImageFormat::Ico,
        ImageFormat::Tiff,
        ImageFormat::WebP,
        ImageFormat::Pnm,
        ImageFormat::Tga,
        ImageFormat::Dds,
    ];

    for &format in &formats {
        if let Ok(img) = image::load_from_memory_with_format(bytes, format) {
            return Ok(img);
        }
    }

    Err(anyhow::anyhow!(
        "Unrecognized image format. Stegsolve-rs tried all decoders but failed."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_invalid_bytes() {
        let result = load_frames_from_bytes(&[0, 1, 2, 3]);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_png() {
        // Transparent 1x1 PNG
        let png_bytes = vec![
            0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
            0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00,
            0x00, 0x1F, 0x15, 0xC4, 0x89, 0x00, 0x00, 0x00, 0x0A, 0x49, 0x44, 0x41, 0x54, 0x78,
            0x9C, 0x63, 0x00, 0x01, 0x00, 0x00, 0x05, 0x00, 0x01, 0x0D, 0x0A, 0x2D, 0xB4, 0x00,
            0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
        ];
        let frames = load_frames_from_bytes(&png_bytes).unwrap();
        assert_eq!(frames.len(), 1);
        assert_eq!(frames[0].width(), 1);
        assert_eq!(frames[0].height(), 1);
    }
}
