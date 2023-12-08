#![windows_subsystem = "windows"]

mod utilities;

use std::error::{Error};
use std::{fs};
use image::{RgbaImage, imageops};
use native_windows_derive::{NwgUi};
use native_windows_gui::{NativeUi};
use crate::utilities::{load_font_data, render_text, tile_background};

extern crate native_windows_gui as nwg;

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


