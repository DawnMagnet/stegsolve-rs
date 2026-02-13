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
        out.push_str(" ");
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
