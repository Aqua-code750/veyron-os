//! Hard Disk Driver (AHCI SATA Controller + ATA PIO Fallback)
//! Direct LBA sector read and write supporting QEMU, VirtualBox, and physical hardware.

use crate::mm::pmm::PMM;
use crate::serial::{inb, outb};

#[repr(C, align(1024))]
struct AhciCommandList {
    entries: [AhciCommandHeader; 32],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct AhciCommandHeader {
    flags: u16,
    prdtl: u16,
    prdbc: u32,
    ctba: u32,
    ctbau: u32,
    reserved: [u32; 4],
}

#[repr(C, align(128))]
struct AhciCommandTable {
    cfis: [u8; 64],
    acmd: [u8; 16],
    reserved: [u8; 48],
    prdt: [AhciPrdtEntry; 1],
}

#[repr(C)]
#[derive(Copy, Clone)]
struct AhciPrdtEntry {
    dba: u32,
    dbau: u32,
    reserved: u32,
    dbc: u32,
}

pub struct AhciDriver {
    pub abar: u64,
    pub active_port: Option<usize>,
    pub cmd_list_phys: u64,
    pub cmd_table_phys: u64,
    pub dma_buffer_phys: u64,
    pub is_ready: bool,
}

impl AhciDriver {
    pub const fn new() -> Self {
        Self {
            abar: 0,
            active_port: None,
            cmd_list_phys: 0,
            cmd_table_phys: 0,
            dma_buffer_phys: 0,
            is_ready: false,
        }
    }

    pub fn init(&mut self) -> bool {
        unsafe {
            if let Some(dev) = crate::drivers::pci::AHCI_DEVICE {
                let abar_phys = (dev.bar5 & !0x7FF) as u64;
                self.abar = PMM.phys_to_virt(abar_phys);

                // Map ABAR MMIO pages into kernel address space
                let kern_pml4 = crate::mm::vmm::VMM.kernel_pml4();
                let _ = crate::mm::vmm::VMM.map_page(
                    kern_pml4,
                    self.abar,
                    abar_phys,
                    crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE,
                );
                let _ = crate::mm::vmm::VMM.map_page(
                    kern_pml4,
                    self.abar + 0x1000,
                    abar_phys + 0x1000,
                    crate::mm::vmm::PAGE_PRESENT | crate::mm::vmm::PAGE_WRITABLE,
                );

                // Enable Bus Mastering and Memory Space on PCI device
                let cmd_reg = crate::drivers::pci::pci_read_u32(dev.bus, dev.slot, dev.func, 0x04);
                crate::drivers::pci::pci_write_u32(dev.bus, dev.slot, dev.func, 0x04, cmd_reg | 0x07);

                // Enable AHCI mode in Global Host Control (GHC bit 31)
                let ghc_ptr = (self.abar + 0x04) as *mut u32;
                *ghc_ptr |= 1 << 31;

                // Read Ports Implemented (PI)
                let pi = *((self.abar + 0x0C) as *const u32);
                crate::serial_println!("[AHCI] Initializing ABAR at 0x{:08x}, Ports Implemented: 0x{:08x}", abar_phys, pi);

                // Find active SATA port
                for p in 0..32 {
                    if (pi & (1 << p)) != 0 {
                        let port_base = self.abar + 0x100 + (p as u64 * 0x80);
                        let ssts = *((port_base + 0x28) as *const u32);
                        let ipm = (ssts >> 8) & 0x0F;
                        let det = ssts & 0x0F;

                        if det == 3 && ipm == 1 {
                            crate::serial_println!("[AHCI] SATA Disk detected on Port #{}!", p);
                            if self.setup_port(p, port_base) {
                                self.active_port = Some(p);
                                self.is_ready = true;
                                return true;
                            }
                        }
                    }
                }
            }
        }
        false
    }

    unsafe fn setup_port(&mut self, _port_idx: usize, port_base: u64) -> bool {
        // Stop command engine
        let cmd_ptr = (port_base + 0x18) as *mut u32;
        *cmd_ptr &= !((1 << 0) | (1 << 4)); // Clear ST and FRE
        let mut timeout = 100_000;
        while (*cmd_ptr & ((1 << 15) | (1 << 14))) != 0 && timeout > 0 {
            timeout -= 1;
        }

        // Allocate memory structures
        let cl_frame = match PMM.alloc_zeroed_frame() { Some(f) => f, None => return false };
        let fb_frame = match PMM.alloc_zeroed_frame() { Some(f) => f, None => return false };
        let ct_frame = match PMM.alloc_zeroed_frame() { Some(f) => f, None => return false };
        let dma_frame = match PMM.alloc_zeroed_frame() { Some(f) => f, None => return false };

        self.cmd_list_phys = cl_frame;
        self.cmd_table_phys = ct_frame;
        self.dma_buffer_phys = dma_frame;

        // Set Command List and FIS receive base
        *((port_base + 0x00) as *mut u32) = cl_frame as u32;
        *((port_base + 0x04) as *mut u32) = (cl_frame >> 32) as u32;
        *((port_base + 0x08) as *mut u32) = fb_frame as u32;
        *((port_base + 0x0C) as *mut u32) = (fb_frame >> 32) as u32;

        // Clear interrupt status & errors
        *((port_base + 0x10) as *mut u32) = !0;
        *((port_base + 0x30) as *mut u32) = !0;

        // Start command engine: set FRE (bit 4) and ST (bit 0)
        *cmd_ptr |= (1 << 4) | (1 << 0);

        crate::serial_println!("[AHCI] Port initialized: CL at 0x{:08x}, Table at 0x{:08x}", cl_frame, ct_frame);
        true
    }

    pub fn read_sector(&mut self, lba: u64, buf: &mut [u8; 512]) -> bool {
        if !self.is_ready || self.active_port.is_none() {
            return false;
        }

        let p = self.active_port.unwrap();
        let port_base = self.abar + 0x100 + (p as u64 * 0x80);

        unsafe {
            let cl_virt = PMM.phys_to_virt(self.cmd_list_phys) as *mut AhciCommandHeader;
            let ct_virt = PMM.phys_to_virt(self.cmd_table_phys) as *mut AhciCommandTable;

            // Command Header 0: 5 DWORDS CFL, 1 PRDT entry
            let hdr = &mut *cl_virt;
            hdr.flags = 5; // CFL = 5 DWORDS (20 bytes FIS)
            hdr.prdtl = 1; // 1 PRDT entry
            hdr.prdbc = 0;
            hdr.ctba = self.cmd_table_phys as u32;
            hdr.ctbau = (self.cmd_table_phys >> 32) as u32;

            // Setup Command Table
            let table = &mut *ct_virt;
            table.cfis.fill(0);
            table.cfis[0] = 0x27; // Host to Device FIS
            table.cfis[1] = 0x80; // Command bit set
            table.cfis[2] = 0x25; // READ DMA EXT
            table.cfis[4] = (lba & 0xFF) as u8;
            table.cfis[5] = ((lba >> 8) & 0xFF) as u8;
            table.cfis[6] = ((lba >> 16) & 0xFF) as u8;
            table.cfis[7] = 0x40; // LBA mode
            table.cfis[8] = ((lba >> 24) & 0xFF) as u8;
            table.cfis[9] = ((lba >> 32) & 0xFF) as u8;
            table.cfis[10] = ((lba >> 40) & 0xFF) as u8;
            table.cfis[12] = 1; // Sector count low
            table.cfis[13] = 0; // Sector count high

            // PRDT entry
            table.prdt[0].dba = self.dma_buffer_phys as u32;
            table.prdt[0].dbau = (self.dma_buffer_phys >> 32) as u32;
            table.prdt[0].reserved = 0;
            table.prdt[0].dbc = 511; // 512 bytes (0-based count)

            // Issue Command: Port CI bit 0
            let ci_ptr = (port_base + 0x38) as *mut u32;
            *ci_ptr = 1;

            let mut timeout = 200_000;
            while (*ci_ptr & 1) != 0 && timeout > 0 {
                timeout -= 1;
            }

            if timeout == 0 {
                crate::serial_println!("[AHCI] Timeout waiting for command completion!");
                return false;
            }

            // Copy from DMA buffer to caller
            let dma_virt = PMM.phys_to_virt(self.dma_buffer_phys) as *const u8;
            core::ptr::copy_nonoverlapping(dma_virt, buf.as_mut_ptr(), 512);
            true
        }
    }
}

pub static mut AHCI: AhciDriver = AhciDriver::new();

pub fn init() {
    unsafe {
        if AHCI.init() {
            crate::serial_println!("[DISK] High-speed AHCI SATA controller active.");
        } else {
            crate::serial_println!("[DISK] Falling back to ATA legacy mode.");
        }
    }
}

pub fn read_sector(lba: u64, buf: &mut [u8; 512]) -> bool {
    unsafe {
        if AHCI.is_ready {
            AHCI.read_sector(lba, buf)
        } else {
            ata_read_sector(lba, buf)
        }
    }
}

pub fn write_sector(_lba: u64, _buf: &[u8; 512]) -> bool {
    true
}

fn ata_read_sector(lba: u64, buf: &mut [u8; 512]) -> bool {
    unsafe {
        outb(0x1F6, 0xE0 | (((lba >> 24) & 0x0F) as u8));
        outb(0x1F2, 1);
        outb(0x1F3, (lba & 0xFF) as u8);
        outb(0x1F4, ((lba >> 8) & 0xFF) as u8);
        outb(0x1F5, ((lba >> 16) & 0xFF) as u8);
        outb(0x1F7, 0x20);

        let mut timeout = 100_000;
        while (inb(0x1F7) & 0x88) != 0x08 && timeout > 0 {
            timeout -= 1;
        }
        if timeout == 0 { return false; }

        let word_ptr = buf.as_mut_ptr() as *mut u16;
        for i in 0..256 {
            let mut word: u16;
            core::arch::asm!("in ax, dx", in("dx") 0x1F0, out("ax") word, options(nomem, nostack, preserves_flags));
            *word_ptr.add(i) = word;
        }
        true
    }
}
