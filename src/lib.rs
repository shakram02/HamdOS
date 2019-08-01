#![feature(asm)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

mod serial;
mod vga_driver;

use core::fmt::Write;
use core::panic::PanicInfo;
use vga_driver::{Color, ScreenCharAttr, VGA_WRITER};

static HELLO: &str = "Computer Vision has become ubiquitous in our society, with applications in\n> search\n> image understanding\n> apps\n> mapping\n> drones, and self-driving cars.\n\nCore to many of these applications are visual recognition tasks such as image classification, localization and detection. Recent developments in neural network (aka “deep learning”) approaches have greatly advanced the performance of these state-of-the-art visual recognition systems. This course is a deep dive into details of the deep learning architectures with a focus on lear@3\x08\x08ning. Wörölö";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    VGA_WRITER.lock().clear_screen();

    VGA_WRITER
        .lock()
        .apply_text_attr(ScreenCharAttr::new(Color::White, Color::Cyan)); // Cyan background, white text
    VGA_WRITER.lock().print(HELLO.as_bytes());
    VGA_WRITER.lock().print_attributed_text(
        "\n bytes: &[u8] ",
        ScreenCharAttr::new(Color::LightBlue, Color::Brown),
    );
    write!(VGA_WRITER.lock(), "Formats! {}", 4).unwrap();
    println!();
    println!("Hello {} {}", "!", 3);
    for i in 0..14 {
        println!("{}", i);
    }

    // The custom test frameworks feature generates a main function that
    // calls test_runner, but this function is ignored because we use
    // the #[no_main] attribute and provide our own entry point.
    #[cfg(test)]
    test_main(); // Generated

    // Go to a terminal state
    unsafe {
        asm!("cli");
        asm!("hlt");
    };
    panic!("END");
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);

    unsafe {
        asm!("hlt");
    }
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);

    unsafe { asm!("hlt") }
    loop {}
}

#[cfg(test)]
fn test_runner(tests: &[&dyn Fn()]) {
    serial_println!("Running {} tests", tests.len());
    for test in tests {
        test();
    }

    exit_qemu(QemuExitCode::Success);
}

#[test_case]
fn trivial_assertion() {
    serial_print!("trivial assertion... ");
    assert_eq!(1, 1);
    serial_println!("[ok]");
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4); // Defined in .cargo/config
        port.write(exit_code as u32);
    }
}
