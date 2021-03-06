#![no_std]
#![no_main]

extern crate rlibc;

use core::panic::PanicInfo;

#[no_mangle]
unsafe fn _start() -> ! {
    loop {}
}

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
