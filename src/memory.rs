use x86_64::{PhysAddr, VirtAddr};
use x86_64::structures::paging::page_table::PageTableEntry;
use x86_64::structures::paging::PageTable;

/// Returns a mutable reference to the active level 4 table.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`. Also, this function must be only called once
/// to avoid aliasing `&mut` references (which is undefined behavior).
pub unsafe fn active_level_4_page_table(physical_memory_offset: u64) -> &'static mut PageTable {
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

/// Translates the given virtual address to the mapped physical address, or
/// `None` if the address is not mapped.
///
/// This function is unsafe because the caller must guarantee that the
/// complete physical memory is mapped to virtual memory at the passed
/// `physical_memory_offset`.
pub unsafe fn translate_addr(addr: VirtAddr, physical_memory_offset: u64) -> Option<PhysAddr> {
    translate_addr_inner(addr, physical_memory_offset)
}

/// Private function that is called by `translate_addr`.
///
/// This function is safe to limit the scope of `unsafe` because Rust treats
/// the whole body of unsafe functions as an unsafe block. This function must
/// only be reachable through `unsafe fn` from outside of this module.
fn translate_addr_inner(addr: VirtAddr, physical_memory_offset: u64) -> Option<PhysAddr> {
    use x86_64::structures::paging::page_table::FrameError;
    use x86_64::registers::control::Cr3;

    // read the active level 4 frame from the CR3 register
    let (level_4_table_frame, _) = Cr3::read();

    // Remember that the virtual address contains different page table level indices
    // in its value
    let table_indices = [
        addr.p4_index(), addr.p3_index(), addr.p2_index(), addr.p1_index()
    ];

    let mut frame = level_4_table_frame;
    // traverse the multi-level page table
    // use the index of the corresponding page
    // table entry to get the next table
    for &table_entry_index in &table_indices {
        // We're just adding an offset (from Cargo.toml -> bootloader) to
        // form the virtual address space, what we're trying to achieve
        // here is to get the virtual address of the next page table entry
        // based on its physical address and the memory offset, as the kernel
        // can't write to physical addresses directly when paging is enabled
        let addr = frame.start_address().as_u64() + physical_memory_offset;
        let table_ptr: *const PageTable = VirtAddr::new(addr).as_ptr();
        let table = unsafe { &(*table_ptr) };

        let entry: &PageTableEntry = &table[table_entry_index];
        frame = match entry.frame() {
            Ok(frame) => frame,
            Err(FrameError::FrameNotPresent) => return None,
            Err(FrameError::HugeFrame) => panic!("Huge pages aren't supported")
        };
    }

    // After finding the address of the level 1 table, we add
    // the offset of the table entry
    Some(frame.start_address() + u64::from(addr.page_offset()))
}
