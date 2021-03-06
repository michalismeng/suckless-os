# BOOTBOOT supports multiple platforms, but our kernel only supports x86.
PLATFORM=x86
#PLATFORM=rpi
#PLATFORM=icicle

# Path to the EFI image. This is used for running QEMU with EFI.
OVMF=./efi/edk2.git/ovmf-x64/OVMF-pure-efi.fd
# Path to the BOOTBOOT project.
BOOTBOOT=bootboot
# Path to the mkbootimg tool.
MKBOOT=$(BOOTBOOT)/mkbootimg
# Number of processors to use.
ncores=1
# If we use the 'isa-debug-exit' device in our kernel to exit QEMU, our kernel
# returns 17. We use this to treat 17 as a successful exit code for the
# Makefile.
QEMU_EXIT_SUCCESS=17

.PHONY: kernel

all: $(BOOTBOOT) $(MKBOOT)/mkbootimg initrd disk

# Clone BOOTBOOT from GitLab.
$(BOOTBOOT):
	@git clone https://gitlab.com/bztsrc/bootboot.git

# Compile the mkbootimg tool.
$(MKBOOT)/mkbootimg:
	@make -C $(MKBOOT) all

# Compile the Rust kernel.
kernel:
	@make -C kernel

# Create an initial RAM disk image.
initrd: kernel
	@mkdir initrd initrd/sys 2>/dev/null | true
ifeq ($(PLATFORM),x86)
	cp kernel/sos.x86_64.elf initrd/sys/core
else
	@echo "*** Unsupported platform '$(PLATFORM)'! ***"
	@exit 1
endif

# Create a disk image based on the initrd. This will be loaded by BOOTBOOT.
disk: $(MKBOOT)/mkbootimg initrd mkbootimg.json
	$(MKBOOT)/mkbootimg mkbootimg.json disk-$(PLATFORM).img
	@rm -rf initrd

run-x86: disk
	qemu-system-x86_64 -kernel $(BOOTBOOT)/dist/bootboot.bin \
					   -drive file=disk-x86.img,format=raw \
					   -m 128M -smp $(ncores) -serial stdio \
					   -D qemu.log -d int,cpu_reset --no-reboot \
					   -device isa-debug-exit,iobase=0xf4,iosize=0x04; \
					   exit $$(($(QEMU_EXIT_SUCCESS)-$$?))

run-x86-debug: disk
	# Use -s -S to listen and wait for GDB.
	qemu-system-x86_64 -kernel $(BOOTBOOT)/dist/bootboot.bin \
					   -drive file=disk-x86.img,format=raw \
					   -m 128M -smp $(ncores) -serial stdio \
					   -s -S

run-x86-efi: disk
	qemu-system-x86_64 -kernel $(BOOTBOOT)/dist/bootboot.bin \
					   -drive file=disk-x86.img,format=raw \
					   -m 128 -smp $(ncores) -serial stdio -bios $(OVMF) \
					   -D qemu.log -d int,cpu_reset --no-reboot \
					   -device isa-debug-exit,iobase=0xf4,iosize=0x04; \
					   exit $$(($(QEMU_EXIT_SUCCESS)-$$?))

doc:
	@make -C kernel doc

clean:
	@make -C kernel clean
	rm -rf initrd *.bin *.img 2>/dev/null || true
