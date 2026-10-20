//! Interrupt Descriptor Table (IDT) and Interrupt Dispatcher
//! Handles CPU exceptions, hardware IRQs (Timer, Keyboard, Mouse), and Ring 3 Syscalls.

use core::arch::asm;
use super::pic;

#[repr(C, packed)]
#[derive(Copy, Clone)]
pub struct IdtEntry {
    offset_low: u16,
    selector: u16,
    ist: u8,
    type_attr: u8,
    offset_mid: u16,
    offset_high: u32,
    zero: u32,
}

impl IdtEntry {
    pub const fn empty() -> Self {
        Self {
            offset_low: 0,
            selector: 0,
            ist: 0,
            type_attr: 0,
            offset_mid: 0,
            offset_high: 0,
            zero: 0,
        }
    }

    pub fn set_handler(&mut self, handler: u64, ist: u8, dpl: u8) {
        self.offset_low = handler as u16;
        self.selector = 0x08; // Kernel code segment
        self.ist = ist;
        self.type_attr = 0x8E | (dpl << 5); // 0x8E: 64-bit Interrupt Gate, Present
        self.offset_mid = (handler >> 16) as u16;
        self.offset_high = (handler >> 32) as u32;
        self.zero = 0;
    }
}

#[repr(C, packed)]
struct IdtDescriptor {
    size: u16,
    offset: u64,
}

#[repr(C)]
#[derive(Debug)]
pub struct InterruptFrame {
    pub r15: u64,
    pub r14: u64,
    pub r13: u64,
    pub r12: u64,
    pub r11: u64,
    pub r10: u64,
    pub r9: u64,
    pub r8: u64,
    pub rbp: u64,
    pub rdi: u64,
    pub rsi: u64,
    pub rdx: u64,
    pub rcx: u64,
    pub rbx: u64,
    pub rax: u64,
    pub int_num: u64,
    pub error_code: u64,
    pub rip: u64,
    pub cs: u64,
    pub rflags: u64,
    pub rsp: u64,
    pub ss: u64,
}

static mut IDT: [IdtEntry; 256] = [IdtEntry::empty(); 256];

extern "C" {
    fn isr0(); fn isr1(); fn isr2(); fn isr3(); fn isr4(); fn isr5(); fn isr6(); fn isr7();
    fn isr8(); fn isr9(); fn isr10(); fn isr11(); fn isr12(); fn isr13(); fn isr14(); fn isr15();
    fn isr16(); fn isr17(); fn isr18(); fn isr19(); fn isr20(); fn isr21(); fn isr22(); fn isr23();
    fn isr24(); fn isr25(); fn isr26(); fn isr27(); fn isr28(); fn isr29(); fn isr30(); fn isr31();
    fn irq0(); fn irq1(); fn irq2(); fn irq3(); fn irq4(); fn irq5(); fn irq6(); fn irq7();
    fn irq8(); fn irq9(); fn irq10(); fn irq11(); fn irq12(); fn irq13(); fn irq14(); fn irq15();
    fn isr128(); // 0x80 syscall
}

#[no_mangle]
pub extern "C" fn interrupt_handler(frame: &mut InterruptFrame) {
    let int_num = frame.int_num;

    match int_num {
        // CPU Exceptions
        0..=31 => {
            crate::serial_println!(
                "[EXCEPTION #{}] Error Code: 0x{:x}, RIP: 0x{:016x}, CS: 0x{:x}",
                int_num, frame.error_code, frame.rip, frame.cs
            );

            if int_num == 14 {
                // Page Fault: read CR2
                let cr2: u64;
                unsafe {
                    asm!("mov {}, cr2", out(reg) cr2, options(nomem, nostack, preserves_flags));
                }
                crate::serial_println!("[PAGE FAULT] Attempted access to virtual address: 0x{:016x}", cr2);
            }

            crate::kprintln!("\n[FATAL] CPU Exception #{}: Halting.", int_num);
            loop {
                unsafe { asm!("hlt"); }
            }
        }
        // Hardware IRQs
        32..=47 => {
            let irq = (int_num - 32) as u8;

            if irq == 0 {
                // Timer tick
                crate::drivers::timer::on_timer_tick();
            } else if irq == 1 {
                // Keyboard
                crate::drivers::keyboard::on_keyboard_irq();
            } else if irq == 12 {
                // Mouse
                crate::drivers::mouse::on_mouse_irq();
            }

            unsafe {
                pic::send_eoi(irq);
            }
        }
        // Syscall (0x80)
        128 => {
            frame.rax = crate::kernel_syscall_dispatch(
                frame.rax, frame.rdi, frame.rsi, frame.rdx, frame.r10, frame.r8, frame.r9
            );
        }
        _ => {
            crate::serial_println!("[INT] Unhandled interrupt vector {}", int_num);
        }
    }
}

pub fn init() {
    unsafe {
        let isr_table: [unsafe extern "C" fn(); 32] = [
            isr0, isr1, isr2, isr3, isr4, isr5, isr6, isr7,
            isr8, isr9, isr10, isr11, isr12, isr13, isr14, isr15,
            isr16, isr17, isr18, isr19, isr20, isr21, isr22, isr23,
            isr24, isr25, isr26, isr27, isr28, isr29, isr30, isr31,
        ];

        for i in 0..32 {
            let ist = if i == 8 { 1 } else { 0 }; // IST 1 for Double Fault
            IDT[i].set_handler(isr_table[i] as u64, ist, 0);
        }

        let irq_table: [unsafe extern "C" fn(); 16] = [
            irq0, irq1, irq2, irq3, irq4, irq5, irq6, irq7,
            irq8, irq9, irq10, irq11, irq12, irq13, irq14, irq15,
        ];

        for i in 0..16 {
            IDT[32 + i].set_handler(irq_table[i] as u64, 0, 0);
        }

        // 0x80 Syscall: Ring 3 DPL=3
        IDT[128].set_handler(isr128 as *const () as u64, 0, 3);

        let desc = IdtDescriptor {
            size: (core::mem::size_of_val(&IDT) - 1) as u16,
            offset: IDT.as_ptr() as u64,
        };

        asm!("lidt [{}]", in(reg) &desc, options(readonly, nostack, preserves_flags));
        crate::serial_println!("[IDT] IDT loaded with 256 interrupt gates.");
    }
}

pub fn enable_interrupts() {
    unsafe {
        asm!("sti");
    }
}

pub fn disable_interrupts() {
    unsafe {
        asm!("cli");
    }
}
