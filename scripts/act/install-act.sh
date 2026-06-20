#!/usr/bin/env bash
# Install act (https://github.com/nektos/act) for local GitHub Actions verification.
# Pattern from jules-skills/local-action-verification (sovereign, no cloud).
set -euo pipefail

INSTALL_DIR="${HOME}/.local/bin"

echo "Installing act..."

ARCH=$(uname -m)
case "$ARCH" in
  x86_64)  ARCH="x86_64" ;;
  aarch64|arm64) ARCH="arm64" ;;
  *)
    echo "Unsupported architecture: $ARCH"
    exit 1
    ;;
esac

mkdir -p "$INSTALL_DIR"

if command -v sudo &>/dev/null && sudo -n true 2>/dev/null; then
  echo "  Installing to /usr/local/bin (system-wide)..."
  curl -sL https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash -s -- -b /usr/local/bin
else
  echo "  Installing to ${INSTALL_DIR} (user-local)..."
  curl -sL https://raw.githubusercontent.com/nektos/act/master/install.sh | bash -s -- -b "$INSTALL_DIR"
  if [[ ":$PATH:" != *":${INSTALL_DIR}:"* ]]; then
    echo "  ${INSTALL_DIR} is not on PATH — add: export PATH=\"${INSTALL_DIR}:\$PATH\""
    export PATH="${INSTALL_DIR}:$PATH"
  fi
fi

if command -v act &>/dev/null; then
  echo "act installed: $(act --version)"
else
  echo "Installation failed — act not found on PATH."
  exit 1
fi
