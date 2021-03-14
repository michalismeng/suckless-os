# Suckless Operating System

This is an attempt to make a simplistic yet working operating system. Our main
focus will be on creating an elegant and solid design, even if we end up
sacrificing performance in terms of speed and space.

Our kernel is going to be a micro-kernel. Our primary target is x86-64, although
it would be interesting to explore other architectures as well.

We use the BOOTBOOT bootloader, a multi-platform bootloader designed mainly for
micro-kernels. We are interested in its ability to directly setup a x86-64 long
mode environment, with color mode enabled. You can find more information at
https://gitlab.com/bztsrc/bootboot.

## Kernel features

Here is a list of the kernel features that we plan to implement over time:

- [ ] Logging mechanism
- [ ] Physical memory manager
- [ ] Virtual memory manager
- [ ] Interprocess communication mechanism
- [ ] Process scheduler

After implementing the above, we will turn our attention to the kernel's
ecosystem: the necessary drivers to support basic user interaction.

## How to build

Building a bootable image of the kernel is a three step process:

1. Build the Rust kernel and produce a freestanding ELF archive.
2. Put the ELF archive in an initrd directory.
3. Pack initrd into a bootable ``.img`` file, by using BOOTBOOT tools.

The above can be accomplished by running

   ``` bash
   $ make kernel
   $ make initrd
   $ make disk
   ```
Note that each ``make`` rule depends on the previous one(s), so running the last
command is equivalent to running all of them. In addition, if you don't have
BOOTBOOT, the above commands will clone the project from GitLab and build it.
For more information on the build process, take a look at the Makefile in the
project's root directory.

## How to run

To run the project, execute:

   ``` bash
   $ make run-x86
   ```

The above command will start a QEMU virtual machine running our kernel.

You can also specify the number of cores of the virtual machine. For example:

   ``` bash
   $ make ncores=4 run-x86
   ```

Do not expect to see anything printed on screen. The kernel only outputs
information to the serial port.

