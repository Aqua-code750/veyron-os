//! 8259 Programmable Interrupt Controller (PIC)
//! Remaps hardware IRQs 0..15 to CPU interrupt vectors 32..47.

use crate::serial::{inb, outb};

const PIC1_COMMAND: u16 = 0x20;
const PIC1_DATA: u16    = 0x21;
const PIC2_COMMAND: u16 = 0xA0;
const PIC2_DATA: u16    = 0xA1;

const ICW1_INIT: u8 = 0x11;
const ICW4_8086: u8 = 0x01;
const PIC_EOI: u8   = 0x20;

pub const PIC1_OFFSET: u8 = 32;
pub const PIC2_OFFSET: u8 = 40;

pub unsafe fn remap() {
    let a1 = inb(PIC1_DATA);
    let a2 = inb(PIC2_DATA);

    // Starts the initialization sequence (in cascade mode)
    outb(PIC1_COMMAND, ICW1_INIT);
    io_wait();
    outb(PIC2_COMMAND, ICW1_INIT);
    io_wait();

    // ICW2: Master PIC vector offset
    outb(PIC1_DATA, PIC1_OFFSET);
    io_wait();
    // ICW2: Slave PIC vector offset
    outb(PIC2_DATA, PIC2_OFFSET);
    io_wait();

    // ICW3: tell Master PIC that there is a slave PIC at IRQ2 (0000 0100)
    outb(PIC1_DATA, 4);
    io_wait();
    // ICW3: tell Slave PIC its cascade identity (0000 0010)
    outb(PIC2_DATA, 2);
    io_wait();

    // ICW4: have the PICs use 8086 mode (and not 8080 mode)
    outb(PIC1_DATA, ICW4_8086);
    io_wait();
    outb(PIC2_DATA, ICW4_8086);
    io_wait();

    // Restore saved masks
    outb(PIC1_DATA, a1);
    outb(PIC2_DATA, a2);

    crate::serial_println!("[PIC] 8259 PIC remapped to vectors 32..47.");
}

pub unsafe fn send_eoi(irq: u8) {
    if irq >= 8 {
        outb(PIC2_COMMAND, PIC_EOI);
    }
    outb(PIC1_COMMAND, PIC_EOI);
}

pub unsafe fn set_mask(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_line = if irq < 8 { irq } else { irq - 8 };
    let value = inb(port) | (1 << irq_line);
    outb(port, value);
}

pub unsafe fn clear_mask(irq: u8) {
    let port = if irq < 8 { PIC1_DATA } else { PIC2_DATA };
    let irq_line = if irq < 8 { irq } else { irq - 8 };
    let value = inb(port) & !(1 << irq_line);
    outb(port, value);
}

#[inline]
pub unsafe fn io_wait() {
    outb(0x80, 0);
}
