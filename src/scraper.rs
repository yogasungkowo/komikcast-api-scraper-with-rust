use std::collections::HashSet;
use regex::Regex;
use scraper::{Html, Selector};
use url::Url;

use crate::models::{
    ChapterDetailData, ChapterImage, ChapterInnerDetail, ChapterMangaRef, ChapterNav,
    ChapterSummary, DetailChapterItem, FirstOrLatestChapter, GenreItem, LatestChapterWithNumber,
    MangaCard, MangaDetail, MangaFilters, MangaListData, Pagination, RankCard, RankingData,
};

pub const BASE_URL: &str = "https://komikcast.app";

pub fn clean(v: &str) -> String {
    v.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub fn slug_from_href(href: &str) -> Option<String> {
    let path = if let Some(idx) = href.find("://") {
        let rest = &href[idx + 3..];
        if let Some(slash_idx) = rest.find('/') {
            &rest[slash_idx..]
        } else {
            ""
        }
    } else {
        href
    };

    let segs: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();
    if segs.len() >= 4 && segs[2] == "chapter" {
        return Some(segs[3].to_string());
    }
    segs.get(1).map(|s| s.to_string())
}

pub fn number_from_text(v: &str) -> Option<f64> {
    let re = Regex::new(r"(?i)(?:chapter|ch\.?)\s*[-–:]?\s*(\d+(?:\.\d+)?)").ok()?;
    let cleaned = clean(v);
    let caps = re.captures(&cleaned)?;
    caps.get(1)?.as_str().parse::<f64>().ok()
}

pub fn chapter_number_from_slug(v: &str) -> Option<f64> {
    let s = clean(v);
    let re1 = Regex::new(r"(?i)/chapter/(\d+(?:\.\d+)?)/?$").ok()?;
    if let Some(caps) = re1.captures(&s) {
        if let Some(num_match) = caps.get(1) {
            if let Ok(num) = num_match.as_str().parse::<f64>() {
                return Some(num);
            }
        }
    }

    let re2 = Regex::new(r"^(\d+(?:\.\d+)?)$").ok()?;
    if let Some(caps) = re2.captures(&s) {
        if let Some(num_match) = caps.get(1) {
            if let Ok(num) = num_match.as_str().parse::<f64>() {
                return Some(num);
            }
        }
    }

    number_from_text(&s)
}

pub fn parse_manga_cards(html: &str) -> Vec<MangaCard> {
    let document = Html::parse_document(html);
    let card_sel = Selector::parse("article.system-content-card").unwrap();
    let link_sel = Selector::parse("a[href*='/manga/']").unwrap();
    let h3_sel = Selector::parse("h3").unwrap();
    let img_sel = Selector::parse("img").unwrap();
    let badge_sel = Selector::parse("[class*='system-badge']").unwrap();
    let rating_sel = Selector::parse("[class*='system-rating']").unwrap();
    let ch_sel = Selector::parse("a[href*='/chapter/']").unwrap();
    let label_sel = Selector::parse("[class*='__label']").unwrap();
    let time_sel = Selector::parse("time").unwrap();

    let type_re = Regex::new(r"(?i)Manga|Manhwa|Manhua").unwrap();

    let mut results = Vec::new();

    for card in document.select(&card_sel) {
        let first_link = card.select(&link_sel).next();
        let href = first_link
            .and_then(|el| el.value().attr("href"))
            .unwrap_or("");
        let title = first_link
            .and_then(|el| el.value().attr("aria-label"))
            .map(clean)
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                card.select(&h3_sel)
                    .next()
                    .map(|el| clean(&el.text().collect::<String>()))
                    .unwrap_or_default()
            });

        let poster = card
            .select(&img_sel)
            .next()
            .and_then(|el| el.value().attr("src"))
            .map(|s| s.to_string());

        let badge_text = card
            .select(&badge_sel)
            .next()
            .map(|el| clean(&el.text().collect::<String>()))
            .unwrap_or_default();

        let manga_type = if type_re.is_match(&badge_text) {
            Some(badge_text)
        } else {
            None
        };

        let rating = card
            .select(&rating_sel)
            .last()
            .map(|el| clean(&el.text().collect::<String>()))
            .and_then(|s| s.parse::<f64>().ok());

        let mut chapters = Vec::new();
        for ch in card.select(&ch_sel) {
            let ch_href = ch.value().attr("href").unwrap_or("");
            let ch_slug = slug_from_href(ch_href);
            let label = ch
                .select(&label_sel)
                .next()
                .map(|el| clean(&el.text().collect::<String>()))
                .unwrap_or_default();
            let date = ch
                .select(&time_sel)
                .next()
                .map(|el| clean(&el.text().collect::<String>()))
                .filter(|s| !s.is_empty());

            let number = ch_slug
                .as_deref()
                .and_then(chapter_number_from_slug);

            chapters.push(ChapterSummary {
                number,
                slug: ch_slug,
                title: label,
                date,
            });
        }

        let latest_chapter = chapters.first().map(|ch| LatestChapterWithNumber {
            number: ch.number,
            slug: ch.slug.clone(),
            title: ch.title.clone(),
            date: ch.date.clone(),
            chapter_number: ch.number,
        });

        let slug = slug_from_href(href);

        let card_item = MangaCard {
            title,
            slug,
            poster,
            manga_type,
            rating,
            latest_chapters: chapters,
            latest_chapter,
        };

        if let Some(ref s) = card_item.slug {
            if s != "manga" {
                results.push(card_item);
            }
        }
    }

    results
}

pub fn parse_pagination(document: &Html, current_page: u32) -> Pagination {
    let pagination_sel =
        Selector::parse("nav[aria-label*='halaman'] a, .system-pagination a").unwrap();
    let page_re = Regex::new(r"page=(\d+)").unwrap();
    let next_rel_re = Regex::new(r"(?i)^next$").unwrap();

    let mut has_next = false;
    let mut next_page = None;
    let mut last_page = current_page;

    for el in document.select(&pagination_sel) {
        let href = el.value().attr("href").unwrap_or("");
        let rel = el.value().attr("rel").unwrap_or("");
        let is_next_rel = next_rel_re.is_match(rel);

        let m_page = page_re
            .captures(href)
            .and_then(|c| c.get(1))
            .and_then(|m| m.as_str().parse::<u32>().ok());

        if is_next_rel || m_page.is_some() {
            if let Some(p) = m_page {
                if p > last_page {
                    last_page = p;
                }
            }
        }

        if is_next_rel {
            has_next = true;
            next_page = m_page.or(Some(current_page + 1));
        }
    }

    let has_prev = current_page > 1;
    let prev_page = if has_prev {
        Some(current_page - 1)
    } else {
        None
    };

    Pagination {
        current_page,
        last_page,
        has_next,
        next_page,
        has_prev,
        prev_page,
    }
}

pub fn parse_manga_detail(html: &str, slug: &str) -> MangaDetail {
    let document = Html::parse_document(html);
    let main_sel = Selector::parse("main").unwrap();
    let main_el = document.select(&main_sel).next();

    let h1_sel = Selector::parse("h1").unwrap();
    let img_sel = Selector::parse("img[data-cover-image], [data-cover] img").unwrap();
    let badge_sel = Selector::parse("[class*='system-badge']").unwrap();
    let rating_sel = Selector::parse("[role='img'][aria-label*='Rating']").unwrap();
    let genre_sel = Selector::parse("a[href*='genre=']").unwrap();
    let headings_sel = Selector::parse("h2, h3").unwrap();
    let meta_desc_sel = Selector::parse("meta[name='description']").unwrap();
    let chapter_list_sel = Selector::parse(
        ".reader-chapter-list a[href*='/chapter/'], ul[class*='chapter'] a[href*='/chapter/'], li a[href*='/chapter/']",
    )
    .unwrap();
    let chapter_link_fallback_sel =
        Selector::parse(&format!("a[href*='/manga/{}/chapter/']", slug)).unwrap();
    let truncate_sel = Selector::parse("span.truncate").unwrap();
    let span_sel = Selector::parse("span").unwrap();
    let font_mono_sel = Selector::parse("span.font-mono").unwrap();

    let title = main_el
        .and_then(|m| m.select(&h1_sel).next())
        .map(|el| clean(&el.text().collect::<String>()))
        .unwrap_or_default();

    let poster = main_el
        .and_then(|m| m.select(&img_sel).next())
        .and_then(|el| el.value().attr("src"))
        .map(|s| s.to_string());

    let badges: Vec<String> = main_el
        .map(|m| {
            m.select(&badge_sel)
                .map(|el| clean(&el.text().collect::<String>()))
                .collect()
        })
        .unwrap_or_default();

    let type_re = Regex::new(r"(?i)(Manga|Manhwa|Manhua)").unwrap();
    let manga_type = badges.iter().find_map(|b| {
        type_re
            .captures(b)
            .and_then(|c| c.get(1))
            .map(|m| m.as_str().to_string())
    });

    let status_re = Regex::new(r"(?i)Ongoing|Completed|Tamat|Berjalan").unwrap();
    let status = badges.iter().find(|b| status_re.is_match(b)).cloned();

    let year_re = Regex::new(r"^(19|20)\d{2}$").unwrap();
    let year = badges.iter().find(|b| year_re.is_match(b)).cloned();

    let rating = main_el
        .and_then(|m| m.select(&rating_sel).next())
        .map(|el| clean(&el.text().collect::<String>()))
        .and_then(|s| s.parse::<f64>().ok());

    let main_text = main_el
        .map(|m| clean(&m.text().collect::<String>()))
        .unwrap_or_default();

    let rank_re = Regex::new(r"(?i)Rank\s*#?\s*(\d+)").unwrap();
    let rank = rank_re
        .captures(&main_text)
        .and_then(|c| c.get(1))
        .and_then(|m| m.as_str().parse::<u32>().ok());

    let author_re = Regex::new(
        r"(?i)Author\s*([-–—]?\s*[\w\s.&',]+?)(?:Artist|Status|Type|Genre|Sinopsis|Chapter|$)",
    )
    .unwrap();
    let author_prefix_re = Regex::new(r"^[-–—]\s*").unwrap();
    let author = author_re.captures(&main_text).and_then(|c| {
        c.get(1).map(|m| {
            let s = author_prefix_re.replace(m.as_str(), "");
            clean(&s)
        })
    }).filter(|s| !s.is_empty());

    let mut genres = Vec::new();
    let mut seen_genres = HashSet::new();
    let genre_slug_re = Regex::new(r"[?&]genre=([a-z0-9-]+)").unwrap();

    for el in document.select(&genre_sel) {
        let name = clean(&el.text().collect::<String>());
        let href = el.value().attr("href").unwrap_or("");
        if let Some(caps) = genre_slug_re.captures(href) {
            if let Some(g_slug_match) = caps.get(1) {
                let g_slug = g_slug_match.as_str().to_string();
                if !name.is_empty() && !seen_genres.contains(&g_slug) {
                    seen_genres.insert(g_slug.clone());
                    genres.push(GenreItem { name, slug: g_slug });
                }
            }
        }
    }

    let mut synopsis: Option<String> = None;
    if let Some(m) = main_el {
        let sinopsis_re = Regex::new(r"(?i)sinopsis").unwrap();
        for heading in m.select(&headings_sel) {
            let heading_text = clean(&heading.text().collect::<String>());
            if sinopsis_re.is_match(&heading_text) {
                // Find next element sibling
                let mut sibling = heading.next_sibling();
                while let Some(node) = sibling {
                    if let Some(el) = scraper::ElementRef::wrap(node) {
                        let text = clean(&el.text().collect::<String>());
                        if !text.is_empty() {
                            synopsis = Some(text);
                            break;
                        }
                    }
                    sibling = node.next_sibling();
                }
                if synopsis.is_some() {
                    break;
                }
            }
        }
    }

    if synopsis.is_none() {
        if let Some(meta) = document.select(&meta_desc_sel).next() {
            if let Some(content) = meta.value().attr("content") {
                let prefix_re = Regex::new(r"(?i)^Ini adalah sinopsis untuk\s*").unwrap();
                let clean_desc = clean(&prefix_re.replace(content, ""));
                if !clean_desc.is_empty() {
                    synopsis = Some(clean_desc);
                }
            }
        }
    }

    let mut chapters = Vec::new();
    let mut seen_ch_slugs = HashSet::new();

    let chapter_elements: Vec<_> = {
        let list_elements: Vec<_> = document.select(&chapter_list_sel).collect();
        if !list_elements.is_empty() {
            list_elements
        } else {
            document.select(&chapter_link_fallback_sel).collect()
        }
    };

    for el in chapter_elements {
        let href = el.value().attr("href").unwrap_or("");
        let segs: Vec<&str> = href.split('/').filter(|s| !s.is_empty()).collect();
        let ch_slug = match segs.last() {
            Some(s) => s.to_string(),
            None => continue,
        };

        if seen_ch_slugs.contains(&ch_slug) {
            continue;
        }
        seen_ch_slugs.insert(ch_slug.clone());

        let label = el
            .select(&truncate_sel)
            .next()
            .map(|e| clean(&e.text().collect::<String>()))
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| {
                el.select(&span_sel)
                    .next()
                    .map(|e| clean(&e.text().collect::<String>()))
                    .unwrap_or_default()
            });

        let date = el
            .select(&font_mono_sel)
            .next()
            .map(|e| clean(&e.text().collect::<String>()))
            .filter(|s| !s.is_empty());

        let num = ch_slug.parse::<f64>().ok();
        let title_str = if !label.is_empty() {
            label
        } else {
            format!("Chapter {}", ch_slug)
        };

        let ch_part = ch_slug.strip_suffix(".00").unwrap_or(&ch_slug);
        let slug_str = format!("{}-chapter-{}", slug, ch_part);

        chapters.push(DetailChapterItem {
            number: num,
            title: title_str,
            slug: slug_str,
            date,
            url: href.to_string(),
        });
    }

    // Ensure chapters are sorted descending: latest chapter at index 0, chapter 1 at the end
    chapters.sort_by(|a, b| {
        match (b.number, a.number) {
            (Some(bn), Some(an)) => bn.partial_cmp(&an).unwrap_or(std::cmp::Ordering::Equal),
            (Some(_), None) => std::cmp::Ordering::Less,
            (None, Some(_)) => std::cmp::Ordering::Greater,
            (None, None) => std::cmp::Ordering::Equal,
        }
    });

    let total_chapters = chapters.len();
    let latest_chapter_number = chapters
        .iter()
        .filter_map(|c| c.number)
        .fold(None, |acc: Option<f64>, n| {
            Some(acc.map_or(n, |cur| if n > cur { n } else { cur }))
        });

    let first_chapter = chapters.last().map(|c| FirstOrLatestChapter {
        slug: c.slug.clone(),
        url: c.url.clone(),
        number: c.number,
    });

    let latest_chapter = chapters.first().map(|c| FirstOrLatestChapter {
        slug: c.slug.clone(),
        url: c.url.clone(),
        number: c.number,
    });

    MangaDetail {
        title,
        slug: slug.to_string(),
        poster,
        manga_type,
        status,
        year,
        rating,
        rank,
        author,
        genres,
        synopsis,
        total_chapters,
        latest_chapter_number,
        first_chapter,
        latest_chapter,
        chapters,
    }
}

pub fn parse_chapter(html: &str, manga_slug: &str, chapter_slug: &str) -> ChapterDetailData {
    let document = Html::parse_document(html);
    let title_sel = Selector::parse("title").unwrap();
    let img_primary_sel =
        Selector::parse("main img[src], [data-cover-image] img[src]").unwrap();
    let img_fallback_sel = Selector::parse(
        "img[src*='uploads'], img[src*='img.'], img[src*='.jpg'], img[src*='.webp'], img[src*='.png']",
    )
    .unwrap();
    let prev_sel = Selector::parse("a[rel='prev']").unwrap();
    let next_sel = Selector::parse("a[rel='next']").unwrap();
    let a_sel = Selector::parse("a").unwrap();

    let raw_title = document
        .select(&title_sel)
        .next()
        .map(|el| el.text().collect::<String>())
        .unwrap_or_default();
    let strip_title_re = Regex::new(r"(?i)—\s*Komikcast.*$").unwrap();
    let title = clean(&strip_title_re.replace(&raw_title, ""));

    let mut images = Vec::new();
    for el in document.select(&img_primary_sel) {
        if let Some(src) = el.value().attr("src") {
            if src.starts_with("http://") || src.starts_with("https://") {
                images.push(src.to_string());
            }
        }
    }

    if images.is_empty() {
        for el in document.select(&img_fallback_sel) {
            if let Some(src) = el.value().attr("src") {
                if src.starts_with("http://") || src.starts_with("https://") {
                    images.push(src.to_string());
                }
            }
        }
    }

    let final_images: Vec<ChapterImage> = images
        .into_iter()
        .enumerate()
        .map(|(i, url)| ChapterImage {
            order: i + 1,
            url,
        })
        .collect();

    let prev_url = document
        .select(&prev_sel)
        .next()
        .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
        .or_else(|| {
            let prev_re = Regex::new(r"(?i)prev|sebelumnya").unwrap();
            document.select(&a_sel).find_map(|el| {
                let text = clean(&el.text().collect::<String>());
                if prev_re.is_match(&text) {
                    el.value().attr("href").map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    let next_url = document
        .select(&next_sel)
        .next()
        .and_then(|el| el.value().attr("href").map(|s| s.to_string()))
        .or_else(|| {
            let next_re = Regex::new(r"(?i)next|berikutnya").unwrap();
            document.select(&a_sel).find_map(|el| {
                let text = clean(&el.text().collect::<String>());
                if next_re.is_match(&text) {
                    el.value().attr("href").map(|s| s.to_string())
                } else {
                    None
                }
            })
        });

    let series_url = format!("/manga/{}", manga_slug);

    let clean_ch_slug = chapter_slug.strip_suffix(".00").unwrap_or(chapter_slug);
    let ch_num = clean_ch_slug.parse::<f64>().ok();
    let fe_slug = format!("{}-chapter-{}", manga_slug, clean_ch_slug);

    let nav = |u_opt: Option<&str>| -> Option<ChapterNav> {
        let u = u_opt?;
        let parsed = Url::parse(u).or_else(|_| Url::parse(&format!("{}{}", BASE_URL, u))).ok()?;
        let path = parsed.path();
        let last_seg = path.split('/').filter(|s| !s.is_empty()).last()?;
        let seg_clean = last_seg.strip_suffix(".00").unwrap_or(last_seg);
        Some(ChapterNav {
            slug: format!("{}-chapter-{}", manga_slug, seg_clean),
            chapter_number: seg_clean.parse::<f64>().ok(),
            url: path.to_string(),
        })
    };

    let prev_nav = nav(prev_url.as_deref());
    let next_nav = nav(next_url.as_deref());

    let strip_manga_title_re = Regex::new(r"(?i)\s*Chapter.*$").unwrap();
    let manga_title_candidate = clean(&strip_manga_title_re.replace(&title, ""));
    let manga_title = if !manga_title_candidate.is_empty() {
        manga_title_candidate
    } else {
        manga_slug.to_string()
    };

    ChapterDetailData {
        title: title.clone(),
        manga_slug: manga_slug.to_string(),
        chapter_slug: fe_slug.clone(),
        chapter_number: ch_num,
        images: final_images.clone(),
        prev_url,
        next_url,
        series_url,
        manga: ChapterMangaRef {
            slug: manga_slug.to_string(),
            title: manga_title,
        },
        chapter: ChapterInnerDetail {
            title,
            slug: fe_slug,
            chapter_number: ch_num,
            images: final_images,
        },
        prev: prev_nav,
        next: next_nav,
        all_chapters: Vec::new(),
    }
}

pub struct ScraperClient {
    client: reqwest::Client,
}

impl ScraperClient {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
            .default_headers({
                let mut h = reqwest::header::HeaderMap::new();
                h.insert(
                    reqwest::header::ACCEPT,
                    "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"
                        .parse()
                        .unwrap(),
                );
                h.insert(
                    reqwest::header::ACCEPT_LANGUAGE,
                    "id-ID,id;q=0.9,en;q=0.8".parse().unwrap(),
                );
                h
            })
            .build()
            .expect("Failed to create reqwest client");

        Self { client }
    }

    pub async fn fetch_html(&self, path: &str) -> Result<String, (reqwest::StatusCode, String)> {
        let url = if path.starts_with("http://") || path.starts_with("https://") {
            path.to_string()
        } else {
            format!("{}{}", BASE_URL, path)
        };

        let resp = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| (reqwest::StatusCode::BAD_GATEWAY, e.to_string()))?;

        let status = resp.status();
        if !status.is_success() {
            return Err((status, format!("HTTP error: {}", status)));
        }

        let text = resp
            .text()
            .await
            .map_err(|e| (reqwest::StatusCode::BAD_GATEWAY, e.to_string()))?;

        if !text.contains("<html") {
            return Err((
                reqwest::StatusCode::BAD_GATEWAY,
                "Invalid HTML response".to_string(),
            ));
        }

        Ok(text)
    }

    pub async fn get_manga_list(
        &self,
        page: Option<u32>,
        manga_type: Option<String>,
        order: Option<String>,
        genre: Option<String>,
        status: Option<String>,
        search: Option<String>,
    ) -> Result<MangaListData, (reqwest::StatusCode, String)> {
        let current_page = page.unwrap_or(1);
        let mut query_params = Vec::new();

        if current_page > 1 {
            query_params.push(format!("page={}", current_page));
        }

        if let Some(ref o) = order {
            let mapped = match o.to_lowercase().as_str() {
                "update" => Some("latest_update"),
                "latest" => Some("new_manga"),
                "popular" => Some("popular"),
                "rating" => Some("rating"),
                "title" => Some("title"),
                _ => None,
            };
            if let Some(m) = mapped {
                query_params.push(format!("sort={}", m));
            }
        }

        if let Some(ref t) = manga_type {
            if t.to_lowercase() != "all" {
                query_params.push(format!("type={}", urlencoding(t.to_lowercase().as_str())));
            }
        }

        if let Some(ref s) = status {
            if s.to_lowercase() != "all" {
                query_params.push(format!("status={}", urlencoding(s.to_lowercase().as_str())));
            }
        }

        if let Some(ref g) = genre {
            query_params.push(format!("genre={}", urlencoding(g.as_str())));
        }

        if let Some(ref s) = search {
            query_params.push(format!("search={}", urlencoding(s.as_str())));
        }

        let path = if query_params.is_empty() {
            "/manga".to_string()
        } else {
            format!("/manga?{}", query_params.join("&"))
        };

        let html = self.fetch_html(&path).await?;
        let mangas = parse_manga_cards(&html);
        let document = Html::parse_document(&html);
        let pagination = parse_pagination(&document, current_page);

        Ok(MangaListData {
            mangas,
            pagination,
            filters: MangaFilters {
                page: current_page,
                type_: manga_type,
                status,
                order,
                genre,
                search,
            },
        })
    }

    pub async fn get_manga_detail(
        &self,
        slug: &str,
    ) -> Result<MangaDetail, (reqwest::StatusCode, String)> {
        let html = self.fetch_html(&format!("/manga/{}", slug)).await?;
        Ok(parse_manga_detail(&html, slug))
    }

    pub async fn get_chapter(
        &self,
        manga_slug: &str,
        chapter_identifier: &str,
    ) -> Result<ChapterDetailData, (reqwest::StatusCode, String)> {
        let mut identifier = chapter_identifier.trim().to_string();
        let fe_re = Regex::new(r"(?i)-chapter-(\d+(?:\.\d+)?)$").unwrap();
        if let Some(caps) = fe_re.captures(&identifier) {
            if let Some(m) = caps.get(1) {
                identifier = m.as_str().to_string();
            }
        }

        let chapter_part = identifier.strip_suffix(".00").unwrap_or(&identifier);
        let html = self
            .fetch_html(&format!("/manga/{}/chapter/{}", manga_slug, chapter_part))
            .await?;
        Ok(parse_chapter(&html, manga_slug, chapter_part))
    }

    pub async fn get_genres(
        &self,
        cached_genres: &tokio::sync::RwLock<Option<Vec<GenreItem>>>,
    ) -> Result<Vec<GenreItem>, (reqwest::StatusCode, String)> {
        {
            let read_guard = cached_genres.read().await;
            if let Some(ref g) = *read_guard {
                if !g.is_empty() {
                    return Ok(g.clone());
                }
            }
        }

        let html = self.fetch_html("/manga").await?;
        let mut seen = HashSet::new();
        let mut genres = {
            let document = Html::parse_document(&html);
            let sel = Selector::parse("select[name='genre[]'] option").unwrap();

            let mut list = Vec::new();
            for opt in document.select(&sel) {
                let g_slug = opt.value().attr("value").map(clean).unwrap_or_default();
                let name = clean(&opt.text().collect::<String>());
                if !g_slug.is_empty() && g_slug != "all" && !name.is_empty() && !seen.contains(&g_slug) {
                    seen.insert(g_slug.clone());
                    list.push(GenreItem { name, slug: g_slug });
                }
            }
            list
        };

        if genres.is_empty() {
            if let Ok(detail_html) = self.fetch_html("/manga/overgeared").await {
                let fallback_list = {
                    let detail_doc = Html::parse_document(&detail_html);
                    let g_sel = Selector::parse("a[href*='genre=']").unwrap();
                    let g_regex = Regex::new(r"[?&]genre=([a-z0-9-]+)").unwrap();
                    let mut list = Vec::new();
                    for a in detail_doc.select(&g_sel) {
                        let name = clean(&a.text().collect::<String>());
                        let href = a.value().attr("href").unwrap_or("");
                        if let Some(caps) = g_regex.captures(href) {
                            if let Some(slug_m) = caps.get(1) {
                                let g_slug = slug_m.as_str().to_string();
                                if !name.is_empty() && !seen.contains(&g_slug) {
                                    seen.insert(g_slug.clone());
                                    list.push(GenreItem { name, slug: g_slug });
                                }
                            }
                        }
                    }
                    list
                };
                genres = fallback_list;
            }
        }

        if !genres.is_empty() {
            let mut write_guard = cached_genres.write().await;
            *write_guard = Some(genres.clone());
        }

        Ok(genres)
    }

    pub async fn get_ranking(
        &self,
        period: Option<String>,
    ) -> Result<RankingData, (reqwest::StatusCode, String)> {
        let html = self.fetch_html("/manga?sort=popular").await?;
        let cards = parse_manga_cards(&html);
        let rankings = cards
            .into_iter()
            .enumerate()
            .map(|(i, m)| RankCard {
                rank: i + 1,
                title: m.title,
                slug: m.slug,
                poster: m.poster,
                rating: m.rating,
                manga_type: m.manga_type,
            })
            .collect();

        Ok(RankingData {
            rankings,
            period: period.unwrap_or_else(|| "all".to_string()),
        })
    }
}

fn urlencoding(s: &str) -> String {
    url::form_urlencoded::byte_serialize(s.as_bytes()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CARD_HTML: &str = r#"
      <article class="system-content-card system-card">
        <a href="https://komikcast.app/manga/the-last-human" aria-label="The Last Human">
          <div data-cover class="system-cover system-cover--poster">
            <img src="https://img.example/poster.jpg" alt="The Last Human" class="system-cover__image">
            <span class="system-badge system-badge--manhua">Manhua</span>
          </div>
          <span role="img" aria-label="Rating 5.2 dari 10" class="system-rating"><span>5.2</span></span>
        </a>
        <h3 class="system-content-card__title">
          <a href="https://komikcast.app/manga/the-last-human">The Last Human</a>
        </h3>
        <div class="system-entry-list">
          <a href="https://komikcast.app/manga/the-last-human/chapter/12.00">
            <span class="system-entry-list__label">Chapter 12.00</span>
            <time class="system-entry-list__time">3 jam yang lalu</time>
          </a>
        </div>
      </article>"#;

    #[test]
    fn test_slug_from_href() {
        assert_eq!(
            slug_from_href("https://komikcast.app/manga/the-last-human"),
            Some("the-last-human".to_string())
        );
        assert_eq!(
            slug_from_href("https://komikcast.app/manga/the-last-human/chapter/12.00"),
            Some("12.00".to_string())
        );
        assert_eq!(slug_from_href("/manga/abc"), Some("abc".to_string()));
    }

    #[test]
    fn test_chapter_number_from_slug() {
        assert_eq!(chapter_number_from_slug("34.00"), Some(34.0));
        assert_eq!(chapter_number_from_slug("34.50"), Some(34.5));
    }

    #[test]
    fn test_parse_manga_cards() {
        let cards = parse_manga_cards(CARD_HTML);
        assert_eq!(cards.len(), 1);
        let c = &cards[0];
        assert_eq!(c.title, "The Last Human");
        assert_eq!(c.slug, Some("the-last-human".to_string()));
        assert_eq!(c.poster, Some("https://img.example/poster.jpg".to_string()));
        assert_eq!(c.manga_type, Some("Manhua".to_string()));
        assert_eq!(c.rating, Some(5.2));
        assert_eq!(c.latest_chapters[0].slug, Some("12.00".to_string()));
        assert_eq!(c.latest_chapters[0].number, Some(12.0));
    }

    #[test]
    fn test_parse_detail_chapters() {
        let html = r#"<main>
        <h1>The Last Human</h1>
        <img data-cover-image src="poster.jpg" alt="The Last Human">
        <span class="system-badge">Manhua</span>
        <span class="system-badge">Ongoing</span>
        <span class="system-badge">2020</span>
        <a href="https://komikcast.app/manga?genre=horror"><span class="system-badge">Horror</span></a>
        <h2>Sinopsis</h2><p>Synopsis text</p>
        <a href="https://komikcast.app/manga/the-last-human/chapter/601.00"><span class="truncate">Chapter 601</span><span class="font-mono">5 bln</span></a>
        <a href="https://komikcast.app/manga/the-last-human/chapter/1.00"><span class="truncate">Chapter 1</span><span class="font-mono">9 bln</span></a>
      </main>"#;
        let d = parse_manga_detail(html, "the-last-human");
        assert_eq!(d.title, "The Last Human");
        assert_eq!(d.manga_type, Some("Manhua".to_string()));
        assert_eq!(d.status, Some("Ongoing".to_string()));
        assert_eq!(d.year, Some("2020".to_string()));
        assert_eq!(
            d.genres,
            vec![GenreItem {
                name: "Horror".to_string(),
                slug: "horror".to_string()
            }]
        );
        assert_eq!(d.synopsis, Some("Synopsis text".to_string()));
        assert_eq!(d.total_chapters, 2);
        assert_eq!(
            d.latest_chapter.as_ref().unwrap().slug,
            "the-last-human-chapter-601"
        );
        assert_eq!(d.latest_chapter.as_ref().unwrap().number, Some(601.0));
        assert_eq!(
            d.first_chapter.as_ref().unwrap().slug,
            "the-last-human-chapter-1"
        );
        assert_eq!(d.chapters[0].number, Some(601.0));
    }

    #[test]
    fn test_parse_chapter() {
        let html = r#"<html><head><title>X Chapter 2 — Komikcast</title></head>
        <body><main>
          <img src="https://img.example/1.jpg" alt="page 1">
          <img src="https://img.example/2.jpg" alt="page 2">
          <a rel="prev" href="https://komikcast.app/manga/x/chapter/1.00">Prev</a>
          <a rel="next" href="https://komikcast.app/manga/x/chapter/3.00">Next</a>
        </main></body></html>"#;
        let ch = parse_chapter(html, "x", "2");
        assert_eq!(ch.title, "X Chapter 2");
        assert_eq!(ch.images.len(), 2);
        assert_eq!(
            ch.images[0],
            ChapterImage {
                order: 1,
                url: "https://img.example/1.jpg".to_string()
            }
        );
        assert_eq!(ch.prev.as_ref().unwrap().slug, "x-chapter-1");
        assert_eq!(ch.prev.as_ref().unwrap().chapter_number, Some(1.0));
        assert_eq!(ch.next.as_ref().unwrap().slug, "x-chapter-3");
        assert_eq!(ch.chapter_number, Some(2.0));
    }

    #[test]
    fn test_parse_pagination() {
        let html = r#"<nav aria-label="Navigasi halaman" class="system-pagination">
        <span class="system-pagination__item" aria-current="page">1</span>
        <a class="system-pagination__item" href="https://komikcast.app/manga?page=2">2</a>
        <a class="system-pagination__item" href="https://komikcast.app/manga?page=369">369</a>
        <a class="system-pagination__item" href="https://komikcast.app/manga?page=2" rel="next">Next</a>
      </nav>"#;
        let document = Html::parse_document(html);
        assert_eq!(
            parse_pagination(&document, 1),
            Pagination {
                current_page: 1,
                last_page: 369,
                has_next: true,
                next_page: Some(2),
                has_prev: false,
                prev_page: None,
            }
        );
    }
}
