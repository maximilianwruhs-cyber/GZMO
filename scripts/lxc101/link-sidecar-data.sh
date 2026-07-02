#!/usr/bin/env bash
# Ensure canonical state lives under /opt/gzmo/ and survey_GZMO uses symlinks.
# Safe to re-run (idempotent).
set -euo pipefail

GZMO_ROOT="${GZMO_ROOT:-/opt/gzmo}"
REPO="${GZMO_ROOT}/survey_GZMO"

mkdir -p "${GZMO_ROOT}/data" "${GZMO_ROOT}/memory" "${GZMO_ROOT}/skills" "${GZMO_ROOT}/wiki"
mkdir -p /home/maximilian/knowledge

link_dir() {
  local name="$1"
  local target="${GZMO_ROOT}/${name}"
  local link="${REPO}/${name}"
  if [[ -L "${link}" ]]; then
    local current
    current="$(readlink -f "${link}")"
    if [[ "${current}" == "${target}" ]]; then
      return 0
    fi
    rm -f "${link}"
  elif [[ -d "${link}" ]]; then
    if [[ -z "$(ls -A "${link}" 2>/dev/null)" ]]; then
      rmdir "${link}"
    else
      echo "[*] Merging ${link} -> ${target}"
      rsync -a "${link}/" "${target}/"
      rm -rf "${link}"
    fi
  elif [[ -e "${link}" ]]; then
    rm -rf "${link}"
  fi
  ln -sfn "${target}" "${link}"
}

link_file() {
  local name="$1"
  local target="${GZMO_ROOT}/${name}"
  local link="${REPO}/${name}"
  if [[ -L "${link}" ]]; then
    local current
    current="$(readlink -f "${link}")"
    if [[ "${current}" == "${target}" ]]; then
      return 0
    fi
    rm -f "${link}"
  elif [[ -f "${link}" ]]; then
    if [[ ! -f "${target}" ]] || [[ "${link}" -nt "${target}" ]]; then
      cp -a "${link}" "${target}"
    fi
    rm -f "${link}"
  fi
  ln -sfn "${target}" "${link}"
}

link_dir data
link_dir memory
link_dir skills
link_dir wiki
link_file SOUL.md
link_file DREAMS.md

if [[ -f "${GZMO_ROOT}/gzmo.toml" ]]; then
  cp -a "${GZMO_ROOT}/gzmo.toml" "${REPO}/gzmo.toml"
fi

echo "[OK] Sidecar data links:"
ls -la "${REPO}/data" "${REPO}/memory" "${REPO}/skills" "${REPO}/wiki" \
  "${REPO}/SOUL.md" "${REPO}/DREAMS.md"
