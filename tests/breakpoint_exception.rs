#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![feature(asm)]
#![test_runner(ham_dos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::panic::PanicInfo;
#[cfg(test)]
use ham_dos::{init, serial_print, serial_println};

#[test_case]
fn test_breakpoint_exception() {
    serial_print!("test_breakpoint_exception...");
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
    serial_println!("[ok]");
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ham_dos::test_panic_handler(info);
}

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    init();
    test_main();

    unsafe {
        asm!("hlt");
    }
    loop {}
}
