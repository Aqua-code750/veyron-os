//! System Call Dispatcher for Veyron OS
//! Provides Ring 3 user mode applications with a secure kernel interface.

use crate::drivers::mouse::{get_state, MouseState};
use crate::drivers::keyboard::{pop_event, KeyEvent};
use crate::framebuffer::FRAMEBUFFER;

pub const SYS_YIELD: u64            = 0;
pub const SYS_EXIT: u64             = 1;
pub const SYS_PRINT: u64            = 2;
pub const SYS_GET_TIME: u64         = 3;
pub const SYS_SLEEP: u64            = 4;
pub const SYS_GET_MOUSE: u64        = 5;
pub const SYS_GET_KEY: u64          = 6;
pub const SYS_GET_FRAMEBUFFER: u64  = 7;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FramebufferInfo {
    pub ptr: u64,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
}

#[no_mangle]
pub fn kernel_syscall_dispatch(
    num: u64,
    arg1: u64,
    arg2: u64,
    _arg3: u64,
    _arg4: u64,
    _arg5: u64,
    _arg6: u64,
) -> u64 {
    match num {
        SYS_YIELD => {
            crate::sched::yield_now();
            0
        }
        SYS_EXIT => {
            crate::serial_println!("[SYSCALL] Process exited with status {}", arg1);
            crate::sched::yield_now();
            0
        }
        SYS_PRINT => {
            let buf_ptr = arg1 as *const u8;
            let len = arg2 as usize;
            if !buf_ptr.is_null() && len > 0 && len <= 4096 {
                let slice = unsafe { core::slice::from_raw_parts(buf_ptr, len) };
                if let Ok(s) = core::str::from_utf8(slice) {
                    crate::kprint!("{}", s);
                }
            }
            len as u64
        }
        SYS_GET_TIME => {
            let t = crate::drivers::rtc::read_rtc();
            let packed = ((t.year as u64) << 32)
                | ((t.month as u64) << 24)
                | ((t.day as u64) << 16)
                | ((t.hours as u64) << 8)
                | (t.minutes as u64);
            packed
        }
        SYS_SLEEP => {
            crate::drivers::timer::sleep_ms(arg1);
            0
        }
        SYS_GET_MOUSE => {
            let out_ptr = arg1 as *mut MouseState;
            if !out_ptr.is_null() {
                let state = get_state();
                unsafe {
                    *out_ptr = state;
                }
                1
            } else {
                0
            }
        }
        SYS_GET_KEY => {
            let out_ptr = arg1 as *mut KeyEvent;
            if !out_ptr.is_null() {
                if let Some(event) = pop_event() {
                    unsafe {
                        *out_ptr = event;
                    }
                    1
                } else {
                    0
                }
            } else {
                0
            }
        }
        SYS_GET_FRAMEBUFFER => {
            let out_ptr = arg1 as *mut FramebufferInfo;
            if !out_ptr.is_null() {
                unsafe {
                    *out_ptr = FramebufferInfo {
                        ptr: FRAMEBUFFER.ptr as u64,
                        width: FRAMEBUFFER.width as u64,
                        height: FRAMEBUFFER.height as u64,
                        pitch: FRAMEBUFFER.pitch as u64,
                    };
                }
                1
            } else {
                0
            }
        }
        _ => {
            crate::serial_println!("[SYSCALL] Unknown syscall #{}", num);
            !0u64
        }
    }
}
