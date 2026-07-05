#![no_std]

use core::panic::PanicInfo;

const VGA_BUFFER: *mut u8 = 0xb8000 as *mut u8;
const VGA_COLOR_WHITE_ON_BLACK: u8 = 0x0f;
const VGA_WIDTH: usize = 80;
const VGA_HEIGHT: usize = 25;

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    for i in 0..VGA_WIDTH * VGA_HEIGHT {
        unsafe {
            VGA_BUFFER.add(i * 2).write_volatile(b' ');
            VGA_BUFFER.add(i * 2 + 1).write_volatile(VGA_COLOR_WHITE_ON_BLACK);
        }
    }

    let message = b"Hello, World!";

    for (i, &byte) in message.iter().enumerate() {
        unsafe {
            VGA_BUFFER.add(i * 2).write_volatile(byte);
            VGA_BUFFER.add(i * 2 + 1).write_volatile(VGA_COLOR_WHITE_ON_BLACK);
        }
    }

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
