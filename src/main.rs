// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

slint::include_modules!();

mod extract;
mod filters;
mod hex;
mod loader;
mod state;
mod stereo;
mod ui_helpers;

use crate::extract::extract_bits_from_image;
use crate::filters::process_image_bytes;
use crate::hex::{format_binary, format_hex_ascii, format_hex_compact};
use crate::loader::load_frames_from_bytes;
use crate::state::{AppState, FilterNavigation, FrameNavigation, MaskUpdate};
use crate::stereo::render_stereo;
use crate::ui_helpers::update_ui_for_frame;
use slint::{Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

fn channel_order_from_index(index: i32) -> [usize; 4] {
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

fn refresh_ui(ui: &AppWindow, state: &AppState) {
    if let Some(img) = state.current_frame() {
        update_ui_for_frame(
            ui,
            img,
            state.filter(),
            state.file_name(),
            state.frame_index(),
            state.frame_count(),
            ui.get_stereo_enabled(),
            ui.get_stereo_offset() as i32,
            state.show_mask_overlay(),
            state.extract_mask(),
        );
    }
}

fn update_extract_preview(dialog: &DataExtractDialog, state: &AppState) {
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

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let data_dialog = DataExtractDialog::new()?;

    let state = Rc::new(RefCell::new(AppState::new()));
    let playback_timer = Rc::new(Timer::default());

    let ui_handle = ui.as_weak();
    let dialog_handle = data_dialog.as_weak();

    // Open file
    let state_clone = state.clone();
    let playback_timer_clone = playback_timer.clone();
    let ui_weak_open = ui_handle.clone();
    ui.on_open_file(move || {
        let ui = ui_weak_open.unwrap();
        if let Some(path) = rfd::FileDialog::new()
            .add_filter(
                "Images",
                &[
                    "png", "jpg", "jpeg", "bmp", "gif", "webp", "tiff", "tga", "pnm",
                ],
            )
            .add_filter("All Files", &["*"])
            .pick_file()
        {
            let file_name = path
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let bytes = match std::fs::read(&path) {
                Ok(bytes) => bytes,
                Err(e) => {
                    ui.set_image_info(format!("Error reading file: {}", e).into());
                    return;
                }
            };

            let frames = match load_frames_from_bytes(&bytes) {
                Ok(frames) => frames,
                Err(e) => {
                    ui.set_image_info(format!("Error: {}", e).into());
                    return;
                }
            };

            {
                let mut state = state_clone.borrow_mut();
                state.set_frames(frames, file_name);
            }

            playback_timer_clone.stop();
            ui.set_is_playing(false);

            let state = state_clone.borrow();
            refresh_ui(&ui, &state);
        }
    });

    // Filter navigation
    let state_clone = state.clone();
    let ui_weak_next_filter = ui_handle.clone();
    ui.on_next_filter(move || {
        let ui = ui_weak_next_filter.unwrap();
        {
            let mut state = state_clone.borrow_mut();
            state.next_filter();
        }
        let state = state_clone.borrow();
        refresh_ui(&ui, &state);
    });

    let state_clone = state.clone();
    let ui_weak_prev_filter = ui_handle.clone();
    ui.on_prev_filter(move || {
        let ui = ui_weak_prev_filter.unwrap();
        {
            let mut state = state_clone.borrow_mut();
            state.prev_filter();
        }
        let state = state_clone.borrow();
        refresh_ui(&ui, &state);
    });

    // Frame navigation
    let state_clone = state.clone();
    let ui_weak_next_frame = ui_handle.clone();
    ui.on_next_frame(move || {
        let ui = ui_weak_next_frame.unwrap();
        {
            let mut state = state_clone.borrow_mut();
            state.next_frame();
        }
        let state = state_clone.borrow();
        refresh_ui(&ui, &state);
    });

    let state_clone = state.clone();
    let ui_weak_prev_frame = ui_handle.clone();
    ui.on_prev_frame(move || {
        let ui = ui_weak_prev_frame.unwrap();
        {
            let mut state = state_clone.borrow_mut();
            state.prev_frame();
        }
        let state = state_clone.borrow();
        refresh_ui(&ui, &state);
    });

    // Play/pause
    let state_clone = state.clone();
    let playback_timer_clone = playback_timer.clone();
    let ui_handle_for_timer = ui_handle.clone();
    let ui_weak_play = ui_handle.clone();
    ui.on_toggle_play(move || {
        let ui = ui_weak_play.unwrap();
        let can_play = { state_clone.borrow().frame_count() > 1 };
        if !can_play {
            return;
        }

        let playing = { state_clone.borrow().is_playing() };
        if playing {
            playback_timer_clone.stop();
            state_clone.borrow_mut().set_playing(false);
            ui.set_is_playing(false);
            return;
        }

        state_clone.borrow_mut().set_playing(true);
        ui.set_is_playing(true);

        let state_for_timer = state_clone.clone();
        let ui_weak_inner = ui_handle_for_timer.clone();
        playback_timer_clone.start(TimerMode::Repeated, Duration::from_millis(150), move || {
            let ui = ui_weak_inner.unwrap();
            {
                let mut state = state_for_timer.borrow_mut();
                state.next_frame();
            }
            let state = state_for_timer.borrow();
            refresh_ui(&ui, &state);
        });
    });

    // Stereo
    let state_clone = state.clone();
    let ui_weak_offset = ui_handle.clone();
    ui.on_stereo_offset_changed(move |_offset| {
        let ui = ui_weak_offset.unwrap();
        let state = state_clone.borrow();
        refresh_ui(&ui, &state);
    });

    let state_clone = state.clone();
    let ui_weak_stereo = ui_handle.clone();
    ui.on_stereo_enabled_changed(move |_enabled| {
        let ui = ui_weak_stereo.unwrap();
        let state = state_clone.borrow();
        refresh_ui(&ui, &state);
    });

    // Export PNG
    let state_clone = state.clone();
    let ui_weak_export = ui_handle.clone();
    ui.on_export_frame(move || {
        let ui = ui_weak_export.unwrap();
        let save_path = rfd::FileDialog::new()
            .add_filter("PNG", &["png"])
            .save_file();
        let Some(path) = save_path else {
            return;
        };

        let (width, height, bytes) = {
            let state = state_clone.borrow();
            let img = match state.current_frame() {
                Some(img) => img,
                None => return,
            };
            let filter = state.filter();
            if ui.get_stereo_enabled() {
                let stereo_img = render_stereo(img, ui.get_stereo_offset() as i32);
                process_image_bytes(&stereo_img, filter)
            } else {
                process_image_bytes(img, filter)
            }
        };

        let Some(out_img) = image::RgbaImage::from_raw(width, height, bytes) else {
            ui.set_image_info("Error: failed to build image buffer".into());
            return;
        };

        if let Err(e) = out_img.save(&path) {
            ui.set_image_info(format!("Error saving image: {}", e).into());
        }
    });

    // Copy hex preview
    let state_clone = state.clone();
    let ui_weak_copy = ui_handle.clone();
    ui.on_copy_hex(move || {
        let ui = ui_weak_copy.unwrap();
        let bytes = {
            let state = state_clone.borrow();
            let img = match state.current_frame() {
                Some(img) => img,
                None => return,
            };
            let filter = state.filter();
            let (_width, _height, bytes) = if ui.get_stereo_enabled() {
                let stereo_img = render_stereo(img, ui.get_stereo_offset() as i32);
                process_image_bytes(&stereo_img, filter)
            } else {
                process_image_bytes(img, filter)
            };
            bytes
        };

        let hex_text = format_hex_ascii(&bytes, 4096);
        match arboard::Clipboard::new() {
            Ok(mut clipboard) => {
                if clipboard.set_text(hex_text).is_ok() {
                    ui.set_image_info("Copied hex preview to clipboard".into());
                }
            }
            Err(e) => {
                ui.set_image_info(format!("Clipboard error: {}", e).into());
            }
        }
    });

    // Data Extract dialog
    let state_clone = state.clone();
    data_dialog.on_update_mask(move |idx, checked| {
        state_clone.borrow_mut().set_mask_bit(idx, checked);
    });

    let state_clone = state.clone();
    let ui_weak_overlay = ui_handle.clone();
    data_dialog.on_update_overlay(move |enabled| {
        state_clone.borrow_mut().set_show_mask_overlay(enabled);
        if let Some(ui) = ui_weak_overlay.upgrade() {
            let state = state_clone.borrow();
            refresh_ui(&ui, &state);
        }
    });

    let state_clone = state.clone();
    let dialog_weak = dialog_handle.clone();
    data_dialog.on_request_preview(move || {
        let dialog = dialog_weak.unwrap();
        let state = state_clone.borrow();
        update_extract_preview(&dialog, &state);
    });

    let dialog_weak = dialog_handle.clone();
    data_dialog.on_close_dialog(move || {
        let dialog = dialog_weak.unwrap();
        dialog.hide().ok();
    });

    let state_clone = state.clone();
    let dialog_weak = dialog_handle.clone();
    data_dialog.on_copy_hex_view(move || {
        let dialog = dialog_weak.unwrap();
        let state = state_clone.borrow();
        let img = match state.current_frame() {
            Some(img) => img,
            None => return,
        };

        let bytes = extract_bits_from_image(
            img,
            state.extract_mask(),
            dialog.get_lsb_first(),
            dialog.get_row_order(),
            channel_order_from_index(dialog.get_color_order()),
        );
        let text = format_hex_ascii(&bytes, usize::MAX);
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard.set_text(text).ok();
        }
    });

    let state_clone = state.clone();
    let dialog_weak = dialog_handle.clone();
    data_dialog.on_copy_bin(move || {
        let dialog = dialog_weak.unwrap();
        let state = state_clone.borrow();
        let img = match state.current_frame() {
            Some(img) => img,
            None => return,
        };

        let bytes = extract_bits_from_image(
            img,
            state.extract_mask(),
            dialog.get_lsb_first(),
            dialog.get_row_order(),
            channel_order_from_index(dialog.get_color_order()),
        );
        let text = format_binary(&bytes, usize::MAX);
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard.set_text(text).ok();
        }
    });

    let state_clone = state.clone();
    let dialog_weak = dialog_handle.clone();
    data_dialog.on_copy_hex_raw(move || {
        let dialog = dialog_weak.unwrap();
        let state = state_clone.borrow();
        let img = match state.current_frame() {
            Some(img) => img,
            None => return,
        };

        let bytes = extract_bits_from_image(
            img,
            state.extract_mask(),
            dialog.get_lsb_first(),
            dialog.get_row_order(),
            channel_order_from_index(dialog.get_color_order()),
        );
        let text = format_hex_compact(&bytes, usize::MAX);
        if let Ok(mut clipboard) = arboard::Clipboard::new() {
            clipboard.set_text(text).ok();
        }
    });

    let state_clone = state.clone();
    let dialog_weak = dialog_handle.clone();
    ui.on_open_data_extract(move || {
        let dialog = dialog_weak.unwrap();
        dialog.show().ok();
        let state = state_clone.borrow();
        update_extract_preview(&dialog, &state);
    });

    ui.run()
}
