#![no_std]
#![allow(warnings)]

extern crate alloc;

use core::panic::PanicInfo;

pub mod heap;
pub mod interrupts;
pub mod keyboard;
pub mod vga;

mod generated {
    include!("../build/main_body.rs");
    use crate::println;
    use alloc::string::{String, ToString};
    use alloc::vec::Vec;

    pub fn run() {
        main();
    }
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    vga::clear_screen();
    interrupts::init();
    generated::run();

    loop {
        unsafe { core::arch::asm!("hlt") };
    }
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
