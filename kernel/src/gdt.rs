use x86_64::structures::tss::TaskStateSegment;
use x86_64::{
    structures::gdt::{Descriptor, GlobalDescriptorTable},
    VirtAddr,
};

pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;

static mut GDT: GlobalDescriptorTable = GlobalDescriptorTable::new();
static mut TSS: TaskStateSegment = TaskStateSegment::new();

const STACK_SIZE: usize = 4096;
// Warn: This stack is common among processors
static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

unsafe fn init_tss() {
    TSS.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] =
        VirtAddr::from_ptr(&STACK) + STACK_SIZE
}

/// Setup entries of the Global Descriptor Table. An entry for CS and TSS
/// is mandatory. Also add an entry for FS and GS.
/// ### Safety
/// This function should be called only once and by only one processor.
pub unsafe fn init() {
    init_tss();

    let cs_selector = GDT.add_entry(Descriptor::kernel_code_segment());
    let ds_selector = GDT.add_entry(Descriptor::kernel_data_segment());
    let tss_selector = GDT.add_entry(Descriptor::tss_segment(&TSS));

    // In x86_64, the only relevant segment registers are CS, TSS and perhaps
    // the general purpose ones, FS and GS.
    x86_64::instructions::segmentation::set_cs(cs_selector);
    x86_64::instructions::tables::load_tss(tss_selector);

    x86_64::instructions::segmentation::load_fs(ds_selector);
    x86_64::instructions::segmentation::load_gs(ds_selector);
}

/// Load the Global Descriptor Table
/// ### Safety
/// This function should be called once per processor and only after a call to
/// [`init`] has finished.
pub unsafe fn load() {
    GDT.load();
}
