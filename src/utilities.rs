use image::{DynamicImage, RgbaImage, imageops};

fn tile_background(bg_image: &DynamicImage, width: u32, height: u32) -> RgbaImage {
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
