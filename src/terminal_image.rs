use std::cmp;

use ratatui::{prelude::Rect, widgets::Widget};
use viuer::{print_from_file, Config};

pub struct TerminalImage {
    path: String,
}

impl TerminalImage {
    pub fn new(path: &str) -> TerminalImage {
        TerminalImage {
            path: path.to_string(),
        }
    }
}

impl Widget for TerminalImage {
    fn render(self, area: Rect, _: &mut ratatui::buffer::Buffer) {
        let y = area.y as i16;
        let width = area.width;
        let height = area.height;

        let size = cmp::min(width, height);

        // Ensure the image is rendered correctly by not rendering outside of box width
        let image_width = match (size * 2) > width {
            true => width,
            false => (size * 2) + 1,
        };

        // Height is adjusted automatically
        let image_height = size;

        // Center image to half of width minus half of image width (and padding of 1)
        let x = ((width / 2) - (image_width / 2)) + 1;

        let conf = Config {
            x,
            y,
            width: Some(image_width as u32),
            height: Some(image_height as u32),
            ..Default::default()
        };

        if let Err(e) = print_from_file(self.path, &conf) {
            eprintln!("Image printing failed: {}", e);
        }
    }
}
