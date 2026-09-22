use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use tokio::sync::RwLock;

use crate::{
    models::{
        ApiErrorResponse, ApiResponse, GenreItem, MangaListQuery, RankingQuery, SearchQuery,
    },
    scraper::{ScraperClient, BASE_URL},
};

#[derive(Clone)]
pub struct AppState {
    pub scraper: Arc<ScraperClient>,
    pub cached_genres: Arc<RwLock<Option<Vec<GenreItem>>>>,
    pub port: u16,
}

pub async fn root_handler(State(state): State<AppState>) -> impl IntoResponse {
    let port = state.port;
    let body = json!({
        "status": "Ok",
        "message": "Komikcast API (komikcast.app)",
        "source": BASE_URL,
        "endpoints": {
            "GET /explore": "Explore manga with filters (query: page, genre, type, status, order, search)",
            "GET /manga": "Alias to /explore (query: page, genre, type, status, order, sort, search)",
            "GET /ranking": "Manga rankings (optional query: period=daily|weekly|monthly|all)",
            "GET /manga/:slug": "Manga detail with chapter list (e.g. /manga/tales-demons-gods)",
            "GET /manga/:slug/:chapterSlug": "Read chapter by chapter slug (e.g. /manga/tales-demons-gods/tales-of-demons-and-gods-chapter-1)",
            "GET /manga/:slug/chapter/:chapter": "Read chapter by chapter number or slug (e.g. /manga/tales-demons-gods/chapter/1)",
            "GET /search?q=keyword": "Search manga (alias to /explore?search=keyword)",
            "GET /genres": "List all 140+ manga genres"
        },
        "filter_options": {
            "type": ["manga", "manhwa", "manhua"],
            "status": ["ongoing", "completed"],
            "order": ["update", "latest", "popular", "title"],
            "genre_example": "shoujo-ai,action"
        },
        "curl_examples": [
            format!("curl \"http://localhost:{}/explore?genre=shoujo-ai,action&type=manga&status=ongoing&order=update\"", port),
            format!("curl \"http://localhost:{}/ranking\"", port),
            format!("curl \"http://localhost:{}/manga/tales-demons-gods\"", port),
            format!("curl \"http://localhost:{}/manga/tales-demons-gods/tales-of-demons-and-gods-chapter-1\"", port),
            format!("curl \"http://localhost:{}/manga/tales-demons-gods/chapter/1\"", port),
        ]
    });

    Json(body)
}

pub async fn manga_list_handler(
    State(state): State<AppState>,
    Query(query): Query<MangaListQuery>,
) -> Response {
    let order = query.order.or(query.sort);
    let search = query.search.or(query.q);

    match state
        .scraper
        .get_manga_list(
            query.page,
            query.manga_type,
            order,
            query.genre,
            query.status,
            search,
        )
        .await
    {
        Ok(data) => Json(ApiResponse {
            status: "Ok".to_string(),
            data,
        })
        .into_response(),
        Err((_, err_msg)) => {
            tracing::error!("[handleMangaList] {}", err_msg);
            (
                StatusCode::BAD_GATEWAY,
                Json(ApiErrorResponse::new(
                    "Failed to fetch manga list",
                    Some(err_msg),
                )),
            )
                .into_response()
        }
    }
}

pub async fn ranking_handler(
    State(state): State<AppState>,
    Query(query): Query<RankingQuery>,
) -> Response {
    match state.scraper.get_ranking(query.period).await {
        Ok(data) => Json(ApiResponse {
            status: "Ok".to_string(),
            data,
        })
        .into_response(),
        Err((_, err_msg)) => {
            tracing::error!("[/ranking] {}", err_msg);
            (
                StatusCode::BAD_GATEWAY,
                Json(ApiErrorResponse::new(
                    "Failed to fetch ranking",
                    Some(err_msg),
                )),
            )
                .into_response()
        }
    }
}

pub async fn genres_handler(State(state): State<AppState>) -> Response {
    match state.scraper.get_genres(&state.cached_genres).await {
        Ok(data) => Json(ApiResponse {
            status: "Ok".to_string(),
            data,
        })
        .into_response(),
        Err((_, err_msg)) => {
            tracing::error!("[/genres] {}", err_msg);
            (
                StatusCode::BAD_GATEWAY,
                Json(ApiErrorResponse::new(
                    "Failed to fetch genres",
                    Some(err_msg),
                )),
            )
                .into_response()
        }
    }
}

pub async fn search_handler(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Response {
    let search_keyword = match query.q.or(query.search) {
        Some(k) if !k.trim().is_empty() => k,
        _ => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ApiErrorResponse::new(
                    "Query parameter 'q' or 'search' is required",
                    None,
                )),
            )
                .into_response();
        }
    };

    match state
        .scraper
        .get_manga_list(
            query.page,
            None,
            None,
            None,
            None,
            Some(search_keyword),
        )
        .await
    {
        Ok(data) => Json(ApiResponse {
            status: "Ok".to_string(),
            data,
        })
        .into_response(),
        Err((_, err_msg)) => {
            tracing::error!("[/search] {}", err_msg);
            (
                StatusCode::BAD_GATEWAY,
                Json(ApiErrorResponse::new(
                    "Failed to search manga",
                    Some(err_msg),
                )),
            )
                .into_response()
        }
    }
}

pub async fn manga_detail_handler(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> Response {
    match state.scraper.get_manga_detail(&slug).await {
        Ok(data) => Json(ApiResponse {
            status: "Ok".to_string(),
            data,
        })
        .into_response(),
        Err((status, err_msg)) => {
            tracing::error!("[/manga/:slug] {}", err_msg);
            let code = if status == reqwest::StatusCode::NOT_FOUND {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_GATEWAY
            };
            let message = if code == StatusCode::NOT_FOUND {
                "Manga not found"
            } else {
                "Failed to fetch manga detail"
            };
            (code, Json(ApiErrorResponse::new(message, Some(err_msg)))).into_response()
        }
    }
}

pub async fn chapter_handler(
    State(state): State<AppState>,
    Path((slug, chapter)): Path<(String, String)>,
) -> Response {
    match state.scraper.get_chapter(&slug, &chapter).await {
        Ok(data) => Json(ApiResponse {
            status: "Ok".to_string(),
            data,
        })
        .into_response(),
        Err((status, err_msg)) => {
            tracing::error!("[chapter_handler] {}", err_msg);
            let code = if status == reqwest::StatusCode::NOT_FOUND {
                StatusCode::NOT_FOUND
            } else {
                StatusCode::BAD_GATEWAY
            };
            let message = if code == StatusCode::NOT_FOUND {
                "Chapter not found"
            } else {
                "Failed to fetch chapter"
            };
            (code, Json(ApiErrorResponse::new(message, Some(err_msg)))).into_response()
        }
    }
}

pub async fn not_found_handler() -> impl IntoResponse {
    (
        StatusCode::NOT_FOUND,
        Json(ApiErrorResponse::new("Route not found", None)),
    )
}
