#![windows_subsystem = "windows"]

use std::collections::{HashMap};
use std::error::{Error};
use std::{fs};
use image::{DynamicImage, Rgba, RgbaImage, imageops};
use native_windows_derive::{NwgUi};
use native_windows_gui::{NativeUi};
extern crate native_windows_gui as nwg;

// Struct to store character data from the font file
struct CharData {
    id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
    yoffset: i32,
    xadvance: u32,
}

// Structure to define the UI elements for the input dialog
#[derive(Default, NwgUi)]
pub struct InputDialog {
    #[nwg_resource(source_bin: Some(ICON_DATA))]
    window_icon: nwg::Icon,

    // Main window configuration
    #[nwg_control(size: (300, 175), center: true, title: "Minecraft Titles [Texture Generator]", flags: "WINDOW|VISIBLE")]
    #[nwg_events(OnWindowClose: [InputDialog::exit])]
    window: nwg::Window,

    // Label for the input field
    #[nwg_control(size: (280, 25), position: (10, 10), text: "Please enter the text to render:")]
    label: nwg::Label,

    // Text input field for entering text to render
    #[nwg_control(size: (280, 25), position: (10, 40))]
    input: nwg::TextInput,

    // Checkbox to enable or disable kerning
    #[nwg_control(size: (280, 25), position: (10, 70), text: "Use kerning")]
    use_kerning_checkbox: nwg::CheckBox,

    // Button to trigger text rendering
    #[nwg_control(size: (280, 25), position: (10, 100), text: "Ok")]
    #[nwg_events(OnButtonClick: [InputDialog::exit])]
    button: nwg::Button,

    #[nwg_control(size: (100, 25), position: (10, 130), text: "About")]
    #[nwg_events(OnButtonClick: [InputDialog::about])]
    about_button: nwg::Button,

    // Layout configuration for the window
    #[nwg_layout(parent: window, spacing: 1)]
    grid_layout: nwg::GridLayout,
}

impl InputDialog {
    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

    fn about(&self) {
        nwg::simple_message("ⓘAbout", "Copyright 2023 Archie★\nVisit my GitHub: https://github.com/ghosthesia\nsource_code:\nhttps://github.com/ArchieC0des/minecraft_titles_texture_generator_bitmap");
    }
}
//load icon
const ICON_DATA: &[u8] = include_bytes!("assets/icon.ico");

fn main() -> Result<(), Box<dyn Error>> {

    // Initialize the GUI framework and set default font
    nwg::init().expect("Failed to init Native Windows GUI");
    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");

    // Build the UI from the defined structure
    let ui = InputDialog::build_ui(Default::default()).expect("Failed to build UI");

    // Set the window icon
    ui.window.set_icon(Some(&ui.window_icon));

    // Start the event dispatch loop for the GUI
    nwg::dispatch_thread_events();

    // Get the entered text and kerning preference from the UI
    let text_to_render = ui.input.text();
    let use_kerning = ui.use_kerning_checkbox.check_state() == nwg::CheckBoxState::Checked;

    // Load font data and images
    const FONT_DATA: &[u8] = include_bytes!("./assets/MinecraftDebugger-bitmap.fnt");
    const FONT_IMAGE: &[u8] = include_bytes!("./assets/MinecraftDebugger-bitmap.png");
    const BACKGROUND_IMAGE: &[u8] = include_bytes!("./assets/uv_checker.png");

    let font_image = image::load_from_memory(FONT_IMAGE)?;
    let bg_image = image::load_from_memory(BACKGROUND_IMAGE)?;

    let (font_data, kerning_pairs) = load_font_data(FONT_DATA)?;

// Render the text and create a final image
    let rendered_image: RgbaImage = render_text(&font_data, &kerning_pairs, &font_image, &text_to_render, use_kerning, 1.5)?;

// Calculate the width and height for the final image with tiled background
    let text_layer_width = rendered_image.width();
    let text_layer_height = rendered_image.height();
    let tiled_bg_height = text_layer_height.max(32); // Ensure at least 32 pixels high

// Create the tiled background and overlay the rendered image on it
    let mut tiled_bg = tile_background(&bg_image, text_layer_width, tiled_bg_height);
    imageops::overlay(&mut tiled_bg, &rendered_image, -1, 0);

    // Create the directory if it doesn't exist
    fs::create_dir_all("./title_texture_map")?;

    // Now save the file in the newly created (or already existing) directory
    tiled_bg.save("./title_texture_map/title_texture_map.png")?;

    Ok(())
}

// Function to load font data from a .fnt file
fn load_font_data(font_data_bytes: &[u8]) -> Result<(HashMap<u32, CharData>, HashMap<(u32, u32), i32>), Box<dyn Error>> {
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

fn render_text(
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

fn tile_background(bg_image: &DynamicImage, width: u32, height: u32) -> RgbaImage {
    let bg_width = bg_image.width();
    let bg_height = bg_image.height();

    // Calculate the number of horizontal tiles needed, ensuring it's a multiple of 16
    let num_horizontal_tiles = ((width + bg_width - 1) / bg_width).max(1);
    let tiled_width = num_horizontal_tiles * bg_width;

    let mut tiled_bg = RgbaImage::new(tiled_width, height);

    let mut y = 0;
    while y < height {
        let mut x = 0;
        while x < tiled_width {
            let crop = bg_image.crop_imm(0, 0, bg_width, bg_height);
            imageops::overlay(&mut tiled_bg, &crop, x as i64, y as i64);
            x += bg_width;
        }
        y += bg_height;
    }

    tiled_bg
}
