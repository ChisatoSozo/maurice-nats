#![allow(non_camel_case_types)]

use actix_cors::Cors;
use actix_web::{http::header, App, HttpServer};

use logger::init_logger;
use paperclip::actix::OpenApiExt;
use routes::get_youtube_videos::get_youtube_videos;

pub mod logger;
pub mod procedures;
pub mod routes;

const JSON_SPEC_PATH: &str = "/api/spec/v2.json";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    init_logger();

    let host = std::env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("WEB_DATA_SERVICE_PORT").unwrap_or_else(|_| "3000".to_string());
    let port_num = port.parse::<u16>().expect("Failed to parse port");

    HttpServer::new(move || {
        App::new()
            .wrap(
                Cors::default()
                    .allow_any_origin()
                    .allowed_methods(vec!["GET", "POST"])
                    .allowed_headers(vec![header::AUTHORIZATION, header::ACCEPT])
                    .allowed_header(header::CONTENT_TYPE)
                    .max_age(3600),
            )
            .wrap_api()
            .with_json_spec_at(JSON_SPEC_PATH)
            .service(get_youtube_videos)
            .build()
    })
    .workers(4)
    .client_request_timeout(std::time::Duration::from_secs(600))
    .client_disconnect_timeout(std::time::Duration::from_secs(600))
    .bind((host.clone(), port_num))?
    .run()
    .await
}
