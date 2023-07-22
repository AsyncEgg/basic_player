use std::{
    cmp,
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
    prelude::{Alignment, Backend, Constraint, CrosstermBackend, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{block::Position, Block, Borders},
    Frame, Terminal,
};
use viuer::{print_from_file, Config};

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
}

pub fn run(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<(), Box<dyn Error>> {
    let path = "songs/Flamethrower.mp3";

    let tag = Tag::read_from_path(path).unwrap();
    save_album_cover(&tag)?;
    let app = App { tag };
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

    f.render_widget(block, center_area(chunks[0], 1));

    let inner_chunks = Layout::default()
        .direction(Direction::Vertical)
        .margin(1)
        .constraints([Constraint::Min(1), Constraint::Length(3)].as_ref())
        .split(chunks[0]);

    render_image(".temp/album_cover.jpg", inner_chunks[0]);

    let block = Block::default()
        .title(app.tag.artist().unwrap())
        .title_position(Position::Bottom)
        .title_alignment(Alignment::Center);

    f.render_widget(block, inner_chunks[1])
}

fn center_area(area: Rect, padding: u16) -> Rect {
    let y = area.y;
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
    let x = ((width / 2) - (image_width / 2)) + padding;
    Rect {
        x,
        y,
        width: image_width,
        height: image_height,
    }
}

//Using this with ratatui's custom widgit is just too slow and information that is unused is passed to this function so i decided to remove it for this function
fn render_image(path: &str, area: Rect) {
    let area = center_area(area, 2);
    let conf = Config {
        x: area.x,
        y: area.y as i16,
        width: Some(area.width as u32),
        height: Some(area.height as u32),
        ..Default::default()
    };

    if let Err(e) = print_from_file(path, &conf) {
        eprintln!("Image printing failed: {}", e);
    }
}

fn save_album_cover(tag: &Tag) -> Result<(), Box<dyn Error>> {
    // Look for APIC frame
    for picture in tag.pictures() {
        // Check if it is cover picture
        if picture.picture_type == id3::frame::PictureType::CoverFront {
            // Set a name and path for your image file
            let image_path = ".temp/album_cover.jpg";
            // Create a new file and write the image data to it
            let mut img_file = File::create(image_path)?;
            img_file.write_all(&picture.data)?;
        }
    }
    Ok(())
}
