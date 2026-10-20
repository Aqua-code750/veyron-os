//! Kernel Heap Allocator
//! Implements a linked-list free block allocator and GlobalAlloc for Rust collections.

use core::alloc::{GlobalAlloc, Layout};
use core::ptr::null_mut;
use super::pmm::{PMM, PAGE_SIZE};

const HEAP_INITIAL_PAGES: usize = 4096; // 16 MB initial contiguous heap

#[repr(C)]
struct BlockHeader {
    size: usize,
    is_free: bool,
    next: *mut BlockHeader,
}

pub struct KernelAllocator {
    head: *mut BlockHeader,
    initialized: bool,
}

unsafe impl Sync for KernelAllocator {}
unsafe impl Send for KernelAllocator {}

impl KernelAllocator {
    pub const fn new() -> Self {
        Self {
            head: null_mut(),
            initialized: false,
        }
    }

    pub fn init(&mut self) {
        unsafe {
            if let Some(phys) = PMM.alloc_contiguous_frames(HEAP_INITIAL_PAGES) {
                let virt = PMM.phys_to_virt(phys) as *mut u8;
                let total_bytes = HEAP_INITIAL_PAGES * PAGE_SIZE;

                // Zero out heap
                core::ptr::write_bytes(virt, 0, total_bytes);

                let header = virt as *mut BlockHeader;
                (*header).size = total_bytes - core::mem::size_of::<BlockHeader>();
                (*header).is_free = true;
                (*header).next = null_mut();

                self.head = header;
                self.initialized = true;

                crate::serial_println!(
                    "[HEAP] Kernel heap initialized at 0x{:016x} ({} MB contiguous)",
                    virt as u64,
                    total_bytes / (1024 * 1024)
                );
            } else {
                crate::serial_println!("[HEAP] Out of memory: could not allocate 16 MB contiguous heap!");
            }
        }
    }

    pub unsafe fn allocate(&mut self, layout: Layout) -> *mut u8 {
        if !self.initialized || self.head.is_null() {
            return null_mut();
        }

        let align = layout.align().max(16);
        let needed_size = (layout.size() + align - 1) & !(align - 1);
        let header_size = core::mem::size_of::<BlockHeader>();

        let mut curr = self.head;
        while !curr.is_null() {
            let block = &mut *curr;
            if block.is_free && block.size >= needed_size {
                if block.size >= needed_size + header_size + 32 {
                    let new_block_ptr = (curr as usize + header_size + needed_size) as *mut BlockHeader;
                    (*new_block_ptr).size = block.size - needed_size - header_size;
                    (*new_block_ptr).is_free = true;
                    (*new_block_ptr).next = block.next;

                    block.size = needed_size;
                    block.is_free = false;
                    block.next = new_block_ptr;
                } else {
                    block.is_free = false;
                }

                let data_ptr = (curr as usize + header_size) as *mut u8;
                return data_ptr;
            }
            curr = block.next;
        }

        null_mut()
    }

    pub unsafe fn deallocate(&mut self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() || self.head.is_null() {
            return;
        }

        let header_size = core::mem::size_of::<BlockHeader>();
        let block_ptr = (ptr as usize - header_size) as *mut BlockHeader;
        (*block_ptr).is_free = true;

        let mut curr = self.head;
        while !curr.is_null() {
            let next = (*curr).next;
            if !next.is_null() && (*curr).is_free && (*next).is_free {
                let expected_next = curr as usize + header_size + (*curr).size;
                if expected_next == next as usize {
                    (*curr).size += header_size + (*next).size;
                    (*curr).next = (*next).next;
                    continue;
                }
            }
            curr = (*curr).next;
        }
    }
}

pub struct LockedHeap;

#[global_allocator]
pub static GLOBAL_ALLOCATOR: LockedHeap = LockedHeap;

pub static mut KERNEL_HEAP: KernelAllocator = KernelAllocator::new();

unsafe impl GlobalAlloc for LockedHeap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        KERNEL_HEAP.allocate(layout)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        KERNEL_HEAP.deallocate(ptr, layout)
    }
}
