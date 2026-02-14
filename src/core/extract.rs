//! # Bit Extraction
//!
//! Provides functionality for extracting hidden bits from image color channels.
//! This module supports LSB/MSB extraction, various channel orderings, and
//! both row-major and column-major scanning.

use image::{DynamicImage, GenericImageView};

/// Extracts bits from an image based on the provided mask and order.
///
/// This function iterates through the pixels of the image and pulls out specific
/// bits from the RGBA channels according to the bitmask and bit order.
///
/// # Arguments
///
/// * `img` - The source image to extract data from.
/// * `mask` - A 32-bit integer where each bit represents whether to extract that bit
///   (bits 0-7: Red, 8-15: Green, 16-23: Blue, 24-31: Alpha).
/// * `lsb_first` - If true, bits are extracted from Least Significant Bit to Most.
/// * `row_order` - If true, pixels are scanned row-by-row; otherwise column-by-column.
/// * `channel_order` - An array of 4 indices (0-3) specifying the order of RGBA channels.
///
/// # Returns
///
/// A `Vec<u8>` containing the extracted and reconstructed bytes.
///
/// # Examples
///
/// ```rust
/// use stegsolve_rs::core::extract::extract_bits_from_image;
/// use image::{DynamicImage, RgbaImage};
///
/// let img = DynamicImage::ImageRgba8(RgbaImage::new(10, 10));
/// let data = extract_bits_from_image(&img, 1, true, true, [0, 1, 2, 3]);
/// assert!(data.len() > 0);
/// ```
pub fn extract_bits_from_image(
    img: &DynamicImage,
    mask: u32,
    lsb_first: bool,
    row_order: bool,
    channel_order: [usize; 4],
) -> Vec<u8> {
    let (width, height) = img.dimensions();
    let src = img.to_rgba8();

    let mut selected_bits: Vec<(usize, u8)> = Vec::new();
    let bit_range: Vec<u8> = if lsb_first {
        (0..8).collect()
    } else {
        (0..8).rev().collect()
    };
    for bit in bit_range {
        for &channel in channel_order.iter() {
            let idx = channel * 8 + bit as usize;
            if (mask >> idx) & 1 == 1 {
                selected_bits.push((channel, bit));
            }
        }
    }

    let mut out: Vec<u8> = Vec::new();
    let mut cur: u8 = 0;
    let mut count: usize = 0;

    let iter_coords: Vec<(u32, u32)> = if row_order {
        (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .collect()
    } else {
        (0..width)
            .flat_map(|x| (0..height).map(move |y| (x, y)))
            .collect()
    };

    for (x, y) in iter_coords {
        let [r, g, b, a] = src.get_pixel(x, y).0;
        for (channel, bit) in &selected_bits {
            let val = match channel {
                0 => (r >> bit) & 1,
                1 => (g >> bit) & 1,
                2 => (b >> bit) & 1,
                _ => (a >> bit) & 1,
            };

            if lsb_first {
                let bit_pos = count % 8;
                cur |= val << bit_pos;
            } else {
                cur = (cur << 1) | val;
            }
            count += 1;

            if count.is_multiple_of(8) {
                out.push(cur);
                cur = 0;
            }
        }
    }

    if !count.is_multiple_of(8) {
        if !lsb_first {
            cur <<= 8 - (count % 8) as u8;
        }
        out.push(cur);
    }

    out
}

/// Maps an integer index to a channel order array.
///
/// * `0` -> RGBA
/// * `1` -> RBGA
/// * `2` -> GRBA
/// * `3` -> GBRA
/// * `4` -> BRGA
/// * `5` -> BGRA
pub fn channel_order_from_index(index: i32) -> [usize; 4] {
    match index {
        0 => [0, 1, 2, 3],
        1 => [0, 2, 1, 3],
        2 => [1, 0, 2, 3],
        3 => [1, 2, 0, 3],
        4 => [2, 0, 1, 3],
        5 => [2, 1, 0, 3],
        _ => [0, 1, 2, 3],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn create_test_image() -> DynamicImage {
        let mut img = RgbaImage::new(2, 2);
        // Pixel (0,0): R=1, G=2, B=4, A=8
        img.put_pixel(0, 0, Rgba([1, 2, 4, 8]));
        // Pixel (1,0): R=16, G=32, B=64, A=128
        img.put_pixel(1, 0, Rgba([16, 32, 64, 128]));
        DynamicImage::ImageRgba8(img)
    }

    #[test]
    fn test_extract_r0_lsb() {
        let img = create_test_image();
        let mask = 1; // R0
        let data = extract_bits_from_image(&img, mask, true, true, [0, 1, 2, 3]);
        // Pixel 0: R=1 (bit 0 is 1)
        // Pixel 1: R=16 (bit 0 is 0)
        // Bit stream: 1, 0 -> 1 (LSB first) -> 1
        assert_eq!(data[0], 1);
    }

    #[test]
    fn test_extract_r0_msb() {
        let img = create_test_image();
        let mask = 1 << 7; // R7
        let data = extract_bits_from_image(&img, mask, false, true, [0, 1, 2, 3]);
        // Pixel 0: R=1 (bit 7 is 0)
        // Pixel 1: R=16 (bit 7 is 0)
        assert_eq!(data[0], 0);

        let mut img_buf = RgbaImage::new(2, 1);
        img_buf.put_pixel(0, 0, Rgba([128, 0, 0, 255]));
        img_buf.put_pixel(1, 0, Rgba([128, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(img_buf);
        let data = extract_bits_from_image(&img, mask, false, true, [0, 1, 2, 3]);
        // Bit stream: 1, 1 -> 11000000 (binary) -> 192
        assert_eq!(data[0], 192);
    }

    #[test]
    fn test_extract_column_order() {
        let mut img_buf = RgbaImage::new(2, 2);
        img_buf.put_pixel(0, 0, Rgba([1, 0, 0, 255]));
        img_buf.put_pixel(0, 1, Rgba([1, 0, 0, 255]));
        img_buf.put_pixel(1, 0, Rgba([0, 0, 0, 255]));
        img_buf.put_pixel(1, 1, Rgba([0, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(img_buf);

        // Row order: (0,0), (1,0), (0,1), (1,1) -> R: 1, 0, 1, 0
        let data_row = extract_bits_from_image(&img, 1, true, true, [0, 1, 2, 3]);
        assert_eq!(data_row[0], 0b0101); // 1,0,1,0 LSB first -> 1 | 0<<1 | 1<<2 | 0<<3 = 5

        // Column order: (0,0), (0,1), (1,0), (1,1) -> R: 1, 1, 0, 0
        let data_col = extract_bits_from_image(&img, 1, true, false, [0, 1, 2, 3]);
        assert_eq!(data_col[0], 0b0011); // 1,1,0,0 LSB first -> 1 | 1<<1 | 0<<2 | 0<<3 = 3
    }

    #[test]
    fn test_all_channel_orderings() {
        let mut img_buf = RgbaImage::new(1, 1);
        img_buf.put_pixel(0, 0, Rgba([1, 2, 4, 8]));
        let img = DynamicImage::ImageRgba8(img_buf);
        let mask = 1 | (1 << 8) | (1 << 16) | (1 << 24); // R0, G0, B0, A0

        // RGBA: 1, 2, 4, 8 bits 0: 1, 0, 0, 0 -> 1
        let data = extract_bits_from_image(&img, mask, true, true, [0, 1, 2, 3]);
        assert_eq!(data[0], 0b0001);

        // BGRA: 4, 2, 1, 8 bits 0: 0, 0, 1, 0 -> 4
        let data = extract_bits_from_image(&img, mask, true, true, [2, 1, 0, 3]);
        assert_eq!(data[0], 0b0100);
    }

    #[test]
    fn test_multi_bit_mask() {
        let mut img_buf = RgbaImage::new(1, 1);
        img_buf.put_pixel(0, 0, Rgba([3, 0, 0, 255])); // R0=1, R1=1
        let img = DynamicImage::ImageRgba8(img_buf);
        let mask = 3; // R0 and R1
        let data = extract_bits_from_image(&img, mask, true, true, [0, 1, 2, 3]);
        // R0 then R1 -> 1, 1 -> 3
        assert_eq!(data[0], 3);
    }

    #[test]
    fn test_partial_byte() {
        let mut img_buf = RgbaImage::new(1, 1);
        img_buf.put_pixel(0, 0, Rgba([1, 0, 0, 255]));
        let img = DynamicImage::ImageRgba8(img_buf);
        let data = extract_bits_from_image(&img, 1, true, true, [0, 1, 2, 3]);
        assert_eq!(data.len(), 1);
        assert_eq!(data[0], 1);
    }

    #[test]
    fn test_channel_order_mapping() {
        assert_eq!(channel_order_from_index(0), [0, 1, 2, 3]);
        assert_eq!(channel_order_from_index(5), [2, 1, 0, 3]);
        assert_eq!(channel_order_from_index(99), [0, 1, 2, 3]);
    }
}
