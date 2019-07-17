const VGA_BUFFER_ADDR: usize = 0xB8000; // VGA buffer address
const SCREEN_WIDTH: usize = 320;
const SCREEN_HEIGHT: usize = 200;

pub struct VgaBuffer {
    row: usize,
    col: usize,
}

impl VgaBuffer {
    pub fn new() -> VgaBuffer {
        VgaBuffer { row: 0, col: 0 }
    }
    pub fn clear_screen(&self) {
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

    pub fn apply_text_attr(&self, text_attr: u8) {
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

    pub fn print(&self, bytes: &[u8]) {
        for (i, &byte) in bytes.iter().enumerate() {
            let addr = VGA_BUFFER_ADDR + (i * 2);
            unsafe {
                *(addr as *mut u8) = byte;
            }
        }
    }
}
