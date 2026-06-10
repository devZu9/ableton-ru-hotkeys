# Ableton RU Hotkeys

**Версия:** v1.0.3

Утилита для корректной работы горячих клавиш в **Ableton Live** при русской раскладке клавиатуры.

## Проблема

В Ableton Live горячие клавиши перестают работать, когда выбрана русская раскладка. Например:
- `Ctrl+Ы` не воспринимается как `Ctrl+S`
- `Ctrl+Ш` не воспринимается как `Ctrl+I`
- `Ctrl+Shift+Ь` не воспринимается как `Ctrl+Shift+M`

## Решение

Утилита отслеживает нажатия клавиш через низкоуровневый хук (`WH_KEYBOARD_LL`) и при обнаружении зажатого `Ctrl`/`Alt`/`Win` с русской раскладкой отправляет окну Ableton Live сообщение `WM_INPUTLANGCHANGEREQUEST` для переключения раскладки на английскую.

После отпускания всех модификаторов раскладка возвращается на русскую.

## Как это работает

1. Запустите `ableton-ru-hotkeys.exe`
2. Переключитесь в Ableton Live
3. Зажмите `Ctrl` (или `Alt`/`Win`) — раскладка автоматически переключится на EN
4. Нажмите нужную клавишу — Ableton видит английское сочетание
5. Отпустите все модификаторы — раскладка вернётся на RU

**Важно:** `Shift` не является триггером, поэтому заглавные русские буквы работают как обычно.

## Требования

- Windows 10/11
- Ableton Live 10/11/12
- Установленная русская и английская раскладки клавиатуры

## Архитектура

Проект имеет кросс-платформенную структуру:

```
src/
├── main.rs           # точка входа
├── core.rs           # ядро: общая логика, маппинг клавиш
└── platform/
    ├── mod.rs        # селектор платформы (cfg)
    ├── windows.rs    # Windows: WH_KEYBOARD_LL, SendMessageW
    └── macos.rs      # macOS: заглушка (в разработке)
```

Сборка под нужную платформу происходит автоматически:
- **Windows:** `cargo build --release` → `target/release/ableton-ru-hotkeys.exe`
- **macOS:** `cargo build --release` → `target/release/ableton-ru-hotkeys`

## Технологии

- **Язык:** Rust (edition 2024)
- **WinAPI:** `SetWindowsHookExW` (WH_KEYBOARD_LL), `SendMessageW` (WM_INPUTLANGCHANGEREQUEST), `GetKeyboardLayout`, `GetForegroundWindow`
- **Крейты:** `windows` 0.58 (только Windows)

Сделано методом **вайб-кодинга** в оболочке [OpenCode](https://opencode.ai/go?ref=DHSKBMGTK0) на модели `opencode/deepseek-v4-flash-free`.

## Установка

Скачайте `ableton-ru-hotkeys.exe` из [релизов](https://github.com/devZu9/ableton-ru-hotkeys/releases) и запустите. Никакой установки не требуется.

## Сборка из исходников

```bash
git clone https://github.com/devZu9/ableton-ru-hotkeys.git
cd ableton-ru-hotkeys
cargo build --release
./target/release/ableton-ru-hotkeys.exe
```

## Лицензия

MIT
