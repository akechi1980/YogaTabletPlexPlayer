use std::path::PathBuf;

/// 海报文件按 `rating_key` 命名存放在单一目录下。
#[derive(Clone, Debug)]
pub struct ThumbnailCache {
    root: PathBuf,
}

impl ThumbnailCache {
    pub fn new(root: PathBuf) -> Self {
        Self { root }
    }

    pub fn ensure_dir(&self) -> std::io::Result<()> {
        std::fs::create_dir_all(&self.root)
    }
    pub fn poster_path(&self, rating_key: &str) -> PathBuf {
        self.root.join(format!("{rating_key}.jpg"))
    }
}
