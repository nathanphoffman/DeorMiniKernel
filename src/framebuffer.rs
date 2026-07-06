use core::fmt;

use crate::font9x16::FONT9X16;
use crate::multiboot2::FramebufferInfo;

const GLYPH_WIDTH: usize = 9;
const GLYPH_HEIGHT: usize = 16;
// Scale is a ratio (SCALE_NUM / SCALE_DEN) rather than a float -- this is
// freestanding code with no FPU/SSE state set up at boot (see boot.asm), so
// an f32 here would risk hitting an unhandled #UD/#NM fault. 3/2 = 1.5x.
// Each destination pixel samples its nearest source pixel (nearest-neighbor
// scaling), so any ratio works, not just whole numbers.
const SCALE_NUM: usize = 1;
const SCALE_DEN: usize = 1;
const CELL_WIDTH: usize = (GLYPH_WIDTH * SCALE_NUM) / SCALE_DEN;
const CELL_HEIGHT: usize = (GLYPH_HEIGHT * SCALE_NUM) / SCALE_DEN;
const FG: (u8, u8, u8) = (0xFF, 0xFF, 0xFF);
const BG: (u8, u8, u8) = (0x00, 0x00, 0x00);

// All framebuffer geometry is fixed once by `init`, before interrupts are
// enabled, and never written again -- but `putc` (and therefore every field
// read here) is also reachable from the keyboard IRQ handler, invisible to
// normal call-graph analysis. As with the cursor position, raw volatile
// access to plain statics avoids ever handing out a `&mut` the optimizer
// could assume is exclusive across that interrupt boundary.
static mut FB_ADDR: usize = 0;
static mut FB_PITCH: usize = 0;
static mut FB_BYTES_PER_PIXEL: usize = 0;
static mut FB_COLS: usize = 0;
static mut FB_ROWS: usize = 0;
static mut RED_POS: u8 = 0;
static mut RED_SIZE: u8 = 0;
static mut GREEN_POS: u8 = 0;
static mut GREEN_SIZE: u8 = 0;
static mut BLUE_POS: u8 = 0;
static mut BLUE_SIZE: u8 = 0;
static mut CURSOR_COL: usize = 0;
static mut CURSOR_ROW: usize = 0;

/// Stores the framebuffer geometry GRUB reported and clears the screen.
/// Halts forever if no usable (direct-RGB) framebuffer was found -- there's
/// no text-mode fallback to report the failure through.
pub fn init(mb_info_ptr: *const u8) {
    let info = match unsafe { crate::multiboot2::find_framebuffer(mb_info_ptr) } {
        Some(info) => info,
        None => loop {
            unsafe { core::arch::asm!("cli", "hlt") };
        },
    };

    unsafe {
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FB_ADDR), info.addr as usize);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(FB_PITCH), info.pitch as usize);
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(FB_BYTES_PER_PIXEL),
            (info.bpp as usize + 7) / 8,
        );
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(FB_COLS),
            info.width as usize / CELL_WIDTH,
        );
        core::ptr::write_volatile(
            core::ptr::addr_of_mut!(FB_ROWS),
            info.height as usize / CELL_HEIGHT,
        );
        core::ptr::write_volatile(core::ptr::addr_of_mut!(RED_POS), info.red_pos);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(RED_SIZE), info.red_size);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(GREEN_POS), info.green_pos);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(GREEN_SIZE), info.green_size);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(BLUE_POS), info.blue_pos);
        core::ptr::write_volatile(core::ptr::addr_of_mut!(BLUE_SIZE), info.blue_size);
    }

    clear_screen();
}

fn geometry() -> (usize, usize, usize, usize, usize) {
    unsafe {
        (
            core::ptr::read_volatile(core::ptr::addr_of!(FB_ADDR)),
            core::ptr::read_volatile(core::ptr::addr_of!(FB_PITCH)),
            core::ptr::read_volatile(core::ptr::addr_of!(FB_BYTES_PER_PIXEL)),
            core::ptr::read_volatile(core::ptr::addr_of!(FB_COLS)),
            core::ptr::read_volatile(core::ptr::addr_of!(FB_ROWS)),
        )
    }
}

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

// Packs an (r, g, b) triple into the pixel format GRUB actually gave us --
// works whether the framebuffer is BGRX8888, RGBX8888, or 16-bit 5-6-5,
// since we shift each channel by the field position/size it reported rather
// than assuming a fixed layout.
fn pack_color(rgb: (u8, u8, u8)) -> u32 {
    unsafe {
        let red_pos = core::ptr::read_volatile(core::ptr::addr_of!(RED_POS));
        let red_size = core::ptr::read_volatile(core::ptr::addr_of!(RED_SIZE));
        let green_pos = core::ptr::read_volatile(core::ptr::addr_of!(GREEN_POS));
        let green_size = core::ptr::read_volatile(core::ptr::addr_of!(GREEN_SIZE));
        let blue_pos = core::ptr::read_volatile(core::ptr::addr_of!(BLUE_POS));
        let blue_size = core::ptr::read_volatile(core::ptr::addr_of!(BLUE_SIZE));

        let r = ((rgb.0 as u32) >> (8 - red_size)) << red_pos;
        let g = ((rgb.1 as u32) >> (8 - green_size)) << green_pos;
        let b = ((rgb.2 as u32) >> (8 - blue_size)) << blue_pos;
        r | g | b
    }
}

fn put_pixel(x: usize, y: usize, color: u32) {
    let (fb_addr, pitch, bytes_per_pixel, _, _) = geometry();
    let offset = y * pitch + x * bytes_per_pixel;
    unsafe {
        let ptr = (fb_addr as *mut u8).add(offset);
        for i in 0..bytes_per_pixel {
            ptr.add(i).write_volatile(((color >> (8 * i)) & 0xFF) as u8);
        }
    }
}

fn draw_glyph(col: usize, row: usize, byte: u8) {
    let glyph = &FONT9X16[byte as usize];
    let fg = pack_color(FG);
    let bg = pack_color(BG);
    let base_x = col * CELL_WIDTH;
    let base_y = row * CELL_HEIGHT;
    for dy in 0..CELL_HEIGHT {
        let src_y = (dy * SCALE_DEN) / SCALE_NUM;
        let bits = glyph[src_y];
        for dx in 0..CELL_WIDTH {
            let src_x = (dx * SCALE_DEN) / SCALE_NUM;
            let set = (bits >> src_x) & 1 != 0;
            let color = if set { fg } else { bg };
            put_pixel(base_x + dx, base_y + dy, color);
        }
    }
}

fn write_byte(byte: u8) {
    let (_, _, _, cols, _) = geometry();
    if byte == b'\n' {
        new_line();
        return;
    }
    let (mut col, _) = cursor();
    if col >= cols {
        new_line();
        col = 0;
    }
    let (_, row) = cursor();
    draw_glyph(col, row, byte);
    set_cursor(col + 1, row);
}

fn new_line() {
    let (_, _, _, _, rows) = geometry();
    let (_, mut row) = cursor();
    row += 1;
    if row >= rows {
        scroll_up();
        row = rows - 1;
    }
    set_cursor(0, row);
}

// Shifts the whole framebuffer up by one glyph row (a straight memmove of
// pixel rows) and blanks the row that scrolled in, like a real terminal
// scrolling instead of wrapping back to the top and overwriting old text.
fn scroll_up() {
    let (fb_addr, pitch, _, _, rows) = geometry();
    let row_bytes = pitch * CELL_HEIGHT;
    let total_bytes = pitch * rows * CELL_HEIGHT;
    unsafe {
        let base = fb_addr as *mut u8;
        core::ptr::copy(base.add(row_bytes), base, total_bytes - row_bytes);
        core::ptr::write_bytes(base.add(total_bytes - row_bytes), 0, row_bytes);
    }
}

// Never crosses into the row above -- a user can only erase back to the
// start of the screen line they're currently on, not into previously
// printed lines above it.
fn backspace() {
    let (col, row) = cursor();
    if col == 0 {
        return;
    }
    let col = col - 1;
    set_cursor(col, row);
    draw_glyph(col, row, b' ');
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
    let (_, pitch, _, _, rows) = geometry();
    let (fb_addr, _, _, _, _) = geometry();
    unsafe {
        core::ptr::write_bytes(fb_addr as *mut u8, 0, pitch * rows * CELL_HEIGHT);
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
    ($($arg:tt)*) => ($crate::framebuffer::_print(core::format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::framebuffer::_print(core::format_args!("{}\n", core::format_args!($($arg)*))));
}
