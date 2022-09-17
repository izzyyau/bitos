#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(bitos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bitos::println;



use bootloader::{BootInfo,entry_point};
entry_point!(kernel_main);
#[no_mangle]
//defind Rust function as entry point
fn kernel_main(boot_info : &'static BootInfo) ->! {
    use x86_64::VirtAddr;
    use bitos::memory::active_level_4_table;
    println!("Hello World{}","!");
    //initalize the idt 
    bitos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let l4_table = unsafe{active_level_4_table(phys_mem_offset)};
    for(i,entry) in l4_table.iter().enumerate(){
        if!entry.is_unused(){
            println!("L4 Entry {} : {:?}", i, entry);
        }
    }


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