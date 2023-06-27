// TODO: add like a thing where it says "oooh loading / now playing / progress bar?"
// TODO: figure out a way to accept the "OMG ARE YOU STILL HERE" prompt
// TODO: actually I think I need the ublock back
// TODO: pub pub D:

mod api_stuff;
use api_stuff::ApiResponse;

use thirtyfour::common::capabilities::firefox::FirefoxPreferences;
use anyhow::{Context, Result};
use std::{ io, time::Duration, fs, vec };
use tui::{
    backend::CrosstermBackend,
    widgets::{Block, Borders, Gauge},
    layout::{Constraint, Direction, Layout},
    Terminal
}; use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use thirtyfour::prelude::*;
use thirtyfour::extensions::addons::firefox::FirefoxTools;
use thirtyfour::FirefoxCapabilities;
use tui::{
    text::Text,
    widgets::Paragraph,
    style::{Color, Style},
};
use tokio;

use std::fs::OpenOptions;
use std::io::prelude::*;



struct SlidingWindow {
    l: i8,
    r: i8,
    len: i8,
    curr: i8,
    capacity: i8,
}

impl SlidingWindow {
    fn new(l: i8, r: i8, curr: i8, capacity: i8) -> Self {
        Self { l, r, len: r - l + 1, curr, capacity}
    }

    fn next(&mut self) {
        self.curr += 1;

        if self.curr > self.capacity - 1 {
            self.curr = self.capacity - 1;
        }

        if self.curr > self.r {
            self.r = self.curr;
            self.l = self.r - (self.len - 1);
        }
    }

    fn prev(&mut self) {
        self.curr -= 1;

        if self.curr < 0 {
            self.curr = 0;
        }

        if self.curr < self.l {
            self.l = self.curr;
            self.r = self.l + (self.len - 1);
        }
    }

    fn get_pos(&self) -> i8 {
        self.curr as i8
    }
}


fn log_to_file(message: &str) -> std::io::Result<()> {
    let mut file = OpenOptions::new()
        .write(true)
        .append(true)
        .open("log.txt")?;
    writeln!(file, "{}", message)
}

async fn get_links(query: &str, result_count: u8) -> Result<ApiResponse> {
    let api_key = fs::read_to_string("APIKEY").context("could not read APIKEY")?;
    let search_url = "https://www.googleapis.com/youtube/v3/search";

    let part = "snippet";
    let item_type = "video";
    let url = format!("{}?part={}&type={}&maxResults={}&key={}&q={}", search_url, part, item_type, result_count, api_key, query);

    let response = reqwest::get(&url).await?;

    Ok(response.json().await?)
}

const RICKROLL_URL: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";
const RESULT_COUNT: u8 = 10;

// default port is 4444, must start selenium server with java -jar selenuimum.jar standaklonne
// before use
async fn start_browser() -> Result<WebDriver> {
    let mut firefox_capabilities = FirefoxCapabilities::new();

    let mut prefs = FirefoxPreferences::new();
    prefs.set("media.autoplay.default", 0).unwrap();
    firefox_capabilities.set_preferences(prefs).unwrap();
    firefox_capabilities.add_firefox_arg("--headless").unwrap();

    let driver = WebDriver::new("http://localhost:4444", firefox_capabilities).await?;

    Ok(driver)
}

async fn add_extension(driver: &WebDriver) -> Result<()> {
    let tools = FirefoxTools::new(driver.handle.clone());
    tools.install_addon("/home/bane/git/poormans-spotify/addons/ublock.xpi", Some(true)).await?;
    tools.install_addon("/home/bane/git/poormans-spotify/addons/unhook.xpi", Some(true)).await?;
    Ok(())
}

async fn open_link(driver: &WebDriver, link: &str) -> WebDriverResult<()> {
    driver.goto(link).await?;
    Ok(())
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
        )?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_results(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    response: Option<&ApiResponse>,
    search_input: &str,
    sliding_window: &mut SlidingWindow,
    ) -> Result<()> {

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Main Menu")
            .borders(Borders::ALL);
        f.render_widget(block, size);

        let results_per_page = 5;
        let result_height = 10;

        let mut constraints = vec![Constraint::Percentage(result_height); results_per_page];
        constraints.push(
            Constraint::Percentage(10),     // search bar
        );

        constraints.push(
            Constraint::Min(0)              // playing :o
        );

        constraints.push(
            Constraint::Percentage(5),      // progress bar
        );

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(constraints.as_ref())
            .split(f.size());

        if response.is_none() {
            for i in 0..results_per_page {
                let result = Paragraph::new("No results found.")
                    .block(Block::default().title("Result").borders(Borders::ALL));
                f.render_widget(result, chunks[i]);
            }
        } else {

            let selected_result = sliding_window.get_pos();
            let l = sliding_window.l;
            let r = sliding_window.r;

            for i in l..=r {
                let i = i as usize;
                let response = response.unwrap();
                let item = response.items.get(i).unwrap();
                let title = item.snippet.title.clone();

                let style = if i == selected_result as usize && selected_result > -1 {
                    Style::default().bg(Color::Blue)
                } else {
                    Style::default()
                };

                let result = Paragraph::new(Text::styled(title, style))
                    .block(Block::default().title("Result").borders(Borders::ALL));


                f.render_widget(result, chunks[i - l as usize]);
            }
        }

        let search_block = Paragraph::new(search_input)
            .block(Block::default().title("Search").borders(Borders::ALL));

        f.render_widget(search_block, chunks[results_per_page]);

        let progress_value = 0.3;  
        let is_playing = true;  

        let progress_bar = Gauge::default()
            .block(
                Block::default()
                .borders(Borders::ALL)
                )
            .gauge_style(Style::default().fg(Color::White))
            .percent((progress_value * 100.0) as u16);  // progress_value is a float between 0 and 1

        let play_pause_status = Paragraph::new(Text::styled(
                if is_playing { "Playing" } else { "Paused" },
                Style::default().fg(Color::White),
                )).block(Block::default().borders(Borders::ALL));

        f.render_widget(play_pause_status, chunks[results_per_page + 1]);
        f.render_widget(progress_bar, chunks[results_per_page + 2]);
    })?;
    Ok(())
}

enum Mode {
    Normal,
    Insert,
}


#[tokio::main]
async fn main() -> Result<()> {

    let mut terminal = setup_terminal()?;

    terminal.clear()?;

    let mut search_input = String::new();
    let mut result: ApiResponse;
    let mut response: Option<&ApiResponse> = None;
    let mut sliding_window = SlidingWindow::new(0, 4, 0, RESULT_COUNT as i8);

    draw_results(&mut terminal, None, &search_input, &mut sliding_window)?;
    let mut mode = Mode::Normal;
    let mut event_ocurred: bool;

    let driver = start_browser().await?;

    
    add_extension(&driver).await?; // BLACK MAGIC: removing this line breaks the program
                                   
    driver.close_window().await?;
    let windows = driver.windows().await?;
    driver.switch_to_window(windows[0].clone()).await?;

    // game loop
    loop {
        event_ocurred = event::poll(Duration::from_millis(100))?;

        if !event_ocurred {
            continue;
        }

        // this is stupid
        match event::read()? {
            Event::Key(key_event) => match mode {
                Mode::Normal => match key_event.code {
                    // refresh screen
                    KeyCode::Esc => {
                        terminal.clear()?;
                        draw_results(&mut terminal, response, &search_input, &mut sliding_window)?;
                    }
                    KeyCode::Char('q') => {
                        break;
                    }
                    KeyCode::Char('i') => {
                        mode = Mode::Insert;
                    }
                    //down
                    KeyCode::Char('j') => {
                        sliding_window.next();
                        draw_results(&mut terminal, response, &search_input, &mut sliding_window)?;
                    }
                    //up
                    KeyCode::Char('k') => {
                        sliding_window.prev();
                        draw_results(&mut terminal, response, &search_input, &mut sliding_window)?;
                    }
                    KeyCode::Enter => {
                        draw_results(&mut terminal, response, &search_input, &mut sliding_window)?;

                        if sliding_window.get_pos() >= 0 && sliding_window.get_pos() < RESULT_COUNT as i8 {
                            let i = sliding_window.get_pos() as usize;
                            let video_id = response.unwrap().items.get(i).unwrap().id.videoId.clone();
                            let link = format!("https://www.youtube.com/watch?v={}", &video_id);

                            open_link(&driver, &link).await?;
                        }
                    }
                    _ => {}
                }

                Mode::Insert => match key_event.code {
                    KeyCode::Esc => {
                        mode = Mode::Normal;
                    }
                    KeyCode::Backspace => {
                        search_input.pop();
                        draw_results(&mut terminal, response, &search_input, &mut sliding_window)?;
                    }
                    KeyCode::Enter => {
                        result = get_links(&search_input, RESULT_COUNT).await?;

                        response = Some(&result);

                        draw_results(&mut terminal, response, &search_input, &mut sliding_window)?;
                    }
                    KeyCode::Char(c) => {
                        search_input.push(c);
                        draw_results(&mut terminal, response, &search_input, &mut sliding_window)?;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    restore_terminal(&mut terminal)?;

    Ok(())
}
