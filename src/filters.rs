use image::{DynamicImage, GenericImageView};
use slint::{Rgba8Pixel, SharedPixelBuffer};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FilterMode {
    Normal,
    Invert,
    Gray,
    RandomMap,
    BinaryThreshold,
    RedPlane(u8),
    GreenPlane(u8),
    BluePlane(u8),
    AlphaPlane(u8),
}

impl FilterMode {
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

    pub fn to_string(&self) -> String {
        match self {
            FilterMode::Normal => "Normal Image".to_string(),
            FilterMode::Invert => "Inverted".to_string(),
            FilterMode::Gray => "Grayscale".to_string(),
            FilterMode::RandomMap => "Random Color Map".to_string(),
            FilterMode::BinaryThreshold => "Binary Threshold".to_string(),
            FilterMode::RedPlane(i) => format!("Red Plane {}", i),
            FilterMode::GreenPlane(i) => format!("Green Plane {}", i),
            FilterMode::BluePlane(i) => format!("Blue Plane {}", i),
            FilterMode::AlphaPlane(i) => format!("Alpha Plane {}", i),
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
