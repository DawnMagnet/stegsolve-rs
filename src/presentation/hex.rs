//! # Hex Utilities
//!
//! This module provides utilities for formatting binary data into human-readable strings.
//! It is used for displaying extracted steganography data in various views.
//!
//! ## Key Features
//! - Hex + ASCII side-by-side dump
//! - Compact hex-only representation
//! - Binary bit-string representation

/// Formats bytes as a hex-ascii dump similar to a hex editor.
///
/// Each line displays 16 bytes, with their hexadecimal representation on the left
/// and ASCII representation on the right.
///
/// # Arguments
///
/// * `bytes` - The data to format.
/// * `max_bytes` - Maximum number of bytes to include in the output.
///
/// # Returns
///
/// A formatted string representing the data.
pub fn format_hex_ascii(bytes: &[u8], max_bytes: usize) -> String {
    let mut out = String::new();
    let limit = bytes.len().min(max_bytes);

    for (i, chunk) in bytes[..limit].chunks(16).enumerate() {
        let offset = i * 16;
        out.push_str(&format!("{:08x}  ", offset));
        for j in 0..16 {
            if j < chunk.len() {
                out.push_str(&format!("{:02x} ", chunk[j]));
            } else {
                out.push_str("   ");
            }
        }
        out.push(' ');
        for &b in chunk {
            let ch = if b.is_ascii_graphic() || b == b' ' {
                b as char
            } else {
                '.'
            };
            out.push(ch);
        }
        out.push('\n');
    }

    if bytes.len() > limit {
        out.push_str(&format!("... ({} bytes total)\n", bytes.len()));
    }

    out
}

pub fn format_hex_compact(bytes: &[u8], max_bytes: usize) -> String {
    let mut out = String::new();
    let limit = bytes.len().min(max_bytes);

    for &b in &bytes[..limit] {
        out.push_str(&format!("{:02x}", b));
    }

    if bytes.len() > limit {
        out.push_str(&format!("... ({} bytes total)", bytes.len()));
    }

    out
}

pub fn format_binary(bytes: &[u8], max_bytes: usize) -> String {
    let mut out = String::new();
    let limit = bytes.len().min(max_bytes);

    for (i, &b) in bytes[..limit].iter().enumerate() {
        for bit in (0..8).rev() {
            out.push(if (b >> bit) & 1 == 1 { '1' } else { '0' });
        }
        if i + 1 < limit {
            out.push(' ');
        }
    }

    if bytes.len() > limit {
        out.push_str(&format!(" ... ({} bytes total)", bytes.len()));
    }

    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_hex_ascii() {
        let data = b"Hello, World!";
        let formatted = format_hex_ascii(data, 100);
        assert!(formatted.contains("48 65 6c 6c 6f")); // Hex for "Hello"
        assert!(formatted.contains("Hello, World!")); // ASCII part
    }

    #[test]
    fn test_format_hex_compact() {
        let data = vec![0xDE, 0xAD, 0xBE, 0xEF];
        assert_eq!(format_hex_compact(&data, 4), "deadbeef");
    }

    #[test]
    fn test_format_binary() {
        let data = vec![0b10101010];
        assert_eq!(format_binary(&data, 1), "10101010");
    }

    #[test]
    fn test_max_bytes_limit() {
        let data = vec![0; 100];
        let formatted = format_hex_compact(&data, 10);
        assert!(formatted.contains("... (100 bytes total)"));
    }
}
