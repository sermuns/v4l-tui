use std::io;

mod app;

use crate::app::App;

fn main() -> io::Result<()> {
    let mut app = App::new()?;

    ratatui::run(|terminal| app.run(terminal))?;

    Ok(())
}
