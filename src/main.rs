mod action;
mod app;
mod device;
mod focused_block;
mod notification;

use crate::app::App;

fn main() -> anyhow::Result<()> {
    let mut app = App::new();

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
