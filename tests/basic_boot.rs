#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ham_dos::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(asm)]

use core::panic::PanicInfo;
<<<<<<< HEAD
use ham_dos::{println, serial_print, serial_println};
=======
use ham_dos::{serial_print,serial_println,println};
>>>>>>> not using makefile anymore, use cargo xrun instead

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    unsafe {
        asm!("hlt");
    }
    loop {}
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    ham_dos::test_panic_handler(info);
}

#[test_case]
fn test_println() {
    serial_print!("test_println... ");
    println!("test_println output");
    serial_println!("[ok]");
}
