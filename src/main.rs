#![feature(asm)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ham_dos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

use bootloader::{entry_point, BootInfo};
use x86_64::structures::paging::PageTable;
use x86_64::VirtAddr;

use ham_dos::println;
use ham_dos::vga_driver::{Color, ScreenCharAttr, VGA_WRITER};
#[cfg(test)]
use ham_dos::{exit_qemu, init, serial_println, QemuExitCode};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static BootInfo) -> ! {
    VGA_WRITER
        .lock()
        .clear_text_and_apply_attr(ScreenCharAttr::new(Color::White, Color::Cyan));
    ham_dos::init();

    use ham_dos::memory;
    use x86_64::structures::paging::{MapperAllSizes, Page};

    // new: initialize a mapper
    let mut mapper = unsafe { memory::init(boot_info.physical_memory_offset) };
    let mut frame_allocator = memory::EmptyFrameAllocator;

    // The create example mapping function maps the physical address 0xb8000 to the given
    // virtual address
    let page = Page::containing_address(VirtAddr::new(0x1000));
    memory::create_example_mapping(page, &mut mapper, &mut frame_allocator);

    let page_ptr: *mut u64 = page.start_address().as_mut_ptr();
    unsafe { page_ptr.offset(400).write_volatile(0x_f021_f077_f065_f04e) };

    // The custom test frameworks feature generates a main function that
    // calls test_runner, but this function is ignored because we use
    // the #[no_main] attribute and provide our own entry point.
    #[cfg(test)]
    test_main(); // Generated

    // Go to a terminal state
    ham_dos::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    ham_dos::hlt_loop();
}

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
