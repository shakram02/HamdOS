#![feature(asm)]
#![no_std]

mod vga_driver;

use core::panic::PanicInfo;
use vga_driver::VgaBuffer;
static HELLO: &str = "Lorem ipsum dolor sit amet,\nno\x08n purus ut euismod vestibulum,non";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let mut vga = VgaBuffer::new();
    vga.clear_screen();

    vga.apply_text_attr(0x3f); // Cyan background, white text
    vga.print(HELLO.as_bytes());

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
