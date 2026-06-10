use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::UI::Shell::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Console::{SetConsoleOutputCP, GetConsoleWindow};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::System::Registry::*;
use windows::core::{PCWSTR, w};
use crate::core::*;
use crate::settings;

const WM_TRAYICON: u32 = WM_APP + 1;
const IDM_TOGGLE_CONSOLE: usize = 200;
const IDM_ABOUT: usize = 201;
const IDM_SETTINGS: usize = 202;
const IDM_EXIT: usize = 203;
const IDI_APP: u16 = 101;
const IDD_SETTINGS: u16 = 102;
const IDD_SPLASH: u16 = 104;
const IDC_AUTOSTART: i32 = 1000;
const IDC_START_MINIMIZED: i32 = 1001;
const IDC_SAVE: i32 = 1002;
const IDC_SPLASH_ICON: i32 = 1003;

const RU_HKL: usize = 0x04190419;
const EN_HKL: usize = 0x04090409;

static mut HOOK_HANDLE: HHOOK = HHOOK(std::ptr::null_mut());
static mut LAYOUT_IS_EN: bool = false;
static mut RESTORE_HWND: HWND = HWND(std::ptr::null_mut());
static mut CTRL_HELD: bool = false;
static mut ALT_HELD: bool = false;
static mut WIN_HELD: bool = false;

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
        let _ = RegCloseKey(hkey);
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
    unsafe {
        match msg {
            WM_INITDIALOG => {
                let (autostart, minimized) = settings::load();
                let _ = SendMessageW(GetDlgItem(hwnd, IDC_AUTOSTART).unwrap(), BM_SETCHECK, WPARAM(if autostart { 1 } else { 0 }), LPARAM(0));
                let _ = SendMessageW(GetDlgItem(hwnd, IDC_START_MINIMIZED).unwrap(), BM_SETCHECK, WPARAM(if minimized { 1 } else { 0 }), LPARAM(0));
                1
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as i32;
                if id == IDC_SAVE {
                    let autostart = SendMessageW(GetDlgItem(hwnd, IDC_AUTOSTART).unwrap(), BM_GETCHECK, WPARAM(0), LPARAM(0)).0 != 0;
                    let minimized = SendMessageW(GetDlgItem(hwnd, IDC_START_MINIMIZED).unwrap(), BM_GETCHECK, WPARAM(0), LPARAM(0)).0 != 0;
                    settings::save(autostart, minimized);
                    apply_autostart(autostart);
                    let _ = EndDialog(hwnd, 1);
                    1
                } else if id == 2 {
                    let _ = EndDialog(hwnd, 0);
                    1
                } else {
                    0
                }
            }
            _ => 0,
        }
    }
}

unsafe extern "system" fn tray_wnd_proc(
    hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM,
) -> LRESULT {
    unsafe {
        match msg {
            WM_CREATE => LRESULT(0),
            WM_TRAYICON => {
                let lpm = lparam.0 as u32;
                if lpm == WM_LBUTTONUP || lpm == WM_RBUTTONUP {
                    let console = GetConsoleWindow();
                    let visible = IsWindowVisible(console).as_bool();
                    if lpm == WM_RBUTTONUP {
                        let mut pt = POINT { x: 0, y: 0 };
                        let _ = GetCursorPos(&mut pt);
                        let _ = SetForegroundWindow(hwnd);
                        let menu = CreatePopupMenu().unwrap();
                        let toggle_label = if visible { w!("Скрыть окно") } else { w!("Показать окно") };
                        let _ = AppendMenuW(menu, MF_STRING, IDM_TOGGLE_CONSOLE, toggle_label);
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
                        let _ = AppendMenuW(menu, MF_STRING, IDM_ABOUT, w!("О программе"));
                        let _ = AppendMenuW(menu, MF_STRING, IDM_SETTINGS, w!("Настройки"));
                        let _ = AppendMenuW(menu, MF_SEPARATOR, 0, None);
                        let _ = AppendMenuW(menu, MF_STRING, IDM_EXIT, w!("Выход"));
                        let _ = TrackPopupMenu(menu, TPM_RIGHTBUTTON, pt.x, pt.y, 0, hwnd, None);
                        let _ = DestroyMenu(menu);
                    } else {
                        if visible { let _ = ShowWindow(console, SW_HIDE); } else { let _ = ShowWindow(console, SW_SHOW); }
                    }
                }
                LRESULT(0)
            }
            WM_COMMAND => {
                let id = (wparam.0 & 0xFFFF) as usize;
                match id {
                    IDM_TOGGLE_CONSOLE => {
                        let console = GetConsoleWindow();
                        let visible = IsWindowVisible(console).as_bool();
                        if visible { let _ = ShowWindow(console, SW_HIDE); } else { let _ = ShowWindow(console, SW_SHOW); }
                    }
                    IDM_ABOUT => {
                        let _ = ShellExecuteW(None, w!("open"), w!("https://github.com/devZu9/ableton-ru-hotkeys"), None, None, SW_SHOWNORMAL);
                    }
                    IDM_SETTINGS => {
                        if let Ok(inst) = GetModuleHandleA(None) {
                            DialogBoxParamW(inst, PCWSTR(IDD_SETTINGS as *const u16), hwnd, Some(settings_dlg_proc), LPARAM(0));
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
                let _ = Shell_NotifyIconW(NIM_DELETE, &nid);
                PostQuitMessage(0);
                LRESULT(0)
            }
            _ => DefWindowProcW(hwnd, msg, wparam, lparam),
        }
    }
}

unsafe extern "system" fn splash_dlg_proc(
    hwnd: HWND, msg: u32, _wparam: WPARAM, _lparam: LPARAM,
) -> isize {
    unsafe {
        match msg {
            WM_INITDIALOG => {
                let _ = SetTimer(hwnd, 1, 3000, None);
                if let Ok(inst) = GetModuleHandleA(None) {
                    if let Ok(h) = LoadImageW(inst, PCWSTR(IDI_APP as *const u16), GDI_IMAGE_TYPE(1), 64, 64, IMAGE_FLAGS(0)) {
                        let icon_ctrl = GetDlgItem(hwnd, IDC_SPLASH_ICON).unwrap();
                        let _ = SendMessageW(icon_ctrl, 0x0173, WPARAM(h.0 as usize), LPARAM(0));
                    }
                }
                1
            }
            WM_TIMER => {
                let _ = KillTimer(hwnd, 1);
                let _ = EndDialog(hwnd, 0);
                1
            }
            _ => 0,
        }
    }
}

pub fn run() {
    unsafe {
        let _ = SetLastError(ERROR_SUCCESS);
        if let Ok(mtx) = CreateMutexW(None, false, w!("Local\\AbletonRUHotkeysMutex")) {
            if GetLastError() == ERROR_ALREADY_EXISTS {
                println!("Ableton RU Hotkeys уже запущена.");
                return;
            }
            let _ = mtx;
        }
        let _ = SetConsoleOutputCP(65001);

        let console = GetConsoleWindow();
        let _ = ShowWindow(console, SW_HIDE);

        let splash_inst = GetModuleHandleA(None).unwrap();
        let _ = DialogBoxParamW(splash_inst, PCWSTR(IDD_SPLASH as *const u16), None, Some(splash_dlg_proc), LPARAM(0));

        let (_, minimized) = settings::load();
        if !minimized {
            let _ = ShowWindow(console, SW_SHOW);
        }

        let instance = GetModuleHandleA(None).unwrap();

        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), instance, 0)
            .expect("Не удалось установить keyboard hook");
        HOOK_HANDLE = hook;

        let hwnd = CreateWindowExW(
            WS_EX_NOACTIVATE, w!("STATIC"), w!(""), WS_POPUP,
            0, 0, 0, 0, HWND_MESSAGE, None, instance, None,
        ).unwrap();
        let _ = SetWindowLongPtrW(hwnd, GWLP_WNDPROC, tray_wnd_proc as *const () as isize);

        let mut nid: NOTIFYICONDATAW = std::mem::zeroed();
        nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = hwnd;
        nid.uID = 1;
        nid.uFlags = NIF_MESSAGE | NIF_ICON | NIF_TIP;
        nid.uCallbackMessage = WM_TRAYICON;
        if let Ok(icon) = LoadIconW(instance, PCWSTR(IDI_APP as *const u16)) {
            nid.hIcon = icon;
        }
        let tip: Vec<u16> = "Ableton RU Hotkeys\0".encode_utf16().collect();
        let count = (tip.len() - 1).min(127);
        nid.szTip[..count].copy_from_slice(&tip[..count]);
        let _ = Shell_NotifyIconW(NIM_ADD, &nid);

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
