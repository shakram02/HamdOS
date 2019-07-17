#![no_std]

mod vga_driver;

use core::panic::PanicInfo;
use vga_driver::VgaBuffer;
static HELLO: &str = "HelloWOWOWOWO World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let vga = VgaBuffer::new();
	vga.clear_screen();
    vga.apply_text_attr(0x3f); // Cyan background, white text
	vga.print(HELLO.as_bytes());
    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
