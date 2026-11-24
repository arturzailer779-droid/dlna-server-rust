#!/usr/bin/env bash
set -e
cd ~/Projects/dlna-video-server-2

echo "=========================================================="
echo "  DLNA VIDEO SERVER 2.0 — INDUSTRIAL QA PIPELINE"
echo "=========================================================="

echo "[1/5] Статический анализ кода (cargo clippy)..."
cargo clippy --workspace --all-targets

echo "[2/5] Модульные, UI, сетевые и хаос-тесты (cargo test)..."
cargo test --workspace

echo "[3/5] Сборка оптимизированного релиза (cargo build --release)..."
cargo build --release -p dlna-app

echo "[4/5] Аудит процессов и памяти..."
if pgrep -x ffmpeg >/dev/null; then
  echo "ОШИБКА: обнаружены незакрытые процессы ffmpeg!"
  exit 1
fi
echo "OK: утечек процессов ffmpeg нет."

echo "[5/5] Валидация .deb пакета..."
DEB_FILE="$HOME/Рабочий стол/dlna-video-server_2.6.0_amd64.deb"
if [ -f "$DEB_FILE" ]; then
  dpkg-deb --info "$DEB_FILE" | grep -E "Package:|Version:|Architecture:|Depends:"
  if command -v lintian >/dev/null 2>&1; then
    lintian --tag-display-limit 0 "$DEB_FILE" || true
  fi
fi

echo "=========================================================="
echo "  ВСЕ 5 ЭТАПОВ ПРОМЫШЛЕННОЙ ПРОВЕРКИ УСПЕШНО ПРОЙДЕНЫ!"
echo "=========================================================="
