//! PCI Bus Enumeration Driver
//! Scans PCI buses, slots, and multi-function devices.
//! Detects AHCI SATA disk controllers, VirtIO devices, and network adapters.

const PCI_CONFIG_ADDRESS: u16 = 0xCF8;
const PCI_CONFIG_DATA: u16    = 0xCFC;

use core::arch::asm;

#[derive(Debug, Copy, Clone)]
pub struct PciDeviceInfo {
    pub bus: u8,
    pub slot: u8,
    pub func: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_id: u8,
    pub subclass_id: u8,
    pub prog_if: u8,
    pub bar5: u32,
    pub irq: u8,
}

pub static mut AHCI_DEVICE: Option<PciDeviceInfo> = None;

pub fn pci_read_u32(bus: u8, slot: u8, func: u8, offset: u8) -> u32 {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    unsafe {
        asm!("out dx, eax", in("dx") PCI_CONFIG_ADDRESS, in("eax") address, options(nomem, nostack, preserves_flags));
        let val: u32;
        asm!("in eax, dx", in("dx") PCI_CONFIG_DATA, out("eax") val, options(nomem, nostack, preserves_flags));
        val
    }
}

pub fn pci_write_u32(bus: u8, slot: u8, func: u8, offset: u8, val: u32) {
    let address = ((bus as u32) << 16)
        | ((slot as u32) << 11)
        | ((func as u32) << 8)
        | ((offset as u32) & 0xFC)
        | 0x80000000;

    unsafe {
        asm!("out dx, eax", in("dx") PCI_CONFIG_ADDRESS, in("eax") address, options(nomem, nostack, preserves_flags));
        asm!("out dx, eax", in("dx") PCI_CONFIG_DATA, in("eax") val, options(nomem, nostack, preserves_flags));
    }
}

pub fn scan_pci() -> usize {
    let mut count = 0;
    for bus in 0..=7 {
        for slot in 0..32 {
            for func in 0..8 {
                let vendor = (pci_read_u32(bus, slot, func, 0x00) & 0xFFFF) as u16;
                if vendor == 0xFFFF {
                    if func == 0 {
                        break; // No device in this slot
                    }
                    continue;
                }

                let dev_id = (pci_read_u32(bus, slot, func, 0x00) >> 16) as u16;
                let class_reg = pci_read_u32(bus, slot, func, 0x08);
                let class_id = ((class_reg >> 24) & 0xFF) as u8;
                let subclass_id = ((class_reg >> 16) & 0xFF) as u8;
                let prog_if = ((class_reg >> 8) & 0xFF) as u8;

                let bar5 = pci_read_u32(bus, slot, func, 0x24);
                let intr = pci_read_u32(bus, slot, func, 0x3C);
                let irq = (intr & 0xFF) as u8;

                let dev_info = PciDeviceInfo {
                    bus,
                    slot,
                    func,
                    vendor_id: vendor,
                    device_id: dev_id,
                    class_id,
                    subclass_id,
                    prog_if,
                    bar5,
                    irq,
                };

                crate::serial_println!(
                    "[PCI] {}:{:02}.{} -> Vendor: 0x{:04x}, Device: 0x{:04x}, Class: 0x{:02x}.{:02x} (ProgIF: 0x{:02x})",
                    bus, slot, func, vendor, dev_id, class_id, subclass_id, prog_if
                );

                // Check for AHCI SATA Controller (Class 1, Subclass 6)
                if class_id == 0x01 && subclass_id == 0x06 {
                    crate::serial_println!("      ^-- AHCI SATA Disk Controller detected! (ABAR = 0x{:08x})", bar5);
                    unsafe {
                        AHCI_DEVICE = Some(dev_info);
                    }
                } else if vendor == 0x1AF4 {
                    crate::serial_println!("      ^-- VirtIO Device detected! (Device ID: 0x{:04x})", dev_id);
                } else if class_id == 0x02 {
                    crate::serial_println!("      ^-- Ethernet Network Adapter detected!");
                }

                count += 1;
            }
        }
    }
    crate::serial_println!("[PCI] Scan complete: {} PCI functions discovered.", count);
    count
}
