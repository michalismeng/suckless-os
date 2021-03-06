use crate::bootboot::{self, MMapEnt};
use crate::{
    kdbg_ctx,
    light::{kdebug, utils},
};
use core::marker::PhantomData;
use x86_64::{
    structures::paging::{FrameAllocator, PageSize, PhysFrame, Size4KiB},
    PhysAddr,
};

/// The size of the physical frame to allocate.
pub type FrameSize = Size4KiB;
type Region = (PhysAddr, PhysAddr);

pub static mut FRAME_ALLOCATOR: Option<BootInfoBumpFrameAllocator> = None;

/// Parse the memory map given by the bootloader.
/// ### Safety
/// This function is unsafe, because the caller must ensure that the memory
/// map from the bootloader is properly initialized.
pub unsafe fn parse_memory_map() -> &'static [MMapEnt] {
    let num_mmap_entries = ((bootboot::bootboot.size - 128) / 16) as isize;
    core::slice::from_raw_parts(
        &bootboot::bootboot.mmap as *const bootboot::MMapEnt,
        num_mmap_entries as usize,
    )
}

/// Get the current frame allocator. Panic if it is not initialized.
/// ### Safety
/// This function is unsafe, because it leaks unprotected mutable
/// references to the allocator.
pub unsafe fn get_frame_allocator() -> &'static mut BootInfoBumpFrameAllocator {
    FRAME_ALLOCATOR.as_mut().expect("Allocator is None")
}

/// Initialize a bump physical frame allocator. The free frames are
/// found via the memory map that the bootloader passes to the kernel.
/// The size of the allocated frame is set to [`FrameSize`].
/// ### Safety
/// This function must be called once and by only one processor.
pub unsafe fn init(memory_map: &'static [bootboot::MMapEnt]) {
    FRAME_ALLOCATOR = Some(BootInfoBumpFrameAllocator {
        memory_map,
        next: 0,
        size: PhantomData::<FrameSize>,
    })
}

impl bootboot::MMapEnt {
    pub fn is_usable(&self) -> bool {
        (self.size & 0xF) == 1
    }

    pub fn get_size(&self) -> u64 {
        self.size & !0xF
    }

    fn to_region(&self) -> Region {
        (
            PhysAddr::new(self.ptr),
            PhysAddr::new(self.ptr + Self::get_size(self)),
        )
    }
}

pub struct BootInfoBumpFrameAllocator<S: PageSize = FrameSize> {
    memory_map: &'static [bootboot::MMapEnt],
    next: usize,
    size: PhantomData<S>,
}

impl<S: PageSize> BootInfoBumpFrameAllocator<S> {
    /// Print the memory regions of this physical frame allocator. These
    /// correspond to the regions that the bootloader passed to us.
    pub fn print_memory_regions(&self) {
        let mut buf = [0u8; 8];
        for e in self.memory_map {
            kdbg_ctx!(
                kdebug::print(b"Region: ")
                utils::int_to_bytes_hex(e.ptr, &mut buf)
                kdebug::print(&buf)
                kdebug::print(b" size: ")
                utils::int_to_bytes_hex(e.get_size(), &mut buf)
                kdebug::print(&buf)
                kdebug::print(b"\n")
            )
        }
    }

    fn squeeze_region_bounds((start, end): Region) -> Region {
        (start.align_up(S::SIZE), end.align_down(S::SIZE))
    }

    fn usable_frames(&self) -> impl Iterator<Item = PhysFrame<S>> {
        self.memory_map
            .iter()
            .filter(|&e| e.is_usable())
            .map(|e| e.to_region())
            .map(Self::squeeze_region_bounds)
            .map(|r| r.0.as_u64()..r.1.as_u64())
            .flat_map(|r| r.step_by(S::SIZE as usize))
            .map(|a| PhysFrame::containing_address(PhysAddr::new(a)))
    }
}

unsafe impl<S: PageSize> FrameAllocator<S> for BootInfoBumpFrameAllocator<S> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<S>> {
        let frame = self.usable_frames().nth(self.next);
        self.next += 1;
        frame
    }
}
