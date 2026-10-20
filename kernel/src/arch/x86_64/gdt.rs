//! Global Descriptor Table (GDT) and Task State Segment (TSS)
//! Configures 64-bit Ring 0 and Ring 3 segments and the kernel interrupt stack.

use core::arch::asm;

pub const KERNEL_CODE_SELECTOR: u16 = 0x08;
pub const KERNEL_DATA_SELECTOR: u16 = 0x10;
pub const USER_DATA_SELECTOR: u16 = 0x18 | 3;
pub const USER_CODE_SELECTOR: u16 = 0x20 | 3;
pub const TSS_SELECTOR: u16 = 0x28;

#[repr(C, packed)]
pub struct TSS {
    reserved0: u32,
    pub rsp0: u64,
    pub rsp1: u64,
    pub rsp2: u64,
    reserved1: u64,
    pub ist1: u64,
    pub ist2: u64,
    pub ist3: u64,
    pub ist4: u64,
    pub ist5: u64,
    pub ist6: u64,
    pub ist7: u64,
    reserved2: u64,
    reserved3: u16,
    pub iomap_base: u16,
}

impl TSS {
    pub const fn new() -> Self {
        Self {
            reserved0: 0,
            rsp0: 0,
            rsp1: 0,
            rsp2: 0,
            reserved1: 0,
            ist1: 0,
            ist2: 0,
            ist3: 0,
            ist4: 0,
            ist5: 0,
            ist6: 0,
            ist7: 0,
            reserved2: 0,
            reserved3: 0,
            iomap_base: core::mem::size_of::<TSS>() as u16,
        }
    }
}

#[repr(C, packed)]
struct GdtDescriptor {
    size: u16,
    offset: u64,
}

#[repr(C, align(16))]
struct Gdt {
    entries: [u64; 7], // 0: null, 1: kcode, 2: kdata, 3: udata, 4: ucode, 5-6: tss (16 bytes)
}

static mut TSS_INSTANCE: TSS = TSS::new();
static mut GDT_INSTANCE: Gdt = Gdt { entries: [0; 7] };
static mut DOUBLE_FAULT_STACK: [u8; 16384] = [0; 16384];

pub fn init() {
    unsafe {
        // Set up IST1 for double fault
        let df_stack_top = DOUBLE_FAULT_STACK.as_ptr() as u64 + DOUBLE_FAULT_STACK.len() as u64;
        TSS_INSTANCE.ist1 = df_stack_top;

        // 0: Null descriptor
        GDT_INSTANCE.entries[0] = 0;
        // 1: Kernel Code 64-bit (0x08): Present, Ring 0, Executable, Readable, 64-bit Long Mode
        GDT_INSTANCE.entries[1] = 0x00209A0000000000;
        // 2: Kernel Data 64-bit (0x10): Present, Ring 0, Writable
        GDT_INSTANCE.entries[2] = 0x0000920000000000;
        // 3: User Data 64-bit (0x18 | 3): Present, Ring 3, Writable
        GDT_INSTANCE.entries[3] = 0x0000F20000000000;
        // 4: User Code 64-bit (0x20 | 3): Present, Ring 3, Executable, Readable, 64-bit Long Mode
        GDT_INSTANCE.entries[4] = 0x0020FA0000000000;

        // 5 & 6: TSS Descriptor (16 bytes)
        let tss_ptr = &TSS_INSTANCE as *const _ as u64;
        let tss_limit = core::mem::size_of::<TSS>() as u64 - 1;
        let tss_low = (tss_limit & 0xFFFF)
            | ((tss_ptr & 0xFFFFFF) << 16)
            | (0x89u64 << 40) // Present, 64-bit TSS (Available)
            | (((tss_limit >> 16) & 0xF) << 48)
            | (((tss_ptr >> 24) & 0xFF) << 56);
        let tss_high = tss_ptr >> 32;

        GDT_INSTANCE.entries[5] = tss_low;
        GDT_INSTANCE.entries[6] = tss_high;

        let descriptor = GdtDescriptor {
            size: (core::mem::size_of::<Gdt>() - 1) as u16,
            offset: GDT_INSTANCE.entries.as_ptr() as u64,
        };

        // Load GDT
        asm!("lgdt [{}]", in(reg) &descriptor, options(readonly, nostack, preserves_flags));

        // Reload CS and data segment registers
        asm!(
            "push 0x08",
            "lea rax, [rip + 2f]",
            "push rax",
            "retfq",
            "2:",
            "mov ax, 0x10",
            "mov ds, ax",
            "mov es, ax",
            "mov ss, ax",
            "mov fs, ax",
            "mov gs, ax",
            out("rax") _,
            options(nostack)
        );

        // Load TSS
        asm!("ltr ax", in("ax") TSS_SELECTOR, options(nostack, preserves_flags));

        crate::serial_println!("[GDT] GDT and TSS successfully loaded.");
    }
}

pub fn set_kernel_stack(stack_top: u64) {
    unsafe {
        TSS_INSTANCE.rsp0 = stack_top;
    }
}
