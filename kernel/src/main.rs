#![no_std]
#![no_main]

extern crate rlibc;

use sos::light;

#[no_mangle]
unsafe fn _start() -> ! {
    light::initialize()
}
