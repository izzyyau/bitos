
use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame};


lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
        //create a new interrupt descriptor table on the stack with static life time and initalize 
        //it when firt use it.
        let mut idt = InterruptDescriptorTable::new();
        //add the interrupt handler for interrupt to idt
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        idt.double_fault.set_handler_fn(double_fault_handler);
        idt
    };
}
pub fn init_idt(){
    //let CPU to use the updated IDT,idt has to be valid in the entire program
    //life in order to use load()
    IDT.load();

}
    
//Interrupt handler for interrupt
extern "x86-interrupt" fn breakpoint_handler(
    stack_frame:InterruptStackFrame
){
    println!("Exception: Breakpoint\n{:#?}",stack_frame);
}

//Interrupt handler for Double fault
extern "x86-interrupt" fn double_fault_handler(stack_frame:InterruptStackFrame, _error_code:u64)->! {
    panic!("Exception:Double fault\n{:#?}",stack_frame);

}

//test for breakpoint interrupt
#[test_case]
fn test_breakpoint_handler(){
    //invoke a breakpoint interrupt
    x86_64::instructions::interrupts::int3();
}