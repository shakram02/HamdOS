#![feature(asm)]
#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(ham_dos::test_runner)]
#![reexport_test_harness_main = "test_main"]

use core::fmt::Write;
use core::panic::PanicInfo;

#[cfg(test)]
use ham_dos::{exit_qemu, init, QemuExitCode, serial_println};
use ham_dos::{print, println};
use ham_dos::vga_driver::{Color, ScreenCharAttr, VGA_WRITER};

static HELLO: &str = "Computer Vision has become ubiquitous in our society, with applications in\n> search\n> image understanding\n> apps\n> mapping\n> drones, and self-driving cars.\nand self-driving cars.\nCore to many of these applications are visual recognition tasks such as image classification, localization and detection. Recent developments in neural network (aka “deep learning”) approaches have greatly advanced the performance of these state-of-the-art visual recognition systems. This course is a deep dive into details of the deep learning\n\n This course is a deep dive into details of the deep learning\n architectures with a focus on lear@3\x08\x08ning. Wörölö";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    ham_dos::init();
    VGA_WRITER.lock().clear_text_and_apply_attr(ScreenCharAttr::new(Color::White, Color::Cyan));
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

// our panic handler in test mode
#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("[failed]\n");
    serial_println!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    loop {}
}
