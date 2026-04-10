# Plex Poster Launcher

轻量级桌面启动器：以海报墙形式浏览 Plex 电影库，并用外部 VLC 播放器打开影片。

## MVP 范围

- 本地保存配置：
  - Plex 服务器 URL
  - Plex Token
  - VLC 可执行文件路径
- 从 Plex 读取电影库
- 在可滚动网格中显示海报
- 按标题关键词筛选已加载的影片
- 使用外部 `vlc.exe`（或本机 VLC 路径）以 Plex 媒体 URL 启动播放
- 在本地缓存海报缩略图

## 技术栈

- Rust
- `eframe` / `egui`：简单桌面 UI
- `reqwest`：Plex HTTP 请求
- `quick-xml`：解析 Plex XML 响应
- `serde` + `toml`：配置持久化

## 项目结构

```text
.
|-- Cargo.toml
|-- README.md
|-- config.example.toml
`-- src
    |-- app.rs
    |-- cache.rs
    |-- config.rs
    |-- main.rs
    |-- models.rs
    |-- player.rs
    |-- plex.rs
    `-- ui
        |-- mod.rs
        `-- poster_grid.rs
```

## 构建

先安装 Rust 工具链，然后执行：

```powershell
cargo build --release
```

## GitHub Actions 构建

仓库内含工作流 `.github/workflows/windows-win32.yml`。

- 在 `windows-latest` 上构建
- 目标为 `i686-pc-windows-msvc`（32 位 Windows 可执行文件）
- 将可直接分发的 `.zip` 作为工作流构件上传
- 压缩包内包含 `.exe`、`config.toml`、字体文件和 `README.md`
- 仅在最新提交信息以 `[deploy]` 开头时在 push 时自动运行
- 仍可通过 `workflow_dispatch` 手动触发

若在 Ubuntu 上开发但需要 Win32 发布产物，推荐使用此流程。

## 运行

```powershell
cargo run --release
```

## 本地启动

在 Ubuntu 本地开发时，直接执行：

```bash
./start.sh
```

若想用 release 模式启动：

```bash
./start.sh --release
```

如果你使用 VS Code / Cursor 一类 IDE，也可以直接点调试按钮。

- 已提供本地调试配置：`.vscode/launch.json`
- 需要安装 `CodeLLDB` 扩展
- 选择 `Debug Plex Poster Launcher` 后即可直接启动
- 调试时会自动使用和 `start.sh` 一致的 `XDG_CONFIG_HOME` / `XDG_CACHE_HOME`

这个脚本会做两件事：

- 把配置和缓存固定到仓库目录下，避免你去系统目录里找文件
- 若配置文件不存在，则自动生成一份可编辑的初始配置

如果仓库根目录已经有 `./config.toml`，程序会优先读取它。

脚本默认使用下面两个目录：

- 配置文件：`./.local/config/plexposterlauncher/config.toml`
- 缓存目录：`./.local/cache/plexposterlauncher`

应用本身默认会将配置与缩略图缓存放在当前用户的应用数据目录下；`start.sh` 通过设置 `XDG_CONFIG_HOME` 和 `XDG_CACHE_HOME`，把它们重定向到仓库本地目录，方便开发。

配置保存在名为 `config.toml` 的 TOML 文件中，可直接编辑，也可在应用界面中修改并保存。

配置文件读取优先级如下：

- 若当前工作目录存在 `./config.toml`，优先读取这份
- 否则使用平台默认应用配置目录

## 字体与标题显示

为保证 UTF-8 文本正常显示，并让未来的 Win32 版本和当前 Ubuntu 开发环境尽量保持一致，程序现在默认内置了一份 CJK 字体。

- 默认会使用仓库内置的 `DroidSansFallbackFull.ttf`
- 也可以直接把字体文件放在仓库根目录，例如 `./DroidSansFallbackFull.ttf`
- 如果你之后要在别的机器上运行，也可以手动放入 `./fonts/DroidSansFallbackFull.ttf`
- 也支持放入 `./fonts/NotoSansCJKsc-Regular.otf` 或 `./fonts/NotoSansCJKjp-Regular.otf`

这样可以明显改善中文、日文片名和简介的显示问题，同时减少不同平台因为系统字体差异带来的显示波动。

启动 VLC 时，程序现在也会把影片标题一起传过去，因此播放器窗口或播放列表会尽量显示影片名，而不是只显示一串媒体 URL。

## 配置项说明

- `server_url`：Plex 服务器地址，例如 `http://192.168.1.100:32400`
- `token`：Plex Token
- `vlc_path`：`vlc.exe`（或 Linux/macOS 上 `vlc`）的完整路径
- `selected_library_id`：上次选中的电影库 ID

上述字段由应用自动读写；通过配置文件设置 VLC 路径与 Plex 连接信息已支持。

## 示例配置

见 `config.example.toml`。

```toml
server_url = "http://192.168.1.100:32400"
token = "replace-with-your-plex-token"
vlc_path = "/snap/bin/vlc"
selected_library_id = ""
```

配置时可以这样理解：

- `server_url` 填你自己的 Plex 地址，例如 `http://192.168.1.100:32400`
- `token` 填你 Plex 服务器对应账号的 token
- `vlc_path` 在你这台 Ubuntu 上可以直接填 `/snap/bin/vlc`
- `selected_library_id` 第一次可以留空，等应用读到电影库后会自动保存

第一次启动最简单的流程就是：

1. 执行 `./start.sh`
2. 打开 `./config.toml`
3. 把 `token` 改成你自己的 Plex Token
4. 再次执行 `./start.sh`

## 当前行为

- 从 Plex 加载电影库列表。
- 选择库后加载第一页海报数据。
- 海报墙支持滚动与手动分页（「加载更多」等）。
- 搜索仅过滤**已加载到内存**的影片。
- 点击「Play」启动外部 VLC。
- 点击海报在右侧打开详情面板。

## 已知限制

- 现阶段仅支持**电影库**。
- 播放当前使用条目中 Plex 暴露的**第一个**媒体分片 URL。
- 面向固定私有 Plex 环境（尤其局域网）使用。
- 缩略图下载失败时显示占位图。
- 若本地无法编译 Windows 版本，预期通过 GitHub Actions 产出 Win32 构建。
