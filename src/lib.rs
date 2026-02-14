//! # Stegsolve-RS
//!
//! A Rust implementation of Stegsolve for steganography analysis.
//!
//! This library provides functionality for:
//! - Loading and processing images
//! - Applying various analysis filters
//! - Extracting hidden data using LSB and bit-plane techniques
//! - Rendering stereo views and formatted output

slint::include_modules!();

pub mod core;
pub mod presentation;
pub mod ui;

// Re-export commonly used types
pub use core::extract::extract_bits_from_image;
pub use core::loader::load_frames_from_bytes;
pub use core::FilterMode;
