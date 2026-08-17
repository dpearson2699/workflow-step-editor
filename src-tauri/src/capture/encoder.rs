//! PNG encoding of BGRA frame crops.
//!
//! Pure byte arithmetic over an in-memory frame: crop a pixel
//! rectangle out of the BGRA buffer, convert to RGB, and encode one
//! PNG. Runs only on the single capture worker behind the bounded
//! queue, never on the tap callback.

use crate::capture::broker::FrameData;
use crate::capture::geometry::CropPx;

/// Encodes `crop` of `frame` as an RGB PNG. The crop is clamped to the
/// actual buffer dimensions; an empty result after clamping is an
/// error (a crop must never silently produce a zero-size image).
pub fn encode_crop_png(frame: &FrameData, crop: CropPx) -> Result<Vec<u8>, String> {
    let x = crop.x.min(frame.width_px);
    let y = crop.y.min(frame.height_px);
    let w = crop.w.min(frame.width_px - x);
    let h = crop.h.min(frame.height_px - y);
    if w == 0 || h == 0 {
        return Err(format!(
            "empty crop {crop:?} for a {}x{} frame",
            frame.width_px, frame.height_px
        ));
    }

    let mut rgb = Vec::with_capacity(w as usize * h as usize * 3);
    for row in y..y + h {
        let row_start = row as usize * frame.bytes_per_row + x as usize * 4;
        let row_end = row_start + w as usize * 4;
        let Some(row_bytes) = frame.pixels.get(row_start..row_end) else {
            return Err(format!(
                "frame buffer truncated at row {row}: {} bytes, bytes_per_row {}",
                frame.pixels.len(),
                frame.bytes_per_row
            ));
        };
        for bgra in row_bytes.chunks_exact(4) {
            rgb.extend_from_slice(&[bgra[2], bgra[1], bgra[0]]);
        }
    }

    let mut bytes = Vec::new();
    {
        let mut encoder = png::Encoder::new(&mut bytes, w, h);
        encoder.set_color(png::ColorType::Rgb);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.set_compression(png::Compression::Fast);
        let mut writer = encoder
            .write_header()
            .map_err(|error| format!("png header: {error}"))?;
        writer
            .write_image_data(&rgb)
            .map_err(|error| format!("png data: {error}"))?;
        writer
            .finish()
            .map_err(|error| format!("png finish: {error}"))?;
    }
    Ok(bytes)
}

#[cfg(test)]
mod tests {
    use crate::capture::geometry::{DisplayGeometry, RectPt};

    use super::*;

    /// A 4x3 BGRA frame with row padding whose pixel at (x, y) encodes
    /// its coordinates: B=x, G=y, R=9.
    fn coordinate_frame() -> FrameData {
        let (width, height, bytes_per_row) = (4_u32, 3_u32, 20_usize);
        let mut pixels = vec![0_u8; bytes_per_row * height as usize];
        for y in 0..height {
            for x in 0..width {
                let offset = y as usize * bytes_per_row + x as usize * 4;
                pixels[offset] = x as u8;
                pixels[offset + 1] = y as u8;
                pixels[offset + 2] = 9;
                pixels[offset + 3] = 255;
            }
        }
        FrameData {
            display: DisplayGeometry {
                id: 1,
                frame_pt: RectPt::new(0.0, 0.0, 4.0, 3.0),
                scale: 1.0,
            },
            width_px: width,
            height_px: height,
            bytes_per_row,
            ts_ns: 0,
            pixels,
        }
    }

    fn decode(bytes: &[u8]) -> (u32, u32, Vec<u8>) {
        let decoder = png::Decoder::new(std::io::Cursor::new(bytes));
        let mut reader = decoder.read_info().unwrap();
        let mut buffer = vec![0; reader.output_buffer_size().unwrap()];
        let info = reader.next_frame(&mut buffer).unwrap();
        buffer.truncate(info.buffer_size());
        (info.width, info.height, buffer)
    }

    #[test]
    fn crops_encode_to_decodable_rgb_pngs_with_exact_pixels() {
        let frame = coordinate_frame();
        let bytes = encode_crop_png(
            &frame,
            CropPx {
                x: 1,
                y: 1,
                w: 2,
                h: 2,
            },
        )
        .unwrap();
        let (w, h, rgb) = decode(&bytes);
        assert_eq!((w, h), (2, 2));
        // Row-major RGB: (1,1), (2,1), (1,2), (2,2) with R=9, G=y, B=x.
        assert_eq!(
            rgb,
            vec![9, 1, 1, 9, 1, 2, 9, 2, 1, 9, 2, 2],
        );
    }

    #[test]
    fn oversized_crops_clamp_to_the_buffer() {
        let frame = coordinate_frame();
        let bytes = encode_crop_png(
            &frame,
            CropPx {
                x: 2,
                y: 0,
                w: 99,
                h: 99,
            },
        )
        .unwrap();
        let (w, h, _) = decode(&bytes);
        assert_eq!((w, h), (2, 3));
    }

    #[test]
    fn an_empty_crop_is_an_error() {
        let frame = coordinate_frame();
        let error = encode_crop_png(
            &frame,
            CropPx {
                x: 4,
                y: 0,
                w: 5,
                h: 5,
            },
        )
        .unwrap_err();
        assert!(error.contains("empty crop"), "got {error}");
    }
}
