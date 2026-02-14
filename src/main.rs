#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::{ComponentHandle, Timer};
use std::{cell::RefCell, rc::Rc};
use stegsolve_rs::ui::{handlers::*, state::AppState};
use stegsolve_rs::{AppWindow, DataExtractDialog};

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let data_dialog = DataExtractDialog::new()?;
    let state = Rc::new(RefCell::new(AppState::new()));
    let playback_timer = Rc::new(Timer::default());

    setup_file_handlers(&ui, &state, &playback_timer);
    setup_filter_handlers(&ui, &state);
    setup_frame_handlers(&ui, &state);
    setup_playback_handlers(&ui, &state, &playback_timer);
    setup_stereo_handlers(&ui, &state);
    setup_export_handlers(&ui, &state);
    setup_extract_handlers(&ui, &data_dialog, &state);

    ui.run()
}
