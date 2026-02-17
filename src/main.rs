#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use slint::{ComponentHandle, Timer};
use std::{cell::RefCell, rc::Rc};
use stegsolve_rs::ui::{handlers::*, state::AppState};
use stegsolve_rs::{AppWindow, DataExtractDialog};

fn main() -> Result<(), slint::PlatformError> {
    let ui = AppWindow::new()?;
    let data_dialog = DataExtractDialog::new()?;
    let state = Rc::new(RefCell::new(AppState::new()));
    // Load persisted theme selection (if any)
    state.borrow_mut().load_theme_config();
    let playback_timer = Rc::new(Timer::default());

    setup_file_handlers(&ui, &state, &playback_timer);
    setup_filter_handlers(&ui, &state);
    setup_frame_handlers(&ui, &state);
    setup_playback_handlers(&ui, &state, &playback_timer);
    setup_stereo_handlers(&ui, &state);
    setup_export_handlers(&ui, &state);
    setup_extract_handlers(&ui, &data_dialog, &state);
    setup_theme_handlers(&ui, &state);

    // Apply theme index to UI so the icon reflects saved selection
    ui.set_theme_index(state.borrow().theme_index());

    ui.run()
}
