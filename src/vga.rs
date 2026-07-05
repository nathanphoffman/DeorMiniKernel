use core::fmt;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;
const COLOR_WHITE_ON_BLACK: u8 = 0x0f;

// Cursor position is shared between ordinary code and `putc`, which is also
// called from the keyboard interrupt handler. A `static mut Writer` accessed
// through `&mut` looks safe here since everything runs on one core, but
// nothing in a normal call graph reaches keyboard_isr_rust -- it's only
// invoked via a hardware interrupt -- so the compiler is free to assume the
// `&mut` it hands out is never aliased and cache or reorder around it. Raw
// volatile reads/writes to plain statics sidestep that: no `&mut` to a
// static is ever created, so there's nothing for the optimizer to assume
// exclusivity over.
static mut CURSOR_COL: usize = 0;
static mut CURSOR_ROW: usize = 0;

fn cursor() -> (usize, usize) {
    unsafe {
        (
            core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_COL)),
            core::ptr::read_volatile(core::ptr::addr_of!(CURSOR_ROW)),
        )
    }
}

fn set_cursor(col: usize, row: usize) {
    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_COL), col);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(CURSOR_ROW), row);
    }
}

fn put_at(col: usize, row: usize, byte: u8) {
    let offset = (row * VGA_WIDTH + col) * 2;
    unsafe {
        VGA_BUFFER.add(offset).write_volatile(byte);
        VGA_BUFFER.add(offset + 1).write_volatile(COLOR_WHITE_ON_BLACK);
    }
}

fn write_byte(byte: u8) {
    if byte == b'\n' {
        new_line();
        return;
    }
    let (mut col, _) = cursor();
    if col >= VGA_WIDTH {
        new_line();
        col = 0;
    }
    let (_, row) = cursor();
    put_at(col, row, byte);
    set_cursor(col + 1, row);
}

fn new_line() {
    let (_, mut row) = cursor();
    row += 1;
    if row >= VGA_HEIGHT {
        row = 0;
    }
    set_cursor(0, row);
}

fn backspace() {
    let (mut col, mut row) = cursor();
    if col == 0 {
        if row == 0 {
            return;
        }
        row -= 1;
        col = VGA_WIDTH - 1;
    } else {
        col -= 1;
    }
    set_cursor(col, row);
    put_at(col, row, b' ');
}

/// Writes a single byte typed at the keyboard directly to the screen,
/// bypassing `core::fmt` -- kept simple since this runs inside an interrupt handler.
pub fn putc(byte: u8) {
    match byte {
        b'\n' => new_line(),
        0x08 => backspace(),
        _ => write_byte(byte),
    }
}

pub fn clear_screen() {
    for i in 0..VGA_WIDTH * VGA_HEIGHT {
        unsafe {
            VGA_BUFFER.add(i * 2).write_volatile(b' ');
            VGA_BUFFER.add(i * 2 + 1).write_volatile(COLOR_WHITE_ON_BLACK);
        }
    }
    set_cursor(0, 0);
}

struct ScreenWriter;

impl fmt::Write for ScreenWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for byte in s.bytes() {
            write_byte(byte);
        }
        Ok(())
    }
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    let _ = ScreenWriter.write_fmt(args);
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
