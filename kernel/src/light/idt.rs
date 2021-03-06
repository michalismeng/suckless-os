use crate::light::kdebug;
use core::panic;
use x86_64::structures::idt::{
    InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode,
};

static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

extern "x86-interrupt" fn breakpoint_handler(
    _stack_frame: &mut InterruptStackFrame,
) {
    unsafe { kdebug::print(b"BREAKPOINT\n") }
}

extern "x86-interrupt" fn segment_not_present_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    unsafe { kdebug::print(b"Segment not present\n") }
    panic!("");
}

extern "x86-interrupt" fn invalid_opcode_handler(
    _stack_frame: &mut InterruptStackFrame,
) {
    unsafe { kdebug::print(b"Invalid opcode\n") }
    panic!("");
}

extern "x86-interrupt" fn general_protection_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) {
    unsafe { kdebug::print(b"GPF occured\n") }
    panic!("");
}

extern "x86-interrupt" fn double_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: u64,
) -> ! {
    unsafe { kdebug::print(b"Double Fault\n") }
    panic!("");
}

extern "x86-interrupt" fn page_fault_handler(
    _stack_frame: &mut InterruptStackFrame,
    _error_code: PageFaultErrorCode,
) {
    unsafe { kdebug::print(b"Page Fault\n") }
    panic!("");
}

/// Setup interrupt handlers for the most common exceptions (i.e the ones I
/// bumped into) that occur due to stack overflows or other such situations.
/// ### Safety
/// This function should be called only once and by only one processor.
pub unsafe fn init() {
    IDT.breakpoint.set_handler_fn(breakpoint_handler);

    IDT.double_fault
        .set_handler_fn(double_fault_handler)
        .set_stack_index(crate::gdt::DOUBLE_FAULT_IST_INDEX);

    IDT.page_fault.set_handler_fn(page_fault_handler);

    IDT.segment_not_present
        .set_handler_fn(segment_not_present_handler);

    IDT.invalid_opcode.set_handler_fn(invalid_opcode_handler);
    IDT.general_protection_fault
        .set_handler_fn(general_protection_fault_handler);
}

/// Load the Interrupt Descriptor Table
/// ### Safety
/// This function should be called once per processor and only after a call to
/// [`init`] has finished.
pub unsafe fn load() {
    IDT.load();
}
