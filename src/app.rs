use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui::{self, ColorImage, TextureHandle, TextureOptions};

use crate::cache::ThumbnailCache;
use crate::config::AppConfig;
use crate::models::{LibrarySection, MediaItem, MoviePage, MovieSort};
use crate::player;
use crate::plex::PlexClient;
use crate::AppPaths;

#[path = "app_ui.rs"]
mod app_ui;

/// 每次向 Plex 请求影片列表的条数（分页大小）。
const PAGE_SIZE: usize = 60;
const CONTINUE_WATCHING_COUNT: usize = 200;
const CONTINUE_WATCHING_HISTORY_SIZE: usize = 400;
const SEARCH_HISTORY_LIMIT: usize = 50;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BrowseMode {
    Library,
    ContinueWatching,
}

/// 顶栏三个浏览页签。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HomeTab {
    All,
    Continue,
    Recent,
}

/// 主界面状态：配置、已加载库/影片、海报纹理，以及后台线程通信。
pub struct PosterLauncherApp {
    config: AppConfig,
    config_path: PathBuf,
    cache: ThumbnailCache,
    libraries: Vec<LibrarySection>,
    items: Vec<MediaItem>,
    textures: HashMap<String, TextureHandle>,
    poster_jobs: HashSet<String>,
    /// 本帧还能在界面线程解码几张海报。多了会把笔的点击堵在解码后面。
    poster_decodes_left: u8,
    search_text: String,
    active_search_query: Option<String>,
    sort_mode: MovieSort,
    home_tab: HomeTab,
    show_settings: bool,
    show_history: bool,
    /// 用户手动点了「更新海报」时记下 rating key，避免每次自动下载都改状态栏。
    poster_refresh_key: Option<String>,
    selected_item_key: Option<String>,
    pending_delete_item: Option<MediaItem>,
    deleting_item_key: Option<String>,
    pending_mark_watched_item: Option<MediaItem>,
    marking_watched_item_key: Option<String>,
    status_text: String,
    error_text: Option<String>,
    is_loading_sections: bool,
    is_loading_movies: bool,
    next_start: usize,
    total_size: usize,
    browse_mode: BrowseMode,
    /// 发往后台线程的指令（加载库、分页、拉取海报）。
    worker_tx: Sender<WorkerCommand>,
    /// 主线程轮询接收的后台结果。
    worker_rx: Receiver<WorkerEvent>,
}

impl PosterLauncherApp {
    pub fn new(cc: &eframe::CreationContext<'_>, paths: AppPaths) -> Self {
        let config = AppConfig::load_or_default(&paths.config_file).unwrap_or_default();
        crate::ui::theme::set_mode(config.theme);
        crate::ui::theme::apply(&cc.egui_ctx);
        let cache = ThumbnailCache::new(paths.cache_dir.clone());
        let _ = cache.ensure_dir();

        // 网络与解码在独立线程，避免阻塞 UI
        let (worker_tx, worker_rx) = spawn_worker(cache.clone());

        let mut app = Self {
            config,
            config_path: paths.config_file,
            cache,
            libraries: Vec::new(),
            items: Vec::new(),
            textures: HashMap::new(),
            poster_jobs: HashSet::new(),
            poster_decodes_left: 0,
            search_text: String::new(),
            active_search_query: None,
            sort_mode: MovieSort::Newest,
            home_tab: HomeTab::All,
            show_settings: false,
            show_history: false,
            poster_refresh_key: None,
            selected_item_key: None,
            pending_delete_item: None,
            deleting_item_key: None,
            pending_mark_watched_item: None,
            marking_watched_item_key: None,
            status_text: "请先在设置中填写 Plex 地址、Token 和 VLC 路径。".to_owned(),
            error_text: None,
            is_loading_sections: false,
            is_loading_movies: false,
            next_start: 0,
            total_size: 0,
            browse_mode: BrowseMode::Library,
            worker_tx,
            worker_rx,
        };

        if app.config.is_ready() {
            app.load_sections();
        }

        app
    }

    fn save_config(&mut self) {
        self.save_config_internal(true);
    }

    fn save_config_silently(&mut self) {
        self.save_config_internal(false);
    }

    fn save_config_internal(&mut self, show_success_message: bool) {
        match self.config.save(&self.config_path) {
            Ok(()) => {
                self.error_text = None;
                if show_success_message {
                    self.status_text = "设置已保存".to_owned();
                }
            }
            Err(error) => self.error_text = Some(error.to_string()),
        }
    }

    fn remember_current_search(&mut self) {
        let query = self.search_text.trim();
        if query.is_empty() {
            return;
        }

        self.config
            .search_history
            .retain(|item| item.trim() != query);
        self.config.search_history.insert(0, query.to_owned());
        self.config.search_history.truncate(SEARCH_HISTORY_LIMIT);
        self.save_config_silently();
    }

    fn delete_search_history_at(&mut self, index: usize) {
        if index >= self.config.search_history.len() {
            return;
        }

        self.config.search_history.remove(index);
        self.save_config_silently();
    }

    fn load_sections(&mut self) {
        if !self.config.is_ready() {
            self.error_text = Some("需要有效的 Plex 地址和 Token。".to_owned());
            return;
        }

        self.error_text = None;
        self.is_loading_sections = true;
        self.status_text = "正在加载片库…".to_owned();
        let _ = self
            .worker_tx
            .send(WorkerCommand::LoadSections(self.config.clone()));
    }

    fn reload_movies(&mut self) {
        if self.config.selected_library_id.trim().is_empty() {
            self.error_text = Some("请先选择一个电影库。".to_owned());
            return;
        }

        self.browse_mode = BrowseMode::Library;
        if self.home_tab == HomeTab::Continue {
            self.home_tab = HomeTab::All;
        }
        self.active_search_query = None;
        self.error_text = None;
        self.items.clear();
        self.textures.clear();
        self.poster_jobs.clear();
        self.selected_item_key = None;
        self.next_start = 0;
        self.total_size = 0;
        self.load_more_movies();
    }

    fn load_more_movies(&mut self) {
        if self.is_loading_movies
            || self.browse_mode != BrowseMode::Library
            || self.config.selected_library_id.trim().is_empty()
        {
            return;
        }

        self.error_text = None;
        self.is_loading_movies = true;
        self.status_text = match self.active_search_query.as_deref() {
            Some(query) => format!(
                "正在搜索「{}」第 {} 页…",
                query,
                self.next_start / PAGE_SIZE + 1
            ),
            None => format!("正在加载第 {} 页…", self.next_start / PAGE_SIZE + 1),
        };
        let _ = self.worker_tx.send(WorkerCommand::LoadMovies {
            config: self.config.clone(),
            section_id: self.config.selected_library_id.clone(),
            search_query: self.active_search_query.clone(),
            sort: self.sort_mode,
            start: self.next_start,
            size: PAGE_SIZE,
        });
    }

    fn apply_search(&mut self) {
        if self.config.selected_library_id.trim().is_empty() {
            self.error_text = Some("请先选择一个电影库。".to_owned());
            return;
        }

        self.home_tab = HomeTab::All;
        let query = self.search_text.trim().to_owned();
        if query.is_empty() {
            self.reload_movies();
            return;
        }

        self.remember_current_search();
        self.browse_mode = BrowseMode::Library;
        self.active_search_query = Some(query);
        self.error_text = None;
        self.items.clear();
        self.textures.clear();
        self.poster_jobs.clear();
        self.selected_item_key = None;
        self.next_start = 0;
        self.total_size = 0;
        self.load_more_movies();
    }

    fn load_continue_watching(&mut self) {
        if self.is_loading_movies {
            return;
        }

        self.browse_mode = BrowseMode::ContinueWatching;
        self.active_search_query = None;
        self.error_text = None;
        self.items.clear();
        self.textures.clear();
        self.poster_jobs.clear();
        self.selected_item_key = None;
        self.next_start = 0;
        self.total_size = 0;
        self.is_loading_movies = true;
        self.status_text = "正在加载继续观看…".to_owned();
        let _ = self
            .worker_tx
            .send(WorkerCommand::LoadContinueWatching(self.config.clone()));
    }

    /// 非阻塞处理后台事件：更新列表/纹理并请求重绘。
    fn process_worker_events(&mut self, ctx: &egui::Context) {
        while let Ok(event) = self.worker_rx.try_recv() {
            match event {
                WorkerEvent::SectionsLoaded(result) => {
                    self.is_loading_sections = false;
                    match result {
                        Ok(sections) => {
                            self.error_text = None;
                            self.libraries = sections;
                            if self.config.selected_library_id.trim().is_empty()
                                || !self
                                    .libraries
                                    .iter()
                                    .any(|library| library.id == self.config.selected_library_id)
                            {
                                if let Some(first) = self.libraries.first() {
                                    self.config.selected_library_id = first.id.clone();
                                }
                            }
                            self.status_text =
                                format!("已载入 {} 个电影库", self.libraries.len());
                            if !self.config.selected_library_id.trim().is_empty() {
                                self.save_config_silently();
                                self.reload_movies();
                            }
                        }
                        Err(error) => {
                            self.error_text = Some(error);
                        }
                    }
                }
                WorkerEvent::MoviesLoaded(result) => {
                    self.is_loading_movies = false;
                    match result {
                        Ok(page) => {
                            self.error_text = None;
                            self.apply_movie_page(page);
                        }
                        Err(error) => {
                            self.error_text = Some(error);
                        }
                    }
                }
                WorkerEvent::PosterReady { rating_key, path } => {
                    self.poster_jobs.remove(&rating_key);
                    if let Some(texture) = load_texture_from_path(ctx, &path, &rating_key) {
                        self.textures.insert(rating_key.clone(), texture);
                    }
                    if self.poster_refresh_key.as_deref() == Some(rating_key.as_str()) {
                        self.poster_refresh_key = None;
                        if let Some(item) =
                            self.items.iter().find(|item| item.rating_key == rating_key)
                        {
                            self.status_text = format!("已更新《{}》的海报", item.title);
                        }
                    }
                }
                WorkerEvent::PosterFailed { rating_key } => {
                    self.poster_jobs.remove(&rating_key);
                    if self.poster_refresh_key.as_deref() == Some(rating_key.as_str()) {
                        self.poster_refresh_key = None;
                        if let Some(item) =
                            self.items.iter().find(|item| item.rating_key == rating_key)
                        {
                            self.error_text = Some(format!("更新《{}》的海报失败。", item.title));
                        }
                    }
                }
                WorkerEvent::MovieDeleted {
                    rating_key,
                    title,
                    result,
                } => {
                    if self.deleting_item_key.as_deref() == Some(rating_key.as_str()) {
                        self.deleting_item_key = None;
                    }

                    match result {
                        Ok(()) => {
                            self.error_text = None;
                            self.remove_deleted_item(&rating_key);
                            self.status_text = format!("已从 Plex 删除《{}》", title);
                        }
                        Err(error) => {
                            self.error_text = Some(format!("删除《{}》失败：{}", title, error));
                        }
                    }
                }
                WorkerEvent::MovieMarkedWatched {
                    rating_key,
                    title,
                    watched,
                    result,
                } => {
                    if self.marking_watched_item_key.as_deref() == Some(rating_key.as_str()) {
                        self.marking_watched_item_key = None;
                    }

                    match result {
                        Ok(()) => {
                            self.error_text = None;
                            if let Some(item) = self
                                .items
                                .iter_mut()
                                .find(|item| item.rating_key == rating_key)
                            {
                                if watched {
                                    item.view_count = Some(item.view_count.unwrap_or_default().max(1));
                                } else {
                                    item.view_count = Some(0);
                                    item.view_offset = None;
                                }
                            }
                            self.status_text = if watched {
                                format!("已将《{}》标为已看", title)
                            } else {
                                format!("已将《{}》标为未看", title)
                            };
                        }
                        Err(error) => {
                            self.error_text = Some(format!("标记《{}》失败：{}", title, error));
                        }
                    }
                }
            }
            ctx.request_repaint();
        }
    }

    fn remove_deleted_item(&mut self, rating_key: &str) {
        self.items.retain(|item| item.rating_key != rating_key);
        self.textures.remove(rating_key);
        self.poster_jobs.remove(rating_key);
        if self.selected_item_key.as_deref() == Some(rating_key) {
            self.selected_item_key = None;
        }
        if self
            .pending_delete_item
            .as_ref()
            .is_some_and(|item| item.rating_key == rating_key)
        {
            self.pending_delete_item = None;
        }
        if self.total_size > self.items.len() {
            self.total_size = self.total_size.saturating_sub(1);
        }

        let poster_path = self.cache.poster_path(rating_key);
        if poster_path.exists() {
            let _ = std::fs::remove_file(poster_path);
        }
    }

    fn apply_movie_page(&mut self, page: MoviePage) {
        self.next_start = page.start + page.items.len();
        self.total_size = page.total_size;
        self.items.extend(page.items);
        self.status_text = match self.browse_mode {
            BrowseMode::Library => format!(
                "当前显示 {} / {} 部",
                self.items.len(),
                self.total_size.max(self.items.len())
            ),
            BrowseMode::ContinueWatching => {
                format!("继续观看：{} 部", self.items.len())
            }
        };
    }

    fn selected_library_title(&self) -> String {
        self.libraries
            .iter()
            .find(|library| library.id == self.config.selected_library_id)
            .map(|library| library.title.clone())
            .unwrap_or_else(|| "Choose library".to_owned())
    }

    fn selected_item(&self) -> Option<&MediaItem> {
        let selected = self.selected_item_key.as_deref()?;
        self.items.iter().find(|item| item.rating_key == selected)
    }

    fn request_delete_item(&mut self, item: MediaItem) {
        if self.deleting_item_key.is_some() {
            return;
        }
        self.pending_delete_item = Some(item);
    }

    fn confirm_delete_item(&mut self) {
        let Some(item) = self.pending_delete_item.take() else {
            return;
        };
        if self.deleting_item_key.is_some() {
            return;
        }

        self.error_text = None;
        self.status_text = format!("正在从 Plex 删除《{}》…", item.title);
        self.deleting_item_key = Some(item.rating_key.clone());
        let _ = self.worker_tx.send(WorkerCommand::DeleteMovie {
            config: self.config.clone(),
            rating_key: item.rating_key,
            title: item.title,
        });
    }

    fn request_mark_watched(&mut self, item: MediaItem) {
        if self.marking_watched_item_key.is_some() {
            return;
        }
        self.pending_mark_watched_item = Some(item);
    }

    fn confirm_mark_watched(&mut self) {
        let Some(item) = self.pending_mark_watched_item.take() else {
            return;
        };
        if self.marking_watched_item_key.is_some() {
            return;
        }

        let mark_watched = item.view_count.unwrap_or_default() == 0;
        self.error_text = None;
        self.status_text = if mark_watched {
            format!("正在标记《{}》为已看…", item.title)
        } else {
            format!("正在标记《{}》为未看…", item.title)
        };
        self.marking_watched_item_key = Some(item.rating_key.clone());
        let _ = self.worker_tx.send(WorkerCommand::MarkMovieWatched {
            config: self.config.clone(),
            rating_key: item.rating_key,
            title: item.title,
            watched: mark_watched,
        });
    }

    fn refresh_poster(&mut self, item: &MediaItem) {
        if self.poster_jobs.contains(&item.rating_key) {
            return;
        }

        let Some(thumb) = item.thumb.clone() else {
            self.error_text = Some(format!("《{}》没有海报地址。", item.title));
            return;
        };

        self.error_text = None;
        self.poster_refresh_key = Some(item.rating_key.clone());
        self.status_text = format!("正在更新《{}》的海报…", item.title);
        self.textures.remove(&item.rating_key);
        self.poster_jobs.insert(item.rating_key.clone());
        let _ = self.worker_tx.send(WorkerCommand::FetchPoster {
            config: self.config.clone(),
            rating_key: item.rating_key.clone(),
            thumb_path: thumb,
        });
    }

    /// 若本地已有缓存则直接加载纹理；否则向后台发起一次海报下载任务。
    fn ensure_poster_texture(&mut self, ctx: &egui::Context, item: &MediaItem) {
        if self.textures.contains_key(&item.rating_key)
            || self.poster_jobs.contains(&item.rating_key)
        {
            return;
        }

        let cached = self.cache.poster_path(&item.rating_key);
        if cached.exists() {
            if self.poster_decodes_left == 0 {
                ctx.request_repaint();
                return;
            }
            self.poster_decodes_left -= 1;
            if let Some(texture) = load_texture_from_path(ctx, &cached, &item.rating_key) {
                self.textures.insert(item.rating_key.clone(), texture);
            }
            return;
        }

        let Some(thumb) = item.thumb.clone() else {
            return;
        };

        self.poster_jobs.insert(item.rating_key.clone());
        let _ = self.worker_tx.send(WorkerCommand::FetchPoster {
            config: self.config.clone(),
            rating_key: item.rating_key.clone(),
            thumb_path: thumb,
        });
    }

    /// 根据媒体分片 key 构造流地址并启动 VLC。
    fn play_item(&mut self, item: &MediaItem) {
        let Some(part_key) = item.part_key.as_deref() else {
            self.error_text = Some(format!("《{}》没有可播放的媒体文件。", item.title));
            return;
        };

        let client =
            match PlexClient::new(self.config.server_url_trimmed(), self.config.token.clone()) {
                Ok(client) => client,
                Err(error) => {
                    self.error_text = Some(error.to_string());
                    return;
                }
            };

        let stream_url = match client.build_stream_url(part_key) {
            Ok(url) => url,
            Err(error) => {
                self.error_text = Some(error.to_string());
                return;
            }
        };

        match player::launch_vlc(
            &self.config.vlc_path,
            &stream_url,
            &item.title,
            item.view_offset,
        ) {
            Ok(()) => {
                self.error_text = None;
                self.status_text = match item.view_offset {
                    Some(offset_ms) if offset_ms >= 1_000 => format!(
                        "已用 VLC 打开《{}》，从 {} 继续",
                        item.title,
                        format_resume_time(offset_ms)
                    ),
                    _ => format!("已用 VLC 打开《{}》", item.title),
                };
            }
            Err(error) => self.error_text = Some(error.to_string()),
        }
    }

    fn activate_tab(&mut self, tab: HomeTab) {
        if self.home_tab == tab
            && match tab {
                HomeTab::Continue => self.browse_mode == BrowseMode::ContinueWatching,
                HomeTab::All => {
                    self.browse_mode == BrowseMode::Library && self.active_search_query.is_none()
                }
                HomeTab::Recent => {
                    self.browse_mode == BrowseMode::Library
                        && self.sort_mode == MovieSort::Newest
                        && self.active_search_query.is_none()
                }
            }
        {
            return;
        }

        self.show_history = false;
        self.home_tab = tab;
        match tab {
            HomeTab::All => {
                self.active_search_query = None;
                self.search_text.clear();
                self.reload_movies();
            }
            HomeTab::Recent => {
                self.sort_mode = MovieSort::Newest;
                self.active_search_query = None;
                self.search_text.clear();
                self.reload_movies();
            }
            HomeTab::Continue => self.load_continue_watching(),
        }
    }

    fn set_sort(&mut self, sort: MovieSort) {
        if self.sort_mode == sort || self.home_tab == HomeTab::Continue {
            return;
        }
        self.sort_mode = sort;
        if self.home_tab == HomeTab::Recent && sort != MovieSort::Newest {
            self.home_tab = HomeTab::All;
        }
        self.reload_library_keeping_search();
    }

    fn reload_library_keeping_search(&mut self) {
        if self.active_search_query.is_some() {
            self.browse_mode = BrowseMode::Library;
            self.error_text = None;
            self.items.clear();
            self.textures.clear();
            self.poster_jobs.clear();
            self.selected_item_key = None;
            self.next_start = 0;
            self.total_size = 0;
            self.load_more_movies();
        } else {
            self.reload_movies();
        }
    }

    fn section_title(&self) -> String {
        if self.active_search_query.is_some() {
            return "搜索结果".to_owned();
        }
        match self.home_tab {
            HomeTab::Continue => "继续观看".to_owned(),
            HomeTab::Recent => "最近添加".to_owned(),
            HomeTab::All => self.selected_library_title(),
        }
    }

    fn section_count(&self) -> usize {
        if self.browse_mode == BrowseMode::ContinueWatching {
            self.items.len()
        } else {
            self.total_size.max(self.items.len())
        }
    }
}

impl eframe::App for PosterLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.poster_decodes_left = 1;
        self.process_worker_events(ctx);
        self.draw_ui(ctx);
    }
}

/// 启动后台线程：串行处理 Plex 请求与海报落盘，通过 channel 与 UI 通信。
fn spawn_worker(cache: ThumbnailCache) -> (Sender<WorkerCommand>, Receiver<WorkerEvent>) {
    let (command_tx, command_rx) = mpsc::channel::<WorkerCommand>();
    let (event_tx, event_rx) = mpsc::channel::<WorkerEvent>();

    thread::spawn(move || {
        let _ = cache.ensure_dir();
        while let Ok(command) = command_rx.recv() {
            match command {
                WorkerCommand::LoadSections(config) => {
                    let result = PlexClient::new(config.server_url_trimmed(), config.token)
                        .and_then(|client| client.fetch_movie_sections())
                        .map_err(|error| error.to_string());
                    let _ = event_tx.send(WorkerEvent::SectionsLoaded(result));
                }
                WorkerCommand::LoadMovies {
                    config,
                    section_id,
                    search_query,
                    sort,
                    start,
                    size,
                } => {
                    let result = PlexClient::new(config.server_url_trimmed(), config.token)
                        .and_then(|client| {
                            if let Some(query) = search_query.as_deref() {
                                client.search_movies_page(
                                    &section_id,
                                    query,
                                    sort,
                                    start,
                                    size,
                                )
                            } else {
                                client.fetch_movies_page(&section_id, sort, start, size)
                            }
                        })
                        .map_err(|error| error.to_string());
                    let _ = event_tx.send(WorkerEvent::MoviesLoaded(result));
                }
                WorkerCommand::LoadContinueWatching(config) => {
                    let result = PlexClient::new(config.server_url_trimmed(), config.token)
                        .and_then(|client| {
                            client.fetch_continue_watching_enhanced(
                                CONTINUE_WATCHING_COUNT,
                                CONTINUE_WATCHING_HISTORY_SIZE,
                            )
                        })
                        .map_err(|error| error.to_string());
                    let _ = event_tx.send(WorkerEvent::MoviesLoaded(result));
                }
                WorkerCommand::FetchPoster {
                    config,
                    rating_key,
                    thumb_path,
                } => match fetch_and_store_poster(&cache, &config, &rating_key, &thumb_path) {
                    Ok(path) => {
                        let _ = event_tx.send(WorkerEvent::PosterReady { rating_key, path });
                    }
                    Err(_) => {
                        let _ = event_tx.send(WorkerEvent::PosterFailed { rating_key });
                    }
                },
                WorkerCommand::DeleteMovie {
                    config,
                    rating_key,
                    title,
                } => {
                    let result = PlexClient::new(config.server_url_trimmed(), config.token)
                        .and_then(|client| client.delete_movie(&rating_key))
                        .map_err(|error| error.to_string());
                    let _ = event_tx.send(WorkerEvent::MovieDeleted {
                        rating_key,
                        title,
                        result,
                    });
                }
                WorkerCommand::MarkMovieWatched {
                    config,
                    rating_key,
                    title,
                    watched,
                } => {
                    let result = PlexClient::new(config.server_url_trimmed(), config.token)
                        .and_then(|client| client.set_movie_watched(&rating_key, watched))
                        .map_err(|error| error.to_string());
                    let _ = event_tx.send(WorkerEvent::MovieMarkedWatched {
                        rating_key,
                        title,
                        watched,
                        result,
                    });
                }
            }
        }
    });

    (command_tx, event_rx)
}

/// 下载缩略图并写入缓存目录，供主线程加载为 GPU 纹理。
fn fetch_and_store_poster(
    cache: &ThumbnailCache,
    config: &AppConfig,
    rating_key: &str,
    thumb_path: &str,
) -> Result<PathBuf, String> {
    let client = PlexClient::new(config.server_url_trimmed(), config.token.clone())
        .map_err(|error| error.to_string())?;
    let bytes = client
        .download_thumbnail(thumb_path)
        .map_err(|error| error.to_string())?;
    let path = cache.poster_path(rating_key);
    std::fs::write(&path, bytes).map_err(|error| error.to_string())?;
    Ok(path)
}

/// 从磁盘读取图片字节并注册为 egui 纹理。
fn load_texture_from_path(
    ctx: &egui::Context,
    path: &Path,
    cache_key: &str,
) -> Option<TextureHandle> {
    let bytes = std::fs::read(path).ok()?;
    // 海报在卡片上大约一百多像素宽，缩小后再上传，避免旧平板每帧都画整张原图。
    let image = image::load_from_memory(&bytes)
        .ok()?
        .thumbnail(280, 420)
        .to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();
    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
    Some(ctx.load_texture(cache_key.to_owned(), color_image, TextureOptions::LINEAR))
}

/// 后台任务类型：与 UI 操作一一对应。
enum WorkerCommand {
    LoadSections(AppConfig),
    LoadMovies {
        config: AppConfig,
        section_id: String,
        search_query: Option<String>,
        sort: MovieSort,
        start: usize,
        size: usize,
    },
    LoadContinueWatching(AppConfig),
    FetchPoster {
        config: AppConfig,
        rating_key: String,
        thumb_path: String,
    },
    DeleteMovie {
        config: AppConfig,
        rating_key: String,
        title: String,
    },
    MarkMovieWatched {
        config: AppConfig,
        rating_key: String,
        title: String,
        /// 目标状态：`true` 标为已看，`false` 标为未看。
        watched: bool,
    },
}

fn format_resume_time(offset_ms: u64) -> String {
    let total_seconds = offset_ms / 1_000;
    let hours = total_seconds / 3_600;
    let minutes = (total_seconds % 3_600) / 60;
    let seconds = total_seconds % 60;

    if hours > 0 {
        format!("{hours:02}:{minutes:02}:{seconds:02}")
    } else {
        format!("{minutes:02}:{seconds:02}")
    }
}

/// 后台完成后回传主线程的事件。
enum WorkerEvent {
    SectionsLoaded(Result<Vec<LibrarySection>, String>),
    MoviesLoaded(Result<MoviePage, String>),
    PosterReady {
        rating_key: String,
        path: PathBuf,
    },
    PosterFailed {
        rating_key: String,
    },
    MovieDeleted {
        rating_key: String,
        title: String,
        result: Result<(), String>,
    },
    MovieMarkedWatched {
        rating_key: String,
        title: String,
        watched: bool,
        result: Result<(), String>,
    },
}
