#![allow(dead_code)]

pub struct ModifierState {
    pub ctrl_held: bool,
    pub alt_held: bool,
    pub win_held: bool,
}

impl ModifierState {
    pub const fn new() -> Self {
        Self { ctrl_held: false, alt_held: false, win_held: false }
    }

    pub fn update(&mut self, vk: u32, is_down: bool) {
        match vk {
            0x11 | 0xA2 | 0xA3 => self.ctrl_held = is_down,
            0x12 | 0xA4 | 0xA5 => self.alt_held = is_down,
            0x5B | 0x5C => self.win_held = is_down,
            _ => {}
        }
    }

    pub fn any_held(&self) -> bool {
        self.ctrl_held || self.alt_held || self.win_held
    }

    pub fn mods_prefix(&self, shift_held: bool) -> String {
        let mut s = String::new();
        if self.ctrl_held { s.push_str("Ctrl+"); }
        if self.alt_held { s.push_str("Alt+"); }
        if self.win_held { s.push_str("Win+"); }
        if shift_held { s.push_str("Shift+"); }
        s
    }
}

pub const ABLETON_TITLE: &str = "Ableton";

pub fn is_trigger(vk: u32) -> bool {
    matches!(vk, 0x11 | 0xA2 | 0xA3 | 0x12 | 0xA4 | 0xA5 | 0x5B | 0x5C)
}

pub fn is_any_modifier(vk: u32) -> bool {
    is_trigger(vk) || matches!(vk, 0x10 | 0xA0 | 0xA1)
}

pub fn vk_name(vk: u32) -> &'static str {
    match vk {
        0x41 => "A", 0x42 => "B", 0x43 => "C", 0x44 => "D",
        0x45 => "E", 0x46 => "F", 0x47 => "G", 0x48 => "H",
        0x49 => "I", 0x4A => "J", 0x4B => "K", 0x4C => "L",
        0x4D => "M", 0x4E => "N", 0x4F => "O", 0x50 => "P",
        0x51 => "Q", 0x52 => "R", 0x53 => "S", 0x54 => "T",
        0x55 => "U", 0x56 => "V", 0x57 => "W", 0x58 => "X",
        0x59 => "Y", 0x5A => "Z",
        0x30 => "0", 0x31 => "1", 0x32 => "2", 0x33 => "3",
        0x34 => "4", 0x35 => "5", 0x36 => "6", 0x37 => "7",
        0x38 => "8", 0x39 => "9",
        0xBA => ";", 0xBB => "=", 0xBC => ",", 0xBD => "-",
        0xBE => ".", 0xBF => "/", 0xC0 => "`",
        0xDB => "[", 0xDC => "\\", 0xDD => "]", 0xDE => "'",
        _ => "?",
    }
}
