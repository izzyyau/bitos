use x86_64::{
    structures::paging::{PageTable, Page},
    VirtAddr,
};
use crate::println;
//Three things we need to do 
//1. access the current active page table 
//2. translate the virtual address to physical address 
//3. modify the page table in order to create a new mapping

 unsafe fn active_level_4_table(physical_memory_offset:VirtAddr)
-> &'static mut PageTable{
    use x86_64::registers::control::Cr3;

    //get the active physical frame of level 4 page table
    let(level_4_table_frame, _) = Cr3::read();

    let phys = level_4_table_frame.start_address();
    let virt = physical_memory_offset + phys.as_u64();
    //get the raw pointer of Page table
    let page_table_ptr:*mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

use x86_64::PhysAddr;
pub unsafe fn translate_addr(addr:VirtAddr, physical_memory_offset:VirtAddr) 
-> Option<PhysAddr>{
    translate_addr_inner(addr,physical_memory_offset)
}

fn translate_addr_inner(addr:VirtAddr,physical_memory_offset:VirtAddr)->Option<PhysAddr>{
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    //read the active level 4 frame from CR3 register
    let(level_4_table_frame, _) = Cr3::read();
    println!("This level 4 table frame is :{:?}",level_4_table_frame);

    //addr is the 64bit address space for each program and VirAddr struct has already 
    //included method to compute the index for each level table
    let table_indexes = [addr.p4_index(),addr.p3_index(),addr.p2_index(),addr.p1_index()];
    let mut frame = level_4_table_frame;


    //traverse multilevel page table
    for &index in &table_indexes{
        println!("index is :{:?}",&index);
        // in 64 bit address space, the lower 12 bits are not translated because it is the page offset
        // virtual address offset is the same as physical address offset
        let virt = physical_memory_offset + frame.start_address().as_u64();
        let table_ptr : *const PageTable = virt.as_ptr();
        //get the corresponding level 4,level 3,level 2,level1 table reference
        let table = unsafe {&*table_ptr};
        //Read the page table entry in the corresponding page table
        let entry = &table[index];
        //get the physical frame in the page table entry, load the next level page table 
        //from physical page table frame
        frame = match entry.frame(){
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("huge pages not supported"),
        };
    }

        

    //calculate the physical address by adding the page offset
    Some(frame.start_address() + u64::from(addr.page_offset()))


    }

    use x86_64::structures::paging::OffsetPageTable;
    //OffsetPageTable
    pub unsafe fn init(physical_memory_offset: VirtAddr) -> OffsetPageTable<'static> {
        //get the mutable reference of level 4 table
        let level_4_table = active_level_4_table(physical_memory_offset);
        //physical_memory_offset: this is where virtual address stared to map physical address
        OffsetPageTable::new(level_4_table, physical_memory_offset)
        
    }
    