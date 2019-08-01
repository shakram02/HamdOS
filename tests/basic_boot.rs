#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]
#![feature(asm)]

use core::panic::PanicInfo;
use ham_dos::{exit_qemu, QemuExitCode};

#[no_mangle] // don't mangle the name of this function
pub extern "C" fn _start() -> ! {
    test_main();

    unsafe {
        asm!("hlt");
    }
    loop {}
}

fn test_runner(tests: &[&dyn Fn()]) {
    for test in tests {
        test();
    }

    exit_qemu(QemuExitCode::Success);
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

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(0, 1);
    serial_println!("[ok]");
}
