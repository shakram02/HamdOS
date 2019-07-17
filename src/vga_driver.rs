const VGA_BUFFER_ADDR: usize = 0xB8000; // VGA buffer address
const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;
const BACKSPACE: u8 = 8;
const LINE_FEED: u8 = 10;

pub struct VgaBuffer {
    row: usize,
    col: usize,
}

impl VgaBuffer {
    pub fn new() -> VgaBuffer {
        VgaBuffer { row: 0, col: 0 }
    }
    pub fn clear_screen(&mut self) {
        for i in 0..SCREEN_HEIGHT {
            for j in 0..SCREEN_WIDTH {
                self.write_byte_to_buffer_at(0x00, i as usize, j as usize);
                self.write_attribute_to_buffer_at(0x00, i as usize, j as usize);
            }
        }
    }

    pub fn apply_text_attr(&mut self, text_attr: u8) {
        for i in 0..SCREEN_HEIGHT {
            for j in 0..SCREEN_WIDTH {
                self.write_attribute_to_buffer_at(text_attr, i, j);
            }
        }
    }

    pub fn print(&mut self, bytes: &[u8]) {
        for &byte in bytes.iter() {
            if byte == LINE_FEED {
                self.row += 1;
                self.col = 0;
            } else if byte == BACKSPACE {
                if self.col > 0 {
                    self.col -= 1;
                }
            } else {
                self.write_byte_to_buffer(byte);
                self.col += 1;
            }

            // Move to newline when we're about to go out of the screen
            if self.col == SCREEN_WIDTH {
                self.row += 1;
                self.col = 0;
            }
        }
    }

    fn write_byte_to_buffer(&self, byte: u8) {
        self.write_byte_to_buffer_at(byte, self.row, self.col);
    }

    fn write_byte_to_buffer_at(&self, byte: u8, row: usize, col: usize) {
        let addr = buffer_address_for(row, col);
        unsafe {
            *(addr as *mut u8) = byte;
        }
    }

    fn write_attribute_to_buffer(&self, attr: u8) {
        self.write_attribute_to_buffer_at(attr, self.row, self.col);
    }

    fn write_attribute_to_buffer_at(&self, attr: u8, row: usize, col: usize) {
        let addr = buffer_address_for(row, col) + 1;
        unsafe {
            *(addr as *mut u8) = attr;
        }
    }
}

fn buffer_address_for(row: usize, col: usize) -> usize {
    VGA_BUFFER_ADDR + ((row * (2 * SCREEN_WIDTH)) + (2 * col))
}
