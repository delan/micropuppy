#![no_std]
#![no_main]

use core::arch::global_asm;
use core::convert::Infallible;
use core::panic::PanicInfo;
use core::ptr;

use fdt::standard_nodes::MemoryRegion;
use ufmt::uwriteln;
use ufmt_write::uWrite;

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
    let mut uart0 = Pl011Writer::new(&uart0);

    uwriteln!(uart0, "woof!!!! wraaaooo!!").unwrap();
}

struct Pl011Writer(*mut u8);

impl Pl011Writer {
    fn new(uart: &MemoryRegion) -> Self {
        // UARTDR is at starting address
        Self(uart.starting_address as *mut u8)
    }
}

impl uWrite for Pl011Writer {
    type Error = Infallible;

    fn write_str(&mut self, value: &str) -> Result<(), Self::Error> {
        for byte in value.bytes() {
            unsafe { ptr::write_volatile(self.0, byte) }
        }

        Ok(())
    }
}
