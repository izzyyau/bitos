use x86_64::VirtAddr;
//TaskStateSegment:Find the kernel level stack if the interrupt arrives in kernel mode
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;

pub const DOUBLE_FAULT_IST_INDEX:u16 = 0;

lazy_static!{
    pub static ref TSS:TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        //define the 0th IST entry as the double fault stack, if kernel hit double fault, CPU would first
        //switch to this stack. 
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            const STACK_SIZE: usize = 4096 *5;
            //bootloader would mapped the stack to read-only page
            static mut STACK: [u8;STACK_SIZE] = [0;STACK_SIZE];
            //create a virtual address by the given pointer
            let stack_start = VirtAddr::from_ptr(unsafe {&STACK});
            let stack_end = stack_start + STACK_SIZE;
            stack_end
        };
        tss
    };
}