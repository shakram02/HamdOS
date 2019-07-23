const VGA_BUFFER_ADDR: usize = 0xB8000; // VGA buffer address
const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;
const BACKSPACE: u8 = 8;
const LINE_FEED: u8 = 10;
use core::fmt;

pub struct VgaBuffer {
    row: usize,
    col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ScreenCharAttr {
    val: u8,
}

impl ScreenCharAttr {
    pub fn new(foreground_color: Color, background_color: Color) -> ScreenCharAttr {
        ScreenCharAttr {
            val: ((background_color as u8) << 4) | (foreground_color as u8),
        }
    }
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

impl VgaBuffer {
    pub fn new() -> VgaBuffer {
        VgaBuffer { row: 0, col: 0 }
    }
    pub fn clear_screen(&mut self) {
        for i in 0..SCREEN_HEIGHT {
            for j in 0..SCREEN_WIDTH {
                self.write_byte_to_buffer_at(0x00, i as usize, j as usize);
                self.write_attribute_to_buffer_at(
                    ScreenCharAttr::new(Color::Black, Color::Black),
                    i as usize,
                    j as usize,
                );
            }
        }
    }

    pub fn apply_text_attr(&mut self, text_attr: ScreenCharAttr) {
        for i in 0..SCREEN_HEIGHT {
            for j in 0..SCREEN_WIDTH {
                self.write_attribute_to_buffer_at(text_attr, i, j);
            }
        }
    }

    pub fn print(&mut self, bytes: &[u8]) {
        for &byte in bytes.iter() {
            self.print_byte(byte)
        }
    }

    pub fn print_attributed_text(&mut self, string: &str, text_attr: ScreenCharAttr) {
        for &byte in string.as_bytes() {
            if is_printable(byte) {
                self.write_attribute_to_buffer(text_attr);
            }

            self.print_byte(byte);
        }
    }

    pub fn print_byte(&mut self, byte: u8) {
        if byte == LINE_FEED {
            self.row += 1;
            self.col = 0;
        } else if byte == BACKSPACE {
            if self.col > 0 {
                self.col -= 1;
            }
        } else if is_printable(byte) {
            self.write_byte_to_buffer(byte);
            self.col += 1;
        } else {
            self.write_byte_to_buffer(0xfe);
            self.col += 1;
        }

        // Move to newline when we're about to go out of the screen
        if self.col == SCREEN_WIDTH {
            self.row += 1;
            self.col = 0;
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

    fn write_attribute_to_buffer(&self, attr: ScreenCharAttr) {
        self.write_attribute_to_buffer_at(attr, self.row, self.col);
    }

    fn write_attribute_to_buffer_at(&self, attr: ScreenCharAttr, row: usize, col: usize) {
        let addr = buffer_address_for(row, col) + 1;
        unsafe {
            *(addr as *mut u8) = attr.val;
        }
    }
}

impl fmt::Write for VgaBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.print(s.as_bytes());
        Ok(())
    }
}

fn buffer_address_for(row: usize, col: usize) -> usize {
    VGA_BUFFER_ADDR + ((row * (2 * SCREEN_WIDTH)) + (2 * col))
}

fn is_printable(byte: u8) -> bool {
    byte >= 0x20 && byte <= 0x7e
}
