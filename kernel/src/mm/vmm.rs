//! Virtual Memory Manager (VMM)
//! 4-level x86_64 page table management and address space manipulation.

use super::pmm::{PMM, PAGE_SIZE};
use core::arch::asm;

pub const PAGE_PRESENT: u64 = 1 << 0;
pub const PAGE_WRITABLE: u64 = 1 << 1;
pub const PAGE_USER: u64 = 1 << 2;
pub const PAGE_NO_EXECUTE: u64 = 1 << 63;

pub fn read_cr3() -> u64 {
    let value: u64;
    unsafe {
        asm!("mov {}, cr3", out(reg) value, options(nomem, nostack, preserves_flags));
    }
    value & !0xFFF // mask off PCID and flags
}

pub fn write_cr3(val: u64) {
    unsafe {
        asm!("mov cr3, {}", in(reg) val, options(nomem, nostack, preserves_flags));
    }
}

pub fn invlpg(virt: u64) {
    unsafe {
        asm!("invlpg [{}]", in(reg) virt, options(nostack, preserves_flags));
    }
}

#[repr(C, align(4096))]
pub struct PageTable {
    pub entries: [u64; 512],
}

impl PageTable {
    pub fn zero(&mut self) {
        self.entries.fill(0);
    }
}

pub struct VirtualMemoryManager {
    kernel_pml4: u64,
}

impl VirtualMemoryManager {
    pub const fn new() -> Self {
        Self { kernel_pml4: 0 }
    }

    pub fn init(&mut self) {
        self.kernel_pml4 = read_cr3();
        crate::serial_println!("[VMM] Kernel active PML4: 0x{:016x}", self.kernel_pml4);
    }

    pub fn kernel_pml4(&self) -> u64 {
        self.kernel_pml4
    }

    pub fn map_page(&mut self, pml4_phys: u64, virt: u64, phys: u64, flags: u64) -> Result<(), &'static str> {
        let pml4_idx = ((virt >> 39) & 0x1FF) as usize;
        let pdpt_idx = ((virt >> 30) & 0x1FF) as usize;
        let pd_idx   = ((virt >> 21) & 0x1FF) as usize;
        let pt_idx   = ((virt >> 12) & 0x1FF) as usize;

        unsafe {
            // PML4 -> PDPT
            let pml4 = &mut *(PMM.phys_to_virt(pml4_phys) as *mut PageTable);
            let pdpt_phys = if (pml4.entries[pml4_idx] & PAGE_PRESENT) == 0 {
                let frame = PMM.alloc_zeroed_frame().ok_or("Out of memory for PDPT")?;
                pml4.entries[pml4_idx] = frame | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
                frame
            } else {
                pml4.entries[pml4_idx] |= flags & PAGE_USER;
                pml4.entries[pml4_idx] & !0xFFF
            };

            // PDPT -> PD
            let pdpt = &mut *(PMM.phys_to_virt(pdpt_phys) as *mut PageTable);
            let pd_phys = if (pdpt.entries[pdpt_idx] & PAGE_PRESENT) == 0 {
                let frame = PMM.alloc_zeroed_frame().ok_or("Out of memory for PD")?;
                pdpt.entries[pdpt_idx] = frame | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
                frame
            } else {
                pdpt.entries[pdpt_idx] |= flags & PAGE_USER;
                pdpt.entries[pdpt_idx] & !0xFFF
            };

            // PD -> PT
            let pd = &mut *(PMM.phys_to_virt(pd_phys) as *mut PageTable);
            let pt_phys = if (pd.entries[pd_idx] & PAGE_PRESENT) == 0 {
                let frame = PMM.alloc_zeroed_frame().ok_or("Out of memory for PT")?;
                pd.entries[pd_idx] = frame | PAGE_PRESENT | PAGE_WRITABLE | (flags & PAGE_USER);
                frame
            } else {
                pd.entries[pd_idx] |= flags & PAGE_USER;
                pd.entries[pd_idx] & !0xFFF
            };

            // PT -> Page
            let pt = &mut *(PMM.phys_to_virt(pt_phys) as *mut PageTable);
            pt.entries[pt_idx] = (phys & !0xFFF) | flags | PAGE_PRESENT;

            invlpg(virt);
        }
        Ok(())
    }

    pub fn create_user_address_space(&mut self) -> Option<u64> {
        unsafe {
            let user_pml4_phys = PMM.alloc_zeroed_frame()?;
            let user_pml4 = &mut *(PMM.phys_to_virt(user_pml4_phys) as *mut PageTable);
            let kern_pml4 = &*(PMM.phys_to_virt(self.kernel_pml4) as *const PageTable);

            // Copy upper half (indices 256..512) for higher-half kernel and HHDM
            for i in 256..512 {
                user_pml4.entries[i] = kern_pml4.entries[i];
            }

            Some(user_pml4_phys)
        }
    }
}

pub static mut VMM: VirtualMemoryManager = VirtualMemoryManager::new();
