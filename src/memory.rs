use bootloader::bootinfo::{MemoryMap, MemoryRegionType};
use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::{MappedPageTable, MapperAllSizes, PageTable, PhysFrame};
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, Size4KiB};
use x86_64::structures::paging::page_table::PageTableEntry;

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn init(physical_memory_offset: u64) -> impl MapperAllSizes {
    let level_4_table = active_level_4_page_table(physical_memory_offset);
    let phys_to_virtual = move |frame: PhysFrame| -> *mut PageTable{
        let phys = frame.start_address().as_u64();
        let virt = VirtAddr::new(phys + physical_memory_offset);
        virt.as_mut_ptr()
    };

    MappedPageTable::new(level_4_table, phys_to_virtual)
}

// traverse the multi-level page table
// use the index of the corresponding page
// table entry to get the next table
// We're just adding an offset (from Cargo.toml -> bootloader) to
// form the virtual address space, what we're trying to achieve
// here is to get the virtual address of the next page table entry
// based on its physical address and the memory offset, as the kernel
// can't write to physical addresses directly when paging is enabled
unsafe fn active_level_4_page_table(physical_memory_offset: u64) -> &'static mut PageTable {
    use x86_64::registers::control::Cr3;

    // Physical address frame (we can't modify the contents of a physcial address
    // since paging is enabled.
    let (level_4_page_table_frame, _) = Cr3::read();

    let phys = level_4_page_table_frame.start_address();
    // So we need to get the virtual address of the level 4 page table
    // and modify page tables using the virtual addresses
    let virt = VirtAddr::new(phys.as_u64() + physical_memory_offset);
    let page_table_ptr: *mut PageTable = virt.as_mut_ptr();

    &mut *page_table_ptr
}

pub fn create_example_mapping(page: Page, mapper: &mut impl Mapper<Size4KiB>, frame_allocator: &mut impl FrameAllocator<Size4KiB>) {
    use x86_64::structures::paging::PageTableFlags as Flags;
    let frame = PhysFrame::containing_address(PhysAddr::new(0xb8000));
    let flags = Flags::PRESENT | Flags::WRITABLE;
    let map_to_result = unsafe {
        mapper.map_to(page, frame, flags, frame_allocator)
    };

    map_to_result.expect("Failed to make page mapping").flush();
}

/// A FrameAllocator that returns usable frames from the bootloader's memory map.
pub struct BootInfoFrameAllocator {
    memory_map: &'static MemoryMap,
    next: usize,
}

impl BootInfoFrameAllocator {
    /// Create a FrameAllocator from the passed memory map.
    ///
    /// This function is unsafe because the caller must guarantee that the passed
    /// memory map is valid. The main requirement is that all frames that are marked
    /// as `USABLE` in it are really unused.
    pub unsafe fn init(memory_map: &'static MemoryMap) -> Self {
        BootInfoFrameAllocator {
            memory_map,
            next: 0,
        }
    }

    /// Returns an iterator over the usable frames specified in the memory map.
    fn usable_frames(&self) -> impl Iterator<Item=PhysFrame> {
        let regions = self.memory_map.iter();
        let usable = regions
            .filter(|&x| x.region_type == MemoryRegionType::Usable);

        let addresses = usable
            .map(|x| x.range.start_addr()..x.range.end_addr());

        // Page start (physical) addresses
        let frame_addresses = addresses
            .flat_map(|x| x.step_by(4096));

        // create `PhysFrame` types from the start addresses
        frame_addresses
            .map(|addr| PhysFrame::containing_address(PhysAddr::new(addr)))
    }
}

unsafe impl FrameAllocator<Size4KiB> for BootInfoFrameAllocator {
    fn allocate_frame(&mut self) -> Option<PhysFrame> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
