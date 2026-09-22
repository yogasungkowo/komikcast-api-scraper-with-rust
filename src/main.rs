mod models;
mod routes;
mod scraper;

use std::sync::Arc;
use axum::{
    middleware,
    response::Response,
    routing::get,
    Router,
};
use tokio::{io::AsyncWriteExt, sync::RwLock};
use tower_http::{compression::CompressionLayer, cors::CorsLayer, trace::TraceLayer};

use routes::{
    chapter_handler, genres_handler, manga_detail_handler, manga_list_handler, not_found_handler,
    ranking_handler, root_handler, search_handler, AppState,
};
use scraper::ScraperClient;

async fn logging_and_headers_middleware(
    req: axum::extract::Request,
    next: axum::middleware::Next,
) -> Response {
    let method = req.method().to_string();
    let uri = req.uri().to_string();
    let now = chrono::Utc::now().to_rfc3339();
    let log_line = format!("[{}] {} {}\n", now, method, uri);

    if let Ok(mut file) = tokio::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("requests.log")
        .await
    {
        tokio::spawn(async move {
            let _ = file.write_all(log_line.as_bytes()).await;
        });
    }

    let mut response = next.run(req).await;
    response.headers_mut().insert(
        axum::http::header::CACHE_CONTROL,
        "no-store, no-cache, must-revalidate".parse().unwrap(),
    );
    response
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(3002);

    let state = AppState {
        scraper: Arc::new(ScraperClient::new()),
        cached_genres: Arc::new(RwLock::new(None)),
        port,
    };

    let app = Router::new()
        .route("/", get(root_handler))
        .route("/explore", get(manga_list_handler))
        .route("/manga", get(manga_list_handler))
        .route("/ranking", get(ranking_handler))
        .route("/genres", get(genres_handler))
        .route("/search", get(search_handler))
        .route("/manga/{slug}", get(manga_detail_handler))
        .route("/manga/{slug}/chapter/{chapter}", get(chapter_handler))
        .route("/manga/{slug}/{chapterSlug}", get(chapter_handler))
        .fallback(not_found_handler)
        .layer(middleware::from_fn(logging_and_headers_middleware))
        .layer(CorsLayer::permissive())
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("Komikcast API running on http://0.0.0.0:{}", port);

    axum::serve(listener, app).await?;

    Ok(())
}
