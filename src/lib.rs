#![feature(asm)]
#![no_std]

mod vga_driver;

use core::fmt::Write;
use core::panic::PanicInfo;
use vga_driver::{Color, ScreenCharAttr, VgaBuffer};

static HELLO: &str = "Computer Vision has become ubiquitous in our society, with applications in\n> search\n> image understanding\n> apps\n> mapping\n> drones, and self-driving cars.\n\nCore to many of these applications are visual recognition tasks such as image classification, localization and detection. Recent developments in neural network (aka “deep learning”) approaches have greatly advanced the performance of these state-of-the-art visual recognition systems. This course is a deep dive into details of the deep learning architectures with a focus on lear@3\x08\x08ning. Wörölö";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut vga = VgaBuffer::new();
    vga.clear_screen();

    vga.apply_text_attr(ScreenCharAttr::new(Color::White, Color::Cyan)); // Cyan background, white text
    vga.print(HELLO.as_bytes());
    vga.print_attributed_text(
        "\n bytes: &[u8] ",
        ScreenCharAttr::new(Color::LightBlue, Color::Brown),
    );
    write!(vga, "Formats! {}", 4).unwrap();
    // Go to a terminal state
    unsafe {
        asm!("cli");
        asm!("hlt");
    }
    panic!("END");
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
