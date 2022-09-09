use x86_64::VirtAddr;
//TaskStateSegment:Find the kernel level stack if the interrupt arrives in kernel mode
use x86_64::structures::tss::TaskStateSegment;
use lazy_static::lazy_static;
use x86_64::structures::gdt::{GlobalDescriptorTable,Descriptor};
use x86_64::structures::gdt::SegmentSelector;


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

    //create a GDT which include the static tts
lazy_static!{
    static ref GDT:(GlobalDescriptorTable,Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code_selector = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss_selector = gdt.add_entry(Descriptor::tss_segment(&TSS));
        (gdt, Selectors{code_selector,tss_selector})


        };
    }

struct Selectors{
    code_selector:SegmentSelector,
    tss_selector:SegmentSelector,

}

pub fn init(){
    use x86_64::instructions::tables::load_tss;
    use x86_64::instructions::segmentation::{CS,Segment};

    
    GDT.0.load();
    unsafe{
        //load code segment register
        //load TSS, now CPU has access to a valid interrupt stack table
        CS::set_reg(GDT.1.code_selector);
        load_tss(GDT.1.tss_selector);
    }

}
