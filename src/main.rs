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


async fn get_links(word: &str) -> Result<()> {
    let api_key = fs::read_to_string("APIKEY").context("could not read APIKEY")?;
    let search_url = "https://www.googleapis.com/youtube/v3/search";

    let query = "joe";
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

#[tokio::main]
async fn main() -> Result<()> {

    let driver = start_browser().await?;
    add_extension(&driver).await?;

    open_link(&driver, RICKROLL_URL).await?;
    play_current_video(&driver).await?;

    Ok(())
}

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

fn show_term() -> Result<()> {

    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("JOE")
            .borders(Borders::ALL);
        f.render_widget(block, size);

    })?;

    thread::sleep(std::time::Duration::from_millis(5000));

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
        )?;
    terminal.show_cursor()?;

    Ok(())
}

