use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Console::{SetConsoleOutputCP, GetConsoleWindow};
use windows::Win32::System::Registry::*;
use windows::core::{PCWSTR, w};
use crate::core::*;

const WM_TRAYICON: u32 = WM_APP + 1;
const IDM_ABOUT: usize = 200;
const IDM_SETTINGS: usize = 201;
const IDM_EXIT: usize = 202;
const REG_PATH: &str = "Software\\AbletonRUHotkeys";
const IDI_APP: u16 = 101;
const IDD_SETTINGS: u16 = 102;
const IDC_AUTOSTART: usize = 1000;
const IDC_START_MINIMIZED: usize = 1001;
const IDC_SAVE: usize = 1002;

const RU_HKL: usize = 0x04190419;
const EN_HKL: usize = 0x04090409;

static mut HOOK_HANDLE: HHOOK = HHOOK(std::ptr::null_mut());
static mut LAYOUT_IS_EN: bool = false;
static mut RESTORE_HWND: HWND = HWND(std::ptr::null_mut());
static mut CTRL_HELD: bool = false;
static mut ALT_HELD: bool = false;
static mut WIN_HELD: bool = false;

fn ok(rc: WIN32_ERROR) -> bool {
    rc == WIN32_ERROR(0)
}

fn open_settings_key(read_only: bool) -> Option<HKEY> {
    unsafe {
        let path_w: Vec<u16> = REG_PATH.encode_utf16().chain(std::iter::once(0)).collect();
        let mut hkey = HKEY::default();
        let access = if read_only { REG_SAM_FLAGS(0x20019) } else { REG_SAM_FLAGS(0x20006) };
        if ok(RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR(path_w.as_ptr()), 0, access, &mut hkey)) {
            Some(hkey)
        } else {
            None
        }
    }
}

fn create_settings_key() -> Option<HKEY> {
    unsafe {
        let path_w: Vec<u16> = REG_PATH.encode_utf16().chain(std::iter::once(0)).collect();
        let mut hkey = HKEY::default();
        if ok(RegCreateKeyW(HKEY_CURRENT_USER, PCWSTR(path_w.as_ptr()), &mut hkey)) {
            Some(hkey)
        } else {
            None
        }
    }
}

fn read_reg_dword(hkey: HKEY, name: &str) -> Option<u32> {
    unsafe {
        let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut value: u32 = 0;
        let mut typ = REG_VALUE_TYPE(0);
        let mut size: u32 = 4;
        let rc = RegQueryValueExW(
            hkey, PCWSTR(name_w.as_ptr()), None,
            Some(&mut typ as *mut REG_VALUE_TYPE),
            Some(&mut value as *mut u32 as *mut u8),
            Some(&mut size as *mut u32),
        );
        if ok(rc) && typ == REG_DWORD { Some(value) } else { None }
    }
}

fn write_reg_dword(hkey: HKEY, name: &str, value: u32) {
    unsafe {
        let name_w: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let data = std::slice::from_raw_parts(&value as *const u32 as *const u8, 4);
        let _ = RegSetValueExW(hkey, PCWSTR(name_w.as_ptr()), 0, REG_DWORD, Some(data));
    }
}

fn is_autostart_enabled() -> bool {
    unsafe {
        if let Some(hkey) = open_settings_key(true) {
            let val = read_reg_dword(hkey, "AutoStart").unwrap_or(1);
            RegCloseKey(hkey);
            val != 0
        } else {
            true
        }
    }
}

fn is_start_minimized_enabled() -> bool {
    unsafe {
        if let Some(hkey) = open_settings_key(true) {
            let val = read_reg_dword(hkey, "StartMinimized").unwrap_or(0);
            RegCloseKey(hkey);
            val != 0
        } else {
            false
        }
    }
}

fn save_settings(autostart: bool, minimized: bool) {
    unsafe {
        if let Some(hkey) = create_settings_key() {
            write_reg_dword(hkey, "AutoStart", if autostart { 1 } else { 0 });
            write_reg_dword(hkey, "StartMinimized", if minimized { 1 } else { 0 });
            RegCloseKey(hkey);
        }
    }
    apply_autostart(autostart);
}

fn apply_autostart(enabled: bool) {
    unsafe {
        let path_w: Vec<u16> = "Software\\Microsoft\\Windows\\CurrentVersion\\Run"
            .encode_utf16().chain(std::iter::once(0)).collect();
        let mut hkey = HKEY::default();
        let _ = RegCreateKeyW(HKEY_CURRENT_USER, PCWSTR(path_w.as_ptr()), &mut hkey);

        if enabled {
            if let Ok(exe) = std::env::current_exe() {
                let path = exe.to_string_lossy().to_string();
                let data_w: Vec<u16> = path.encode_utf16().chain(std::iter::once(0)).collect();
                let bytes = std::slice::from_raw_parts(data_w.as_ptr() as *const u8, data_w.len() * 2);
                let _ = RegSetValueExW(hkey, w!("AbletonRUHotkeys"), 0, REG_SZ, Some(bytes));
            }
        } else {
            let _ = RegDeleteValueW(hkey, w!("AbletonRUHotkeys"));
        }
        RegCloseKey(hkey);
    }
}

fn is_ableton_foreground() -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_invalid() { return false; }
        let mut buf = [0u16; 512];
        let len = GetWindowTextW(hwnd, &mut buf);
        if len == 0 { return false; }
        String::from_utf16_lossy(&buf[..len as usize]).contains(ABLETON_TITLE)
    }
}

fn is_russian_layout() -> bool {
    unsafe {
        let fg = GetForegroundWindow();
        if fg.is_invalid() { return false; }
        let tid = GetWindowThreadProcessId(fg, None);
        GetKeyboardLayout(tid).0 as usize == RU_HKL
    }
}

fn switch_to_en_via_message(hwnd: HWND) {
    unsafe {
        let hkl = HKL(EN_HKL as *mut _);
        let _ = SendMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, WPARAM(1), LPARAM(hkl.0 as isize));
    }
}

fn switch_to_ru_via_message(hwnd: HWND) {
    unsafe {
        let hkl = HKL(RU_HKL as *mut _);
        let _ = SendMessageW(hwnd, WM_INPUTLANGCHANGEREQUEST, WPARAM(1), LPARAM(hkl.0 as isize));
    }
}

fn is_shift_down() -> bool {
    unsafe { GetAsyncKeyState(0x10) >> 15 == -1 }
}

fn any_mod_held() -> bool {
    unsafe { CTRL_HELD || ALT_HELD || WIN_HELD }
}

fn mods_prefix() -> String {
    unsafe {
        let mut s = String::new();
        if CTRL_HELD { s.push_str("Ctrl+"); }
        if ALT_HELD { s.push_str("Alt+"); }
        if WIN_HELD { s.push_str("Win+"); }
        if is_shift_down() { s.push_str("Shift+"); }
        s
    }
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if code < 0 { return CallNextHookEx(None, code, wparam, lparam); }

        if code == HC_ACTION as i32 {
            let kb = &*(lparam.0 as *const KBDLLHOOKSTRUCT);
            let vk = kb.vkCode;
            let msg = wparam.0 as u32;
            let is_down = msg == WM_KEYDOWN || msg == WM_SYSKEYDOWN;
            let is_up = msg == WM_KEYUP || msg == WM_SYSKEYUP;
            let is_injected = kb.flags.contains(LLKHF_INJECTED);

            if !is_down && !is_up || is_injected {
                return CallNextHookEx(None, code, wparam, lparam);
            }

            if is_trigger(vk) {
                if is_down {
                    match vk { 0x11 | 0xA2 | 0xA3 => CTRL_HELD = true, _ => {} }
                    match vk { 0x12 | 0xA4 | 0xA5 => ALT_HELD = true, _ => {} }
                    match vk { 0x5B | 0x5C => WIN_HELD = true, _ => {} }
                } else {
                    match vk { 0x11 | 0xA2 | 0xA3 => CTRL_HELD = false, _ => {} }
                    match vk { 0x12 | 0xA4 | 0xA5 => ALT_HELD = false, _ => {} }
                    match vk { 0x5B | 0x5C => WIN_HELD = false, _ => {} }
                }
            }

            let ableton_fg = is_ableton_foreground();
            let is_ru = if ableton_fg { is_russian_layout() } else { false };

            let fg_hwnd = if ableton_fg { GetForegroundWindow() } else { HWND(std::ptr::null_mut()) };

            if ableton_fg && is_ru && is_down && !is_any_modifier(vk) {
                if any_mod_held() {
                    println!("[RU] {}{}", mods_prefix(), vk_name(vk));
                }
            }

            if ableton_fg && is_ru && is_down && is_trigger(vk) && !LAYOUT_IS_EN {
                println!("[RU] Переключаем на EN (SendMessageW)");
                switch_to_en_via_message(fg_hwnd);
                LAYOUT_IS_EN = true;
                RESTORE_HWND = fg_hwnd;
            }

            if LAYOUT_IS_EN && is_up && is_trigger(vk) {
                if !any_mod_held() {
                    let hwnd = RESTORE_HWND;
                    if !hwnd.is_invalid() {
                        println!("[EN] Возвращаем RU раскладку");
                        switch_to_ru_via_message(hwnd);
                    }
                    LAYOUT_IS_EN = false;
                    RESTORE_HWND = HWND(std::ptr::null_mut());
                }
            }

            if !ableton_fg && LAYOUT_IS_EN {
                LAYOUT_IS_EN = false;
                RESTORE_HWND = HWND(std::ptr::null_mut());
                CTRL_HELD = false;
                ALT_HELD = false;
                WIN_HELD = false;
            }
        }
        CallNextHookEx(None, code, wparam, lparam)
    }
}

unsafe extern "system" fn settings_dlg_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, _lparam: LPARAM,
) -> isize {
    match msg {
        WM_INITDIALOG => {
            unsafe {
                let autostart = is_autostart_enabled();
                let minimized = is_start_minimized_enabled();
                SendMessageW(hwnd, BM_SETCHECK, WPARAM(IDC_AUTOSTART), LPARAM(if autostart { 1 } else { 0 }));
                SendMessageW(hwnd, BM_SETCHECK, WPARAM(IDC_START_MINIMIZED), LPARAM(if minimized { 1 } else { 0 }));
            }
            1
        }
        WM_COMMAND => {
            let id = (wparam.0 & 0xFFFF) as usize;
            if id == IDC_SAVE {
                unsafe {
                    let autostart = SendMessageW(hwnd, BM_GETCHECK, WPARAM(IDC_AUTOSTART), LPARAM(0)).0 != 0;
                    let minimized = SendMessageW(hwnd, BM_GETCHECK, WPARAM(IDC_START_MINIMIZED), LPARAM(0)).0 != 0;
                    save_settings(autostart, minimized);
                    EndDialog(hwnd, 1);
                }
                1
            } else if id == 2 {
                unsafe { EndDialog(hwnd, 0); }
                1
            } else {
                0
            }
        }
        _ => 0,
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => { LRESULT(0) }
            WM_TRAYICON => {
                let lpm = lparam.0 as u32;
                if lpm == WM_LBUTTONUP {
                    let console = GetConsoleWindow();
                    let visible = IsWindowVisible(console);
                    if visible.as_bool() {
                        ShowWindow(console, SW_HIDE);
                    } else {
                        ShowWindow(console, SW_SHOW);
                    }
                } else if lpm == WM_RBUTTONUP {
                    let mut pt = POINT { x: 0, y: 0 };
                    GetCursorPos(&mut pt);
                    SetForegroundWindow(hwnd);
                    let menu = CreatePopupMenu().unwrap();
                    let _ = AppendMenuW(menu, MF_STRING, IDM_ABOUT, w!("О программе"));
                    let _ = AppendMenuW(menu, MF_STRING, IDM_SETTINGS, w!("Настройки"));
                    let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
                    let _ = AppendMenuW(menu, MF_STRING, IDM_EXIT, w!("Выход"));
                    let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
                    let _ = DestroyMenu(menu);
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as usize;
                match id {
                    IDM_ABOUT => {
                        let _ = ShellExecuteW(
                            None, w!("open"),
                            w!("https://github.com/devZu9/ableton-ru-hotkeys"),
                            None, None, SW_SHOWNORMAL,
                        );
                    }
                    IDM_SETTINGS => {
                        if let Ok(inst) = GetModuleHandleA(None) {
                            DialogBoxParamW(
                                inst, PCWSTR(IDD_SETTINGS as *const u16),
                                hwnd, Some(settings_dlg_proc), LPARAM(0),
                            );
                        }
                    }
                    IDM_EXIT => { PostQuitMessage(0); }
                    _ => {}
                }
                LRESULT(0)
            }
            WM_DESTROY => {
                let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
                nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
                nid.hWnd = hwnd;
                nid.uID = 1;
                Shell_NotifyIconW(NIM_DELETE, &nid);
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

pub fn run() {
    unsafe {
        SetConsoleOutputCP(65001).unwrap();

        let instance = GetModuleHandleA(None).unwrap();
        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), instance, 0)
            .expect("Не удалось установить keyboard hook");
        HOOK_HANDLE = hook;

        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE,
            w!("STATIC"),
            w!(""),
            WS_POPUP,
            0, 0, 0, 0,
            HWND_MESSAGE,
            None,
            instance,
            None,
        ).unwrap();

        // Subclass message-only window for tray messages
        SetWindowLongPtrW(hwnd, GWLP_WNDPROC, tray_wnd_proc as isize);

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        let icon = LoadIconW(instance, PCWSTR(IDI_APP as *const u16)).unwrap();
        nid.hIcon = icon;
        let tip: Vec<u16> = "Ableton RU Hotkeys\0".encode_utf16().collect();
        let count = (tip.len() - 1).min(127);
        nid.szTip[..count].copy_from_slice(&tip[..count]);
        nid.szTip[count] = 0;
        Shell_NotifyIconW(NIM_ADD, &nid);

        println!("=== Ableton RU Hotkeys ===");
        println!("Иконка в трее. Левая кнопка — спрятать/показать окно.");
        println!("Правая кнопка — меню.\n");
        println!("Что делает:");
        println!("  - Отслеживает окно Ableton Live");
        println!("  - Если русская раскладка + зажат Ctrl/Alt/Win —");
        println!("    меняет раскладку на EN через SendMessageW");
        println!("  - Shift НЕ является триггером (нужен для заглавных букв)");
        println!("  - Когда все триггеры отпущены — возвращает RU");
        println!("  - При потери фокуса раскладка НЕ меняется\n");

        if is_start_minimized_enabled() {
            ShowWindow(GetConsoleWindow(), SW_HIDE);
        }

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if LAYOUT_IS_EN {
            let hwnd_restore = RESTORE_HWND;
            if !hwnd_restore.is_invalid() {
                switch_to_ru_via_message(hwnd_restore);
            }
        }
        let _ = UnhookWindowsHookEx(HOOK_HANDLE);
        HOOK_HANDLE = HHOOK(std::ptr::null_mut());
    }
}
