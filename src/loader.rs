use image::codecs::gif::GifDecoder;
use image::{AnimationDecoder, DynamicImage, ImageFormat};
use std::io::Cursor;

pub fn load_frames_from_bytes(bytes: &[u8]) -> Result<Vec<DynamicImage>, anyhow::Error> {
    let mut frames: Vec<DynamicImage> = Vec::new();

    if let Ok(format) = image::guess_format(bytes) {
        if format == ImageFormat::Gif {
            if let Ok(decoder) = GifDecoder::new(Cursor::new(bytes)) {
                for frame in decoder.into_frames() {
                    if let Ok(frame) = frame {
                        frames.push(DynamicImage::ImageRgba8(frame.into_buffer()));
                    }
                }
            }
        }
    }

    if frames.is_empty() {
        frames.push(load_image_robust_from_bytes(bytes)?);
    }

    Ok(frames)
}

fn load_image_robust_from_bytes(bytes: &[u8]) -> Result<DynamicImage, anyhow::Error> {
    if let Ok(img) = image::load_from_memory(bytes) {
        return Ok(img);
    }

    let formats = [
        ImageFormat::Png,
        ImageFormat::Jpeg,
        ImageFormat::Gif,
        ImageFormat::Bmp,
        ImageFormat::Ico,
        ImageFormat::Tiff,
        ImageFormat::WebP,
        ImageFormat::Pnm,
        ImageFormat::Tga,
        ImageFormat::Dds,
    ];

    for &format in &formats {
        if let Ok(img) = image::load_from_memory_with_format(bytes, format) {
            return Ok(img);
        }
    }

    Err(anyhow::anyhow!(
        "Unrecognized image format. Stegsolve-rs tried all decoders but failed."
    ))
}
