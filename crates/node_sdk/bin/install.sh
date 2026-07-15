#!/usr/bin/env bash
set -euo pipefail

REPO="MasonChow/source-map-parser"
PACKAGE_NAME="source_map_parser_node"
VERSION="latest"
INSTALL_DIR="${SOURCE_MAP_PARSER_INSTALL_DIR:-$HOME/.source-map-parser}"
BIN_DIR="${SOURCE_MAP_PARSER_BIN_DIR:-$HOME/.local/bin}"

usage() {
  cat <<USAGE
Install source_map_parser_node from GitHub release artifacts.

Usage:
  bash ./node_modules/source_map_parser_node/bin/install.sh
  bash ./node_modules/source_map_parser_node/bin/install.sh --version v0.4.0 --install-dir ~/.source-map-parser --bin-dir ~/.local/bin

Options:
  --version <tag|latest>   Release tag to install. Defaults to latest.
  --install-dir <dir>      Package installation directory. Defaults to ~/.source-map-parser.
  --bin-dir <dir>          Directory for the source-map-parser shim. Defaults to ~/.local/bin.
  -h, --help               Show this help.
USAGE
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --version) VERSION="${2:?missing value for --version}"; shift 2 ;;
    --install-dir) INSTALL_DIR="${2:?missing value for --install-dir}"; shift 2 ;;
    --bin-dir) BIN_DIR="${2:?missing value for --bin-dir}"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "Unknown argument: $1" >&2; usage; exit 1 ;;
  esac
done

need() { command -v "$1" >/dev/null 2>&1 || { echo "Missing required command: $1" >&2; exit 1; }; }
need curl
need tar
need node

case "$(uname -s)" in
  Linux*) OS="linux" ;;
  Darwin*) OS="macos" ;;
  MINGW*|MSYS*|CYGWIN*) OS="windows" ;;
  *) echo "Unsupported OS: $(uname -s)" >&2; exit 1 ;;
esac
case "$(uname -m)" in
  x86_64|amd64) ARCH="x64" ;;
  arm64|aarch64) ARCH="arm64" ;;
  *) ARCH="$(uname -m)" ;;
esac

if [[ "$VERSION" == "latest" ]]; then
  API_URL="https://api.github.com/repos/${REPO}/releases/latest"
else
  API_URL="https://api.github.com/repos/${REPO}/releases/tags/${VERSION}"
fi

ASSET="${PACKAGE_NAME}-${OS}-${ARCH}.tar.gz"
DOWNLOAD_URL="$(curl -fsSL "$API_URL" | ASSET="$ASSET" node -e '
let data = "";
process.stdin.on("data", (chunk) => { data += chunk; });
process.stdin.on("end", () => {
  const release = JSON.parse(data);
  const asset = (release.assets || []).find((item) => item.name === process.env.ASSET);
  if (!asset) process.exit(2);
  console.log(asset.browser_download_url);
});
')" || {
  echo "Could not find release asset ${ASSET}." >&2
  exit 1
}
CHECKSUM_URL="${DOWNLOAD_URL}.sha256"

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT
curl -fL "$DOWNLOAD_URL" -o "$TMP_DIR/$ASSET"
if curl -fsL "$CHECKSUM_URL" -o "$TMP_DIR/$ASSET.sha256"; then
  if command -v sha256sum >/dev/null 2>&1; then
    (cd "$TMP_DIR" && sha256sum -c "$ASSET.sha256")
  else
    (cd "$TMP_DIR" && shasum -a 256 -c "$ASSET.sha256")
  fi
else
  echo "Checksum file not found; continuing without checksum verification." >&2
fi
mkdir -p "$INSTALL_DIR" "$BIN_DIR"
tar -xzf "$TMP_DIR/$ASSET" -C "$INSTALL_DIR" --strip-components=1
cat > "$BIN_DIR/source-map-parser" <<SHIM
#!/usr/bin/env bash
exec node "$INSTALL_DIR/bin/source-map-parser.mjs" "\$@"
SHIM
chmod +x "$BIN_DIR/source-map-parser"
if [[ "$OS" == "windows" ]]; then
  cat > "$BIN_DIR/source-map-parser.cmd" <<SHIM
@echo off
node "$INSTALL_DIR/bin/source-map-parser.mjs" %*
SHIM
fi
echo "Installed source-map-parser to $BIN_DIR/source-map-parser"
echo "If needed, add $BIN_DIR to PATH."
