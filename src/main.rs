//! Plex 海报墙启动器入口：初始化配置/缓存目录，启动 egui 桌面窗口。
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod cache;
mod config;
mod models;
mod player;
mod plex;
mod touch_keyboard;
mod ui;

use std::fs;
use std::path::PathBuf;

use directories::ProjectDirs;
use eframe::egui::{self, FontData, FontDefinitions, FontFamily};

fn main() -> Result<(), eframe::Error> {
    // 确保配置与缓存目录存在，避免首次读写失败
    let paths = app_paths();
    if let Err(error) = std::fs::create_dir_all(&paths.config_dir) {
        eprintln!("failed to create config directory: {error}");
    }
    if let Err(error) = std::fs::create_dir_all(&paths.cache_dir) {
        eprintln!("failed to create cache directory: {error}");
    }

    let native_options = eframe::NativeOptions {
        renderer: eframe::Renderer::Glow,
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 780.0])
            .with_min_inner_size([960.0, 640.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Plex Poster Launcher",
        native_options,
        Box::new(move |cc| {
            configure_fonts(&cc.egui_ctx);
            Box::new(app::PosterLauncherApp::new(cc, paths.clone()))
        }),
    )
}

/// 应用持久化路径：配置文件与海报缓存根目录。
#[derive(Clone, Debug)]
pub struct AppPaths {
    pub config_dir: PathBuf,
    pub config_file: PathBuf,
    pub cache_dir: PathBuf,
}

/// 若当前工作目录存在 config.toml，则优先使用本地配置；否则使用系统应用数据目录。
fn app_paths() -> AppPaths {
    let current_dir = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let local_config_file = current_dir.join("config.toml");
    if local_config_file.exists() {
        return AppPaths {
            config_file: local_config_file,
            config_dir: current_dir.clone(),
            cache_dir: current_dir.join("cache"),
        };
    }

    if let Some(project_dirs) = ProjectDirs::from("com", "humble-ai", "PlexPosterLauncher") {
        let config_dir = project_dirs.config_dir().to_path_buf();
        let cache_dir = project_dirs.cache_dir().to_path_buf();
        return AppPaths {
            config_file: config_dir.join("config.toml"),
            config_dir,
            cache_dir,
        };
    }

    let fallback_root = current_dir;
    AppPaths {
        config_file: fallback_root.join("config.toml"),
        config_dir: fallback_root.clone(),
        cache_dir: fallback_root.join("cache"),
    }
}

fn configure_fonts(ctx: &egui::Context) {
    let Some(font_bytes) = load_cjk_font_bytes() else {
        return;
    };

    let mut fonts = FontDefinitions::default();
    fonts
        .font_data
        .insert("cjk_ui".to_owned(), FontData::from_owned(font_bytes));

    if let Some(family) = fonts.families.get_mut(&FontFamily::Proportional) {
        family.insert(0, "cjk_ui".to_owned());
    }
    if let Some(family) = fonts.families.get_mut(&FontFamily::Monospace) {
        family.push("cjk_ui".to_owned());
    }

    ctx.set_fonts(fonts);
}

fn load_cjk_font_bytes() -> Option<Vec<u8>> {
    let candidates = [
        "./DroidSansFallbackFull.ttf",
        "./NotoSansCJKsc-Regular.otf",
        "./NotoSansCJKjp-Regular.otf",
        "./fonts/DroidSansFallbackFull.ttf",
        "./fonts/NotoSansCJKsc-Regular.otf",
        "./fonts/NotoSansCJKjp-Regular.otf",
    ];

    for path in candidates {
        if let Ok(bytes) = fs::read(path) {
            return Some(bytes);
        }
    }

    Some(include_bytes!("../DroidSansFallbackFull.ttf").to_vec())
}
