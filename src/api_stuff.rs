use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct ApiResponse {
    kind: String,
    etag: String,
    nextPageToken: String,
    regionCode: String,
    pageInfo: PageInfo,
    pub items: Vec<Item>,
}

#[derive(Deserialize, Debug)]
struct PageInfo {
    totalResults: u32,
    resultsPerPage: u8,
}

#[derive(Deserialize, Debug)]
pub struct Item {
    kind: String,
    etag: String,
    pub id: Id,
    pub snippet: Snippet,
}

#[derive(Deserialize, Debug)]
pub struct Snippet {
    publishedAt: String,
    channelId: String,
    pub title: String,
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
pub struct Id {
    pub videoId: String,
}
