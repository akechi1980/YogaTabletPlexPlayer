use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

use eframe::egui::{self, ColorImage, RichText, TextureHandle, TextureOptions};

use crate::cache::ThumbnailCache;
use crate::config::AppConfig;
use crate::models::{LibrarySection, MediaItem, MoviePage};
use crate::player;
use crate::plex::PlexClient;
use crate::ui::poster_grid;
use crate::AppPaths;

/// 每次向 Plex 请求影片列表的条数（分页大小）。
const PAGE_SIZE: usize = 60;
const CONTINUE_WATCHING_COUNT: usize = 200;
const CONTINUE_WATCHING_HISTORY_SIZE: usize = 400;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BrowseMode {
    Library,
    ContinueWatching,
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
    search_text: String,
    selected_item_key: Option<String>,
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
    pub fn new(_cc: &eframe::CreationContext<'_>, paths: AppPaths) -> Self {
        let config = AppConfig::load_or_default(&paths.config_file).unwrap_or_default();
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
            search_text: String::new(),
            selected_item_key: None,
            status_text: "Fill in Plex URL, token, and VLC path first.".to_owned(),
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
        match self.config.save(&self.config_path) {
            Ok(()) => {
                self.error_text = None;
                self.status_text =
                    format!("Configuration saved to {}", self.config_path.display());
            }
            Err(error) => self.error_text = Some(error.to_string()),
        }
    }

    fn load_sections(&mut self) {
        if !self.config.is_ready() {
            self.error_text = Some("A valid Plex URL and token are required.".to_owned());
            return;
        }

        self.error_text = None;
        self.is_loading_sections = true;
        self.status_text = "Loading Plex movie libraries...".to_owned();
        let _ = self
            .worker_tx
            .send(WorkerCommand::LoadSections(self.config.clone()));
    }

    fn reload_movies(&mut self) {
        if self.config.selected_library_id.trim().is_empty() {
            self.error_text = Some("Choose a movie library first.".to_owned());
            return;
        }

        self.browse_mode = BrowseMode::Library;
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
        self.status_text = format!("Loading page {}...", self.next_start / PAGE_SIZE + 1);
        let _ = self.worker_tx.send(WorkerCommand::LoadMovies {
            config: self.config.clone(),
            section_id: self.config.selected_library_id.clone(),
            start: self.next_start,
            size: PAGE_SIZE,
        });
    }

    fn load_continue_watching(&mut self) {
        if self.is_loading_movies {
            return;
        }

        self.browse_mode = BrowseMode::ContinueWatching;
        self.error_text = None;
        self.items.clear();
        self.textures.clear();
        self.poster_jobs.clear();
        self.selected_item_key = None;
        self.next_start = 0;
        self.total_size = 0;
        self.is_loading_movies = true;
        self.status_text = "Loading recent and continue-watching items...".to_owned();
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
                                format!("Loaded {} movie libraries.", self.libraries.len());
                            if !self.config.selected_library_id.trim().is_empty()
                                && self.items.is_empty()
                            {
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
                        self.textures.insert(rating_key, texture);
                    }
                }
                WorkerEvent::PosterFailed { rating_key } => {
                    self.poster_jobs.remove(&rating_key);
                }
            }
            ctx.request_repaint();
        }
    }

    fn apply_movie_page(&mut self, page: MoviePage) {
        self.next_start = page.start + page.items.len();
        self.total_size = page.total_size;
        self.items.extend(page.items);
        self.status_text = match self.browse_mode {
            BrowseMode::Library => format!(
                "Loaded {}/{} movies.",
                self.items.len(),
                self.total_size.max(self.items.len())
            ),
            BrowseMode::ContinueWatching => {
                format!("Loaded {} recent / continue items.", self.items.len())
            }
        };
    }

    fn browse_mode_title(&self) -> &'static str {
        match self.browse_mode {
            BrowseMode::Library => "Library",
            BrowseMode::ContinueWatching => "Recent & Continue Watching",
        }
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

    /// 若本地已有缓存则直接加载纹理；否则向后台发起一次海报下载任务。
    fn ensure_poster_texture(&mut self, ctx: &egui::Context, item: &MediaItem) {
        if self.textures.contains_key(&item.rating_key)
            || self.poster_jobs.contains(&item.rating_key)
        {
            return;
        }

        let cached = self.cache.poster_path(&item.rating_key);
        if cached.exists() {
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
            self.error_text = Some(format!("{} does not expose a playable media part.", item.title));
            return;
        };

        let client = match PlexClient::new(
            self.config.server_url_trimmed(),
            self.config.token.clone(),
        ) {
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
                        "Launched VLC for {} at {}",
                        item.title,
                        format_resume_time(offset_ms)
                    ),
                    _ => format!("Launched VLC for {}", item.title),
                };
            }
            Err(error) => self.error_text = Some(error.to_string()),
        }
    }
}

impl eframe::App for PosterLauncherApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        self.process_worker_events(ctx);

        egui::TopBottomPanel::top("top_bar").show(ctx, |ui| {
            let mut any_text_field_has_focus = false;
            let mut ime_purpose = egui::viewport::IMEPurpose::Normal;

            ui.horizontal_wrapped(|ui| {
                ui.label("Plex URL");
                let server_response = ui.add_sized(
                    [300.0, 0.0],
                    egui::TextEdit::singleline(&mut self.config.server_url)
                        .id_source("config_server_url"),
                );
                any_text_field_has_focus |= server_response.has_focus();

                ui.label("Token");
                let token_response = ui.add_sized(
                    [220.0, 0.0],
                    egui::TextEdit::singleline(&mut self.config.token)
                        .password(true)
                        .id_source("config_token"),
                );
                if token_response.has_focus() {
                    any_text_field_has_focus = true;
                    ime_purpose = egui::viewport::IMEPurpose::Password;
                }

                ui.label("VLC Path");
                let vlc_response = ui.add_sized(
                    [280.0, 0.0],
                    egui::TextEdit::singleline(&mut self.config.vlc_path)
                        .id_source("config_vlc_path"),
                );
                any_text_field_has_focus |= vlc_response.has_focus();

                if ui.button("Save Config").clicked() {
                    self.save_config();
                }
                if ui.button("Load Libraries").clicked() {
                    self.load_sections();
                }
            });

            ui.horizontal(|ui| {
                ui.label("Movie Library");
                egui::ComboBox::from_id_source("library_selector")
                    .selected_text(self.selected_library_title())
                    .show_ui(ui, |ui| {
                        for section in &self.libraries {
                            ui.selectable_value(
                                &mut self.config.selected_library_id,
                                section.id.clone(),
                                &section.title,
                            );
                        }
                    });

                if ui.button("Load Posters").clicked() {
                    self.reload_movies();
                }

                if ui.button("Recent / Continue").clicked() {
                    self.load_continue_watching();
                }

                if ui
                    .add_enabled(
                        self.browse_mode == BrowseMode::Library
                            && !self.is_loading_movies
                            && self.total_size > 0
                            && self.items.len() < self.total_size,
                        egui::Button::new("Load More"),
                    )
                    .clicked()
                {
                    self.load_more_movies();
                }

                ui.separator();
                ui.label("Search");
                let search_response = ui.add_sized(
                    [220.0, 0.0],
                    egui::TextEdit::singleline(&mut self.search_text)
                        .id_source("search_text"),
                );
                any_text_field_has_focus |= search_response.has_focus();

                if ui.button("Close").clicked() {
                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                }
            });

            ctx.send_viewport_cmd(egui::ViewportCommand::IMEAllowed(
                any_text_field_has_focus,
            ));
            ctx.send_viewport_cmd(egui::ViewportCommand::IMEPurpose(ime_purpose));
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            if let Some(error) = self.error_text.as_ref() {
                ui.colored_label(egui::Color32::LIGHT_RED, error);
            } else {
                ui.label(&self.status_text);
            }
        });

        egui::SidePanel::right("details_panel")
            .resizable(false)
            .default_width(260.0)
            .show(ctx, |ui| {
                ui.heading("Details");
                ui.separator();

                if let Some(item) = self.selected_item().cloned() {
                    ui.label(RichText::new(&item.title).strong().size(18.0));
                    ui.label(format!(
                        "Year: {}",
                        item.year
                            .map(|year| year.to_string())
                            .unwrap_or_else(|| "-".to_owned())
                    ));
                    ui.label(format!("Rating Key: {}", item.rating_key));
                    if let Some(section_title) = item.library_section_title.as_deref() {
                        ui.label(format!("Section: {}", section_title));
                    }
                    if let Some(progress) = continue_progress_text(&item) {
                        ui.label(format!("Progress: {}", progress));
                    }
                    ui.separator();
                    ui.label(
                        item.summary
                            .as_deref()
                            .unwrap_or("No summary available."),
                    );
                    ui.add_space(12.0);
                    if ui.button("Play In VLC").clicked() {
                        self.play_item(&item);
                    }
                } else {
                    ui.label("Select a poster to inspect and play it here.");
                }
            });

        egui::CentralPanel::default().show(ctx, |ui| {
            let filtered = poster_grid::filtered_items(&self.items, &self.search_text);
            let columns = poster_grid::poster_columns(ui.available_width());
            let can_load_more = self.browse_mode == BrowseMode::Library
                && !self.is_loading_movies
                && self.total_size > 0
                && self.items.len() < self.total_size;

            ui.horizontal(|ui| {
                ui.label(RichText::new(self.browse_mode_title()).strong());
                ui.label(RichText::new(format!("Loaded {}", self.items.len())).strong());
                ui.label(format!("Filtered {}", filtered.len()));
                if self.is_loading_sections || self.is_loading_movies {
                    ui.spinner();
                }
            });

            ui.separator();

            egui::ScrollArea::vertical().show(ui, |ui| {
                for row in filtered.chunks(columns) {
                    ui.horizontal_top(|ui| {
                        for item_index in row {
                            let item = self.items[*item_index].clone();
                            self.ensure_poster_texture(ctx, &item);

                            ui.vertical(|ui| {
                                let clicked = if let Some(texture) =
                                    self.textures.get(&item.rating_key)
                                {
                                    ui.add(egui::ImageButton::new((
                                        texture.id(),
                                        egui::vec2(
                                            poster_grid::POSTER_WIDTH,
                                            poster_grid::POSTER_HEIGHT,
                                        ),
                                    )))
                                    .clicked()
                                } else {
                                    poster_grid::draw_placeholder(ui, &item.title).clicked()
                                };

                                ui.add_sized(
                                    [poster_grid::POSTER_CARD_WIDTH - 12.0, 0.0],
                                    egui::Label::new(
                                        egui::RichText::new(&item.title).size(14.0),
                                    ),
                                );
                                if let Some(year) = item.year {
                                    ui.small(year.to_string());
                                } else if let Some(progress) = continue_progress_text(&item) {
                                    ui.small(progress);
                                } else {
                                    ui.small("-");
                                }
                                if clicked {
                                    self.selected_item_key = Some(item.rating_key.clone());
                                }
                                if ui.small_button("Play").clicked() {
                                    self.play_item(&item);
                                }
                            });
                        }
                    });
                    ui.add_space(12.0);
                }

                if can_load_more && ui.button("Load Next Page").clicked() {
                    self.load_more_movies();
                }
            });
        });
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
                    start,
                    size,
                } => {
                    let result = PlexClient::new(config.server_url_trimmed(), config.token)
                        .and_then(|client| client.fetch_movies_page(&section_id, start, size))
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
    let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
    let size = [image.width() as usize, image.height() as usize];
    let pixels = image.into_raw();
    let color_image = ColorImage::from_rgba_unmultiplied(size, &pixels);
    Some(ctx.load_texture(
        cache_key.to_owned(),
        color_image,
        TextureOptions::LINEAR,
    ))
}

/// 后台任务类型：与 UI 操作一一对应。
enum WorkerCommand {
    LoadSections(AppConfig),
    LoadMovies {
        config: AppConfig,
        section_id: String,
        start: usize,
        size: usize,
    },
    LoadContinueWatching(AppConfig),
    FetchPoster {
        config: AppConfig,
        rating_key: String,
        thumb_path: String,
    },
}

fn continue_progress_text(item: &MediaItem) -> Option<String> {
    let (view_offset, duration) = (item.view_offset?, item.duration?);
    if duration == 0 {
        return None;
    }

    let percent = ((view_offset as f64 / duration as f64) * 100.0).round() as i32;
    Some(format!("{}%", percent.clamp(0, 100)))
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
}
