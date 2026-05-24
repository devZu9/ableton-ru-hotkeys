use windows::Win32::Foundation::*;
use windows::Win32::UI::Input::KeyboardAndMouse::*;
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::System::Console::SetConsoleOutputCP;
use crate::core::*;

const RU_HKL: usize = 0x04190419;
const EN_HKL: usize = 0x04090409;

static mut LAYOUT_IS_EN: bool = false;
static mut RESTORE_HWND: HWND = HWND(std::ptr::null_mut());
static mut CTRL_HELD: bool = false;
static mut ALT_HELD: bool = false;
static mut WIN_HELD: bool = false;

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
    let mut s = String::new();
    unsafe {
        if CTRL_HELD { s.push_str("Ctrl+"); }
        if ALT_HELD { s.push_str("Alt+"); }
        if WIN_HELD { s.push_str("Win+"); }
    }
    if is_shift_down() { s.push_str("Shift+"); }
    s
}

unsafe extern "system" fn hook_proc(code: i32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    unsafe {
        if code < 0 {
            return CallNextHookEx(None, code, wparam, lparam);
        }

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

            let fg_hwnd = if ableton_fg {
                GetForegroundWindow()
            } else {
                HWND(std::ptr::null_mut())
            };

            if ableton_fg && is_ru && is_down && !is_any_modifier(vk) {
                let held = any_mod_held();
                if held {
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

pub fn run() {
    unsafe {
        let _ = SetConsoleOutputCP(65001);

        let instance = GetModuleHandleA(None).unwrap();

        let hook = SetWindowsHookExW(WH_KEYBOARD_LL, Some(hook_proc), instance, 0)
            .expect("Не удалось установить keyboard hook");

        println!("=== Ableton RU Hotkeys (MVP) ===");
        println!("Запущено. Нажми Ctrl+C для выхода.\n");
        println!("Что делает:");
        println!("  - Отслеживает окно Ableton Live");
        println!("  - Если русская раскладка + зажат Ctrl/Alt/Win —");
        println!("    меняет раскладку на EN через SendMessageW");
        println!("  - Shift НЕ является триггером (нужен для заглавных букв)");
        println!("  - Когда все триггеры (Ctrl/Alt/Win) отпущены — возвращает RU");
        println!("  - При потери фокуса раскладка НЕ меняется\n");

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        if LAYOUT_IS_EN {
            let hwnd = RESTORE_HWND;
            if !hwnd.is_invalid() {
                switch_to_ru_via_message(hwnd);
            }
        }
        let _ = UnhookWindowsHookEx(hook);
    }
}
