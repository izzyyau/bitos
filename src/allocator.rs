use alloc::alloc::{GlobalAlloc,Layout};
use core::ptr::null_mut;

pub struct Dummy;

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