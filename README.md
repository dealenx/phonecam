# phonecam

Камера Android-телефона на Linux-ПК для OBS: либо 4K-окно (GPU-декод),
либо виртуальная камера `/dev/video0`. Без приложений на телефоне — только scrcpy + adb.

## Зависимости

```bash
sudo pacman -S scrcpy android-tools v4l-utils v4l2loopback-dkms   # Arch/CachyOS
sudo apt install scrcpy adb v4l-utils v4l2loopback-dkms           # Ubuntu/Debian
```

Для сборки GUI дополнительно нужен Rust: <https://rustup.rs>.

Модуль loopback с именем камеры (для `phonecam vcam`; `phonecam on` его не требует):

```bash
sudo tee /etc/modprobe.d/plugcam.conf <<'EOF2'
options v4l2loopback exclusive_caps=1 devices=1 card_label="scrcpy-camera"
EOF2
echo v4l2loopback | sudo tee /etc/modules-load.d/plugcam.conf
sudo modprobe v4l2loopback
```

Несколько устройств — через запятую в `card_label`; у нас добавлены слоты под Iriun.

Телефон: *Настройки → О телефоне → 7 раз по номеру сборки* → *Для разработчиков →
Отладка по USB*; режим USB — «Передача файлов».

## Быстрая проверка

```bash
phonecam status   # телефон найден? камера выключена? — значит всё стоит
phonecam vcam     # виртуалка должна подняться за ~2 с
```

## Скрипт

```bash
install -Dm755 phonecam ~/.local/bin/phonecam   # из каталога репо
phonecam on      # 4K-окно IN2023 (GPU-декод) — в OBS через Window Capture
phonecam vcam    # виртуалка scrcpy-camera 1080p30 — в приложениях сразу
phonecam off     # выключить
phonecam status  # что запущено + состояние телефона
```

## GUI (GPUI)

```bash
cargo build --release        # первая сборка ~5–10 минут (грузится дерево gpui)
./target/release/phonecam-gui
```

Кнопки дёргают тот же `phonecam`; настройки (разрешение, битрейт, FPS, камера,
фонарик, кодек) пишутся в `~/.config/phonecam/settings` и переживают закрытие GUI —
читает их и скрипт, и автозапуск.

## Ограничения (проверено)

- **4K realtime** — только в оконном режиме (`--hwdec=vaapi`).
  В режиме v4l2 аппаратный декод запрещён самим scrcpy
  (`ERROR: Hardware decoding is not compatible with V4L2`), софт-декод 4K
  отстаёт и даёт ~1 с задержки. Поэтому `vcam` ограничен 1080p.
- Виртуалка видна в OBS как V4L2-устройство только при запущенном `phonecam vcam`
  (`exclusive_caps=1`).
- Аудио с телефона не настраивалось.
