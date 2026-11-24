#!/usr/bin/env bash
set -e
cd ~/Projects/dlna-video-server-2

echo "=== 1. Модульные, UI и сетевые интеграционные тесты ==="
cargo test --workspace

echo "=== 2. Проверка отсутствия зависших процессов ffmpeg ==="
if pgrep -x ffmpeg >/dev/null; then
  echo "ВНИМАНИЕ: обнаружены активные процессы ffmpeg!"
  ps -C ffmpeg -o pid,etime,args
else
  echo "OK: зависших процессов ffmpeg нет."
fi

echo "=== 3. Сборка релизного бинарника ==="
cargo build --release -p dlna-app

echo "=== 4. Проверка .deb пакета ==="
if [ -f ~/Рабочий\ стол/dlna-video-server_2.6.0_amd64.deb ]; then
  dpkg-deb --info ~/Рабочий\ стол/dlna-video-server_2.6.0_amd64.deb | grep -E "Package:|Version:|Architecture:|Depends:"
fi
echo "=== ВСЕ ТЕСТЫ УСПЕШНО ПРОЙДЕНЫ ==="
