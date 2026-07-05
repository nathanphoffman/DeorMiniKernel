use core::fmt;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const COLOR_WHITE_ON_BLACK: u8 = 0x0f;

pub struct Writer {
    col: usize,
    row: usize,
}

impl Writer {
    fn write_byte(&mut self, byte: u8) {
        if byte == b'\n' {
            self.new_line();
            return;
        }
        if self.col >= VGA_WIDTH {
            self.new_line();
        }
        let offset = (self.row * VGA_WIDTH + self.col) * 2;
        unsafe {
            VGA_BUFFER.add(offset).write_volatile(byte);
            VGA_BUFFER.add(offset + 1).write_volatile(COLOR_WHITE_ON_BLACK);
        }
        self.col += 1;
    }

    fn new_line(&mut self) {
        self.col = 0;
        self.row += 1;
        if self.row >= VGA_HEIGHT {
            self.row = 0;
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
        Ok(())
    }
}

pub static mut WRITER: Writer = Writer { col: 0, row: 0 };

pub fn clear_screen() {
    for i in 0..VGA_WIDTH * VGA_HEIGHT {
        unsafe {
            VGA_BUFFER.add(i * 2).write_volatile(b' ');
            VGA_BUFFER.add(i * 2 + 1).write_volatile(COLOR_WHITE_ON_BLACK);
        }
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    unsafe {
        let _ = WRITER.write_fmt(args);
    }
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga::_print(core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::vga::_print(core::format_args!("{}\n", core::format_args!($($arg)*))));
}
