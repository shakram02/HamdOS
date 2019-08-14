#![feature(asm)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ham_dos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;

#[cfg(test)]
use ham_dos::{exit_qemu, init, QemuExitCode, serial_println};
use ham_dos::println;
use ham_dos::vga_driver::{Color, ScreenCharAttr, VGA_WRITER};

#[no_mangle]
pub extern "C" fn _start() -> ! {
    VGA_WRITER.lock().clear_text_and_apply_attr(ScreenCharAttr::new(Color::White, Color::Cyan));
    ham_dos::init();

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
