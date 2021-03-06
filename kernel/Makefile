all: sos.x86_64.elf

sos.x86_64.elf: $(shell find . -type f -name "*.rs")
	RUSTFLAGS="-Z emit-stack-sizes" cargo xbuild --target ./triplets/sos-x86.json
	cp ./target/sos-x86/debug/sos sos.x86_64.elf
	# strip -s -K mmio -K fb -K bootboot -K environment -K initstack sos.x86_64.elf

sos.x86_64.elf-release:
	cargo xbuild --target ./triplets/sos-x86.json --release
	cp ./target/sos-x86/release/sos sos.x86_64-release.elf
	strip --remove-section=".stack_sizes" -s -K mmio -K fb -K bootboot \
											 -K environment -K initstack \
											 sos.x86_64-release.elf

doc:
	cargo doc

clean:
	rm -f *.elf *.o 2>/dev/null || true
