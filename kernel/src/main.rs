#![no_std]
#![no_main]

use sos::light::utils;

extern crate rlibc;

#[no_mangle]
unsafe fn _start() -> ! {
    utils::print_stack_usage();

    #[allow(clippy::empty_loop)]
    loop {}
}
