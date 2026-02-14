use stegsolve_rs::core::loader::load_frames_from_bytes;
use stegsolve_rs::core::filters::{process_image_bytes, FilterMode};
use stegsolve_rs::core::extract::extract_bits_from_image;
use image::{DynamicImage, Rgba, RgbaImage};

#[test]
fn test_workflow_load_and_filter() {
    // Create a simple red 1x1 pixel PNG-like byte slice (or just use image crate to generate)
    let mut img_buf = RgbaImage::new(1, 1);
    img_buf.put_pixel(0, 0, Rgba([255, 0, 0, 255]));
    let img = DynamicImage::ImageRgba8(img_buf);
    
    // Testing filtered bytes
    let (_, _, bytes) = process_image_bytes(&img, FilterMode::Invert);
    assert_eq!(bytes[0], 0);
    assert_eq!(bytes[1], 255);
    assert_eq!(bytes[2], 255);
}

#[test]
fn test_workflow_extract() {
    let mut img_buf = RgbaImage::new(8, 1);
    for x in 0..8 {
        // Put data in R0 bit
        let val = if x % 2 == 0 { 1 } else { 0 };
        img_buf.put_pixel(x, 0, Rgba([val, 0, 0, 255]));
    }
    let img = DynamicImage::ImageRgba8(img_buf);
    
    // Extract R0 bit
    let data = extract_bits_from_image(&img, 1, true, true, [0, 1, 2, 3]);
    // Bits: 1, 0, 1, 0, 1, 0, 1, 0 -> 1 | 0<<1 | 1<<2 | 0<<3 | 1<<4 | 0<<5 | 1<<6 | 0<<7 = 85
    assert_eq!(data[0], 85);
}
