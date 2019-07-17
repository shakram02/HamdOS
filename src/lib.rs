#![no_std]

use core::panic::PanicInfo;
static HELLO: &[u8] = b"HelloWOWOWOWO World!";
const VGA_BUFFER_ADDR: usize = 0xb8000; // VGA buffer address
const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    clear_screen();
    apply_text_attr(0x3f); // Cyan background, white text

    for (i, &byte) in HELLO.iter().enumerate() {
        unsafe {
            let addr = VGA_BUFFER_ADDR + (i * 2);
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
    for i in 0..SCREEN_WIDTH {
        for j in 0..SCREEN_WIDTH {
            unsafe {
                let addr = VGA_BUFFER_ADDR + (i * SCREEN_WIDTH) + j;
                if j % 2 == 0 {
                    *(addr as *mut u8) = 0x0;
                }
            }
        }
    }
}

fn apply_text_attr(text_attr: u8) {
    for i in 0..SCREEN_WIDTH {
        for j in 0..SCREEN_WIDTH {
            unsafe {
                let addr = VGA_BUFFER_ADDR + (i * SCREEN_WIDTH) + j;
                if j % 2 != 0 {
                    // Set text attribute
                    *(addr as *mut u8) = text_attr;
                }
            }
        }
    }
}
