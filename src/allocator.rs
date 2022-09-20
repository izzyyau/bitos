use alloc::alloc::{GlobalAlloc,Layout};
use core::ptr::null_mut;
use x86_64::{
    structures::paging::{
        mapper::MapToError, FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB,
    },
    VirtAddr,
};
use linked_list_allocator::LockedHeap;
use fixed_size_block::FixedSizeBlockAllocator;

#[global_allocator]
static ALLOCATOR: Locked<FixedSizeBlockAllocator> = Locked::new(
    FixedSizeBlockAllocator::new());

//create a kernel heap, first we define a virtual memory region for heap.
pub const HEAP_START: usize = 0x_4444_4444_0000;
pub const HEAP_SIZE: usize = 100 * 1024; // 100 KiB

pub struct Dummy;
pub mod bump;
pub mod linked_list;
pub mod fixed_size_block;
//unsafe reason: alloc method never return a memory block that have already been used
//GlobalAlloc trait define the functions that a heap allocator must provide. We can 
//implement it by ourself.
unsafe impl GlobalAlloc for Dummy {
    unsafe fn alloc(&self, _layout: Layout) -> *mut u8 {
        //allcator never return any memory
        null_mut()
    }

    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        panic!("dealloc should be never called")
    }
}

//initalize kernel heap

pub fn init_heap(
    mapper: &mut impl Mapper<Size4KiB>,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) -> Result<(), MapToError<Size4KiB>>{
    let page_range = {
    let heap_start = VirtAddr::new(HEAP_START as u64);
    //We want the inclusive bound, so we subtract by 1.
    let heap_end = heap_start + HEAP_SIZE -1u64;
    //Convert the address into Page tyes
    let heap_start_page = Page::containing_address(heap_start);
    let heap_end_page = Page::containing_address(heap_end);
    Page::range_inclusive(heap_start_page, heap_end_page)
    };
    for page in page_range{
        //Use Option::ok_or method to deal with if there is no frame left to be allocated.
        //Use ? to return early in the case of error
        let frame = frame_allocator
            .allocate_frame()
            .ok_or(MapToError::FrameAllocationFailed)?;
        let flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
        unsafe{
            mapper.map_to(page, frame, flags, frame_allocator)?.flush()
        };
    }
    //initalize linked_list_allocator
    unsafe {
        ALLOCATOR.lock().init(HEAP_START, HEAP_SIZE);
    }

    Ok(())
    
}

/// A wrapper around spin::Mutex to permit trait implementations.
pub struct Locked<A> {
    inner: spin::Mutex<A>,
}

impl<A> Locked<A> {
    pub const fn new(inner: A) -> Self {
        Locked {
            inner: spin::Mutex::new(inner),
        }
    }

    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.inner.lock()
    }
}

fn align_up(addr: usize, align: usize) -> usize {
    (addr + align - 1) & !(align - 1)
}
