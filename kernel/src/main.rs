#![no_std]
#![no_main]

extern crate rlibc;

use core::sync::atomic;
use sos::light;
use sos::{
    gdt, kdbg_ctx,
    light::{idt, kdebug, memory, utils},
};
use x86_64::structures::paging::FrameAllocator;

static mut INIT: atomic::AtomicBool = atomic::AtomicBool::new(true);

unsafe fn init() {
    gdt::init();
    idt::init();
}

unsafe fn load_system_tables() {
    gdt::load();
    idt::load();
}

unsafe fn init_memory_allocator() {
    let mut buf = [0u8; 8];
    let entries = memory::parse_memory_map();
    memory::init(entries);

    // Report total memory
    let mut total_memory = 0;
    for e in entries {
        if e.is_usable() {
            total_memory += e.get_size();
        }
    }

    kdbg_ctx!(
        utils::int_to_bytes(total_memory / 1024 / 1024, &mut buf, 10)
        kdebug::print(b"Total memory: ")
        kdebug::print(&buf)
        kdebug::print(b" MB\n")
    );
}

#[no_mangle]
unsafe fn _start() -> ! {
    // Only the BSP should perform this initialization.
    if utils::is_bsp() {
        init();
        load_system_tables();
        init_memory_allocator();
        memory::get_frame_allocator().print_memory_regions();

        // Initialization is over, so let other processors continue.
        INIT.store(false, atomic::Ordering::SeqCst);
    } else {
        while INIT.load(atomic::Ordering::SeqCst) {}
        load_system_tables();
    }

    if utils::is_bsp() {
        let allocator = memory::get_frame_allocator();
        let x = allocator.allocate_frame().unwrap();
        let mut buf = [0u8; 16];
        let addr = x.start_address().as_u64();
        kdbg_ctx!(
            kdebug::print(b"Alloc at: ")
            utils::int_to_bytes_hex(addr, &mut buf)
            kdebug::print(&buf)
            kdebug::print(b"\n")
        )
    }

    #[allow(clippy::empty_loop)]
    loop {}
}
