use std::collections::HashMap;
use std::io::Cursor;

use image::DynamicImage;
use image::GenericImageView as _;
use image::ImageBuffer;
use image::Rgba;
use image::codecs::gif::{GifDecoder, GifEncoder};
use image::{AnimationDecoder, Frame};
use rexif::ExifTag;
use wasm_bindgen::prelude::*;

fn rotate_image(img: DynamicImage, orientation: u16) -> DynamicImage {
    match orientation {
        3 => img.rotate180(),
        6 => img.rotate90(),
        8 => img.rotate270(),
        _ => img,
    }
}

fn color_distance(c1: Rgba<u8>, c2: Rgba<u8>) -> f64 {
    let dr = c1[0] as f64 - c2[0] as f64;
    let dg = c1[1] as f64 - c2[1] as f64;
    let db = c1[2] as f64 - c2[2] as f64;
    let da = c1[3] as f64 - c2[3] as f64;
    (dr * dr + dg * dg + db * db + da * da).sqrt()
}

fn adjust_color(color: Rgba<u8>) -> Rgba<u8> {
    let mut new_color = color;
    new_color[0] = new_color[0].saturating_add(8);
    new_color[1] = new_color[1].saturating_add(8);
    new_color[2] = new_color[2].saturating_add(8);
    new_color
}

fn amogify(img: &DynamicImage) -> DynamicImage {
    let (img_width, img_height) = img.dimensions();

    let mut imgbuf = ImageBuffer::new(img_width, img_height);

    // Copy the original image to the output buffer
    for x in 0..img_width {
        for y in 0..img_height {
            let pixel = img.get_pixel(x, y);
            imgbuf.put_pixel(x, y, pixel);
        }
    }

    for x in (0..img_width - 3).step_by(4) {
        for y in (0..img_height - 4).step_by(5) {
            let mut color_counts: HashMap<Rgba<u8>, usize> = HashMap::new();

            for xx in 0..4 {
                for yy in 0..5 {
                    let pixel = img.get_pixel(x + xx, y + yy);
                    *color_counts.entry(pixel).or_insert(0) += 1;
                }
            }

            let mut sorted_colors: Vec<(Rgba<u8>, usize)> = color_counts.into_iter().collect();
            sorted_colors.sort_by(|a, b| b.1.cmp(&a.1));

            let mut selected_colors = Vec::new();
            for (color, _) in sorted_colors {
                if selected_colors.is_empty()
                    || selected_colors
                        .iter()
                        .all(|&c| color_distance(c, color) > 50.0)
                {
                    selected_colors.push(color);
                } else if selected_colors
                    .iter()
                    .any(|&c| color_distance(c, color) <= 50.0)
                {
                    selected_colors.push(adjust_color(color));
                }
                if selected_colors.len() >= 2 {
                    break;
                }
            }

            let top_color = selected_colors
                .get(0)
                .cloned()
                .unwrap_or(Rgba([255, 255, 255, 255]));
            let second_color = match selected_colors.get(1).cloned() {
                Some(color) => color,
                None => adjust_color(top_color),
            };

            // Set the top 2 colors to the output image
            // First column
            imgbuf.put_pixel(x + 1, y, top_color);
            imgbuf.put_pixel(x + 2, y, top_color);
            imgbuf.put_pixel(x + 3, y, top_color);

            // Second column
            imgbuf.put_pixel(x, y + 1, top_color);
            imgbuf.put_pixel(x + 1, y + 1, top_color);
            imgbuf.put_pixel(x + 2, y + 1, second_color);
            imgbuf.put_pixel(x + 3, y + 1, second_color);

            // Third column completely top
            for xx in 0..4 {
                imgbuf.put_pixel(x + xx, y + 2, top_color);
            }

            // Fourth and fifth column second top second top
            for xx in 1..4 {
                let color = if xx % 2 == 0 { second_color } else { top_color };

                imgbuf.put_pixel(x + xx, y + 3, color);
                imgbuf.put_pixel(x + xx, y + 4, color);
            }
        }
    }

    DynamicImage::ImageRgba8(imgbuf)
}

#[wasm_bindgen]
pub struct ConvertedImage {
    preview: Vec<u8>,
    full: Vec<u8>,
    is_animated: bool,
}

#[wasm_bindgen]
impl ConvertedImage {
    #[wasm_bindgen(constructor)]
    pub fn new(preview: Vec<u8>, full: Vec<u8>, is_animated: bool) -> ConvertedImage {
        ConvertedImage { preview, full, is_animated }
    }

    #[wasm_bindgen(getter)]
    pub fn preview(&self) -> Vec<u8> {
        self.preview.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn full(&self) -> Vec<u8> {
        self.full.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn is_animated(&self) -> bool {
        self.is_animated
    }
}

fn is_gif(bytes: &[u8]) -> bool {
    bytes.len() >= 6 && &bytes[0..6] == b"GIF87a" || &bytes[0..6] == b"GIF89a"
}

fn convert_gif(bytes: &[u8]) -> Result<ConvertedImage, JsValue> {
    let cursor = Cursor::new(bytes);
    let decoder = GifDecoder::new(cursor)
        .map_err(|e| JsValue::from_str(&format!("Failed to decode GIF: {:?}", e)))?;

    let frames = decoder.into_frames()
        .collect_frames()
        .map_err(|e| JsValue::from_str(&format!("Failed to collect frames: {:?}", e)))?;

    if frames.is_empty() {
        return Err(JsValue::from_str("No frames found in GIF"));
    }

    let first_frame = &frames[0];
    let (width, height) = first_frame.buffer().dimensions();

    // Convert each frame
    let mut converted_frames = Vec::new();
    for frame in frames {
        let frame_image = DynamicImage::ImageRgba8(frame.buffer().clone());
        let amogified = amogify(&frame_image);

        // Create new frame with same delay
        let delay = frame.delay();
        let new_frame = Frame::from_parts(
            amogified.to_rgba8(),
            0, 0,
            delay
        );
        converted_frames.push(new_frame);
    }

    // Encode as GIF
    let mut output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut output);
        let mut encoder = GifEncoder::new(&mut cursor);
        encoder.set_repeat(image::codecs::gif::Repeat::Infinite)
            .map_err(|e| JsValue::from_str(&format!("Failed to set repeat: {:?}", e)))?;

        for frame in &converted_frames {
            encoder.encode_frame(frame.clone())
                .map_err(|e| JsValue::from_str(&format!("Failed to encode frame: {:?}", e)))?;
        }
    }

    // Create preview from first frame
    let first_amogified = DynamicImage::ImageRgba8(converted_frames[0].buffer().clone());
    let crop_width = std::cmp::min(200, width);
    let crop_height = std::cmp::min(25, height);
    let crop_x = (width - crop_width) / 2;
    let crop_y = (height - crop_height) / 2;
    let cropped_image = first_amogified.crop_imm(crop_x, crop_y, crop_width, crop_height);

    let mut cropped_output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut cropped_output);
        cropped_image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .map_err(|e| JsValue::from_str(&format!("Failed to write preview: {:?}", e)))?;
    }

    Ok(ConvertedImage::new(cropped_output, output, true))
}

#[wasm_bindgen]
pub fn convert_image(bytes: Vec<u8>) -> Result<ConvertedImage, JsValue> {
    // Check if it's a GIF
    if is_gif(&bytes) {
        return convert_gif(&bytes);
    }

    let input_image: DynamicImage =
        image::load_from_memory(&bytes).or_else(|e| Err(JsValue::from_str(&format!("{:?}", e))))?;

    // Not all files have exif data
    let orientation = match rexif::parse_buffer(&bytes) {
        Ok(exif_data) => exif_data
            .entries
            .iter()
            .find(|entry| entry.tag == ExifTag::Orientation)
            .and_then(|entry| entry.value.to_i64(0))
            .unwrap_or(1),
        Err(_) => 1,
    };

    // Rotate the image based on the orientation
    let rotated_image = rotate_image(input_image, orientation.min(u16::MAX as i64) as u16);

    let amogus_image = amogify(&rotated_image);

    let mut output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut output);
        amogus_image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .or_else(|e| Err(JsValue::from_str(&format!("{:?}", e))))?;
    }

    // Create the middle 200x25 pixels image
    let (width, height) = amogus_image.dimensions();
    let crop_width = std::cmp::min(200, width);
    let crop_height = std::cmp::min(25, height);
    let crop_x = (width - crop_width) / 2;
    let crop_y = (height - crop_height) / 2;
    let cropped_image = amogus_image.crop_imm(crop_x, crop_y, crop_width, crop_height);

    let mut cropped_output = Vec::new();
    {
        let mut cursor = Cursor::new(&mut cropped_output);
        cropped_image
            .write_to(&mut cursor, image::ImageFormat::Png)
            .or_else(|e| Err(JsValue::from_str(&format!("{:?}", e))))?;
    }

    Ok(ConvertedImage::new(cropped_output, output, false))
}
