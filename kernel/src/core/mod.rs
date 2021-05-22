
pub fn initialize() -> ! {

    crate::kdbg_ctx!(
        crate::kdebug::print(b"Made it to core!\n")
    );

    #[allow(clippy::empty_loop)]
    loop {}
}