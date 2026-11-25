//! Kernel Heap Allocator with Bidirectional Free-Block Coalescing
//! Implements core::alloc::GlobalAlloc to enable `extern crate alloc` (Vec, Box, String, etc.)

use core::alloc::{GlobalAlloc, Layout};
use core::sync::atomic::{AtomicUsize, Ordering};

pub const HEAP_START: usize = 0x00400000; // 4 MB
pub const HEAP_SIZE: usize  = 16 * 1024 * 1024; // 16 MB

#[repr(C)]
struct BlockHeader {
    size: usize,        // Total size of block including this header
    is_free: bool,
    next: *mut BlockHeader,
}

pub struct KernelAllocator {
    head: AtomicUsize,
    allocated_bytes: AtomicUsize,
    total_alloc_count: AtomicUsize,
}

unsafe impl Sync for KernelAllocator {}

#[global_allocator]
pub static ALLOCATOR: KernelAllocator = KernelAllocator {
    head: AtomicUsize::new(0),
    allocated_bytes: AtomicUsize::new(0),
    total_alloc_count: AtomicUsize::new(0),
};

impl KernelAllocator {
    pub unsafe fn init(&self, start: usize, size: usize) {
        let initial_block = start as *mut BlockHeader;
        (*initial_block).size = size;
        (*initial_block).is_free = true;
        (*initial_block).next = core::ptr::null_mut();
        self.head.store(start, Ordering::SeqCst);
    }

    pub fn allocated_bytes(&self) -> usize {
        self.allocated_bytes.load(Ordering::Relaxed)
    }

    pub fn free_bytes(&self) -> usize {
        HEAP_SIZE.saturating_sub(self.allocated_bytes())
    }

    pub fn total_allocations(&self) -> usize {
        self.total_alloc_count.load(Ordering::Relaxed)
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let align = layout.align().max(core::mem::align_of::<BlockHeader>());
        let needed_payload = layout.size();
        let header_size = core::mem::size_of::<BlockHeader>();
        
        let mut curr = self.head.load(Ordering::SeqCst) as *mut BlockHeader;
        
        while !curr.is_null() {
            if (*curr).is_free {
                let payload_addr = (curr as usize) + header_size;
                let aligned_payload = (payload_addr + align - 1) & !(align - 1);
                let padding = aligned_payload - payload_addr;
                let total_needed = header_size + padding + needed_payload;

                if (*curr).size >= total_needed {
                    // Check if block can be split
                    let remainder = (*curr).size - total_needed;
                    if remainder >= header_size + 32 {
                        let new_block = ((curr as usize) + total_needed) as *mut BlockHeader;
                        (*new_block).size = remainder;
                        (*new_block).is_free = true;
                        (*new_block).next = (*curr).next;

                        (*curr).size = total_needed;
                        (*curr).next = new_block;
                    }

                    (*curr).is_free = false;
                    self.allocated_bytes.fetch_add((*curr).size, Ordering::Relaxed);
                    self.total_alloc_count.fetch_add(1, Ordering::Relaxed);
                    return aligned_payload as *mut u8;
                }
            }
            curr = (*curr).next;
        }

        core::ptr::null_mut() // Out of memory
    }

    unsafe fn dealloc(&self, ptr: *mut u8, _layout: Layout) {
        if ptr.is_null() { return; }

        let mut curr = self.head.load(Ordering::SeqCst) as *mut BlockHeader;
        let target_addr = ptr as usize;

        while !curr.is_null() {
            let block_start = curr as usize;
            let block_end = block_start + (*curr).size;

            if target_addr >= block_start && target_addr < block_end {
                if !(*curr).is_free {
                    (*curr).is_free = true;
                    self.allocated_bytes.fetch_sub((*curr).size, Ordering::Relaxed);

                    // Coalesce adjacent free blocks
                    self.coalesce();
                }
                return;
            }
            curr = (*curr).next;
        }
    }
}

impl KernelAllocator {
    unsafe fn coalesce(&self) {
        let mut curr = self.head.load(Ordering::SeqCst) as *mut BlockHeader;
        while !curr.is_null() && !(*curr).next.is_null() {
            let next = (*curr).next;
            if (*curr).is_free && (*next).is_free {
                // If adjacent in memory
                if (curr as usize) + (*curr).size == (next as usize) {
                    (*curr).size += (*next).size;
                    (*curr).next = (*next).next;
                    continue;
                }
            }
            curr = (*curr).next;
        }
    }
}

// Allocator shims required by bare-metal Rust
#[no_mangle]
static __rust_no_alloc_shim_is_unstable: u8 = 0;

#[no_mangle]
pub static __rust_alloc_error_handler_should_panic: u8 = 0;

#[no_mangle]
pub unsafe extern "C" fn __rust_alloc_error_handler(size: usize, align: usize) -> ! {
    panic!("Out of memory error: size={}, align={}", size, align);
}
