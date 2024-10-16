use base64::{engine::general_purpose, Engine as _};
use paperclip::actix::Apiv2Schema;
use regex::Regex;
use reqwest::Client;
use serde::Serialize;
use std::error::Error;

#[derive(Debug, Serialize, Apiv2Schema)]
pub struct Song {
    url: String,
    thumbnail_b64: String,
    title: String,
}

pub async fn get_youtube_videos(search: String) -> Result<Vec<Song>, Box<dyn Error>> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
        .build()?;

    let url = format!(
        "https://www.youtube.com/results?search_query={}",
        urlencoding::encode(&search)
    );
    let res = client.get(&url).send().await?.text().await?;

    // Extract the initial data JSON
    let re = Regex::new(r"var ytInitialData = (\{.*?\});").unwrap();
    let json_str = re
        .captures(&res)
        .and_then(|cap| cap.get(1))
        .ok_or("Failed to extract initial data")?
        .as_str();

    // Parse the JSON
    let json: serde_json::Value = serde_json::from_str(json_str)?;

    // Extract video information
    let videos = json["contents"]["twoColumnSearchResultsRenderer"]["primaryContents"]
        ["sectionListRenderer"]["contents"][0]["itemSectionRenderer"]["contents"]
        .as_array()
        .ok_or("Failed to extract video array")?
        .iter()
        .filter_map(|item| {
            let video_renderer = &item["videoRenderer"];
            let id = video_renderer["videoId"].as_str()?;
            let title = video_renderer["title"]["runs"][0]["text"].as_str()?;

            Some((id.to_string(), title.to_string()))
        })
        .collect::<Vec<_>>();

    // Fetch thumbnails and create Song structs
    let mut youtube_videos = Vec::new();
    for (id, title) in videos {
        let thumbnail_url = format!("https://i.ytimg.com/vi/{}/mqdefault.jpg", id);
        let thumbnail_bytes = client.get(&thumbnail_url).send().await?.bytes().await?;
        let thumbnail_base64 = general_purpose::STANDARD.encode(thumbnail_bytes);

        youtube_videos.push(Song {
            url: format!("https://www.youtube.com/watch?v={}", id),
            title,
            thumbnail_b64: format!("data:image/jpeg;base64,{}", thumbnail_base64),
        });
    }

    Ok(youtube_videos)
}
