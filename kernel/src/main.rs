//! Veyron OS - Modern, Lightweight Windows 11-style x86_64 Desktop Operating System
//! Written FROM SCRATCH in Rust (no_std, no_main) for UEFI and BIOS systems.

#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unused)]

extern crate alloc;

pub mod apps;
pub mod arch;
pub mod drivers;
pub mod font;
pub mod framebuffer;
pub mod fs;
pub mod gui;
pub mod mm;
pub mod net;
pub mod sched;
pub mod serial;
pub mod syscall;

use core::panic::PanicInfo;
use framebuffer::{CONSOLE, FRAMEBUFFER, Framebuffer};
use serial::SERIAL;

#[used]
#[link_section = ".limine_requests_start"]
static LIMINE_REQUESTS_START: [u64; 4] = [
    0xf6b8f4b39de7d1ae,
    0xfab91a6940fcb9cf,
    0x785c6ed015d3e316,
    0x181e920a7852b9d9,
];

#[used]
#[link_section = ".limine_requests"]
static BASE_REVISION: [u64; 3] = [0xf9562b2d5c95a6c8, 0x6a7b384944536bdc, 3];

#[repr(C)]
pub struct LimineFramebufferRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *mut LimineFramebufferResponse,
}

#[repr(C)]
pub struct LimineFramebufferResponse {
    pub revision: u64,
    pub framebuffer_count: u64,
    pub framebuffers: *mut *mut LimineFramebuffer,
}

#[repr(C)]
pub struct LimineFramebuffer {
    pub address: *mut u8,
    pub width: u64,
    pub height: u64,
    pub pitch: u64,
    pub bpp: u16,
    pub memory_model: u8,
    pub red_mask_size: u8,
    pub red_mask_shift: u8,
    pub green_mask_size: u8,
    pub green_mask_shift: u8,
    pub blue_mask_size: u8,
    pub blue_mask_shift: u8,
}

unsafe impl Sync for LimineFramebufferRequest {}

#[used]
#[link_section = ".limine_requests"]
pub static mut FRAMEBUFFER_REQUEST: LimineFramebufferRequest = LimineFramebufferRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x9d5827dcd881dd75,
        0xa3148604f6fab11b,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

#[repr(C)]
pub struct LimineHhdmRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *mut LimineHhdmResponse,
}

#[repr(C)]
pub struct LimineHhdmResponse {
    pub revision: u64,
    pub offset: u64,
}

unsafe impl Sync for LimineHhdmRequest {}

#[used]
#[link_section = ".limine_requests"]
pub static mut HHDM_REQUEST: LimineHhdmRequest = LimineHhdmRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x48dcf1cb8ad2b852,
        0x63984e959a98244b,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

#[repr(C)]
#[derive(Copy, Clone)]
pub struct LimineMemmapEntry {
    pub base: u64,
    pub length: u64,
    pub typ: u64,
}

#[repr(C)]
pub struct LimineMemmapResponse {
    pub revision: u64,
    pub entry_count: u64,
    pub entries: *mut *mut LimineMemmapEntry,
}

#[repr(C)]
pub struct LimineMemmapRequest {
    pub id: [u64; 4],
    pub revision: u64,
    pub response: *mut LimineMemmapResponse,
}

unsafe impl Sync for LimineMemmapRequest {}

#[used]
#[link_section = ".limine_requests"]
pub static mut MEMMAP_REQUEST: LimineMemmapRequest = LimineMemmapRequest {
    id: [
        0xc7b1dd30df4c8b88,
        0x0a82e883a194f07b,
        0x67cf3d9d378a806f,
        0xe304acdfc50c3c62,
    ],
    revision: 0,
    response: core::ptr::null_mut(),
};

#[used]
#[link_section = ".limine_requests_end"]
static LIMINE_REQUESTS_END: [u64; 2] = [0xadc0e0531bb10d03, 0x9572709f31764c62];

pub use syscall::kernel_syscall_dispatch;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    // 1. Initialize serial port COM1
    SERIAL.init();
    serial_println!("==================================================");
    serial_println!("        VEYRON OS v0.1.0 (x86_64 UEFI/BIOS)       ");
    serial_println!("    Copyright (c) 2026 Veyron Project scratch     ");
    serial_println!("==================================================");
    serial_println!("[BOOT] Serial console initialized at COM1 (0x3F8)");

    // 2. Initialize Framebuffer & Console
    unsafe {
        let fb_resp = FRAMEBUFFER_REQUEST.response;
        if !fb_resp.is_null() && (*fb_resp).framebuffer_count > 0 {
            let fb = *(*fb_resp).framebuffers;
            FRAMEBUFFER = Framebuffer {
                ptr: (*fb).address as *mut u32,
                width: (*fb).width as usize,
                height: (*fb).height as usize,
                pitch: ((*fb).pitch / 4) as usize,
            };
            CONSOLE.init();
            serial_println!(
                "[VIDEO] GOP Framebuffer: {}x{} (pitch: {} px, bpp: {})",
                (*fb).width,
                (*fb).height,
                (*fb).pitch / 4,
                (*fb).bpp
            );
        }
    }

    kprintln!("  [STAGE 1] Bootloader & Framebuffer Console Online.");

    // 3. Initialize Memory Management (PMM, VMM, Heap)
    unsafe {
        let hhdm_resp = HHDM_REQUEST.response;
        if !hhdm_resp.is_null() {
            mm::pmm::PMM.set_hhdm((*hhdm_resp).offset);
        }

        let mmap_resp = MEMMAP_REQUEST.response;
        if !mmap_resp.is_null() {
            let count = (*mmap_resp).entry_count as usize;
            let mut entries_buf = [LimineMemmapEntry { base: 0, length: 0, typ: 1 }; 64];
            let copy_count = count.min(64);
            for i in 0..copy_count {
                let entry_ptr = *(*mmap_resp).entries.add(i);
                entries_buf[i] = *entry_ptr;
            }
            mm::pmm::PMM.init(&entries_buf[..copy_count]);
        }

        mm::vmm::VMM.init();
        mm::heap::KERNEL_HEAP.init();
    }
    kprintln!("  [STAGE 2] Memory Management: PMM, VMM, Heap Initialized.");

    // 4. Initialize GDT, IDT, PIC, and Interrupts
    arch::x86_64::gdt::init();
    unsafe {
        arch::x86_64::pic::remap();
    }
    arch::x86_64::idt::init();
    arch::x86_64::idt::enable_interrupts();
    kprintln!("  [STAGE 2] GDT, TSS, IDT, and 8259 PIC Remapped. Interrupts Enabled.");

    // 5. Initialize Hardware Drivers
    drivers::timer::init();
    drivers::keyboard::init();
    unsafe {
        drivers::mouse::init(FRAMEBUFFER.width, FRAMEBUFFER.height);
    }
    drivers::pci::scan_pci();
    drivers::disk::init();
    kprintln!("  [STAGE 2] PIT Timer, PS/2 Keyboard/Mouse, and PCI Drivers Loaded.");

    // 6. Mount Filesystem
    let fs_ok = fs::vfs::init();
    if fs_ok {
        kprintln!("  [STAGE 4] VFS Layer & FAT32 Persistent Filesystem Mounted.");
    }

    // 7. Initialize Network Stack
    unsafe {
        net::NET_STACK.init();
    }
    kprintln!("  [STAGE 8] Native TCP/IP Stack Online (DHCP, DNS, HTTP/1.1).");

    // 8. Initialize Preemptive Scheduler
    unsafe {
        sched::SCHEDULER.init();
    }
    kprintln!("  [STAGE 3] Preemptive Scheduler & Syscall Layer Ready.");

    // 9. Initialize Windows 11 Compositor
    unsafe {
        gui::compositor::COMPOSITOR.init();
        show_boot_splash();
    }
    serial_println!("[VEYRON] Entering Windows 11 Desktop Interactive Compositor Loop.");

    // Main Compositor Loop (Running at ~60 FPS)
    loop {
        unsafe {
            gui::compositor::COMPOSITOR.render_frame();
            sched::schedule();
        }
        drivers::timer::sleep_ms(16); // ~60 FPS throttle
    }
}

unsafe fn show_boot_splash() {
    if !FRAMEBUFFER.is_valid() { return; }
    let w = FRAMEBUFFER.width;
    let h = FRAMEBUFFER.height;
    FRAMEBUFFER.clear(0x000000);

    let cx = w / 2;
    let cy = h / 2 - 40;

    // Draw high-resolution Windows 11 4-tile Azure Blue logo
    let tile_w = 28;
    let gap = 5;

    FRAMEBUFFER.fill_rect(cx - tile_w - gap, cy - tile_w - gap, tile_w, tile_w, 0x00A2ED);
    FRAMEBUFFER.fill_rect(cx + gap, cy - tile_w - gap, tile_w, tile_w, 0x0078D4);
    FRAMEBUFFER.fill_rect(cx - tile_w - gap, cy + gap, tile_w, tile_w, 0x0078D4);
    FRAMEBUFFER.fill_rect(cx + gap, cy + gap, tile_w, tile_w, 0x005A9E);

    // Modern spinning progress animation dots
    for i in 0..5 {
        let dot_x = cx - 32 + i * 16;
        let dot_y = cy + 60;
        FRAMEBUFFER.fill_rect(dot_x, dot_y, 6, 6, 0x00A2ED);
        drivers::timer::sleep_ms(70);
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[KERNEL PANIC] {}", info);
    kprintln!("\n!!! KERNEL PANIC !!!");
    kprintln!("{}", info);
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
