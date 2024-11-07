use std::io::Cursor;

use image::DynamicImage;
use image::GenericImageView as _;
use image::ImageBuffer;
use image::Rgba;
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
            let mut colors = [None; 2];
            let mut color_count = 0;

            for xx in 0..4 {
                for yy in 0..5 {
                    let pixel = img.get_pixel(x + xx, y + yy);
                    if !colors.contains(&Some(pixel)) {
                        if color_count < 2 {
                            colors[color_count] = Some(pixel);
                            color_count += 1;
                        }
                    }
                }
            }

            let top_color = colors[0].unwrap();
            let second_color = colors[1].unwrap_or(Rgba([255, 255, 255, 255]));

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
}

#[wasm_bindgen]
impl ConvertedImage {
    #[wasm_bindgen(constructor)]
    pub fn new(preview: Vec<u8>, full: Vec<u8>) -> ConvertedImage {
        ConvertedImage { preview, full }
    }

    #[wasm_bindgen(getter)]
    pub fn preview(&self) -> Vec<u8> {
        self.preview.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn full(&self) -> Vec<u8> {
        self.full.clone()
    }
}

#[wasm_bindgen]
pub fn convert_image(bytes: Vec<u8>) -> Result<ConvertedImage, JsValue> {
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
        Err(e) => 1,
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

    Ok(ConvertedImage::new(cropped_output, output))
}
