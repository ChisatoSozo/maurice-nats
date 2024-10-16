use actix_web::{web::Json, Error};

use paperclip::actix::{api_v2_operation, post, Apiv2Schema};
use serde::{Deserialize, Serialize};

use crate::{procedures, procedures::get_youtube_videos::Song};

#[derive(Debug, Deserialize, Apiv2Schema)]
struct GetYoutubeVideosArgs {
    search: String,
}

#[derive(Debug, Serialize, Apiv2Schema)]
struct GetYoutubeVideosReturn {
    videos: Vec<Song>,
}

#[api_v2_operation]
#[post("/api/get_youtube_videos")]
pub async fn get_youtube_videos(
    body: Json<GetYoutubeVideosArgs>,
) -> Result<Json<GetYoutubeVideosReturn>, Error> {
    let search = body.search.clone();

    let videos = procedures::get_youtube_videos::get_youtube_videos(search)
        .await
        .map_err(|e| actix_web::error::ErrorInternalServerError(e.to_string()))?;

    Ok(Json(GetYoutubeVideosReturn { videos }))
}
