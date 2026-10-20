//! Physical Memory Manager (PMM)
//! Bitmap-based page frame allocator tracking 4KB physical pages.

use crate::serial_println;

pub const PAGE_SIZE: usize = 4096;
const MAX_FRAMES: usize = 524_288; // 2 GB / 4 KB = 524,288 pages
const BITMAP_SIZE: usize = MAX_FRAMES / 64; // 8,192 u64 words = 64 KB

pub struct PhysicalMemoryManager {
    bitmap: [u64; BITMAP_SIZE],
    total_frames: usize,
    used_frames: usize,
    hhdm_offset: u64,
}

impl PhysicalMemoryManager {
    pub const fn new() -> Self {
        Self {
            bitmap: [!0u64; BITMAP_SIZE],
            total_frames: 0,
            used_frames: 0,
            hhdm_offset: 0,
        }
    }

    pub fn set_hhdm(&mut self, offset: u64) {
        self.hhdm_offset = offset;
    }

    pub fn hhdm_offset(&self) -> u64 {
        self.hhdm_offset
    }

    #[inline]
    pub fn phys_to_virt(&self, phys: u64) -> u64 {
        phys + self.hhdm_offset
    }

    #[inline]
    pub fn virt_to_phys(&self, virt: u64) -> u64 {
        virt - self.hhdm_offset
    }

    pub fn init(&mut self, entries: &[crate::LimineMemmapEntry]) {
        for word in self.bitmap.iter_mut() {
            *word = !0u64;
        }

        let mut max_phys = 0u64;

        for entry in entries {
            let base = entry.base;
            let length = entry.length;
            let typ = entry.typ;

            if base + length > max_phys {
                max_phys = base + length;
            }

            // LIMINE_MEMMAP_USABLE == 0
            // Also skip low memory below 1MB (0x100000) to keep real-mode/BIOS tables intact
            if typ == 0 && base >= 0x100000 {
                let start_frame = (base / PAGE_SIZE as u64) as usize;
                let frame_count = (length / PAGE_SIZE as u64) as usize;

                for f in start_frame..start_frame + frame_count {
                    if f < MAX_FRAMES {
                        self.clear_bit(f);
                    }
                }
            }
        }

        self.total_frames = (max_phys / PAGE_SIZE as u64).min(MAX_FRAMES as u64) as usize;
        self.used_frames = 0;
        for f in 0..self.total_frames {
            if self.test_bit(f) {
                self.used_frames += 1;
            }
        }

        serial_println!(
            "[PMM] Total RAM: {} MB, Usable: {} MB ({} free frames)",
            (self.total_frames * PAGE_SIZE) / (1024 * 1024),
            ((self.total_frames - self.used_frames) * PAGE_SIZE) / (1024 * 1024),
            self.total_frames - self.used_frames
        );
    }

    #[inline]
    fn test_bit(&self, frame: usize) -> bool {
        let idx = frame / 64;
        let bit = frame % 64;
        (self.bitmap[idx] & (1 << bit)) != 0
    }

    #[inline]
    fn set_bit(&mut self, frame: usize) {
        let idx = frame / 64;
        let bit = frame % 64;
        self.bitmap[idx] |= 1 << bit;
    }

    #[inline]
    fn clear_bit(&mut self, frame: usize) {
        let idx = frame / 64;
        let bit = frame % 64;
        self.bitmap[idx] &= !(1 << bit);
    }

    pub fn alloc_frame(&mut self) -> Option<u64> {
        for idx in 0..(self.total_frames / 64) {
            if self.bitmap[idx] != !0u64 {
                let free_bit = (!self.bitmap[idx]).trailing_zeros() as usize;
                let frame = idx * 64 + free_bit;
                if frame < self.total_frames {
                    self.set_bit(frame);
                    self.used_frames += 1;
                    return Some((frame * PAGE_SIZE) as u64);
                }
            }
        }
        None
    }

    pub fn alloc_contiguous_frames(&mut self, count: usize) -> Option<u64> {
        let mut consecutive = 0;
        let mut start_frame = 0;

        for f in 256..self.total_frames { // Start above 1 MB
            if !self.test_bit(f) {
                if consecutive == 0 {
                    start_frame = f;
                }
                consecutive += 1;
                if consecutive == count {
                    for i in start_frame..start_frame + count {
                        self.set_bit(i);
                    }
                    self.used_frames += count;
                    return Some((start_frame * PAGE_SIZE) as u64);
                }
            } else {
                consecutive = 0;
            }
        }
        None
    }

    pub fn alloc_zeroed_frame(&mut self) -> Option<u64> {
        if let Some(phys) = self.alloc_frame() {
            let virt = self.phys_to_virt(phys) as *mut u8;
            unsafe {
                core::ptr::write_bytes(virt, 0, PAGE_SIZE);
            }
            Some(phys)
        } else {
            None
        }
    }

    pub fn free_frame(&mut self, phys: u64) {
        let frame = (phys as usize) / PAGE_SIZE;
        if frame < self.total_frames && self.test_bit(frame) {
            self.clear_bit(frame);
            self.used_frames -= 1;
        }
    }

    pub fn total_memory(&self) -> usize {
        self.total_frames * PAGE_SIZE
    }

    pub fn free_memory(&self) -> usize {
        (self.total_frames - self.used_frames) * PAGE_SIZE
    }

    pub fn used_memory(&self) -> usize {
        self.used_frames * PAGE_SIZE
    }
}

pub static mut PMM: PhysicalMemoryManager = PhysicalMemoryManager::new();
