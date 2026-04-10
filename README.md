# Plex Poster Launcher

**中文**  
轻量级桌面启动器：以海报墙形式浏览 Plex 电影库，并用外部 VLC 播放器打开影片。

**English**  
A lightweight desktop launcher: browse your Plex movie libraries as a poster wall and open titles in an external VLC player.

---

## 背景与动机 / Background

**中文**  
不少早年基于 **Win32（含 32 位 Windows）** 的平板（例如部分 Yoga 等机型）在 **Chrome / Chromium 系浏览器** 里播放 **H.265 / HEVC** 时，常受系统解码能力与浏览器支持限制，**无法硬解或无法正常播放**。若只用 **Plex Web** 在浏览器里看片，服务端往往要对 H.265 **实时转码**成浏览器兼容格式，既增加 NAS / 服务器的 CPU 负载，也容易带来卡顿与等待。  
**VLC** 对 H.265 支持成熟，适合在平板本机 **直链播放（Direct Play）**。本工具从 Plex 取媒体地址并用 **外部 VLC** 打开，从而尽量避免为「浏览器能播」而被迫转码，在老旧平板上也能更合理地观看高清资源。

**English**  
Many **older Win32 tablets** (including some **Yoga**-class devices running **32-bit Windows**) struggle with **H.265 / HEVC** inside **Chrome / Chromium**: OS and browser codec support is limited, so **hardware decode may be unavailable** and playback can fail. **Plex Web** in the browser then often forces the server to **transcode** H.265 into something the browser can play—adding **CPU load** on your NAS or Plex host and causing **buffering and delay**.  
**VLC** handles H.265 well and can **Direct Play** from Plex media URLs. This launcher fetches those URLs and opens them in **external VLC**, reducing the need to transcode just so a browser can decode the stream—making high-bitrate libraries more usable on those legacy tablets.

---

## MVP 范围 / MVP scope

**中文**  
- 本地保存配置：Plex 服务器 URL、Plex Token、VLC 可执行文件路径  
- 从 Plex 读取电影库  
- 在可滚动网格中显示海报  
- 按标题关键词筛选已加载的影片  
- 使用外部 `vlc.exe`（或本机 VLC 路径）以 Plex 媒体 URL 启动播放  
- 在本地缓存海报缩略图  

**English**  
- Persist config locally: Plex server URL, Plex token, VLC executable path  
- Read movie libraries from Plex  
- Show posters in a scrollable grid  
- Filter loaded titles by keyword  
- Launch playback via external `vlc.exe` (or your platform’s VLC path) with Plex media URLs  
- Cache poster thumbnails locally  

---

## Plex Token 获取方式 / How to get your Plex token

**中文**  
Token 相当于账户访问 Plex API 的凭证，**请当作密码保管**，勿提交到公开仓库或截图外传。常用获取方式如下（任选其一即可；若 Plex 改版，以 [Plex 官方说明](https://support.plex.tv/articles/204059436-finding-an-authentication-token-x-plex-token/) 为准）：

1. **浏览器开发者工具（Network）**  
   - 在浏览器中登录 [Plex Web](https://app.plex.tv)（或你自己的 Plex 地址）。  
   - 打开开发者工具（F12）→ **Network（网络）** 面板，刷新页面或浏览库。  
   - 选中发往 `plex.tv` 或你的 Plex 服务器的请求，在 **Request Headers** 中查找 **`X-Plex-Token`**，其值即 Token。

2. **Plex Web「查看 XML」类入口**  
   - 在 Plex Web 中对某个条目使用「查看 XML / View XML」等（具体菜单名可能随版本变化）。  
   - 浏览器地址栏 URL 中常带有 **`X-Plex-Token=...`** 查询参数，等号后的字符串即 Token。

3. **本地存储（部分环境）**  
   - 在已登录的 `app.plex.tv` 页面打开开发者工具 → **Application（应用）** → **Local Storage** → 选中 `https://app.plex.tv`，查看是否存在类似 **`myPlexAccessToken`** 的键（名称以你当前 Plex Web 版本为准）。

将获得的字符串填入 `config.toml` 或应用界面中的 **`token`** 字段即可。

**English**  
Your Plex token is a **secret** that grants API access—**treat it like a password**. Do not commit it to public repos or share screenshots. Common methods (use any one that works; if Plex changes their UI, follow [Plex’s official article](https://support.plex.tv/articles/204059436-finding-an-authentication-token-x-plex-token/)):

1. **Browser DevTools (Network)**  
   - Sign in to [Plex Web](https://app.plex.tv) (or your own Plex URL).  
   - Open DevTools (F12) → **Network**, reload or navigate your libraries.  
   - Select a request to `plex.tv` or your Plex server and read **`X-Plex-Token`** from **Request Headers**.

2. **“View XML” (or similar)**  
   - From Plex Web, use an option like **View XML** for an item (wording may vary by version).  
   - The URL in the address bar often contains **`X-Plex-Token=...`**; the value after `=` is your token.

3. **Local storage (where available)**  
   - On a signed-in `app.plex.tv` tab: DevTools → **Application** → **Local Storage** → `https://app.plex.tv` and look for a key such as **`myPlexAccessToken`** (exact names depend on the current Plex Web build).

Paste the value into the **`token`** field in `config.toml` or in the app UI.

---

## 技术栈 / Tech stack

**中文**  
Rust · `eframe` / `egui` · `reqwest` · `quick-xml` · `serde` + `toml`

**English**  
Rust · `eframe` / `egui` · `reqwest` · `quick-xml` · `serde` + `toml`

---

## 项目结构 / Project layout

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

---

## 构建 / Build

**中文**  
先安装 Rust 工具链，然后执行：

**English**  
Install the Rust toolchain, then run:

```powershell
cargo build --release
```

---

## GitHub Actions 构建 / GitHub Actions build

**中文**  
仓库内含工作流 `.github/workflows/windows-win32.yml`。

- 在 `windows-latest` 上构建  
- 目标为 `i686-pc-windows-msvc`（32 位 Windows 可执行文件）  
- 通过静态链接 MSVC C 运行库，尽量避免目标机器额外缺少 `vcruntime140.dll`  
- 将可直接分发的 `.zip` 作为工作流构件上传  
- 压缩包内包含 `.exe`、`config.toml`、字体文件和 `README.md`  
- 仅在最新提交信息以 `[deploy]` 开头时在 push 时自动运行  
- 仍可通过 `workflow_dispatch` 手动触发  

若在 Ubuntu 上开发但需要 Win32 发布产物，推荐使用此流程。  
如果你之前下载过旧版 zip，仍然可能会遇到 `vcruntime140.dll` 缺失；需要等新的 GitHub Actions 构建完成后，重新下载新的压缩包。

**English**  
The repo includes `.github/workflows/windows-win32.yml`.

- Builds on `windows-latest`  
- Target: `i686-pc-windows-msvc` (32-bit Windows executable)  
- Statically links the MSVC C runtime to reduce missing `vcruntime140.dll` on target PCs  
- Uploads a ready-to-share `.zip` as an artifact  
- The zip contains the `.exe`, `config.toml`, fonts, and `README.md`  
- Auto-runs on push only when the latest commit message starts with `[deploy]`  
- Can still be triggered manually via `workflow_dispatch`  

If you develop on Ubuntu but need Win32 binaries, use this workflow. Older zip downloads may still lack `vcruntime140.dll`; download a fresh artifact from a newer Actions run.

---

## 运行 / Run

```powershell
cargo run --release
```

---

## 本地启动 / Local dev startup

**中文**  
在 Ubuntu 本地开发时，直接执行：

**English**  
On Ubuntu for local development, run:

```bash
./start.sh
```

Release 模式 / Release mode:

```bash
./start.sh --release
```

**中文**  
若使用 VS Code / Cursor，也可使用调试配置。

**English**  
VS Code / Cursor debugging is supported.

- 已提供：`.vscode/launch.json`  
- 默认：`Run Plex Poster Launcher`（不依赖 CodeLLDB）  
- 停止按钮可正常结束进程  
- 可选：`Debug Plex Poster Launcher (LLDB Optional)`（若已安装 CodeLLDB）  
- 调试时使用与 `start.sh` 一致的 `XDG_CONFIG_HOME` / `XDG_CACHE_HOME`  

- Provided: `.vscode/launch.json`  
- Default: `Run Plex Poster Launcher` (no CodeLLDB required)  
- Stop button terminates the process cleanly  
- Optional: `Debug Plex Poster Launcher (LLDB Optional)` if CodeLLDB is installed  
- Debug env matches `start.sh` for `XDG_CONFIG_HOME` / `XDG_CACHE_HOME`  

**中文**  
`start.sh` 会：把配置与缓存固定到仓库目录；若配置文件不存在则生成初始配置。若仓库根目录已有 `./config.toml`，程序会优先读取。  
默认目录：配置 `./.local/config/plexposterlauncher/config.toml`，缓存 `./.local/cache/plexposterlauncher`。  
应用默认本机用户应用数据目录；`start.sh` 通过 `XDG_CONFIG_HOME` / `XDG_CACHE_HOME` 重定向到仓库内，便于开发。

**English**  
`start.sh` pins config/cache under the repo and seeds a starter config if missing. If `./config.toml` exists at the repo root, it takes precedence.  
Defaults: config `./.local/config/plexposterlauncher/config.toml`, cache `./.local/cache/plexposterlauncher`.  
The app normally uses the OS app-data locations; `start.sh` redirects via `XDG_*` for development.

**中文**  
配置读取优先级：当前工作目录 `./config.toml` 优先，否则为平台默认应用配置目录。可在 TOML 或应用界面中编辑并保存。

**English**  
Config resolution: `./config.toml` in the working directory wins; otherwise the platform default app config path. Edit via TOML or the in-app UI.

---

## 字体与标题显示 / Fonts and title rendering

**中文**  
为保证 UTF-8 文本正常显示，并让 Win32 与 Ubuntu 开发环境尽量一致，程序默认内置 CJK 字体。

- 默认使用仓库内 `DroidSansFallbackFull.ttf`  
- 也可放在仓库根目录 `./DroidSansFallbackFull.ttf`  
- 其他机器可放 `./fonts/DroidSansFallbackFull.ttf`  
- 亦支持 `./fonts/NotoSansCJKsc-Regular.otf` 或 `./fonts/NotoSansCJKjp-Regular.otf`  

启动 VLC 时会传入影片标题，播放器窗口或列表尽量显示片名而非仅 URL。

**English**  
For reliable UTF-8 and consistent CJK rendering across Win32 and Ubuntu, the app ships a default CJK font.

- Default: bundled `DroidSansFallbackFull.ttf`  
- Optional: `./DroidSansFallbackFull.ttf` at repo root  
- On other machines: `./fonts/DroidSansFallbackFull.ttf`  
- Also supported: `./fonts/NotoSansCJKsc-Regular.otf` or `./fonts/NotoSansCJKjp-Regular.otf`  

VLC is launched with the movie title so the player shows names instead of raw URLs when possible.

---

## 配置项说明 / Configuration keys

**中文**  
- `server_url`：Plex 服务器地址，例如 `http://192.168.1.100:32400`  
- `token`：Plex Token（见上文「Plex Token 获取方式」）  
- `vlc_path`：`vlc.exe`（或 Linux/macOS 上 `vlc`）的完整路径  
- `selected_library_id`：上次选中的电影库 ID  

**English**  
- `server_url`: Plex base URL, e.g. `http://192.168.1.100:32400`  
- `token`: Plex token (see **How to get your Plex token** above)  
- `vlc_path`: Full path to `vlc.exe` (or `vlc` on Linux/macOS)  
- `selected_library_id`: Last selected movie library id  

---

## 示例配置 / Example config

见 `config.example.toml` / See `config.example.toml`.

```toml
server_url = "http://192.168.1.100:32400"
token = "replace-with-your-plex-token"
vlc_path = "/snap/bin/vlc"
selected_library_id = ""
```

**中文**  
- `server_url` 填你的 Plex 地址  
- `token` 填你的 Plex Token  
- Ubuntu + snap 的 VLC 可填 `/snap/bin/vlc`  
- `selected_library_id` 首次可留空，载入库后会自动保存  

**English**  
- Set `server_url` to your Plex server  
- Set `token` to your Plex token  
- On Ubuntu with VLC from snap, `/snap/bin/vlc` is typical  
- Leave `selected_library_id` empty at first; it is filled after libraries load  

**中文**  
最简流程：`./start.sh` → 编辑 `./config.toml` 填入 `token` → 再运行 `./start.sh`。

**English**  
Quick path: run `./start.sh` → edit `./config.toml` with your `token` → run `./start.sh` again.

---

## 当前行为 / Current behavior

**中文**  
- 加载电影库列表；选库后加载第一页海报  
- 海报墙可滚动与手动分页（「加载更多」等）  
- 搜索仅过滤**已加载到内存**的条目  
- 「Play」启动外部 VLC；点击海报在右侧打开详情  

**English**  
- Loads libraries; after selection, loads the first page of posters  
- Scrollable grid with manual pagination (“load more”, etc.)  
- Search filters **only titles already loaded in memory**  
- “Play” launches external VLC; clicking a poster opens details on the right  

---

## 已知限制 / Known limitations

**中文**  
- 现阶段仅支持**电影库**  
- 播放使用条目中 Plex 暴露的**第一个**媒体分片 URL  
- 面向固定私有 Plex 环境（尤其局域网）  
- 缩略图下载失败时显示占位图  
- 若本地无法编译 Windows 版本，可通过 GitHub Actions 产出 Win32 构建  

**English**  
- **Movie libraries only** for now  
- Playback uses the **first** media part URL Plex exposes for the item  
- Intended for a private Plex setup (often LAN)  
- Failed poster downloads show a placeholder  
- Use GitHub Actions for Win32 builds if you cannot compile for Windows locally  
