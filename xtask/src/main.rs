use std::env::current_dir;

use ratasvg::svg;
use ratatui::prelude::*;
use tui_big_text::BigText;

struct Header<'a> {
    title: &'a str,
}

impl Widget for Header<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        BigText::builder()
            .lines([self.title.into()])
            .centered()
            .build()
            .render(area.centered_vertically(Constraint::Length(8)), buf);
    }
}

fn main() {
    let opts = ratasvg::Options {
        width_px: 1280,
        height_px: 720,
        font_size_px: 20,
        background_color: "black",
    };

    let current_dir = current_dir().unwrap();
    let package_name = current_dir.file_name().unwrap().to_str().unwrap();

    let document = ratasvg::build_svg_from_widget(
        Header {
            title: package_name,
        },
        opts,
    );

    svg::save("media/header.svg", &document).unwrap();
}
