#!/usr/bin/env sh
# Install the Realm of Echoes client (`realm` command).
# Usage: curl -fsSL https://raw.githubusercontent.com/bragibytes/space/main/scripts/install.sh | sh

set -e

REPO="bragibytes/space"
INSTALL_DIR="${REALM_INSTALL_DIR:-$HOME/.local/bin}"
VERSION="${REALM_VERSION:-latest}"

main() {
  need_cmd mkdir
  need_cmd chmod
  need_cmd curl
  need_cmd tar

  detect_target
  ensure_install_dir

  echo "→ Installing Realm of Echoes client for ${TARGET}..."

  if try_download_release; then
    echo "✓ Installed to ${INSTALL_DIR}/realm"
  elif try_cargo_install; then
    echo "✓ Installed via cargo to $(cargo home 2>/dev/null || echo ~/.cargo)/bin/realm"
    INSTALL_DIR="$(dirname "$(command -v realm)")"
  else
    echo "✗ Install failed. Open an issue: https://github.com/${REPO}/issues" >&2
    exit 1
  fi

  print_path_hint
  echo ""
  echo "  realm          # full-screen UI"
  echo "  realm --plain  # simple scrollback mode"
  echo ""
  echo "Type 'register' or 'login' when prompted. No setup required."
}

detect_target() {
  OS="$(uname -s)"
  ARCH="$(uname -m)"

  case "${OS}-${ARCH}" in
    Darwin-arm64|Darwin-aarch64)
      TARGET="aarch64-apple-darwin"
      ARCHIVE="realm-aarch64-apple-darwin.tar.gz"
      ;;
    Darwin-x86_64)
      TARGET="x86_64-apple-darwin"
      ARCHIVE="realm-x86_64-apple-darwin.tar.gz"
      ;;
    Linux-x86_64|Linux-amd64)
      TARGET="x86_64-unknown-linux-gnu"
      ARCHIVE="realm-x86_64-unknown-linux-gnu.tar.gz"
      ;;
    *)
      echo "✗ Unsupported platform: ${OS} ${ARCH}" >&2
      echo "  Build from source: cargo install --git https://github.com/${REPO} --bin realm" >&2
      exit 1
      ;;
  esac
}

ensure_install_dir() {
  mkdir -p "${INSTALL_DIR}"
}

try_download_release() {
  TAG="$(resolve_tag)" || return 1
  URL="https://github.com/${REPO}/releases/download/${TAG}/${ARCHIVE}"

  echo "  Downloading ${URL}..."
  TMP="$(mktemp -d)"
  trap 'rm -rf "$TMP"' EXIT INT HUP

  if ! curl -fsSL "${URL}" -o "${TMP}/${ARCHIVE}"; then
    return 1
  fi

  tar -xzf "${TMP}/${ARCHIVE}" -C "${TMP}"
  install -m 755 "${TMP}/realm" "${INSTALL_DIR}/realm"
  return 0
}

resolve_tag() {
  if [ "${VERSION}" != "latest" ]; then
    printf '%s' "${VERSION}"
    return 0
  fi

  TAG="$(curl -fsSL "https://api.github.com/repos/${REPO}/releases/latest" \
    | sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' \
    | head -n1)"

  if [ -z "${TAG}" ]; then
    return 1
  fi
  printf '%s' "${TAG}"
}

try_cargo_install() {
  if ! command -v cargo >/dev/null 2>&1; then
    echo "  No release found and Rust/cargo is not installed." >&2
    return 1
  fi

  echo "  No release binary found — building from source (requires Rust)..."
  cargo install --locked --git "https://github.com/${REPO}.git" --bin realm
}

print_path_hint() {
  case ":${PATH}:" in
    *":${INSTALL_DIR}:"*) ;;
    *)
      echo ""
      echo "Add to your PATH:"
      echo "  export PATH=\"${INSTALL_DIR}:\$PATH\""
      echo ""
      echo "Add that line to ~/.zshrc or ~/.bashrc to keep it permanent."
      ;;
  esac
}

need_cmd() {
  if ! command -v "$1" >/dev/null 2>&1; then
    echo "✗ Missing required command: $1" >&2
    exit 1
  fi
}

main "$@"