#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(bitos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
use bitos::println;




#[no_mangle]
pub extern "C" fn _start() ->! {
    println!("Hello World{}","!");
    //initalize the idt 
    bitos::init();
    //temporaily pause a prgram when the breakpoint instruction int3 is executed.
    //x86_64::instructions::interrupts::int3();


    //cfg is a special conditional compilation, compile code based on flag
    #[cfg(test)]
    test_main();


    loop{}

}


#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
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