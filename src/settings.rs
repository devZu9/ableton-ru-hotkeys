use std::path::PathBuf;

const APP_NAME: &str = "AbletonRUHotkeys";

fn config_dir() -> PathBuf {
    if let Some(appdata) = std::env::var_os("APPDATA") {
        PathBuf::from(appdata).join(APP_NAME)
    } else {
        PathBuf::from(".").join(APP_NAME)
    }
}

fn ini_path() -> PathBuf {
    config_dir().join("settings.ini")
}

pub fn load() -> (bool, bool) {
    let path = ini_path();
    if !path.exists() {
        return (true, false);
    }
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => return (true, false),
    };
    let mut autostart = true;
    let mut minimized = false;
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') || line.starts_with('#') {
            continue;
        }
        if let Some(pos) = line.find('=') {
            let key = line[..pos].trim();
            let val = line[pos + 1..].trim();
            let on = val == "1" || val.eq_ignore_ascii_case("true");
            match key {
                "AutoStart" => autostart = on,
                "StartMinimized" => minimized = on,
                _ => {}
            }
        }
    }
    (autostart, minimized)
}

pub fn save(autostart: bool, minimized: bool) {
    let dir = config_dir();
    let _ = std::fs::create_dir_all(&dir);
    let content = format!(
        "AutoStart={}\nStartMinimized={}\n",
        if autostart { 1 } else { 0 },
        if minimized { 1 } else { 0 },
    );
    let _ = std::fs::write(ini_path(), content);
}
