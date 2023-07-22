use std::{
    error::Error,
    fs::File,
    io::{self, Stdout, Write},
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use id3::{Tag, TagLike};
use ratatui::{
    prelude::{Alignment, Backend, Constraint, CrosstermBackend, Direction, Layout},
    style::{Color, Style},
    widgets::{Block, Borders},
    Frame, Terminal,
};

use crate::terminal_image::TerminalImage;

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>, Box<dyn Error>> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;
    Ok(Terminal::new(CrosstermBackend::new(stdout))?)
}

pub fn restore_terminal(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
) -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    Ok(terminal.show_cursor()?)
}

struct App {
    tag: Tag,
    image_path: String,
}

pub fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    let path = "songs/Flamethrower.mp3";
    let image_path = ".temp/album_cover.jpg".to_string();
    let tag = Tag::read_from_path(path).unwrap();
    save_album_cover(&tag);
    let app = App { tag, image_path };
    loop {
        terminal.draw(|f| ui(f, &app))?;
        if event::poll(Duration::from_millis(250))? {
            if let Event::Key(key) = event::read()? {
                if KeyCode::Char('q') == key.code {
                    break;
                }
            }
        }
    }
    Ok(())
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.size());

    let block = Block::default()
        .title(app.tag.title().unwrap())
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::White));

    f.render_widget(block, chunks[0]);

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Percentage(100), Constraint::Percentage(0)].as_ref())
        .split(chunks[0]);

    let music_image_chunk = Layout::default()
        .direction(Direction::Horizontal)
        .margin(0)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(inner_chunks[0]);

    let image_chunk = music_image_chunk[0];

    let image = TerminalImage::new(&app.image_path);
    f.render_widget(image, image_chunk)
}

fn save_album_cover(tag: &Tag) {
    // Look for APIC frame
    for picture in tag.pictures() {
        // Check if it is cover picture
        if picture.picture_type == id3::frame::PictureType::CoverFront {
            // Set a name and path for your image file
            let image_path = ".temp/album_cover.jpg";
            // Create a new file and write the image data to it
            let mut img_file = File::create(image_path).expect("Failed to create image file");
            img_file
                .write_all(&picture.data)
                .expect("Failed to write data to file");
            println!("Image saved to {}", image_path);
        }
    }
}
