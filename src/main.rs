use std::io;

mod app;
mod device;
mod notification;
mod action;
mod focused_block;

use crate::app::App;

fn main() -> io::Result<()> {
    let mut app = App::new()?;

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
