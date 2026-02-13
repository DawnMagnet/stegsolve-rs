use image::{DynamicImage, GenericImageView};

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
                cur = (cur << 1) | (val as u8);
            }
            count += 1;

            if count % 8 == 0 {
                out.push(cur);
                cur = 0;
            }
        }
    }

    if count % 8 != 0 {
        if !lsb_first {
            cur <<= 8 - (count % 8) as u8;
        }
        out.push(cur);
    }

    out
}
