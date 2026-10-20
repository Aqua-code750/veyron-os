.PHONY: all run run-tcg clean

all:
	python tools/build_os.py

run: all
	qemu-system-x86_64 -m 2G -smp 2 -machine q35 -device virtio-vga -device virtio-net,netdev=n0 -netdev user,id=n0 -drive file=veyron.img,format=raw -device usb-ehci -device usb-kbd -device usb-mouse -bios OVMF.fd

run-tcg: all
	qemu-system-x86_64 -accel tcg -m 2G -smp 2 -machine q35 -device virtio-vga -device virtio-net,netdev=n0 -netdev user,id=n0 -drive file=veyron.img,format=raw -device usb-ehci -device usb-kbd -device usb-mouse -bios OVMF.fd

clean:
	cargo clean
	rm -f veyron.img veyron.iso
