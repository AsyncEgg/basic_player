use std::error::Error;

use ui::{restore_terminal, run, setup_terminal};
mod terminal_image;
mod ui;

fn main() -> Result<(), Box<dyn Error>> {
    let mut terminal = setup_terminal()?;
    run(&mut terminal)?;
    restore_terminal(&mut terminal)?;
    Ok(())
}
