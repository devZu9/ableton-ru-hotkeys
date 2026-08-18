# CHANGELOG

## v1.1.0 (19.08.2026 01:30)

- Добавлено: macOS-поддержка — src/platform/macos.rs (CGEventTap + TIS, FFI на фреймворки без новых крейтов), settings.rs (config_dir ~/Library/Application Support).
- Изменено: версия 1.1.0 (VERSION, README, resource.rc); сборка ресурсов через .cargo/config.toml (явный RC + INCLUDE).
- Исправлено: сборка Windows падала из-за некорректного rc.exe (RC2135/RC1015) — embed-resource подхватывал битый компилятор.

## v1.0.3 (17.08.2026)

- Внедрена система _for_OpenCode: конфигурация opencode (opencode.json, rust-скиллы, команды review/test, агент reviewer), журналы по канону (SESSIONS/ROADMAP/CHANGELOG/SUMMARY), VERSION, проверка трея по windows-tray-testing.