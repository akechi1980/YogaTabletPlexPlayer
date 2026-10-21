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
    /// 影片所属片库名。界面目前不展示，保留给播放和后续详情使用。
    #[allow(dead_code)]
    pub library_section_title: Option<String>,
    pub duration: Option<u64>,
    pub view_offset: Option<u64>,
    pub view_count: Option<u32>,
    pub last_viewed_at: Option<i64>,
    /// 第一个可播放分片的 API 路径，用于构造流 URL。
    pub part_key: Option<String>,
}

/// 片库列表的服务端排序。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MovieSort {
    /// 最近更新在前。
    Newest,
    /// 最早更新在前。
    Oldest,
    /// 按标题排序。
    Title,
}

impl MovieSort {
    /// Plex `sort` 查询参数。
    pub fn plex_value(self) -> &'static str {
        match self {
            MovieSort::Newest => "updatedAt:desc",
            MovieSort::Oldest => "updatedAt:asc",
            MovieSort::Title => "titleSort:asc",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            MovieSort::Newest => "最新优先",
            MovieSort::Oldest => "最早优先",
            MovieSort::Title => "名称排序",
        }
    }
}

impl Default for MovieSort {
    fn default() -> Self {
        Self::Newest
    }
}

/// 某一页影片列表及 Plex 返回的总条数，用于分页。
#[derive(Clone, Debug, Default)]
pub struct MoviePage {
    pub items: Vec<MediaItem>,
    pub start: usize,
    pub total_size: usize,
}
