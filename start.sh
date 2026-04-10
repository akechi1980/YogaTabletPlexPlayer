#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

if ! command -v cargo >/dev/null 2>&1; then
    echo "cargo was not found. Install Rust first: https://rustup.rs" >&2
    exit 1
fi

export XDG_CONFIG_HOME="${XDG_CONFIG_HOME:-$SCRIPT_DIR/.local/config}"
export XDG_CACHE_HOME="${XDG_CACHE_HOME:-$SCRIPT_DIR/.local/cache}"

APP_DIR_NAME="plexposterlauncher"
CONFIG_DIR="$XDG_CONFIG_HOME/$APP_DIR_NAME"
CACHE_DIR="$XDG_CACHE_HOME/$APP_DIR_NAME"
CONFIG_FILE="$CONFIG_DIR/config.toml"

mkdir -p "$CONFIG_DIR" "$CACHE_DIR"

DEFAULT_VLC_PATH="${VLC_PATH:-}"
if [[ -z "$DEFAULT_VLC_PATH" ]]; then
    DEFAULT_VLC_PATH="$(command -v vlc || true)"
fi
if [[ -z "$DEFAULT_VLC_PATH" ]]; then
    DEFAULT_VLC_PATH="/snap/bin/vlc"
fi

if [[ ! -f "$CONFIG_FILE" ]]; then
    cat >"$CONFIG_FILE" <<EOF
# Plex 服务器地址
server_url = "http://192.168.1.100:32400"
# 替换成你自己的 Plex Token
token = "replace-with-your-plex-token"
# 本机 VLC 路径，Ubuntu + snap 默认通常是 /snap/bin/vlc
vlc_path = "$DEFAULT_VLC_PATH"
# 首次运行可留空，应用载入电影库后会自动保存
selected_library_id = ""
EOF
    echo "Created initial config: $CONFIG_FILE"
fi

echo "Config file: $CONFIG_FILE"
echo "Cache dir:   $CACHE_DIR"

if [[ "${1:-}" == "--release" ]]; then
    shift
    exec cargo run --release "$@"
fi

exec cargo run "$@"
