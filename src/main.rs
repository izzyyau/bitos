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
use alloc::{boxed::Box,vec,vec::Vec,rc::Rc};
use bitos::task::{Task,simple_executor::SimpleExecutor};
use bitos::task::keyboard; // new
use bitos::task::executor::Executor; // new

entry_point!(kernel_main);
#[no_mangle]
//defind Rust function as entry point
fn kernel_main(boot_info : &'static BootInfo) ->! {
    use x86_64::VirtAddr;
    println!("Hello World{}","!");
    //initalize the idt 
    bitos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);

    use bitos::memory::BootInfoFrameAllocator;
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe{BootInfoFrameAllocator::init(&boot_info.memory_map)} ;
  

    allocator::init_heap(&mut mapper, &mut frame_allocator)
    .expect("heap initialization failed");

    // //allocate a number on heap
    // let heap_value = Box::new(41);
    // //use {:p} formatting specifier to print the underlying heap pointer
    // println!("heap_value at {:p}",heap_value);

    // //create dynamically sized vector
    // let mut vec = Vec::new();
    // for i in 0..500{
    //     vec.push(i);
    // }
    // println!("vec at {:p}",vec.as_slice());

    //create a reference counted vector -> will be freed when count reaches 0
    // let reference_counted = Rc::new(vec![1,2,3]);
    // let cloned_reference = reference_counted.clone();
    // println!(
    //     "current reference count is {}",
    //     Rc::strong_count(&cloned_reference)
    // );
    // core::mem::drop(reference_counted);
    // println!("reference count is {} now", Rc::strong_count(&cloned_reference));


    let mut executor = Executor::new();
    executor.spawn(Task::new(example_task()));
    executor.spawn(Task::new(keyboard::print_keypresses())); // new
    executor.run();






    //cfg is a special conditional compilation, compile code based on flag
    #[cfg(test)]
    test_main();
    println!("It did not crash!");
    bitos::hlt_loop();

}



async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("async number: {}", number);
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