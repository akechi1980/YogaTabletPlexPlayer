/// Plex 媒体库分区（此处仅使用电影类型）。
#[derive(Clone, Debug, Default)]
pub struct LibrarySection {
    pub id: String,
    pub title: String,
}

/// 单部影片在 UI 与播放逻辑中使用的字段子集。
#[derive(Clone, Debug, Default)]
pub struct MediaItem {
    pub rating_key: String,
    pub title: String,
    pub year: Option<i32>,
    pub summary: Option<String>,
    pub thumb: Option<String>,
    pub library_section_title: Option<String>,
    pub duration: Option<u64>,
    pub view_offset: Option<u64>,
    pub last_viewed_at: Option<i64>,
    /// 第一个可播放分片的 API 路径，用于构造流 URL。
    pub part_key: Option<String>,
}

/// 某一页影片列表及 Plex 返回的总条数，用于分页。
#[derive(Clone, Debug, Default)]
pub struct MoviePage {
    pub items: Vec<MediaItem>,
    pub start: usize,
    pub total_size: usize,
}
