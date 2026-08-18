use std::ffi::{c_char, c_void};

use crate::core::ABLETON_TITLE;

type CFStringRef = *mut c_void;
type CFArrayRef = *mut c_void;
type CFTypeRef = *mut c_void;
type CFMachPortRef = *mut c_void;
type CFRunLoopRef = *mut c_void;
type CFRunLoopSourceRef = *mut c_void;
type CFIndex = isize;
type CFStringEncoding = u32;

type CGEventRef = *mut c_void;
type CGEventTapProxy = *mut c_void;
type CGEventType = u32;
type CGEventFlags = u64;
type CGEventField = u32;
type CGEventMask = u64;
type CGWindowListOption = u32;
type CGWindowID = u32;
type TISInputSourceRef = *mut c_void;
type CGEventTapCallBack =
    unsafe extern "C" fn(CGEventTapProxy, CGEventType, CGEventRef, *mut c_void) -> CGEventRef;

const K_CG_HID_EVENT_TAP: u32 = 0;
const K_CG_HEAD_INSERT_EVENT_TAP: u32 = 0;
const K_CG_EVENT_TAP_OPTION_DEFAULT: u32 = 0;
const K_CG_EVENT_KEY_DOWN: u32 = 10;
const K_CG_EVENT_KEY_UP: u32 = 11;
const K_CG_EVENT_FLAGS_CHANGED: u32 = 12;
const K_CG_KEYBOARD_EVENT_KEYCODE: u32 = 9;
const K_CG_EVENT_FLAG_COMMAND: u64 = 0x0010_0000;
const K_CG_EVENT_FLAG_CONTROL: u64 = 0x0004_0000;
const K_CG_EVENT_FLAG_ALTERNATE: u64 = 0x0008_0000;
const K_CG_WINDOW_LIST_ONSCREEN: u32 = 0x1;
const K_CG_WINDOW_LIST_EXCLUDE_DESKTOP: u32 = 0x1_0000;
const K_CF_STRING_ENCODING_UTF8: u32 = 0x0800_0100;

#[link(name = "CoreFoundation", kind = "framework")]
unsafe extern "C" {
    fn CFStringCreateWithCString(
        alloc: *mut c_void,
        c_str: *const c_char,
        encoding: CFStringEncoding,
    ) -> CFStringRef;
    fn CFStringGetLength(cf: CFStringRef) -> CFIndex;
    fn CFStringGetCString(
        cf: CFStringRef,
        buffer: *mut c_char,
        buffer_size: CFIndex,
        encoding: CFStringEncoding,
    ) -> u8;
    fn CFRelease(cf: CFTypeRef);
    fn CFArrayGetCount(array: CFArrayRef) -> CFIndex;
    fn CFArrayGetValueAtIndex(array: CFArrayRef, index: CFIndex) -> CFTypeRef;
    fn CFDictionaryGetValue(dict: *mut c_void, key: CFTypeRef) -> CFTypeRef;
    fn CFMachPortCreateRunLoopSource(
        alloc: *mut c_void,
        port: CFMachPortRef,
        order: CFIndex,
    ) -> CFRunLoopSourceRef;
    fn CFRunLoopGetCurrent() -> CFRunLoopRef;
    fn CFRunLoopAddSource(rl: CFRunLoopRef, source: CFRunLoopSourceRef, mode: CFStringRef);
    fn CFRunLoopRun();
}

#[link(name = "ApplicationServices", kind = "framework")]
unsafe extern "C" {
    fn CGEventTapCreate(
        tap: u32,
        place: u32,
        options: u32,
        mask: CGEventMask,
        callback: Option<CGEventTapCallBack>,
        user_info: *mut c_void,
    ) -> CFMachPortRef;
    fn CGEventTapEnable(tap: CFMachPortRef, enable: u8);
    fn CGEventGetIntegerValueField(event: CGEventRef, field: CGEventField) -> i64;
    fn CGEventGetFlags(event: CGEventRef) -> CGEventFlags;
    fn CGWindowListCopyWindowInfo(options: CGWindowListOption, relative: CGWindowID) -> CFArrayRef;
    fn AXIsProcessTrusted() -> u8;
    static kCGWindowOwnerName: CFStringRef;
}

#[link(name = "Carbon", kind = "framework")]
unsafe extern "C" {
    fn TISCopyInputSourceList(include_all_layouts: *mut u8) -> CFArrayRef;
    fn TISGetInputSourceProperty(source: TISInputSourceRef, key: CFStringRef) -> CFTypeRef;
    fn TISSelectInputSource(source: TISInputSourceRef) -> i32;
    fn TISCopyCurrentKeyboardInputSource() -> TISInputSourceRef;
}

static mut CTRL_HELD: bool = false;
static mut ALT_HELD: bool = false;
static mut WIN_HELD: bool = false;
static mut LAYOUT_IS_EN: bool = false;
static mut RESTORE_SOURCE: TISInputSourceRef = std::ptr::null_mut();

fn cf_string_from_static(s: &'static str) -> CFStringRef {
    unsafe { CFStringCreateWithCString(std::ptr::null_mut(), s.as_ptr() as *const c_char, K_CF_STRING_ENCODING_UTF8) }
}

fn cf_string_to_owned(cf: CFStringRef) -> Option<String> {
    unsafe {
        let len = CFStringGetLength(cf);
        if len == 0 {
            return Some(String::new());
        }
        let mut buf = vec![0u8; (len * 4 + 1) as usize];
        let ok = CFStringGetCString(cf, buf.as_mut_ptr() as *mut c_char, buf.len() as CFIndex, K_CF_STRING_ENCODING_UTF8);
        if ok == 0 {
            return None;
        }
        let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
        Some(String::from_utf8_lossy(&buf[..end]).into_owned())
    }
}

fn input_source_id(source: TISInputSourceRef) -> Option<String> {
    unsafe {
        let key = cf_string_from_static("TISPropertyInputSourceID");
        let prop = TISGetInputSourceProperty(source, key);
        CFRelease(key);
        if prop.is_null() {
            return None;
        }
        cf_string_to_owned(prop)
    }
}

fn frontmost_is_ableton() -> bool {
    unsafe {
        let list = CGWindowListCopyWindowInfo(
            K_CG_WINDOW_LIST_ONSCREEN | K_CG_WINDOW_LIST_EXCLUDE_DESKTOP,
            0,
        );
        if list.is_null() {
            return false;
        }
        let count = CFArrayGetCount(list);
        let mut found = false;
        for i in 0..count {
            let dict = CFArrayGetValueAtIndex(list, i);
            if dict.is_null() {
                continue;
            }
            let owner = CFDictionaryGetValue(dict, kCGWindowOwnerName);
            if owner.is_null() {
                continue;
            }
            if let Some(name) = cf_string_to_owned(owner) {
                if name.contains(ABLETON_TITLE) {
                    found = true;
                    break;
                }
            }
        }
        CFRelease(list);
        found
    }
}

fn current_is_russian() -> bool {
    unsafe {
        let src = TISCopyCurrentKeyboardInputSource();
        if src.is_null() {
            return false;
        }
        let is_ru = input_source_id(src).map(|id| id.contains("russian")).unwrap_or(false);
        CFRelease(src);
        is_ru
    }
}

fn select_input_source_by_id(pred: impl Fn(&str) -> bool) {
    unsafe {
        let list = TISCopyInputSourceList(std::ptr::null_mut());
        if list.is_null() {
            return;
        }
        let count = CFArrayGetCount(list);
        for i in 0..count {
            let src = CFArrayGetValueAtIndex(list, i);
            if src.is_null() {
                continue;
            }
            if let Some(id) = input_source_id(src) {
                if pred(&id) {
                    let _ = TISSelectInputSource(src);
                    break;
                }
            }
        }
        CFRelease(list);
    }
}

fn select_english_source() {
    select_input_source_by_id(|id| {
        let lower = id.to_lowercase();
        !lower.contains("russian") && (lower.contains(".abc") || lower.contains(".us"))
    });
}

fn restore_russian_source() {
    unsafe {
        if !RESTORE_SOURCE.is_null() {
            let _ = TISSelectInputSource(RESTORE_SOURCE);
            CFRelease(RESTORE_SOURCE);
            RESTORE_SOURCE = std::ptr::null_mut();
        }
    }
}

fn is_modifier_keycode(vk: u32) -> bool {
    matches!(vk, 54 | 55 | 56 | 58 | 59 | 60 | 61 | 62)
}

fn any_mod_held() -> bool {
    unsafe { CTRL_HELD || ALT_HELD || WIN_HELD }
}

fn mods_prefix() -> String {
    unsafe {
        let mut s = String::new();
        if CTRL_HELD { s.push_str("Ctrl+"); }
        if ALT_HELD { s.push_str("Alt+"); }
        if WIN_HELD { s.push_str("Cmd+"); }
        s
    }
}

unsafe extern "C" fn tap_callback(
    _proxy: CGEventTapProxy,
    event_type: CGEventType,
    event: CGEventRef,
    _user_info: *mut c_void,
) -> CGEventRef {
    unsafe {
        handle_event(event_type, event);
    }
    event
}

unsafe fn handle_event(event_type: CGEventType, event: CGEventRef) {
    unsafe {
        let flags = CGEventGetFlags(event);
        CTRL_HELD = flags & K_CG_EVENT_FLAG_CONTROL != 0;
        ALT_HELD = flags & K_CG_EVENT_FLAG_ALTERNATE != 0;
        WIN_HELD = flags & K_CG_EVENT_FLAG_COMMAND != 0;

        let vk = CGEventGetIntegerValueField(event, K_CG_KEYBOARD_EVENT_KEYCODE) as u32;
        let is_down = event_type == K_CG_EVENT_KEY_DOWN;

        let ableton_fg = frontmost_is_ableton();
        let is_ru = if ableton_fg { current_is_russian() } else { false };

        if ableton_fg && is_ru && is_down && !is_modifier_keycode(vk) && any_mod_held() {
            println!("[RU] {}клавиша (keycode {})", mods_prefix(), vk);
        }

        if ableton_fg && is_ru && any_mod_held() && !LAYOUT_IS_EN {
            println!("[RU] Переключаем на EN (TIS)");
            RESTORE_SOURCE = TISCopyCurrentKeyboardInputSource();
            select_english_source();
            LAYOUT_IS_EN = true;
        }

        if LAYOUT_IS_EN && !any_mod_held() {
            println!("[EN] Возвращаем RU раскладку");
            restore_russian_source();
            LAYOUT_IS_EN = false;
        }

        if !ableton_fg && LAYOUT_IS_EN {
            restore_russian_source();
            LAYOUT_IS_EN = false;
            CTRL_HELD = false;
            ALT_HELD = false;
            WIN_HELD = false;
        }
    }
}

pub fn run() {
    println!("=== Ableton RU Hotkeys (macOS) ===");
    unsafe {
        if AXIsProcessTrusted() == 0 {
            println!("Требуется разрешение «Специальные возможности»:");
            println!("  System Settings → Privacy & Security → Accessibility → разрешите терминал");
            println!("  (после включения перезапустите утилиту)");
        }
    }
    let mask = (1u64 << K_CG_EVENT_KEY_DOWN)
        | (1u64 << K_CG_EVENT_KEY_UP)
        | (1u64 << K_CG_EVENT_FLAGS_CHANGED);
    unsafe {
        let tap = CGEventTapCreate(
            K_CG_HID_EVENT_TAP,
            K_CG_HEAD_INSERT_EVENT_TAP,
            K_CG_EVENT_TAP_OPTION_DEFAULT,
            mask,
            Some(tap_callback),
            std::ptr::null_mut(),
        );
        if tap.is_null() {
            println!("Не удалось создать event tap. Проверьте разрешение «Специальные возможности».");
            return;
        }
        CGEventTapEnable(tap, 1);
        println!("=== Ableton RU Hotkeys ===");
        println!("Слушаем события клавиатуры.");
        println!("  - Ableton в фокусе + RU + модификатор (Cmd/Ctrl/Option) → EN");
        println!("  - Shift не является триггером (нужен для заглавных букв)");
        println!("  - Все модификаторы отпущены → возврат RU");
        println!("  - Ctrl+C для выхода");
        let source = CFMachPortCreateRunLoopSource(std::ptr::null_mut(), tap, 0);
        let common = cf_string_from_static("kCFRunLoopCommonModes");
        CFRunLoopAddSource(CFRunLoopGetCurrent(), source, common);
        CFRelease(common);
        CFRunLoopRun();
    }
}
