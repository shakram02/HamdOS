#![no_std]
#![no_main]

use core::panic::PanicInfo;
static HELLO: &[u8] = b"HelloWOWOWOWO World!";

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    apply_text_attr(0x3f); // Cyan background, white text

	let vga_buffer = 0xb8000; // VGA buffer address
    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            let addr = vga_buffer + (i * 2);
            *(addr as *mut u8) = byte;
        }
    }

    loop {}
}

/// This function is called on panic.
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

fn clear_screen() {
    let vga_buffer = 0xb8000; // VGA buffer address
    for i in 0..200 {
        for j in 0..320 {
            unsafe {
                let addr = vga_buffer + (i * 320) + j;
                if j % 2 == 0 {
                    *(addr as *mut u8) = 0x0;
                }
            }
        }
    }
}

fn apply_text_attr(text_attr: u8) {
    let vga_buffer = 0xb8000; // VGA buffer address
    for i in 0..200 {
        for j in 0..320 {
            unsafe {
                let addr = vga_buffer + (i * 320) + j;
                if j % 2 != 0 {
                    // Set text attribute
                    *(addr as *mut u8) = text_attr;
                }
            }
        }
    }
}
