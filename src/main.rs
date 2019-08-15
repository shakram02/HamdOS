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

    use x86_64::registers::control::Cr3;
    use ham_dos::memory::{active_level_4_page_table, translate_addr};

    let l4_table: &'static mut PageTable = unsafe {
        active_level_4_page_table(boot_info.physical_memory_offset)
    };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x20010a,
        // some stack page
        0x57ac_001f_fe48,
        // virtual address mapped to physical address 0
        // The translation of the physical_memory_offset should point to physical address 0,
        // but the translation fails because the mapping uses huge pages for efficiency.
        // Future bootloader version might use the same optimization for kernel and stack pages.
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = unsafe { translate_addr(virt, boot_info.physical_memory_offset) };
        println!("{:#?} -> {:#?}", virt, phys);
    }

    for (i, entry) in l4_table.iter().enumerate() {
        if !entry.is_unused() {
            println!("L4 Entry {}: {:?}", i, entry);
        }
    }

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
