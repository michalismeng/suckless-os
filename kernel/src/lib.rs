#![no_std]
#![feature(asm)]

#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(clippy::upper_case_acronyms)]
#[allow(clippy::redundant_static_lifetimes)]
pub mod bootboot;
pub mod light;

use core::panic::PanicInfo;
use light::kdebug;

#[cfg(not(test))]
#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    unsafe {
        kdebug::print(b"\n### PANIC ###\n");
    }
    loop {}
}
