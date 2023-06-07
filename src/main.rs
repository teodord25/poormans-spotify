use anyhow::{Context, Result};
use std::{ io, thread, time::Duration, fs, vec };
use tui::{
    backend::CrosstermBackend,
    widgets::{Widget, Block, Borders},
    layout::{Constraint, Direction, Layout},
    Terminal
};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use serde::Deserialize;
use reqwest::Error;
use thirtyfour::prelude::*;
use thirtyfour::extensions::addons::firefox::FirefoxTools;
use tui::{
    text::Text,
    widgets::Paragraph,
};
use tokio;

#[derive(Deserialize, Debug)]
struct ApiResponse {
    kind: String,
    etag: String,
    nextPageToken: String,
    regionCode: String,
    pageInfo: PageInfo,
    items: Vec<Item>,
}

#[derive(Deserialize, Debug)]
struct PageInfo {
    totalResults: u32,
    resultsPerPage: u8,
}

#[derive(Deserialize, Debug)]
struct Item {
    kind: String,
    etag: String,
    id: Id,
    snippet: Snippet,
}

#[derive(Deserialize, Debug)]
struct Snippet {
    publishedAt: String,
    channelId: String,
    title: String,
    description: String,
    thumbnails: Thumbnails,
    channelTitle: String,
    liveBroadcastContent: String,
    publishTime: String,

}

#[derive(Deserialize, Debug)]
struct Thumbnails {
    default: Thumbnail,
    medium: Thumbnail,
    high: Thumbnail,
}

#[derive(Deserialize, Debug)]
struct Thumbnail {
    url: String,
}


#[derive(Deserialize, Debug)]
struct Id {
    videoId: String,
}


async fn get_links(query: &str) -> Result<()> {
    let api_key = fs::read_to_string("APIKEY").context("could not read APIKEY")?;
    let search_url = "https://www.googleapis.com/youtube/v3/search";

    let part = "snippet";
    let url = format!("{}?part={}&key={}&q={}", search_url, part, api_key, query);

    let response = reqwest::get(&url).await?;

    let api_response: ApiResponse = response.json().await?;

    for item in api_response.items {
        println!("Video ID: {}", item.id.videoId);
        println!("Title: {}", item.snippet.title);
        println!();
    }

    Ok(())
}

const RICKROLL_URL: &str = "https://www.youtube.com/watch?v=dQw4w9WgXcQ";


async fn play_current_video(driver: &WebDriver) -> WebDriverResult<()> {
    let script = r#"
        var video = document.querySelector("video");
        if (video) {
            video.play();
        }
    "#;
    driver.execute(script, vec![]).await?;
    Ok(())
}


async fn start_browser() -> Result<WebDriver> {
    let caps = DesiredCapabilities::firefox();
    let driver = WebDriver::new("http://localhost:4444", caps).await?;

    Ok(driver)
}

async fn add_extension(driver: &WebDriver) -> Result<()> {
    let tools = FirefoxTools::new(driver.handle.clone());
    tools.install_addon("/home/bane/Downloads/ublock_origin-1.49.2.xpi", Some(true)).await.unwrap();
    Ok(())
}

// default port is 4444, must start selenium server with java -jar selenuimum.jar standaklonne
// before use
async fn open_link(driver: &WebDriver, link: &str) -> WebDriverResult<()> {
    driver.goto(link).await?;
    play_current_video(&driver).await.unwrap();
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

fn draw_menu(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Main Menu")
            .borders(Borders::ALL);
        f.render_widget(block, size);

        let results_per_page = 5;
        let result_height = 10;

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints(
                vec!
                [
                    Constraint::Percentage(result_height); results_per_page
                ].as_ref()
                )
            .split(f.size());

        for i in 0..results_per_page {
            let textyBlock = Paragraph::new("This is some text.")
                .block(Block::default().title("BRUH").borders(Borders::ALL));
            f.render_widget(textyBlock, chunks[i]);
        }

    })?;
    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {

    let mut terminal = setup_terminal()?;
    draw_menu(&mut terminal)?;

    // Loop until 'q' is pressed.
    loop {
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(event) = crossterm::event::read()? {
                if event.code == KeyCode::Char('q') {
                    break;
                }
            }
        }
    }

    restore_terminal(&mut terminal)?;

    Ok(())
}
