## Roadmap

### Lightweight environment

When the BOOTBOOT bootloader gives control to our kernel, it has already setup an environment for us.
As we explain below, this environment comes with some limitations for our kernel implementation.

We assume a BOOTBOOT Level 1 loader. This means that the kernel is loaded at a statically defined base address
and the stack of each processor is 1 KiB in size.

The memory map that BOOTBOOT sets up is the following:

| Address Range                             | Size          | Description                           |
|-------------------------------------------|:-------------:|---------------------------------------|
| 0xFFFFFFFF_FFFFFFFF - 0xFFFFFFFF_FFE02000 | 2 MiB - 8 KiB | Kernel text + data + stack            |
| 0xFFFFFFFF_FFE02000 - 0xFFFFFFFF_F8000000 |   ~126 MiB    | BOOTBOOT structures (e.g framebuffer) |
| 0xFFFFFFFF_F8000000 - 0x00000000_00000000 |      -        | User space, initrd                    |

Stacks begin at address 0 and grow downwards.
BOOTBOOT allocates a stack of 1 KiB in size for each processor. So, for a maximum of 256 processors, we need 256 KiB stack memory
(we will never run on that many proessors, but this is the maximum for our design. Support for more processors seems to have been added on ACPI Revision 3.0).
This means that the maximum kernel size can be 2 MiB - 8 Kib - 256 Kib = ~1.74 MiB.

We immediately face a problem for our Rust kernel. We have a very small stack.
Linux seems to be using between 8 KiB and 16 KiB for x86/x86_64 architectures. Since this is a micro-kernel we could opt for the minimum, if we are careful.
In any case, even a 4 KiB stack per processor would require 1 MiB memory for 256 processors, leaving the rest <1 MiB for the kernel.
An 8 KiB stack wouldn't fit the pre-allocated memory region.

Given that we don't know at the moment neither the exact required stack memory, nor the kernel size we need a more flexible setup.
So we will prepare a new address space, in order to allow this flexibility.

One way to do this is to map the BOOTBOOT and kernel regions lower in memory, to allow more stack space at the top.
Unfortunately, there is a subtle detail, which makes it difficult to implement. If we naively map the kernel code to another location in memory,
then when we switch address spaces, we will immediately page fault. To avoid this we have to map the kernel twice in the new address space.
Once in the desired location, lower in memory. And the second time at exactly the same place where it is in the current address space, i.e address 0xFFFFFFFF_FFE02000.

Given that this procedure can be quite error-prone, we propose a different plan. We will move the stack memory below the BOOTBOOT structures.
This will allow a maximum kernel size of ~1.74MiB as explained above and an arbitrary size for stack memory.
Changing the stack pointer is less tricky. It merely invalidates the current stack frame. This means that after changing the stack pointer, we cannot
reference any local variables and cannot return from the currently executing function. But we can call a new function and establish the new stack frame there.

The memory map we will build for our kernel environment is the following:

| Address Range                             |    Size   | Description                           |
|-------------------------------------------|:---------:|---------------------------------------|
| 0xFFFFFFFF_FFFFFFFF - 0xFFFFFFFF_FFFC0000 |  256 KiB  | Initial max stack set by BOOTBOOT     |
| 0xFFFFFFFF_FFFC0000 - 0xFFFFFFFF_FFE02000 |  ~1.74 MiB| Kernel text + data                    |
| 0xFFFFFFFF_FFE02000 - 0xFFFFFFFF_F8000000 |  ~126 MiB | BOOTBOOT structures (e.g framebuffer) |
| 0xFFFFFFFF_F8000000 -    stack_start      | arbitrary | Stack memory                          |
|      stack_start    - 0x00000000_00000000 |     -     | User space, initrd                    |


TODO: Mention we can't even perform a virtual memory mapping, because the stack size of 1 KiB isn't enough. Workaround: use framebuffer memory.

We will be using constructs with very limited functionality and we will examine the used stack very closely.
Fortunately, there are tools to do that. Enter the `-Z emit-stack-sizes` rust flag, which instructs LLVM to add
stack size metadata into the final ELF archive. We can combine this with cargo's
[stack-size](https://crates.io/crates/stack-sizes) to get a report on the most stack intensive functions.

Assuming `kernel.elf` is an ELF archive with stack size information, then the following produces
a list of the top 10 functions that consume the most stack memory:

``` bash
$ stack-sizes kernel.elf | sort -rnk2 | head -n10
```

Light environment features:

- [x] Simple serial (COM1) support for kernel logging. Use plain buffers, no fancy Rust formatting.
- [x] Global Descriptor Table.
- [x] Interrupt Descriptor Table. Always panic with a simple message.
- [x] Bump physical memory allocator.
- [x] The memory mapping that was discussed above.
- [x] New stacks per processor as discussed above.

