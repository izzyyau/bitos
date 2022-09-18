#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(bitos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bitos::println;
use bitos::memory;
use bitos::allocator;
use bootloader::{BootInfo,entry_point};
use x86_64::structures::paging::frame;
use x86_64::structures::paging::{Translate,Page};
extern crate alloc;
use alloc::boxed::Box;

entry_point!(kernel_main);
#[no_mangle]
//defind Rust function as entry point
fn kernel_main(boot_info : &'static BootInfo) ->! {
    use x86_64::VirtAddr;
    use x86_64::structures::paging::mapper::MapperAllSizes;
    println!("Hello World{}","!");
    //initalize the idt 
    bitos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    use bitos::memory::BootInfoFrameAllocator;
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe{BootInfoFrameAllocator::init(&boot_info.memory_map)} ;
    let x = Box::new(41);
    // // map an unused page
    // let page = Page::containing_address(VirtAddr::new(0));
    // memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);
    // // write the string `New!` to the screen through the new mapping
    // let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    // //The “New!” on the screen is caused by our write to page 0, 
    // //which means that we successfully created a new mapping in the page tables.
    // unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e)};

    allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("heap initialization failed");
    //cfg is a special conditional compilation, compile code based on flag
    #[cfg(test)]
    test_main();
    println!("It did not crash!");
    bitos::hlt_loop();

}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    bitos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bitos::test_panic_handler(info)
}
#[test_case]
fn trivial_assertion() {
    assert_eq!(1, 1);
}