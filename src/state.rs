use crate::filters::FilterMode;
use image::DynamicImage;

pub struct AppState {
    frames: Vec<DynamicImage>,
    index: usize,
    filter: FilterMode,
    file_name: String,
    extract_mask: u32,
    is_playing: bool,
    show_mask_overlay: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            frames: Vec::new(),
            index: 0,
            filter: FilterMode::Normal,
            file_name: String::from(""),
            extract_mask: 0,
            is_playing: false,
            show_mask_overlay: false,
        }
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
        if idx < 0 || idx >= 32 {
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
