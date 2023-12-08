use std::collections::HashMap;
use std::error::Error;
use image::{DynamicImage, RgbaImage, imageops, Rgba};

pub struct CharData {
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    yoffset: i32,
    xadvance: u32,
}

// Function to load font data from a .fnt file
pub fn load_font_data(font_data_bytes: &[u8]) -> Result<(HashMap<u32, CharData>, HashMap<(u32, u32), i32>), Box<dyn Error>> {
    let font_data_str = std::str::from_utf8(font_data_bytes)?;

    let mut char_data_map = HashMap::new();
    let mut kerning_pairs = HashMap::new();

    for line in font_data_str.lines() {

        if line.starts_with("char id=") {
            let char_data = parse_char_line(&line)?;
            char_data_map.insert(char_data.id, char_data);
        } else if line.starts_with("kerning first=") {
            let (first, second, amount) = parse_kerning_line(&line)?;
            kerning_pairs.insert((first, second), amount);
        }
    }

    Ok((char_data_map, kerning_pairs))
}

fn parse_char_line(line: &str) -> Result<CharData, Box<dyn Error>> {
    let parts: HashMap<&str, String> = line.split_whitespace()
        .filter(|part| part.contains('='))
        .map(|part| {
            let mut split = part.split('=');
            (split.next().unwrap(), split.next().unwrap().to_string())
        })
        .collect();

    let id = parts.get("id")
        .ok_or("Error: ID not found")?
        .parse()
        .map_err(|e| format!("Error parsing ID '{}' from line '{}': {}", parts.get("id").unwrap(), line, e))?;

    let x = parts.get("x")
        .ok_or("Error: X coordinate not found")?
        .parse()
        .map_err(|e| format!("Error parsing X coordinate '{}' from line '{}': {}", parts.get("x").unwrap(), line, e))?;

    let y = parts.get("y")
        .ok_or("Error: Y coordinate not found")?
        .parse()
        .map_err(|e| format!("Error parsing Y coordinate '{}' from line '{}': {}", parts.get("y").unwrap(), line, e))?;

    let width = parts.get("width")
        .ok_or("Error: Width not found")?
        .parse()
        .map_err(|e| format!("Error parsing width '{}' from line '{}': {}", parts.get("width").unwrap(), line, e))?;

    let height = parts.get("height")
        .ok_or("Error: Height not found")?
        .parse()
        .map_err(|e| format!("Error parsing height '{}' from line '{}': {}", parts.get("height").unwrap(), line, e))?;

    let yoffset = parts.get("yoffset")
        .ok_or("Error: Y offset not found")?
        .parse()
        .map_err(|e| format!("Error parsing Y offset '{}' from line '{}': {}", parts.get("yoffset").unwrap(), line, e))?;

    let xadvance = parts.get("xadvance")
        .ok_or("Error: Xadvance not found")?
        .parse()
        .map_err(|e| format!("Error parsing Xadvance '{}' from line '{}': {}", parts.get("xadvance").unwrap(), line, e))?;

    Ok(CharData { id, x, y, width, height, yoffset, xadvance })
}

fn parse_kerning_line(line: &str) -> Result<(u32, u32, i32), Box<dyn Error>> {
    let parts: HashMap<&str, String> = line.split_whitespace()
        .filter(|part| part.contains('='))
        .map(|part| {
            let mut split = part.split('=');
            (split.next().unwrap(), split.next().unwrap().to_string())
        })
        .collect();

    let first = parts.get("first")
        .ok_or("Error: First not found")?
        .parse()
        .map_err(|e| format!("Error parsing First '{}' from line '{}': {}", parts.get("first").unwrap(), line, e))?;

    let second = parts.get("second")
        .ok_or("Error: Second not found")?
        .parse()
        .map_err(|e| format!("Error parsing Second '{}' from line '{}': {}", parts.get("second").unwrap(), line, e))?;

    let amount = parts.get("amount")
        .ok_or("Error: Amount not found")?
        .parse()
        .map_err(|e| format!("Error parsing Amount '{}' from line '{}': {}", parts.get("amount").unwrap(), line, e))?;

    Ok((first, second, amount))
}

pub fn render_text(
    font_data: &HashMap<u32, CharData>,
    kerning_pairs: &HashMap<(u32, u32), i32>,
    font_image: &DynamicImage,
    text: &str,
    use_kerning: bool,
    scale_factor: f32,
) -> Result<RgbaImage, Box<dyn Error>> {
    let (total_width, max_height) = text.chars().fold((0, 0), |(width, height), ch| {
        font_data.get(&(ch as u32)).map_or((width, height), |char_data| {
            (width + char_data.xadvance.saturating_sub(2), height.max(char_data.height as i32 + char_data.yoffset))
        })
    });

    let canvas_height = max_height as u32 + 10; // Original padding (5) + 5 extra pixels
    let mut target_image = RgbaImage::new(total_width, canvas_height);
    let mut highlight_image = RgbaImage::new(total_width, canvas_height);

    let base_line: i32 = font_data.values()
        .map(|char_data| char_data.yoffset)
        .max()
        .unwrap_or(0) + 5; // Adjust baseline for the extra canvas height

    for x in 0..total_width {
        target_image.put_pixel(x, base_line as u32, Rgba([255, 0, 0, 255])); // Red color for baseline
    }

    let mut cursor_x: u32 = 0;
    let mut last_char_id: Option<u32> = None;

    for ch in text.chars() {
        let char_id = ch as u32;

        if use_kerning {
            if let Some(last_id) = last_char_id {
                if let Some(kerning) = kerning_pairs.get(&(last_id, char_id)) {
                    cursor_x = (cursor_x as i32 + kerning).max(0) as u32;
                }
            }
        }

        if let Some(char_data) = font_data.get(&char_id) {
            let crop_x = char_data.x.saturating_add(1);
            let crop_width = char_data.width.saturating_sub(2).max(1);
            let char_img = font_image.crop_imm(crop_x, char_data.y, crop_width, char_data.height);
            let render_y = base_line - char_data.height as i32 - char_data.yoffset;

            imageops::overlay(&mut target_image, &char_img, cursor_x.into(), render_y.into());

            cursor_x += char_data.xadvance.saturating_sub(3);
        }

        last_char_id = Some(char_id);
    }

    let highlight_color = Rgba([0, 255, 0, 128]); // 50% transparent green for highlight
    let baseline_color = Rgba([255, 0, 0, 255]); // Red color for baseline
    for x in 0..total_width {
        let mut column_has_text = false;
        for y in 0..canvas_height {
            let pixel = target_image.get_pixel(x, y);
            if pixel.0[3] != 0 && *pixel != baseline_color {
                column_has_text = true;
                break;
            }
        }
        if column_has_text {
            for y in 0..canvas_height {
                highlight_image.put_pixel(x, y, highlight_color);
            }
        }
    }


// Resize the highlight image if necessary
    let new_height = (canvas_height as f32 * scale_factor).round() as u32;
    let final_height = new_height.min(32); // Ensure the height does not exceed 32 pixels
    highlight_image = imageops::resize(&highlight_image, total_width, final_height, imageops::FilterType::Nearest);

// Define new colors (without alpha channel)
    let cyan = Rgba([0, 255, 255, 0]); // Cyan without alpha
    let purple = Rgba([128, 0, 128, 0]); // Purple without alpha

    for y in 0..final_height {
        for x in 0..total_width {
            let original_pixel = highlight_image.get_pixel(x, y);
            let mut new_pixel = *original_pixel; // Create a copy of the original pixel

            if y >= 27 && y <= 32 {
                // Set the cyan color while keeping the original alpha
                new_pixel = Rgba([cyan[0], cyan[1], cyan[2], original_pixel[3]]);
            } else if y >= 21 && y <= 25 {
                // Set the purple color while keeping the original alpha
                new_pixel = Rgba([purple[0], purple[1], purple[2], original_pixel[3]]);
            }

            highlight_image.put_pixel(x, y, new_pixel); // Place the new pixel
        }
    }

// Create the final image and overlay the highlight and text images
    let mut final_image = RgbaImage::new(total_width, final_height);
    imageops::overlay(&mut final_image, &highlight_image, 0, 0); // Place the highlight
    imageops::overlay(&mut final_image, &target_image, 0, 0); // Then, place the original text

    Ok(final_image)
}

// generate background based on an image that gets tiled
pub fn tile_background(bg_image: &DynamicImage, width: u32, height: u32) -> RgbaImage {
    let bg_width = bg_image.width();
    let bg_height = bg_image.height();

    let num_horizontal_tiles = ((width + bg_width - 1) / bg_width).max(1);
    let tiled_width = num_horizontal_tiles * bg_width;
    let tiled_bg = RgbaImage::new(tiled_width, height);

    tile_background_helper(&bg_image, &tiled_bg, bg_width, bg_height, 0, 0, tiled_width, height)
}

fn tile_background_helper(
    bg_image: &DynamicImage,
    tiled_bg: &RgbaImage,
    bg_width: u32,
    bg_height: u32,
    current_x: u32,
    current_y: u32,
    total_width: u32,
    total_height: u32,
) -> RgbaImage {
    let mut new_tiled_bg = tiled_bg.clone();

    if current_y >= total_height {
        return new_tiled_bg;
    }

    if current_x < total_width {
        let crop = bg_image.crop_imm(0, 0, bg_width, bg_height);
        imageops::overlay(&mut new_tiled_bg, &crop, current_x as i64, current_y as i64);

        return tile_background_helper(
            bg_image,
            &new_tiled_bg,
            bg_width,
            bg_height,
            current_x + bg_width,
            current_y,
            total_width,
            total_height,
        );
    }

    tile_background_helper(
        bg_image,
        &new_tiled_bg,
        bg_width,
        bg_height,
        0,
        current_y + bg_height,
        total_width,
        total_height,
    )
}
