use crate::bootboot;

const LETTERS: &[u8] = b"0123456789abcdef";

pub fn get_stack() -> u64 {
    let mut stack: u64;
    unsafe { asm!("mov {}, rsp", out(reg) stack) }
    stack
}

pub fn get_processor_id() -> u8 {
    let cpuid = raw_cpuid::CpuId::new();
    cpuid.get_feature_info().unwrap().initial_local_apic_id()
}

pub fn is_bsp() -> bool {
    unsafe { get_processor_id() as u16 == bootboot::bootboot.bspid }
}

pub fn int_to_bytes_hex(x: u64, result: &mut [u8]) {
    int_to_bytes(x, result, 16)
}

pub fn int_to_bytes(value: u64, result: &mut [u8], base: u64) {
    let mut temp = value;
    let mut i = 0;
    let length = result.len();

    #[allow(clippy::useless_asref)]
    for x in result.as_mut() {
        *x = b'0'
    }

    while temp > 0 && i < length {
        let val = (temp % base) as usize;
        result[(length - 1 - i) as usize] = LETTERS[val];
        i += 1;
        temp /= base;
    }
}

#[allow(dead_code)]
pub fn print_stack_usage() {
    use super::kdebug;
    use crate::kdbg_ctx;

    let id = get_processor_id();
    // Might be off by one because stack begins at 0, i.e core::u64::MAX + 1
    let usage = core::u64::MAX - 1024 * (id as u64) - get_stack();
    let mut buf = [0u8; 16];

    kdbg_ctx!(
        int_to_bytes_hex(id as u64, &mut buf)
        kdebug::print(b"Processor: ")
        kdebug::print(&buf)
        int_to_bytes(usage, &mut buf, 10)
        kdebug::print(b" Stack usage: ")
        kdebug::print(&buf)
        kdebug::print(b" bytes\n")
    )
}
