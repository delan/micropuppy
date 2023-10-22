.POSIX:

QEMUFLAGS =
KERNEL =

# https://krinkinmu.github.io/2020/11/21/EFI-aarch64.html#bonus-testing-in-qemu
# http://www.redfelineninja.org.uk/daniel/2018/02/running-an-iso-installer-image-for-arm64-aarch64-using-qemu-and-kvm/

run-kernel: $(KERNEL)
	qemu-system-aarch64 $(QEMUFLAGS) \
		-M virt -cpu cortex-a53 -m 4096 -nographic \
		-kernel $(KERNEL)
	@echo

run-uefi: QEMU_EFI.img vars.qcow2
	qemu-system-aarch64 $(QEMUFLAGS) \
		-M virt-8.0 -cpu cortex-a53 -m 4096 \
		-drive if=pflash,format=raw,file=QEMU_EFI.img \
		-drive if=pflash,file=vars.qcow2 \
		-drive if=virtio,format=raw,file=nixos-minimal-23.05.4392.b5f8ec6be261-aarch64-linux.iso

QEMU_EFI.img: QEMU_EFI.img.gz
	zcat $? > $@

QEMU_EFI.img.gz:
	curl -LO http://snapshots.linaro.org/components/kernel/leg-virt-tianocore-edk2-upstream/latest/QEMU-AARCH64/RELEASE_GCC5/QEMU_EFI.img.gz

vars.qcow2:
	qemu-img create -f qcow2 vars.qcow2 64M

.PHONY: run-kernel run-uefi
