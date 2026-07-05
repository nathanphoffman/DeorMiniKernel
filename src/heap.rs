use core::alloc::{GlobalAlloc, Layout};
use core::cell::UnsafeCell;

const HEAP_SIZE: usize = 1024 * 1024;

#[repr(align(16))]
struct HeapSpace([u8; HEAP_SIZE]);
static mut HEAP_SPACE: HeapSpace = HeapSpace([0; HEAP_SIZE]);

struct BumpAllocator {
    next: UnsafeCell<usize>,
}

unsafe impl Sync for BumpAllocator {}

unsafe impl GlobalAlloc for BumpAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let next = self.next.get();
        let start = align_up(*next, layout.align());
        let end = start + layout.size();
        if end > HEAP_SIZE {
            return core::ptr::null_mut();
        }
        *next = end;
        HEAP_SPACE.0.as_mut_ptr().add(start)
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {}
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}

#[global_allocator]
static ALLOCATOR: BumpAllocator = BumpAllocator {
    next: UnsafeCell::new(0),
};
