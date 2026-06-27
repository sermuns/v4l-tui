mod action;
mod app;
mod device;
mod focused_block;
mod notification;

use crate::app::App;

fn main() -> color_eyre::Result<()> {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()?;

    let mut app = App::new()?;

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
