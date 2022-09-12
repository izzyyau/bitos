

use crate::println;
use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable,InterruptStackFrame};
use crate::gdt;
//ChainedPics:Primary and Secondary Interrupt Controller
use pic8259::ChainedPics;
use spin;

#[derive(Debug,Clone,Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer  = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex{
    fn as_u8(self) -> u8{
        self as u8
    }

    fn as_usize(self) -> usize{
        usize::from(self.as_u8())
    }

}


pub const PIC_1_OFFSET : u8 = 32;
pub const PIC_2_OFFSET : u8 = PIC_1_OFFSET + 8;
pub static PICS:spin::Mutex<ChainedPics> = spin::Mutex::new(unsafe{ChainedPics::new(PIC_1_OFFSET,PIC_2_OFFSET)});



lazy_static!{
    static ref IDT: InterruptDescriptorTable = {
        //create a new interrupt descriptor table on the stack with static life time and initalize 
        //it when firt use it.
        let mut idt = InterruptDescriptorTable::new();
        //add the interrupt handler for interrupt to idt
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        //tell CPU to use double fault stack when double fault occur
        unsafe{
            idt.double_fault.set_handler_fn(double_fault_handler).set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        } 
        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
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



extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame:InterruptStackFrame){
    println!(".");
    unsafe{
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame:InterruptStackFrame){
    println!("k");
    use x86_64::instructions::port::Port;
    let mut port = Port::new(0x60);
    let scancode:u8 = unsafe{port.read()};

    let key = match scancode{
        0x02 => Some('1'),
        0x03 => Some('2'),
        0x04 => Some('3'),
        0x05 => Some('4'),
        0x06 => Some('5'),
        0x07 => Some('6'),
        0x08 => Some('7'),
        0x09 => Some('8'),
        0x0a => Some('9'),
        0x0b => Some('0'),
        _ => None,
    };

    if let Some(key) = key{
        println!("{}", key);
    }
    //notify PIC the interrupt handler for current interrupt has done its job
    unsafe{
        PICS.lock().notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }

}






//test for breakpoint interrupt
#[test_case]
fn test_breakpoint_handler(){
    //invoke a breakpoint interrupt
    x86_64::instructions::interrupts::int3();
}