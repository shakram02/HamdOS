use core::fmt;

use spin::Mutex;

use lazy_static::lazy_static;

const VGA_BUFFER_ADDR: usize = 0xB8000;
const DEFAULT_TEXT_ATTR: u8 = 0x07;
// VGA buffer address
const SCREEN_WIDTH: usize = 80;
const SCREEN_HEIGHT: usize = 25;
const BACKSPACE: u8 = 8;
const LINE_FEED: u8 = 10;
lazy_static! {
    pub static ref VGA_WRITER: Mutex<VgaBuffer> = Mutex::new(VgaBuffer {
        row: 0,
        col: 0,
        all_screen_attr: ScreenCharAttr { val: DEFAULT_TEXT_ATTR }   // Default light grey
    });
}

pub struct VgaBuffer {
    row: usize,
    col: usize,
    all_screen_attr: ScreenCharAttr,
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
    /// Clears the screen content and applies the default text attribute
    /// which is Light Grey text on Black background
    pub fn clear_screen(&mut self) {
        unsafe {
            let addr = &mut *(VGA_BUFFER_ADDR as *mut [[u16; SCREEN_WIDTH]; SCREEN_HEIGHT]);
            *addr = [[((self.all_screen_attr.val as u16) << 8) as u16; SCREEN_WIDTH]; SCREEN_HEIGHT]
            // This is the default value [LightGray]
        }
    }

    /// Clears the text on the screen and applies the given text attribute
    pub fn clear_text_and_apply_attr(&mut self, text_attr: ScreenCharAttr) {
        // Interpret the VGA buffer as u16 to be able to do batch copy,
        // The structure of each VGA block is as follows (little endian)
        // The addresses of x86 grow from bottom to top
        // 0-7 	    ASCII code point (far right)
        // 8-11 	Foreground color
        // 12-14 	Background color
        // 15 	    Blink (left)
        // That's why each ScreenCharAttr value is shifted left 8 bits
        self.all_screen_attr = text_attr;
        unsafe {
            let addr = &mut *((VGA_BUFFER_ADDR) as *mut [[u16; SCREEN_WIDTH]; SCREEN_HEIGHT]);
            *addr =
                [[((self.all_screen_attr.val as u16) << 8) as u16; SCREEN_WIDTH]; SCREEN_HEIGHT];
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
        // Move to newline when we're about to go out of the screen
        if self.col == SCREEN_WIDTH {
            self.row += 1;
            self.col = 0;
        }

        if self.row == SCREEN_HEIGHT {
            self.row -= 1;
            self.shift_one_row_up();
        }

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
        // The addresses of x86 grow from bottom to top, the text was the previous address
        let addr = buffer_address_for(row, col) + 1;
        unsafe {
            *(addr as *mut u8) = attr.val;
        }
    }

    fn shift_one_row_up(&mut self) {
        self.clear_rect(0, 1, 0, SCREEN_WIDTH);
        let addr = buffer_address_for(0, 0);

        unsafe {
            // Double screen width because each char has text & attribute
            let row_vals = &mut *(addr as *mut [[u8; 2 * SCREEN_WIDTH]; SCREEN_HEIGHT]);
            for i in 0..(SCREEN_HEIGHT - 1) {
                row_vals[i] = row_vals[i + 1];
            }

            // The last line should be cleared before writing anything to it
            // since we don't move the row variable when writing at the final line
            // TODO: save the passed away screen buffer to somewhere
            self.clear_rect(SCREEN_HEIGHT - 1, 1, 0, SCREEN_WIDTH);
        }
    }

    pub fn clear_rect(&mut self, start_row: usize, rows: usize, start_col: usize, cols: usize) {
        for i in start_row..(start_row + rows) {
            for j in start_col..(start_col + cols) {
                self.write_byte_to_buffer_at(0x00, i, j);
                self.write_attribute_to_buffer_at(self.all_screen_attr, i, j);
            }
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
    let row_byte_count = 2 * SCREEN_WIDTH;
    VGA_BUFFER_ADDR + ((row * row_byte_count) + (2 * col))
}

fn is_printable(byte: u8) -> bool {
    byte >= 0x20 && byte <= 0x7e
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        VGA_WRITER.lock().write_fmt(args).unwrap();
    });
}

#[macro_export]
macro_rules! print {
	($($arg:tt)*) => {$crate::vga_driver::_print(format_args!($($arg)*))	};
}

#[macro_export]
macro_rules! println {
	() => (print!("\n"));
	($($arg:tt)*) => {$crate::print!("{}\n",format_args!($($arg)*))};
}
