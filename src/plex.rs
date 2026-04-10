use std::collections::HashSet;
use std::time::Duration;

use quick_xml::de::from_str;
use reqwest::blocking::Client;
use reqwest::Url;
use serde::Deserialize;
use thiserror::Error;

use crate::models::{LibrarySection, MediaItem, MoviePage};

/// 带 Token 的 Plex HTTP 客户端：拉取 XML、拼流地址与缩略图转码 URL。
#[derive(Clone, Debug)]
pub struct PlexClient {
    server_url: String,
    token: String,
    http: Client,
}

impl PlexClient {
    pub fn new(server_url: impl Into<String>, token: impl Into<String>) -> Result<Self, PlexError> {
        let server_url = server_url.into().trim().trim_end_matches('/').to_owned();
        let token = token.into().trim().to_owned();
        let http = Client::builder()
            .timeout(Duration::from_secs(20))
            .build()?;
        Ok(Self {
            server_url,
            token,
            http,
        })
    }

    /// 列出 `type=movie` 的库，供下拉框选择。
    pub fn fetch_movie_sections(&self) -> Result<Vec<LibrarySection>, PlexError> {
        let xml = self.get_xml("/library/sections", &[])?;
        let container: MediaContainer = from_str(&xml)?;
        Ok(container
            .directories
            .into_iter()
            .filter(|section| section.kind.as_deref() == Some("movie"))
            .map(|section| LibrarySection {
                id: section.key,
                title: section.title,
            })
            .collect())
    }

    /// 分页获取某库下全部影片条目（`type=1` 表示电影）。
    pub fn fetch_movies_page(
        &self,
        section_id: &str,
        start: usize,
        size: usize,
    ) -> Result<MoviePage, PlexError> {
        let path = format!("/library/sections/{section_id}/all");
        let start_value = start.to_string();
        let size_value = size.to_string();
        let xml = self.get_xml(
            &path,
            &[
                ("type", "1"),
                ("X-Plex-Container-Start", &start_value),
                ("X-Plex-Container-Size", &size_value),
            ],
        )?;

        let container: MediaContainer = from_str(&xml)?;
        let total_size = container.total_size.unwrap_or(container.size.unwrap_or_default());
        let items = container
            .videos
            .into_iter()
            .map(media_item_from_video)
            .collect();

        Ok(MoviePage {
            items,
            start,
            total_size,
        })
    }

    /// 获取增强版 Continue Watching：先拿 Plex 原生列表，再用历史记录补齐更多未看完条目。
    pub fn fetch_continue_watching_enhanced(
        &self,
        hub_count: usize,
        history_size: usize,
    ) -> Result<MoviePage, PlexError> {
        let mut items = self.fetch_continue_watching(hub_count)?;
        let mut seen: HashSet<String> = items.iter().map(|item| item.rating_key.clone()).collect();

        let history = self.fetch_playback_history(history_size)?;
        let history_rating_keys = dedup_history_rating_keys(&history, &seen);
        let history_viewed_at = history_viewed_at_map(&history);

        for mut item in self.fetch_metadata_items(&history_rating_keys)? {
            if let Some(viewed_at) = history_viewed_at.get(&item.rating_key) {
                item.last_viewed_at = Some(*viewed_at);
            }
            if is_continue_candidate(&item) && seen.insert(item.rating_key.clone()) {
                items.push(item);
            }
        }

        items.sort_by(|left, right| right.last_viewed_at.cmp(&left.last_viewed_at));

        Ok(MoviePage {
            total_size: items.len(),
            items,
            start: 0,
        })
    }

    /// 获取全局 Continue Watching，并尽量用更大的数量上限拿到比首页更多的条目。
    fn fetch_continue_watching(&self, count: usize) -> Result<Vec<MediaItem>, PlexError> {
        let count_value = count.to_string();
        let xml = self.get_xml("/hubs/continueWatching", &[("count", &count_value)])?;
        let container: MediaContainer = from_str(&xml)?;

        let mut items = Vec::new();
        let mut seen = HashSet::new();
        for hub in container.hubs {
            let is_continue_watching = hub.hub_identifier.as_deref() == Some("continueWatching")
                || hub
                    .key
                    .as_deref()
                    .map(|value| value.contains("continueWatching"))
                    .unwrap_or(false);
            if !is_continue_watching {
                continue;
            }

            for video in hub.videos {
                if seen.insert(video.rating_key.clone()) {
                    items.push(media_item_from_video(video));
                }
            }
        }

        Ok(items)
    }

    fn fetch_playback_history(&self, size: usize) -> Result<Vec<HistoryVideoNode>, PlexError> {
        let size_value = size.to_string();
        let xml = self.get_xml(
            "/status/sessions/history/all",
            &[
                ("sort", "viewedAt:desc"),
                ("X-Plex-Container-Start", "0"),
                ("X-Plex-Container-Size", &size_value),
            ],
        )?;
        let container: HistoryContainer = from_str(&xml)?;
        Ok(container.videos)
    }

    fn fetch_metadata_items(&self, rating_keys: &[String]) -> Result<Vec<MediaItem>, PlexError> {
        let mut items = Vec::new();
        for chunk in rating_keys.chunks(50) {
            if chunk.is_empty() {
                continue;
            }

            let path = format!("/library/metadata/{}", chunk.join(","));
            let xml = self.get_xml(&path, &[])?;
            let container: MediaContainer = from_str(&xml)?;
            items.extend(container.videos.into_iter().map(media_item_from_video));
        }

        Ok(items)
    }

    /// 使用 Plex 图片转码接口生成固定尺寸的缩略图 URL。
    pub fn build_thumbnail_url(&self, thumb_path: &str, width: u32, height: u32) -> Result<String, PlexError> {
        let absolute_thumb = self.absolute_url(thumb_path, &[])?;
        let url = self.absolute_url(
            "/photo/:/transcode",
            &[
                ("width", &width.to_string()),
                ("height", &height.to_string()),
                ("minSize", "1"),
                ("upscale", "0"),
                ("url", absolute_thumb.as_str()),
            ],
        )?;
        Ok(url.to_string())
    }

    /// 媒体分片路径加上 `download=1`，得到可被播放器打开的地址。
    pub fn build_stream_url(&self, part_key: &str) -> Result<String, PlexError> {
        let url = self.absolute_url(part_key, &[("download", "1")])?;
        Ok(url.to_string())
    }

    pub fn download_thumbnail(&self, thumb_path: &str) -> Result<Vec<u8>, PlexError> {
        let url = self.build_thumbnail_url(thumb_path, 180, 270)?;
        let response = self.http.get(url).send()?.error_for_status()?;
        Ok(response.bytes()?.to_vec())
    }

    /// GET 请求并返回 XML 文本；查询参数与 Token 统一在 `absolute_url` 中附加。
    fn get_xml(&self, path: &str, params: &[(&str, &str)]) -> Result<String, PlexError> {
        let url = self.absolute_url(path, params)?;
        let response = self.http.get(url).send()?.error_for_status()?;
        Ok(response.text()?)
    }

    /// 相对路径拼到 `server_url`，或解析绝对 URL；始终追加 `X-Plex-Token`。
    fn absolute_url(&self, path: &str, params: &[(&str, &str)]) -> Result<Url, PlexError> {
        let base = Url::parse(&self.server_url)?;
        let mut url = if path.starts_with("http://") || path.starts_with("https://") {
            Url::parse(path)?
        } else {
            base.join(path)?
        };

        {
            let mut pairs = url.query_pairs_mut();
            for (key, value) in params {
                pairs.append_pair(key, value);
            }
            pairs.append_pair("X-Plex-Token", &self.token);
        }

        Ok(url)
    }
}

#[derive(Debug, Error)]
pub enum PlexError {
    #[error("invalid Plex server URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("failed to build HTTP client: {0}")]
    Client(#[from] reqwest::Error),
    #[error("failed to parse Plex response: {0}")]
    Xml(#[from] quick_xml::DeError),
}

/// Plex XML 根容器：反序列化时映射 `@` 属性与子元素。
#[derive(Debug, Deserialize)]
struct MediaContainer {
    #[serde(rename = "@size")]
    size: Option<usize>,
    #[serde(rename = "@totalSize")]
    total_size: Option<usize>,
    #[serde(rename = "Directory", default)]
    directories: Vec<DirectoryNode>,
    #[serde(rename = "Hub", default)]
    hubs: Vec<HubNode>,
    #[serde(rename = "Video", default)]
    videos: Vec<VideoNode>,
}

#[derive(Debug, Deserialize)]
struct HubNode {
    #[serde(rename = "@hubIdentifier")]
    hub_identifier: Option<String>,
    #[serde(rename = "@key")]
    key: Option<String>,
    #[serde(rename = "Video", default)]
    videos: Vec<VideoNode>,
}

#[derive(Debug, Deserialize)]
struct DirectoryNode {
    #[serde(rename = "@key")]
    key: String,
    #[serde(rename = "@title")]
    title: String,
    #[serde(rename = "@type")]
    kind: Option<String>,
}

#[derive(Debug, Deserialize)]
struct VideoNode {
    #[serde(rename = "@ratingKey")]
    rating_key: String,
    #[serde(rename = "@title")]
    title: String,
    #[serde(rename = "@year")]
    year: Option<i32>,
    #[serde(rename = "@summary")]
    summary: Option<String>,
    #[serde(rename = "@thumb")]
    thumb: Option<String>,
    #[serde(rename = "@librarySectionTitle")]
    library_section_title: Option<String>,
    #[serde(rename = "@duration")]
    duration: Option<u64>,
    #[serde(rename = "@viewOffset")]
    view_offset: Option<u64>,
    #[serde(rename = "@lastViewedAt")]
    last_viewed_at: Option<i64>,
    #[serde(rename = "Media", default)]
    media: Vec<MediaNode>,
}

#[derive(Debug, Deserialize)]
struct MediaNode {
    #[serde(rename = "@duration")]
    duration: Option<u64>,
    #[serde(rename = "Part", default)]
    parts: Vec<PartNode>,
}

#[derive(Debug, Deserialize)]
struct HistoryContainer {
    #[serde(rename = "Video", default)]
    videos: Vec<HistoryVideoNode>,
}

#[derive(Debug, Deserialize)]
struct HistoryVideoNode {
    #[serde(rename = "@ratingKey")]
    rating_key: Option<String>,
    #[serde(rename = "@type")]
    kind: Option<String>,
    #[serde(rename = "@viewedAt")]
    viewed_at: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PartNode {
    #[serde(rename = "@key")]
    key: Option<String>,
}

fn media_item_from_video(video: VideoNode) -> MediaItem {
    let part_key = video
        .media
        .iter()
        .flat_map(|media| media.parts.iter())
        .find_map(|part| part.key.clone());
    let duration = video
        .duration
        .or_else(|| video.media.iter().find_map(|media| media.duration));

    MediaItem {
        rating_key: video.rating_key,
        title: video.title,
        year: video.year,
        summary: video.summary.filter(|value| !value.trim().is_empty()),
        thumb: video.thumb,
        library_section_title: video.library_section_title,
        duration,
        view_offset: video.view_offset,
        last_viewed_at: video.last_viewed_at,
        part_key,
    }
}

fn dedup_history_rating_keys(
    history: &[HistoryVideoNode],
    seen: &HashSet<String>,
) -> Vec<String> {
    let mut unique = HashSet::new();
    let mut keys = Vec::new();

    for entry in history {
        if entry.kind.as_deref() != Some("movie") {
            continue;
        }
        let Some(rating_key) = entry.rating_key.as_ref() else {
            continue;
        };
        if seen.contains(rating_key) || !unique.insert(rating_key.clone()) {
            continue;
        }
        keys.push(rating_key.clone());
    }

    keys
}

fn history_viewed_at_map(history: &[HistoryVideoNode]) -> std::collections::HashMap<String, i64> {
    let mut map = std::collections::HashMap::new();
    for entry in history {
        let (Some(rating_key), Some(viewed_at)) = (entry.rating_key.as_ref(), entry.viewed_at)
        else {
            continue;
        };
        map.entry(rating_key.clone()).or_insert(viewed_at);
    }
    map
}

fn is_continue_candidate(item: &MediaItem) -> bool {
    let (Some(view_offset), Some(duration)) = (item.view_offset, item.duration) else {
        return false;
    };

    duration > 0
        && view_offset >= 60_000
        && view_offset < duration.saturating_mul(95) / 100
}
