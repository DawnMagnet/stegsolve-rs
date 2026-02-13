use crate::filters::{
    blend_overlay_bytes, process_image, process_image_bytes, render_mask_overlay_bytes, FilterMode,
};
use crate::stereo::render_stereo;
use crate::AppWindow;
use image::DynamicImage;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

pub fn update_ui_for_frame(
    ui: &AppWindow,
    img: &DynamicImage,
    filter: FilterMode,
    file_name: &str,
    frame_index: usize,
    total_frames: usize,
    stereo_enabled: bool,
    stereo_offset: i32,
    show_mask_overlay: bool,
    mask: u32,
) {
    let display_buffer = if show_mask_overlay && mask != 0 {
        let base_img = if stereo_enabled {
            render_stereo(img, stereo_offset)
        } else {
            img.clone()
        };
        let (width, height, base_bytes) = process_image_bytes(&base_img, filter);
        let (_w, _h, overlay_bytes) = render_mask_overlay_bytes(img, mask);
        let blended = blend_overlay_bytes(&base_bytes, &overlay_bytes);
        bytes_to_buffer(width, height, blended)
    } else if stereo_enabled {
        let stereo_img = render_stereo(img, stereo_offset);
        process_image(&stereo_img, filter)
    } else {
        process_image(img, filter)
    };

    let total = total_frames.max(1);
    ui.set_displayed_image(Image::from_rgba8(display_buffer));
    ui.set_filter_name(filter.to_string().into());
    ui.set_frame_info(format!("Frame {}/{}", frame_index + 1, total).into());
    ui.set_image_info(format!("{}x{} | {}", img.width(), img.height(), file_name).into());
}

fn bytes_to_buffer(width: u32, height: u32, bytes: Vec<u8>) -> SharedPixelBuffer<Rgba8Pixel> {
    let mut buffer = SharedPixelBuffer::<Rgba8Pixel>::new(width, height);
    let raw = buffer.make_mut_slice();

    for (i, pixel) in raw.iter_mut().enumerate() {
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
