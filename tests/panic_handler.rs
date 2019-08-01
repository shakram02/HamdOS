#![no_std]
#![no_main]
#![feature(panic_info_message)]

use core::fmt::Write;
use core::panic::PanicInfo;
use ham_dos::{exit_qemu, serial_print, serial_println, QemuExitCode};

const MESSAGE: &str = "Example panic message from panic_handler test";
const PANIC_LINE: u32 = 14; // panic! line number
#[no_mangle]
pub extern "C" fn _start() -> ! {
    serial_print!("panic_handler... ");
    panic!(MESSAGE); // must be in line `PANIC_LINE`
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    check_message(info);
    check_location(info);

    serial_println!("[ok]");
    exit_qemu(QemuExitCode::Success);
    loop {}
}

fn check_location(info: &PanicInfo) {
    let location = info.location().unwrap_or_else(|| fail("No location info"));

    if location.file() != file!() {
        fail("Wrong file name");
    }

    if location.line() != PANIC_LINE {
        fail("Wrong line number");
    }
}

fn check_message(info: &PanicInfo) {
    let message = info
        .message()
        .unwrap_or_else(|| fail("No panic message found"));

    let mut compare_message = CompareMessage { expected: MESSAGE };
    write!(&mut compare_message, "{}", message).unwrap_or_else(|_| fail("write failed"));

    if !compare_message.expected.is_empty() {
        fail("Message shorter than expected message");
    }
}

fn fail(error: &str) -> ! {
    serial_println!("[failed]");
    serial_println!("{}", error);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}

use core::fmt;

/// Compares a `fmt::Arguments` instance with the `MESSAGE` string
///
/// To use this type, write the `fmt::Arguments` instance to it using the
/// `write` macro. If the message component matches `MESSAGE`, the `expected`
/// field is the empty string.
struct CompareMessage {
    expected: &'static str,
}

impl fmt::Write for CompareMessage {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        if self.expected.starts_with(s) {
            self.expected = &self.expected[s.len()..];
        } else {
            fail("Panic message is not equal to expected message");
        }

        Ok(())
    }
}
