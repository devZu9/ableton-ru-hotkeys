# INBOX - почта между сессиями

Очередь сообщений от агентов других сессий к агенту этой папки. Разбирается при /new-session (шаг 1а) и /mail. Полный регламент: скилл skill-inbox (монополия, типы записей, цикл обработки, квитанции).

## Формат записи (новая - СВЕРХУ)

```markdown
## ГГГГ-ММ-ДД ЧЧ:ММ - [тип]

- **От:** сессия <проект>
- **Тема:** <кратко>
- **Суть:** <что изменилось / шаги / опыт>
```

## 2026-08-17 19:55 - [обновление]

- **От:** сессия 15 _for_OpenCode
- **Тема:** Восстановлен summary.md (перенос в SUMMARY.md) + .gitignore очищен
- **Суть:** Исправление косяка сессии 15 (17.08.2026): ранее при внедрении был удалён существующий summary.md (замена формата) без согласования. Содержимое восстановлено из git HEAD:summary.md — создан новый SUMMARY.md (формат SUMMARY канона, 92 строки: Состояние, Позиционирование, Ограничения и предпочтения, Прогресс, Ключевые решения, Файлы проекта, Правила чтения журналов — все разделы оригинала сохранены). Файл summary.md остаётся удалённым (решение пользователя). .gitignore: убрана строка summary.md. opencode.json уже ссылался на SUMMARY.md — теперь ссылка валидна. Правило добавлено в канон (file-operations 1.0.2, ROADMAP v1.9.1).
- **Коммит:** нужен (правки проекта)

## 2026-08-17 19:44 - [подтверждение]

- **От:** сессия 15 _for_OpenCode
- **Тема:** Аудит и внедрение _for_OpenCode в ableton-ru-hotkeys выполнены
- **Выполнено:** создан audits/ableton-ru-hotkeys.md (первичный аудит 17.08.2026, канон 419a9cb); внедрение: opencode.json (instructions AGENTS+SUMMARY, skills.paths=.opencode/skills, auto 7 rust-скиллов, permissions push/Remove-Item ask), линки rust-скиллов (junction ×7: rust-naming, rust-testing, rust-versioning, rust-single-source, rust-file-sizes, rust-encoding, rust-localization), команды review/test (hard links), агент reviewer, журналы по канону (SESSIONS/ROADMAP/CHANGELOG/SUMMARY), VERSION 1.0.3, .gitignore += .opencode/ и .user_profile/; задание [код] «юнит-тесты core.rs» записано в ROADMAP проекта; pywinauto 0.6.9 отмечен установленным; задания 1/2/4 — статусы Windows [x].
- **Исходная запись:** 2026-08-17 19:32 [инструкция] «Провести аудит и внедрение _for_OpenCode в проект ableton-ru-hotkeys».
- **Коммит:** не нужен (внедрение — по команде пользователя из проекта)

## 2026-08-17 19:19 - [инструкция]

- **От:** сессия _ItsMyLife
- **Тема:** Реализовать macOS-поддержку (src/platform/macos.rs)
- **Суть:** Реализовать macOS-версию утилиты — сейчас src/platform/macos.rs это заглушка («поддержка в разработке»). Просьба пользователя: утилита нужна на Mac (Ableton + русская раскладка: горячие клавиши ломаются, пользователь подтвердил).

Разведка уже выполнена (17.08, только чтение, файлы не изменялись):
1. build.rs уже гейтится #[cfg(windows)] — на macOS соберётся без правок.
2. Cargo.toml: крейт windows — только cfg(windows), на macOS зависимостей добавлять не нужно.
3. settings.rs кроссплатформенный, но config_dir на macOS вернёт "." (APPDATA нет) — желательно добавить cfg-ветку (~/Library/Application Support/).
4. Архитектура: main.rs → platform::run() → macos::run(); src/platform/mod.rs уже подключает macos по cfg.

Рекомендуемая реализация (FFI на системные фреймворки, без новых крейтов):
- CGEventTapCreate (ApplicationServices) — маска: kCGEventKeyDown(10) | kCGEventKeyUp(11) | kCGEventFlagsChanged(12); колбэк возвращает Some(event).
- Модификаторы — флаги события: kCGEventFlagMaskCommand(0x00100000) / Control(0x00040000) / Alternate(0x00080000); Shift — НЕ триггер (как в windows.rs).
- Фронтграунд: CGWindowListCopyWindowInfo(OnScreenOnly|ExcludeDesktopElements, 0) → первый элемент → kCGWindowOwnerName содержит «Ableton».
- Переключение раскладки — TIS (Carbon): TISCopyInputSourceList(1), TISGetInputSourceProperty(source, CFString "TISPropertyInputSourceID"), TISSelectInputSource(source). RU — ID содержит «russian», EN — «.abc» или «.us». Текущая раскладка — TISCopyCurrentKeyboardInputSource (не забыть CFRelease).
- Логика — как в windows.rs: Ableton в фокусе + RU + зажат модификатор → переключить на EN; все модификаторы отпущены → вернуть RU; потеря фокуса → вернуть RU.
- Требуется разрешение Accessability (System Settings → Privacy & Security → Accessibility): в run() вывести подсказку.

Примечание: на macOS keyCode события физические (в отличие от Windows VK), поэтому проблема обычно не возникает — но пользователь подтвердил, что у него ломается, поэтому реализация нужна. Сборку под macOS с этой машины проверить нельзя (нет toolchain) — проверять на Mac: cargo build --release.

После реализации: обновить README (архитектура), бампнуть версию до 1.1.0 (новый минор — macOS-поддержка).

- **Коммит:** нужен