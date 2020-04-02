use crate::rdev::{Event, EventType, GrabCallback, GrabError};
use crate::windows::common::{convert, get_name, set_key_hook, set_mouse_hook, HookError, HOOK};
use std::ptr::null_mut;
use std::time::SystemTime;
use winapi::um::winuser::{CallNextHookEx, GetMessageA, HC_ACTION};

fn default_callback(event: Event) -> Option<Event> {
    println!("Default : Event {:?}", event);
    Some(event)
}
static mut GLOBAL_CALLBACK: GrabCallback = default_callback;

unsafe extern "system" fn raw_callback(code: i32, param: usize, lpdata: isize) -> isize {
    if code == HC_ACTION {
        let opt = convert(param, lpdata);
        if let Some(event_type) = opt {
            let name = match &event_type {
                EventType::KeyPress(_key) => get_name(lpdata),
                _ => None,
            };
            let event = Event {
                event_type,
                time: SystemTime::now(),
                name,
            };
            if GLOBAL_CALLBACK(event).is_none() {
                // https://stackoverflow.com/questions/42756284/blocking-windows-mouse-click-using-setwindowshookex
                // https://android.developreference.com/article/14560004/Blocking+windows+mouse+click+using+SetWindowsHookEx()
                // https://cboard.cprogramming.com/windows-programming/99678-setwindowshookex-wm_keyboard_ll.html
                // let _result = CallNextHookEx(HOOK, code, param, lpdata);
                return 1;
            }
        }
    }
    CallNextHookEx(HOOK, code, param, lpdata)
}
impl From<HookError> for GrabError {
    fn from(error: HookError) -> Self {
        match error {
            HookError::Mouse(code) => GrabError::MouseHookError(code),
            HookError::Key(code) => GrabError::KeyHookError(code),
        }
    }
}

pub fn grab(callback: GrabCallback) -> Result<(), GrabError> {
    unsafe {
        GLOBAL_CALLBACK = callback;
        set_key_hook(raw_callback)?;
        set_mouse_hook(raw_callback)?;

        GetMessageA(null_mut(), null_mut(), 0, 0);
    }
    Ok(())
}
