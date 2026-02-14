use crate::core::extract::{channel_order_from_index, extract_bits_from_image};
use crate::core::filters::{
    blend_overlay_bytes, process_image, process_image_bytes, render_mask_overlay_bytes,
};
use crate::presentation::hex::format_hex_ascii;
use crate::presentation::stereo::render_stereo;
use crate::ui::state::AppState;
use crate::{AppWindow, DataExtractDialog};
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

pub fn refresh_ui(ui: &AppWindow, state: &AppState) {
    let img = match state.current_frame() {
        Some(img) => img,
        None => return,
    };

    let filter = state.filter();
    let stereo_enabled = ui.get_stereo_enabled();
    let stereo_offset = ui.get_stereo_offset() as i32;
    let show_mask_overlay = state.show_mask_overlay();
    let mask = state.extract_mask();

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

    let total = state.frame_count().max(1);
    ui.set_displayed_image(Image::from_rgba8(display_buffer));
    ui.set_filter_name(filter.to_string().into());
    ui.set_frame_info(format!("Frame {}/{}", state.frame_index() + 1, total).into());
    ui.set_image_info(format!("{}x{} | {}", img.width(), img.height(), state.file_name()).into());
}

pub fn update_extract_preview(dialog: &DataExtractDialog, state: &AppState) {
    let img = match state.current_frame() {
        Some(img) => img,
        None => {
            dialog.set_preview_text("No image loaded".into());
            return;
        }
    };

    let bytes = extract_bits_from_image(
        img,
        state.extract_mask(),
        dialog.get_lsb_first(),
        dialog.get_row_order(),
        channel_order_from_index(dialog.get_color_order()),
    );
    dialog.set_preview_text(format_hex_ascii(&bytes, 512).into());
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
