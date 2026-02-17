use crate::core::filters::FilterMode;
use image::DynamicImage;
use serde::{Deserialize, Serialize};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThemeMode {
    System,
    Light,
    Dark,
}

pub struct AppState {
    frames: Vec<DynamicImage>,
    index: usize,
    filter: FilterMode,
    file_name: String,
    extract_mask: u32,
    is_playing: bool,
    show_mask_overlay: bool,
    theme_mode: ThemeMode,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            frames: Vec::new(),
            index: 0,
            filter: FilterMode::Normal,
            file_name: String::from(""),
            extract_mask: 0,
            is_playing: false,
            show_mask_overlay: false,
            theme_mode: ThemeMode::System,
        }
    }
}

impl AppState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_frames(&mut self, frames: Vec<DynamicImage>, file_name: String) {
        self.frames = frames;
        self.index = 0;
        self.filter = FilterMode::Normal;
        self.file_name = file_name;
        self.is_playing = false;
    }

    pub fn current_frame(&self) -> Option<&DynamicImage> {
        self.frames.get(self.index)
    }

    pub fn frame_count(&self) -> usize {
        self.frames.len()
    }

    pub fn frame_index(&self) -> usize {
        self.index
    }

    pub fn filter(&self) -> FilterMode {
        self.filter
    }

    pub fn file_name(&self) -> &str {
        &self.file_name
    }

    pub fn is_playing(&self) -> bool {
        self.is_playing
    }

    pub fn set_playing(&mut self, playing: bool) {
        self.is_playing = playing;
    }

    pub fn extract_mask(&self) -> u32 {
        self.extract_mask
    }

    pub fn show_mask_overlay(&self) -> bool {
        self.show_mask_overlay
    }

    pub fn set_show_mask_overlay(&mut self, enabled: bool) {
        self.show_mask_overlay = enabled;
    }

    pub fn theme_mode(&self) -> ThemeMode {
        self.theme_mode
    }

    pub fn set_theme_mode(&mut self, mode: ThemeMode) {
        self.theme_mode = mode;
    }

    pub fn theme_index(&self) -> i32 {
        match self.theme_mode {
            ThemeMode::System => 0,
            ThemeMode::Light => 1,
            ThemeMode::Dark => 2,
        }
    }

    pub fn load_theme_config(&mut self) {
        // Persist only theme selection using confy to avoid serializing image data
        #[derive(Serialize, Deserialize)]
        struct ThemeConfig {
            theme_mode: ThemeMode,
        }

        impl Default for ThemeConfig {
            fn default() -> Self {
                ThemeConfig {
                    theme_mode: ThemeMode::System,
                }
            }
        }

        if let Ok(cfg) = confy::load::<ThemeConfig>("stegsolve-rs", None) {
            self.theme_mode = cfg.theme_mode;
        }
    }

    pub fn save_theme_config(&self) {
        #[derive(Serialize, Deserialize)]
        struct ThemeConfig {
            theme_mode: ThemeMode,
        }

        impl Default for ThemeConfig {
            fn default() -> Self {
                ThemeConfig {
                    theme_mode: ThemeMode::System,
                }
            }
        }

        let cfg = ThemeConfig {
            theme_mode: self.theme_mode,
        };
        let _ = confy::store("stegsolve-rs", None, cfg);
    }
}

pub trait FrameNavigation {
    fn next_frame(&mut self);
    fn prev_frame(&mut self);
}

impl FrameNavigation for AppState {
    fn next_frame(&mut self) {
        if self.frames.is_empty() {
            return;
        }
        self.index = (self.index + 1) % self.frames.len();
    }

    fn prev_frame(&mut self) {
        if self.frames.is_empty() {
            return;
        }
        if self.index == 0 {
            self.index = self.frames.len() - 1;
        } else {
            self.index -= 1;
        }
    }
}

pub trait FilterNavigation {
    fn next_filter(&mut self);
    fn prev_filter(&mut self);
}

impl FilterNavigation for AppState {
    fn next_filter(&mut self) {
        self.filter = self.filter.next();
    }

    fn prev_filter(&mut self) {
        self.filter = self.filter.prev();
    }
}

pub trait MaskUpdate {
    fn set_mask_bit(&mut self, idx: i32, checked: bool);
}

impl MaskUpdate for AppState {
    fn set_mask_bit(&mut self, idx: i32, checked: bool) {
        if !(0..32).contains(&idx) {
            return;
        }
        let bit = 1u32 << (idx as u32);
        if checked {
            self.extract_mask |= bit;
        } else {
            self.extract_mask &= !bit;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{DynamicImage, RgbaImage};

    #[test]
    fn test_new_state() {
        let state = AppState::new();
        assert_eq!(state.frame_count(), 0);
        assert_eq!(state.filter(), FilterMode::Normal);
        assert_eq!(state.extract_mask(), 0);
    }

    #[test]
    fn test_frame_navigation() {
        let mut state = AppState::new();
        let frames = vec![
            DynamicImage::ImageRgba8(RgbaImage::new(1, 1)),
            DynamicImage::ImageRgba8(RgbaImage::new(1, 1)),
        ];
        state.set_frames(frames, "test.png".to_string());

        assert_eq!(state.frame_index(), 0);
        state.next_frame();
        assert_eq!(state.frame_index(), 1);
        state.next_frame();
        assert_eq!(state.frame_index(), 0); // Wrap

        state.prev_frame();
        assert_eq!(state.frame_index(), 1); // Wrap
    }

    #[test]
    fn test_filter_navigation() {
        let mut state = AppState::new();
        assert_eq!(state.filter(), FilterMode::Normal);
        state.next_filter();
        assert_ne!(state.filter(), FilterMode::Normal);
        state.prev_filter();
        assert_eq!(state.filter(), FilterMode::Normal);
    }

    #[test]
    fn test_mask_bits() {
        let mut state = AppState::new();
        state.set_mask_bit(0, true);
        assert_eq!(state.extract_mask(), 1);
        state.set_mask_bit(8, true);
        assert_eq!(state.extract_mask(), 1 | (1 << 8));
        state.set_mask_bit(0, false);
        assert_eq!(state.extract_mask(), 1 << 8);
    }
}
