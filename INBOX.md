# INBOX - почта между сессиями

Очередь сообщений от агентов других сессий к агенту этой папки. Разбирается при /new-session (шаг 1а) и /mail. Полный регламент: скилл skill-inbox (монополия, типы записей, цикл обработки, квитанции).

## Формат записи (новая - СВЕРХУ)

```markdown
## ГГГГ-ММ-ДД ЧЧ:ММ - [тип]

- **От:** сессия <проект>
- **Тема:** <кратко>
- **Суть:** <что изменилось / шаги / опыт>
```

## 2026-08-19 01:25 — [инструкция]

- **От:** сессия 1 ableton-ru-hotkeys (Windows)
- **Платформа:** [Mac]
- **Тема:** Проверить macOS-поддержку v1.1.0 (сборка и работа)
- **Суть:** В проекте реализована macOS-поддержка (версия 1.1.0, коммит 2db7c7e — на Windows-машине запушен). Windows-агент не может собрать под macOS (нет toolchain) — проверка за macOS-агентом.

Что реализовано:
- src/platform/macos.rs: CGEventTap (kCGEventKeyDown=10, kCGEventKeyUp=11, kCGEventFlagsChanged=12) + TIS; FFI на фреймворки ApplicationServices/CoreFoundation/Carbon (#[link(kind="framework")]) БЕЗ новых крейтов.
- Логика как в windows.rs: Ableton в фокусе (CGWindowListCopyWindowInfo → kCGWindowOwnerName содержит «Ableton») + русская раскладка (TISPropertyInputSourceID содержит «russian») + зажат мод (Cmd/Control/Alternate из флагов события) → переключить на EN (TISSelectInputSource, EN = «.abc»/«.us»); все моды отпущены → вернуть RU (сохранённый исходный источник); потеря фокуса → вернуть RU; Shift НЕ триггер.
- В run(): подсказка про разрешение Accessibility (AXIsProcessTrusted).
- src/settings.rs: config_dir для macOS — ~/Library/Application Support/AbletonRUHotkeys.
- Cargo.toml: build-dependencies embed-resource перенесены в [target.'cfg(windows)'.build-dependencies]; build.rs гейтится #[cfg(windows)] — на macOS embed-resource не используется.

Задачи для macOS-агента:
1. git pull (доставка через git) и сборка: cargo build --release — убедиться, что macos.rs компилируется БЕЗ ошибок и предупреждений (проект: компиляция без предупреждений обязательна).
2. Выдать разрешение Accessibility терминалу (System Settings → Privacy & Security → Accessibility) и проверить работу с Ableton + русской раскладкой: зажат мод → EN, отпущен → RU, потеря фокуса → RU, Shift не триггер.
3. Если всё ок: обновить статус аудита audits/ableton-ru-hotkeys.md задание 6 (macOS: [x]) и при необходимости журналы/README.
4. Прислать квитанцию [подтверждение] в INBOX.md этого проекта (адресовано сессии 1 ableton-ru-hotkeys Windows): что собрано/проверено, версия, дата.

Замечание: на macOS keycode событий физические (в отличие от Windows VK) — переключение по модам (флаги события), не по keycode, поэтому vk_name/is_trigger из core.rs на macOS не используются.

- **Коммит:** не нужен (инструкция; доставка — пуш отправителя)
