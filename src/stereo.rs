use image::{DynamicImage, GenericImageView};

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
