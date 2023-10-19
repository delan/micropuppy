#![no_std]
#![no_main]

use core::arch::global_asm;
use core::panic::PanicInfo;
use core::ptr;

global_asm!(include_str!("start.s"));

#[panic_handler]
fn on_panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() {
    // TODO not this
    // https://qemu-project.gitlab.io/qemu/system/arm/virt.html#hardware-configuration-information-for-bare-metal-programming
    // https://doc.coreboot.org/mainboard/emulation/qemu-aarch64.html
    // https://mail.gnu.org/archive/html/qemu-discuss/2019-10/msg00014.html
    let fdt = fdt::Fdt::new(include_bytes!("../../qemu/virt-8.0.dtb")).unwrap();
    let uart0 = fdt.find_compatible(&["arm,pl011"]).unwrap();
    let uart0 = uart0.reg().unwrap().next().unwrap();
    let uart0_uartdr = uart0.starting_address as *mut u8;

    let out_str = b"woof!!!! wraaaooo!!";
    for byte in out_str {
        unsafe {
            ptr::write_volatile(uart0_uartdr, *byte);
        }
    }
}
