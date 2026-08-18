# ableton-ru-hotkeys — Summary

## Состояние

- **Текущая версия:** v1.0.3
- **Стадия:** стабильная (релизы на GitHub `devZu9/ableton-ru-hotkeys`, последний тег v1.0.1)
- **Платформа:** Windows (основная, одна машина); macOS — в разработке (заглушка `src/platform/macos.rs`)

## Позиционирование

Утилита для Windows, которая корректно обрабатывает горячие клавиши в Ableton Live при активной русской раскладке: переключает раскладку на английскую при зажатии Ctrl/Alt/Win и возвращает обратно после отпускания. Системный трей, настройки (INI), автозагрузка, сплэш-скрин.

## Ограничения и предпочтения

- Только Rust + WinAPI; без Tauri, без многопоточности
- Глобальный хук `WH_KEYBOARD_LL`, событийное отслеживание модификаторов, детект окна Ableton, детект русской раскладки (0x04190419)
- Переключение раскладки только через `SendMessageW(WM_INPUTLANGCHANGEREQUEST)`
- Shift НЕ является триггером (нужен для заглавных русских букв)
- При потере фокуса раскладка НЕ меняется (без автоматического восстановления)
- Все сообщения пользователю на русском
- Вывод консоли в UTF-8 через `SetConsoleOutputCP(65001)`
- Кросс-платформенная структура: `core.rs` (общее) + `platform/windows.rs` + `platform/macos.rs` (заглушка)
- `windows` зависимость только под `cfg(windows)`
- Иконка в трее всегда присутствует; консоль скрывается/показывается левым кликом по трею
- Правое меню трея: «О программе» (GitHub), «Настройки» (диалог), «Выход»
- Настройки: автозагрузка + запуск свёрнутым — хранятся в `%APPDATA%\AbletonRUHotkeys\settings.ini`
- .exe с метаданными (версия, описание, язык) и встроенной иконкой
- Сплэш-скрин при запуске (модальный диалог, автозакрытие через 3 секунды)
- Консоль скрыта во время сплэша — `ShowWindow(SW_HIDE)` до показа диалога

## Прогресс

### Готово

- Сплэш-скрин при запуске: модальный диалог (`IDD_SPLASH` в resource.rc) с названием, версией и описанием; закрывается через 3с (`SetTimer` → `EndDialog`)
- Консоль скрывается до сплэша (`ShowWindow(GetConsoleWindow(), SW_HIDE)`); показывается после сплэша только если «Запускать в свёрнутом виде» выключено
- Кросс-платформенное разделение: `src/core.rs` (общая логика — `ModifierState`, `vk_name()`, `is_trigger()`, `is_any_modifier()`, `ABLETON_TITLE`), `src/platform/mod.rs` (cfg селектор), `src/platform/windows.rs` (WH_KEYBOARD_LL, трей, меню, настройки, автозагрузка, сплэш), `src/platform/macos.rs` (заглушка), `src/main.rs` (тонкая точка входа)
- `src/settings.rs`: INI-настройки (`%APPDATA%\AbletonRUHotkeys\settings.ini`), по умолчанию: `AutoStart=1`, `StartMinimized=1`
- `Cargo.toml`: `windows` только под `[target.'cfg(windows)'.dependencies]`; `[build-dependencies] embed-resource = "3.0"`
- `build.rs`: `embed_resource::compile("resource/resource.rc")` на Windows
- `resource/resource.rc` (UTF-8 BOM): иконка, `IDD_SETTINGS` (автозагрузка + свёрнутый), `IDD_SPLASH`, `VS_VERSIONINFO` (v1.0.1.0, русский)
- `resource/icon.ico`: иконка (фиолетовый фон + белая «RU», 16–256 px)
- Системный трей: `STATIC`-окно с `HWND_MESSAGE`, подкласс через `SetWindowLongPtrW(GWLP_WNDPROC)`, левый клик — переключение консоли
- Меню трея: «Показать окно / Скрыть окно» (динамический текст), «О программе» → GitHub, «Настройки», «Выход»
- Диалог настроек: `DialogBoxParamW` с `BM_SETCHECK`/`BM_GETCHECK` через `GetDlgItem().unwrap()` (исправлено — ранее отправлялось на HWND диалога, а не контрола)
- Автозагрузка: `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`
- README.md на русском
- Релизы GitHub: `v0.1.0-rc1` (MVP), `v0.2.0-rc1` (реструктуризация), `v1.0.1` (стабильный: трей + настройки + сплэш + все исправления)
- Защита от второго экземпляра (named mutex `Local\AbletonRUHotkeysMutex`)
- Иконка пользователя (триколор + буква A) — v1.0.2; иконка на сплэше
- Консоль не показывается во время сплэша, по умолчанию запуск свёрнутым — v1.0.1

### В работе

- (нет)

### Заблокировано

- (нет)

## Ключевые решения

- `SendMessageW(WM_INPUTLANGCHANGEREQUEST)` вместо `AttachThreadInput` + `ActivateKeyboardLayout` — первый не работал для RU→EN, второй работает в обе стороны
- Событийное отслеживание модификаторов (`CTRL_HELD/ALT_HELD/WIN_HELD`) вместо `GetAsyncKeyState` — последний возвращал устаревшие значения внутри хука
- `RESTORE_HWND` сохраняется при переключении на EN, чтобы возвращать раскладку правильному окну в многопоточном UI Ableton
- `SetWindowLongPtrW(GWLP_WNDPROC)` для подкласса `STATIC`-окна вместо `RegisterClassW` (нет в выбранных фичах `windows` v0.58)
- `SendMessageW(BM_SETCHECK/BM_GETCHECK)` через `GetDlgItem` вместо `CheckDlgButton`/`IsDlgButtonChecked` (тоже нет в фичах)
- `embed-resource` v3.0 для встраивания версии и иконки
- Настройки из реестра перенесены в INI-файл — проще редактировать и бэкапить
- Сплэш через `DialogBoxParamW` (модальный, блокирует на 3с) — самый простой способ, не требует регистрации класса окна
- Консоль скрывается до сплэша вызовом `ShowWindow(SW_HIDE)` в самом начале `run()`

## Файлы проекта

- `src\core.rs` — общая логика
- `src\settings.rs` — INI-настройки
- `src\platform\windows.rs` — полная реализация Windows
- `src\platform\macos.rs` — заглушка macOS
- `src\platform\mod.rs` — cfg-селектор
- `src\main.rs` — точка входа
- `resource\resource.rc` — ресурсы (диалоги, версия, иконка)
- `resource\icon.ico` — иконка приложения
- `build.rs` — сборка ресурсов
- `Cargo.toml` — зависимости
- `https://github.com/devZu9/ableton-ru-hotkeys` — репозиторий

## Правила чтения журналов

- **SESSIONS.md** — только заголовки сессий + тело текущей сессии.
- **ROADMAP.md** — только насущные `[ ]` текущей версии.
- **CHANGELOG.md** — НЕ читать (для пользователя).
- **Аудит:** `C:\_dev\_for_OpenCode\audits\ableton-ru-hotkeys.md`