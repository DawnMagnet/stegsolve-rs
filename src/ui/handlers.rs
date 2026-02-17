use crate::AppWindow;
use crate::DataExtractDialog;
use crate::core::extract::{channel_order_from_index, extract_bits_from_image};
use crate::core::filters::process_image_bytes;
use crate::core::loader::load_frames_from_bytes;
use crate::presentation::hex::{format_binary, format_hex_ascii, format_hex_compact};
use crate::presentation::stereo::render_stereo;
use crate::ui::helpers::{refresh_ui, update_extract_preview};
use crate::ui::state::ThemeMode;
use crate::ui::state::{AppState, FilterNavigation, FrameNavigation, MaskUpdate};
use slint::{ComponentHandle, SharedString, Timer, TimerMode};
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

pub fn setup_file_handlers(
    ui: &AppWindow,
    state: &Rc<RefCell<AppState>>,
    playback_timer: &Rc<Timer>,
) {
    let ui_handle = ui.as_weak();
    let state_clone = state.clone();
    let playback_timer_clone = playback_timer.clone();
    ui.on_open_file(move || {
        let ui = ui_handle.unwrap();
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
                    ui.set_image_info(SharedString::from(format!("Error reading file: {}", e)));
                    return;
                }
            };
            let frames = match load_frames_from_bytes(&bytes) {
                Ok(frames) => frames,
                Err(e) => {
                    ui.set_image_info(SharedString::from(format!("Error: {}", e)));
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
}

pub fn setup_filter_handlers(ui: &AppWindow, state: &Rc<RefCell<AppState>>) {
    let ui_handle = ui.as_weak();
    let state_clone = state.clone();
    ui.on_next_filter(move || {
        let ui = ui_handle.unwrap();
        state_clone.borrow_mut().next_filter();
        refresh_ui(&ui, &state_clone.borrow());
    });
    let state_clone = state.clone();
    let ui_handle = ui.as_weak();
    ui.on_prev_filter(move || {
        let ui = ui_handle.unwrap();
        state_clone.borrow_mut().prev_filter();
        refresh_ui(&ui, &state_clone.borrow());
    });
}

pub fn setup_frame_handlers(ui: &AppWindow, state: &Rc<RefCell<AppState>>) {
    let ui_handle = ui.as_weak();
    let state_clone = state.clone();
    ui.on_next_frame(move || {
        let ui = ui_handle.unwrap();
        state_clone.borrow_mut().next_frame();
        refresh_ui(&ui, &state_clone.borrow());
    });
    let state_clone = state.clone();
    let ui_handle = ui.as_weak();
    ui.on_prev_frame(move || {
        let ui = ui_handle.unwrap();
        state_clone.borrow_mut().prev_frame();
        refresh_ui(&ui, &state_clone.borrow());
    });
}

pub fn setup_playback_handlers(
    ui: &AppWindow,
    state: &Rc<RefCell<AppState>>,
    playback_timer: &Rc<Timer>,
) {
    let ui_handle = ui.as_weak();
    let state_clone = state.clone();
    let playback_timer_clone = playback_timer.clone();
    let ui_handle_for_timer = ui.as_weak();
    ui.on_toggle_play(move || {
        let ui = ui_handle.unwrap();
        let can_play = state_clone.borrow().frame_count() > 1;
        if !can_play {
            return;
        }

        let playing = state_clone.borrow().is_playing();
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
            state_for_timer.borrow_mut().next_frame();
            refresh_ui(&ui, &state_for_timer.borrow());
        });
    });
}

pub fn setup_stereo_handlers(ui: &AppWindow, state: &Rc<RefCell<AppState>>) {
    let ui_handle = ui.as_weak();
    let state_clone = state.clone();
    ui.on_stereo_offset_changed(move |_| {
        let ui = ui_handle.unwrap();
        refresh_ui(&ui, &state_clone.borrow());
    });
    let state_clone = state.clone();
    let ui_handle = ui.as_weak();
    ui.on_stereo_enabled_changed(move |_| {
        let ui = ui_handle.unwrap();
        refresh_ui(&ui, &state_clone.borrow());
    });
}

pub fn setup_export_handlers(ui: &AppWindow, state: &Rc<RefCell<AppState>>) {
    let ui_handle = ui.as_weak();
    let state_clone = state.clone();
    ui.on_export_frame(move || {
        let ui = ui_handle.unwrap();
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
        if let Some(out_img) = image::RgbaImage::from_raw(width, height, bytes) {
            if let Err(e) = out_img.save(&path) {
                ui.set_image_info(SharedString::from(format!("Error saving image: {}", e)));
            }
        } else {
            ui.set_image_info(SharedString::from("Error: failed to build image buffer"));
        }
    });

    let state_clone = state.clone();
    let ui_handle = ui.as_weak();
    ui.on_copy_hex(move || {
        let ui = ui_handle.unwrap();
        let bytes = {
            let state = state_clone.borrow();
            let img = match state.current_frame() {
                Some(img) => img,
                None => return,
            };
            let filter = state.filter();
            let (_, _, bytes) = if ui.get_stereo_enabled() {
                let stereo_img = render_stereo(img, ui.get_stereo_offset() as i32);
                process_image_bytes(&stereo_img, filter)
            } else {
                process_image_bytes(img, filter)
            };
            bytes
        };
        let hex_text = format_hex_ascii(&bytes, 4096);
        if arboard::Clipboard::new()
            .and_then(|mut cb| cb.set_text(hex_text))
            .is_ok()
        {
            ui.set_image_info(SharedString::from("Copied hex preview to clipboard"));
        }
    });
}

pub fn setup_extract_handlers(
    ui: &AppWindow,
    data_dialog: &DataExtractDialog,
    state: &Rc<RefCell<AppState>>,
) {
    let state_clone = state.clone();
    data_dialog.on_update_mask(move |idx, checked| {
        state_clone.borrow_mut().set_mask_bit(idx, checked);
    });

    let state_clone = state.clone();
    let ui_handle = ui.as_weak();
    data_dialog.on_update_overlay(move |enabled| {
        state_clone.borrow_mut().set_show_mask_overlay(enabled);
        if let Some(ui) = ui_handle.upgrade() {
            refresh_ui(&ui, &state_clone.borrow());
        }
    });

    let state_clone = state.clone();
    let dialog_handle = data_dialog.as_weak();
    data_dialog.on_request_preview(move || {
        let dialog = dialog_handle.unwrap();
        update_extract_preview(&dialog, &state_clone.borrow());
    });

    let dialog_handle = data_dialog.as_weak();
    data_dialog.on_close_dialog(move || {
        dialog_handle.unwrap().hide().ok();
    });

    let state_clone = state.clone();
    let dialog_handle = data_dialog.as_weak();
    data_dialog.on_copy_hex_view(move || {
        let dialog = dialog_handle.unwrap();
        let state = state_clone.borrow();
        let Some(img) = state.current_frame() else {
            return;
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
    let dialog_handle = data_dialog.as_weak();
    data_dialog.on_copy_bin(move || {
        let dialog = dialog_handle.unwrap();
        let state = state_clone.borrow();
        let Some(img) = state.current_frame() else {
            return;
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
    let dialog_handle = data_dialog.as_weak();
    data_dialog.on_copy_hex_raw(move || {
        let dialog = dialog_handle.unwrap();
        let state = state_clone.borrow();
        let Some(img) = state.current_frame() else {
            return;
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
    let dialog_handle = data_dialog.as_weak();
    ui.on_open_data_extract(move || {
        let dialog = dialog_handle.unwrap();
        dialog.show().ok();
        update_extract_preview(&dialog, &state_clone.borrow());
    });
}

pub fn setup_theme_handlers(ui: &AppWindow, state: &Rc<RefCell<AppState>>) {
    // Apply saved theme on setup
    apply_theme_to_ui(ui, state.borrow().theme_mode());

    let ui_handle = ui.as_weak();
    let state_clone = state.clone();
    ui.on_toggle_theme(move || {
        let ui = ui_handle.unwrap();
        let mut s = state_clone.borrow_mut();
        let current = s.theme_mode();
        let next = match current {
            ThemeMode::System => ThemeMode::Light,
            ThemeMode::Light => ThemeMode::Dark,
            ThemeMode::Dark => ThemeMode::System,
        };
        s.set_theme_mode(next);
        s.save_theme_config();
        ui.set_theme_index(s.theme_index());
        apply_theme_to_ui(&ui, next);
    });
}

fn apply_theme_to_ui(ui: &AppWindow, mode: ThemeMode) {
    use slint::ComponentHandle;
    use slint::language::ColorScheme;
    let scheme = match mode {
        ThemeMode::System => ColorScheme::Unknown,
        ThemeMode::Light => ColorScheme::Light,
        ThemeMode::Dark => ColorScheme::Dark,
    };
    ui.global::<crate::Palette>().set_color_scheme(scheme);
}
