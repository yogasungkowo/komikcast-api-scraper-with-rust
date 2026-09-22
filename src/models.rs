use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub status: String,
    pub data: T,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiErrorResponse {
    pub status: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl ApiErrorResponse {
    pub fn new(message: impl Into<String>, error: Option<String>) -> Self {
        Self {
            status: "Error".to_string(),
            message: message.into(),
            error,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterSummary {
    pub number: Option<f64>,
    pub slug: Option<String>,
    pub title: String,
    pub date: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LatestChapterWithNumber {
    pub number: Option<f64>,
    pub slug: Option<String>,
    pub title: String,
    pub date: Option<String>,
    pub chapter_number: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MangaCard {
    pub title: String,
    pub slug: Option<String>,
    pub poster: Option<String>,
    #[serde(rename = "type")]
    pub manga_type: Option<String>,
    pub rating: Option<f64>,
    pub latest_chapters: Vec<ChapterSummary>,
    pub latest_chapter: Option<LatestChapterWithNumber>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Pagination {
    pub current_page: u32,
    pub last_page: u32,
    pub has_next: bool,
    pub next_page: Option<u32>,
    pub has_prev: bool,
    pub prev_page: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MangaFilters {
    pub page: u32,
    #[serde(rename = "type")]
    pub type_: Option<String>,
    pub status: Option<String>,
    pub order: Option<String>,
    pub genre: Option<String>,
    pub search: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MangaListData {
    pub mangas: Vec<MangaCard>,
    pub pagination: Pagination,
    pub filters: MangaFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GenreItem {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DetailChapterItem {
    pub number: Option<f64>,
    pub title: String,
    pub slug: String,
    pub date: Option<String>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FirstOrLatestChapter {
    pub slug: String,
    pub url: String,
    pub number: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MangaDetail {
    pub title: String,
    pub slug: String,
    pub poster: Option<String>,
    #[serde(rename = "type")]
    pub manga_type: Option<String>,
    pub status: Option<String>,
    pub year: Option<String>,
    pub rating: Option<f64>,
    pub rank: Option<u32>,
    pub author: Option<String>,
    pub genres: Vec<GenreItem>,
    pub synopsis: Option<String>,
    pub total_chapters: usize,
    pub latest_chapter_number: Option<f64>,
    pub first_chapter: Option<FirstOrLatestChapter>,
    pub latest_chapter: Option<FirstOrLatestChapter>,
    pub chapters: Vec<DetailChapterItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterImage {
    pub order: usize,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterNav {
    pub slug: String,
    pub chapter_number: Option<f64>,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterMangaRef {
    pub slug: String,
    pub title: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterInnerDetail {
    pub title: String,
    pub slug: String,
    pub chapter_number: Option<f64>,
    pub images: Vec<ChapterImage>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChapterDetailData {
    pub title: String,
    pub manga_slug: String,
    pub chapter_slug: String,
    pub chapter_number: Option<f64>,
    pub images: Vec<ChapterImage>,
    pub prev_url: Option<String>,
    pub next_url: Option<String>,
    pub series_url: String,
    pub manga: ChapterMangaRef,
    pub chapter: ChapterInnerDetail,
    pub prev: Option<ChapterNav>,
    pub next: Option<ChapterNav>,
    pub all_chapters: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankCard {
    pub rank: usize,
    pub title: String,
    pub slug: Option<String>,
    pub poster: Option<String>,
    pub rating: Option<f64>,
    #[serde(rename = "type")]
    pub manga_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RankingData {
    pub rankings: Vec<RankCard>,
    pub period: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MangaListQuery {
    pub page: Option<u32>,
    #[serde(rename = "type")]
    pub manga_type: Option<String>,
    pub sort: Option<String>,
    pub order: Option<String>,
    pub genre: Option<String>,
    pub status: Option<String>,
    pub search: Option<String>,
    pub q: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RankingQuery {
    pub period: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub search: Option<String>,
    pub page: Option<u32>,
}
