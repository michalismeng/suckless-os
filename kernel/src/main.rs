#![no_std]
#![no_main]

extern crate rlibc;

use core::sync::atomic;
use sos::{gdt, light::utils};

static mut INIT: atomic::AtomicBool = atomic::AtomicBool::new(true);

unsafe fn init() {
    gdt::init();
}

unsafe fn load_system_tables() {
    gdt::load();
}

#[no_mangle]
unsafe fn _start() -> ! {
    // Only the BSP should perform this initialization.
    if utils::is_bsp() {
        init();
        load_system_tables();

        // Initialization is over, so let other processors continue.
        INIT.store(false, atomic::Ordering::SeqCst);
    } else {
        while INIT.load(atomic::Ordering::SeqCst) {}
        load_system_tables();
    }

    utils::print_stack_usage();

    #[allow(clippy::empty_loop)]
    loop {}
}
