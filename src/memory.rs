use x86_64::{
    structures::paging::{PageTable, Page,PhysFrame,Mapper,Size4KiB,FrameAllocator},
    VirtAddr,
    PhysAddr,

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
    
    /// Creates an example mapping for the given page to frame `0xb8000`.
pub fn create_example_mapping(
    page: Page,
    mapper: &mut OffsetPageTable,
    frame_allocator: &mut impl FrameAllocator<Size4KiB>,
) {
    use x86_64::structures::paging::PageTableFlags as Flags;

    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;

    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };
    map_to_result.expect("map_to failed").flush();
}

use bootloader::bootinfo::MemoryMap;

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    //memory_map consists a list of MemoryRegion structs, which contain the start
    //address,the length, ant the type of each memory region or memory frame
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.

    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            //increased by every time allocating frame
            next: 0,
        }
    }
}


use bootloader::bootinfo::MemoryRegionType;

impl BootInfoFrameAllocator {
    /// Converts the memory map into an iterator of usable physical memory frame.
    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame> {
        // First,call the iter method to convert the memory map to an iterator of MemoryReigons
        let regions = self.memory_map.iter();
        // Second, use filter method to skip any reserved or unavailable regions.
        let usable_regions = regions
            .filter(|r| r.region_type == MemoryRegionType::Usable);
        // Afterwards, we use the map combinator and Rust’s range syntax to transform our iterator of memory regions to an iterator of address ranges.
        let addr_ranges = usable_regions
            .map(|r| r.range.start_addr()..r.range.end_addr());
        // Then, transform to an iterator of frame start addresses,choose every 4096th address.
        // because 4096 bytes is the page size.
        let frame_addresses = addr_ranges.flat_map(|r| r.step_by(4096));
        // Finally, convert the start addresses to PhysFrame types to construct an Iterator<Item = PhysFrame>
        frame_addresses
            .map(|addr|PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
