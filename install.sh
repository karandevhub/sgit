#!/bin/bash
# sgit installer — https://github.com/karandevhub/sgit

set -euo pipefail

REPO="karandevhub/sgit"
BINARY_NAME="sgit"
RELEASES_URL="https://github.com/$REPO/releases"
DOWNLOAD_DIR="${TMPDIR:-/tmp}/sgit-install-$$"
TARGET="${1:-latest}"

if [ -t 1 ]; then
    RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
    CYAN='\033[0;36m'; BOLD='\033[1m'; RESET='\033[0m'
else
    RED=''; GREEN=''; YELLOW=''; CYAN=''; BOLD=''; RESET=''
fi

info()  { printf "  ${CYAN}→${RESET}  %s\n" "$*"; }
ok()    { printf "  ${GREEN}✓${RESET}  %s\n" "$*"; }
warn()  { printf "  ${YELLOW}!${RESET}  %s\n" "$*"; }
die()   { printf "  ${RED}✗${RESET}  %s\n" "$*" >&2; exit 1; }

if [[ -n "$TARGET" ]] && \
   [[ ! "$TARGET" =~ ^(latest|[0-9]+\.[0-9]+\.[0-9]+(-[^[:space:]]+)?)$ ]]; then
    die "Usage: install.sh [latest|VERSION]   e.g. install.sh 0.2.0"
fi

DOWNLOADER=""
if command -v curl >/dev/null 2>&1; then
    DOWNLOADER="curl"
elif command -v wget >/dev/null 2>&1; then
    DOWNLOADER="wget"
else
    die "Either curl or wget is required but neither was found."
fi

download_file() {
    local url="$1" out="${2:-}"
    if [ "$DOWNLOADER" = "curl" ]; then
        if [ -n "$out" ]; then curl -fsSL -o "$out" "$url"
        else                 curl -fsSL       "$url"; fi
    else
        if [ -n "$out" ]; then wget -q -O "$out" "$url"
        else                 wget -q -O -   "$url"; fi
    fi
}

case "$(uname -s)" in
    Darwin)             os="darwin" ;;
    Linux)              os="linux"  ;;
    MINGW*|MSYS*|CYGWIN*)
        warn "Detected Windows. Running PowerShell installer..."
        powershell -NoProfile -ExecutionPolicy Bypass \
            -Command "iex (irm 'https://raw.githubusercontent.com/$REPO/main/install.ps1')"
        exit 0
        ;;
    *) die "Unsupported OS: $(uname -s)" ;;
esac

case "$(uname -m)" in
    x86_64|amd64)      arch="x86_64"  ;;
    arm64|aarch64)     arch="aarch64" ;;
    *) die "Unsupported architecture: $(uname -m)" ;;
esac

if [ "$os" = "darwin" ] && [ "$arch" = "x86_64" ]; then
    if [ "$(sysctl -n sysctl.proc_translated 2>/dev/null)" = "1" ]; then
        arch="aarch64"
        info "Rosetta detected — using arm64 binary"
    fi
fi

platform="${os}-${arch}"
info "Platform: $platform"

resolve_latest() {
    if [ "$DOWNLOADER" = "curl" ]; then
        curl -fsSI "$RELEASES_URL/latest" \
          | awk -F'/' 'tolower($1) ~ /^location:/ { sub(/\r$/,"",$NF); print $NF }' \
          | sed 's/^v//'
    else
        wget -q --server-response --spider "$RELEASES_URL/latest" 2>&1 \
          | awk -F'/' '/Location:/ { sub(/\r$/,"",$NF); print $NF }' \
          | sed 's/^v//'
    fi
}

if [ "$TARGET" = "latest" ]; then
    info "Resolving latest version..."
    VERSION="$(resolve_latest)"
else
    VERSION="$TARGET"
fi
[ -n "$VERSION" ] || die "Could not resolve version from $RELEASES_URL/latest"
info "Version: v$VERSION"

BASE_URL="$RELEASES_URL/download/v$VERSION"
ARCHIVE_NAME="${BINARY_NAME}-${platform}.tar.gz"
CHECKSUM_NAME="${ARCHIVE_NAME}.sha256"

mkdir -p "$DOWNLOAD_DIR"
trap 'rm -rf "$DOWNLOAD_DIR"' EXIT

info "Downloading $ARCHIVE_NAME..."
download_file "$BASE_URL/$ARCHIVE_NAME"  "$DOWNLOAD_DIR/$ARCHIVE_NAME" \
  || die "Download failed. Check https://github.com/$REPO/releases for available assets."

info "Verifying checksum..."
download_file "$BASE_URL/$CHECKSUM_NAME" "$DOWNLOAD_DIR/$CHECKSUM_NAME" \
  || die "Could not download checksum file."

EXPECTED="$(awk '{print $1}' "$DOWNLOAD_DIR/$CHECKSUM_NAME")"

if command -v shasum >/dev/null 2>&1; then
    ACTUAL="$(shasum -a 256 "$DOWNLOAD_DIR/$ARCHIVE_NAME" | awk '{print $1}')"
else
    ACTUAL="$(sha256sum "$DOWNLOAD_DIR/$ARCHIVE_NAME" | awk '{print $1}')"
fi

[ "$ACTUAL" = "$EXPECTED" ] || die "Checksum mismatch! Expected=$EXPECTED Got=$ACTUAL"
ok "Checksum verified"

tar -xzf "$DOWNLOAD_DIR/$ARCHIVE_NAME" -C "$DOWNLOAD_DIR"
BINARY_PATH="$DOWNLOAD_DIR/$BINARY_NAME"
chmod +x "$BINARY_PATH"

INSTALL_DIR=""

if [ -w "/opt/homebrew/bin" ]; then
    INSTALL_DIR="/opt/homebrew/bin"
elif [ -w "/usr/local/bin" ]; then
    INSTALL_DIR="/usr/local/bin"
elif command -v sudo >/dev/null 2>&1; then
    INSTALL_DIR="/usr/local/bin"
    info "Requesting sudo to install to $INSTALL_DIR (so it works without restarting terminal)"
    sudo mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
    INSTALL_DIR=""  
elif mkdir -p "$HOME/.local/bin" 2>/dev/null; then
    INSTALL_DIR="$HOME/.local/bin"
else
    die "Could not find a suitable installation directory."
fi

if [ -n "$INSTALL_DIR" ]; then
    mv "$BINARY_PATH" "$INSTALL_DIR/$BINARY_NAME"
fi

if [ -n "$INSTALL_DIR" ] && ! echo "$PATH" | grep -q "$INSTALL_DIR"; then
    SHELL_PROFILE=""
    case "$(basename "${SHELL:-sh}")" in
        zsh)  SHELL_PROFILE="$HOME/.zshrc" ;;
        bash)
            if [ -f "$HOME/.bash_profile" ]; then
                SHELL_PROFILE="$HOME/.bash_profile"
            else
                SHELL_PROFILE="$HOME/.bashrc"
            fi
            ;;
        *)    SHELL_PROFILE="$HOME/.profile" ;;
    esac

    PATH_LINE='export PATH="$HOME/.local/bin:$PATH"'

    if ! grep -qF "$PATH_LINE" "$SHELL_PROFILE" 2>/dev/null; then
        echo "" >> "$SHELL_PROFILE"
        echo "# Added by sgit installer" >> "$SHELL_PROFILE"
        echo "$PATH_LINE" >> "$SHELL_PROFILE"
    fi

    warn "Run 'source $SHELL_PROFILE' or open a new terminal for sgit to be available."
fi

echo ""
printf "  ${GREEN}${BOLD}✅  sgit v$VERSION installed!${RESET}\n"
echo ""
echo "  Quick start:"
printf "    ${CYAN}sgit index${RESET}                    # build the search index\n"
printf "    ${CYAN}sgit log \"auth bug\"${RESET}           # semantic search\n"
printf "    ${CYAN}sgit log --help${RESET}               # all options\n"
echo ""

curl -s "https://hits.seeyoufarm.com/api/count/incr/badge.svg?url=https%3A%2F%2Fgithub.com%2Fkarandevhub%2Fsgit%2Fdownload&count_bg=%230099CC&title_bg=%23555555&title=downloads&edge_flat=false" > /dev/null 2>&1 || true
