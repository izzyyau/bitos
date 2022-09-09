#![no_std]
#![no_main]
#![feature(abi_x86_interrupt)]

use core::panic::PanicInfo;
use bitos::{serial_println, QemuExitCode,exit_qemu,};
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame};


#[no_mangle]
pub extern "C" fn _start() -> !{
    serial_println!("stack_overflow::stack_overflow...\t");
    bitos::gdt::init();
    //test need its own idt with a custom double fault handler
    init_test_idt();

    stack_overflow();
    panic!("Executation continued after stack overflow");

}

#[allow(unconditional_recursion)]
fn stack_overflow(){
    stack_overflow();
    //prevert compiler to do the tail call elimination
    //optimize the function call to loop, that is not what we want
    //we want stack overflow. 
    volatile::Volatile::new(0).read();
}

//construct a TEST_IDT
lazy_static!{
    static ref TEST_IDT:InterruptDescriptorTable={
        let mut idt = InterruptDescriptorTable::new();
        unsafe{
            idt.double_fault.set_handler_fn(test_double_fault_handler).set_stack_index(
                bitos::gdt::DOUBLE_FAULT_IST_INDEX);
            
        }
        idt
    };
}

pub fn init_test_idt(){
    TEST_IDT.load();
}

extern "x86-interrupt" fn test_double_fault_handler(_stack_frame:InterruptStackFrame,
    _error_code:u64)-> !{
        serial_println!("[ok]");
        exit_qemu(QemuExitCode::Success);
        loop{}

}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    bitos::test_panic_handler(info)
}