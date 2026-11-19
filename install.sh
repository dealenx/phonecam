#!/usr/bin/env bash
# Установка phonecam одной строкой:
#   curl -fsSL https://raw.githubusercontent.com/dealenx/phonecam/master/install.sh | bash
# Ставит в ~/.local/bin (без sudo): скрипт phonecam + GUI из релиза.
set -euo pipefail
BIN="$HOME/.local/bin"
REPO="dealenx/phonecam"

mkdir -p "$BIN"

# 1) скрипт — прямо из raw
curl -fsSL "https://raw.githubusercontent.com/$REPO/master/phonecam" -o "$BIN/phonecam"
chmod +x "$BIN/phonecam"

# 2) GUI — свежий бинарь из последнего релиза (нужен grep/sed/tar/sha256sum)
url="$(curl -fsSL "https://api.github.com/repos/$REPO/releases/latest" \
  | grep -o '"browser_download_url": *"[^"]*\.tar\.gz"' | head -1 | cut -d'"' -f4)"
[ -n "$url" ] || { echo "релиз не найден: https://github.com/$REPO/releases"; exit 1; }
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl -fsSL "$url" -o "$tmp/r.tgz"
tar -xzf "$tmp/r.tgz" -C "$tmp"
install -m755 "$tmp/phonecam/phonecam-gui" "$BIN/phonecam-gui"

echo "Установлено в $BIN:"
echo "  phonecam on|vcam|off|status   — камера из терминала"
echo "  phonecam-gui                  — окно с кнопками"
command -v scrcpy >/dev/null || echo "Внимание: не установлен scrcpy — поставь пакет scrcpy (см. README)"
command -v adb >/dev/null || echo "Внимание: не установлен adb — поставь android-tools"
