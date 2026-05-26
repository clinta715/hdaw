#[cfg(target_os = "windows")]
pub(crate) mod keyboard {
    use std::cell::RefCell;
    pub const SPACE: u32 = 0x20;
    pub const Z: u32 = 0x5A;
    pub const Y: u32 = 0x59;
    pub const L: u32 = 0x4C;
    pub const T: u32 = 0x54;
    pub const M: u32 = 0x4D;
    pub const HOME: u32 = 0x24;
    pub const END: u32 = 0x23;
    pub const OEM_PLUS: u32 = 0xBB;
    pub const OEM_MINUS: u32 = 0xBD;
    pub const LEFT: u32 = 0x25;
    pub const RIGHT: u32 = 0x27;
    pub const CONTROL: u32 = 0x11;
    pub const DELETE: u32 = 0x2E;
    pub const C: u32 = 0x43;
    pub const V: u32 = 0x56;
    pub const A: u32 = 0x41;
    pub const R: u32 = 0x52;
    pub const P: u32 = 0x50;
    pub const S: u32 = 0x53;
    pub const N: u32 = 0x4E;
    pub const ESCAPE: u32 = 0x1B;

    pub struct KeyState {
        pressed: [bool; 256],
    }

    impl KeyState {
        pub fn new() -> Self { Self { pressed: [false; 256] } }
        pub fn was_pressed(&mut self, key: u32) -> bool {
            unsafe {
                use windows::Win32::UI::Input::KeyboardAndMouse::GetAsyncKeyState;
                let is_down = GetAsyncKeyState(key as i32) < 0;
                let was = is_down && !self.pressed[key as usize];
                self.pressed[key as usize] = is_down;
                was
            }
        }
    }

    impl Default for KeyState { fn default() -> Self { Self::new() } }

    thread_local! { pub static KEYS: RefCell<KeyState> = RefCell::new(KeyState::new()); }
}

#[cfg(target_os = "windows")]
pub(crate) fn app_has_focus() -> bool {
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowThreadProcessId};
    let hwnd = unsafe { GetForegroundWindow() };
    if hwnd.0.is_null() { return false; }
    let mut pid: u32 = 0;
    unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)); }
    pid == std::process::id()
}

#[cfg(not(target_os = "windows"))]
pub(crate) fn app_has_focus() -> bool { true }
