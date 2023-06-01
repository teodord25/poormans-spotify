use anyhow::{Context, Result};
use std::{
    io,
    thread,
    time::Duration,
    fs
};
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

#[tokio::main]
async fn main() -> Result<()> {
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

fn showTerm() -> Result<()> {

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

