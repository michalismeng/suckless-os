#![no_std]
#![no_main]
#![feature(asm)]

extern crate rlibc;

use core::{sync::atomic, u64};
use sos::{bootboot, light};
use sos::{
    gdt, kdbg_ctx,
    light::{idt, kdebug, memory, utils},
    vmemory,
};
use x86_64::{
    structures::paging::{
        FrameAllocator, Mapper, Page, PageTableFlags,
    },
    VirtAddr,
};

static mut INIT: atomic::AtomicBool = atomic::AtomicBool::new(true);
const STACK_TOP: u64 = 0xFFFF_FFFF_F800_0000;
const STACK_SIZE: u64 = 4096;

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

unsafe fn move_to_permanent_stack() -> ! {
    let stack = STACK_TOP - utils::get_processor_id() as u64 * 4096;
    asm!("mov rsp, {}", in(reg) stack);
    begin_new_environment()
}

fn begin_new_environment() -> ! {
    let mut buf = [0u8; 16];
    kdbg_ctx!(
        kdebug::print(b"Stack at: 0x")
        utils::int_to_bytes_hex(utils::get_stack(), &mut buf)
        kdebug::print(&buf)
        kdebug::print(b". Everything is OK.\n")
    );

    #[allow(clippy::empty_loop)]
    loop {}
}

unsafe fn allocate_stack_space() -> ! {
    use bootboot::bootboot;
    let total_stack_memory = bootboot.numcores as u64 * STACK_SIZE;
    let stack_bottom = STACK_TOP - total_stack_memory;

    let pe_flags = PageTableFlags::PRESENT | PageTableFlags::WRITABLE;
    let allocator = memory::get_frame_allocator();
    let mut mapper = vmemory::init(VirtAddr::new(0));

    // Map stack memory for each processor.
    for i in 0..bootboot.numcores as u64 {
        let frame = allocator.allocate_frame().unwrap();
        let vaddr = VirtAddr::new(stack_bottom + i * STACK_SIZE);
        let page = Page::from_start_address_unchecked(vaddr);

        mapper
            .map_to(page, frame, pe_flags, allocator)
            .unwrap()
            .flush();
    }

    // Initialization is over, so let other processors continue.
    INIT.store(false, atomic::Ordering::SeqCst);
    move_to_permanent_stack()
}

#[no_mangle]
unsafe fn _start() -> ! {
    // Only the BSP should perform this initialization.
    if utils::is_bsp() {
        init();
        load_system_tables();
        init_memory_allocator();
        memory::get_frame_allocator().print_memory_regions();

        // Temporarily set the BSP stack inside the framebuffer. This will allow
        // us to allocate memory for the permanent stack region.
        let stack = bootboot::bootboot.fb_ptr as u64 + 4096;
        asm!("mov rsp, {}", in(reg) stack);
        allocate_stack_space()
    } else {
        // Wait until BSP initialization is over.
        while INIT.load(atomic::Ordering::SeqCst) {}
        load_system_tables();
        move_to_permanent_stack()
    }
}
