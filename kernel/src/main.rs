#![no_std]
#![no_main]
#![deny(clippy::undocumented_unsafe_blocks)]

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
    // SAFETY: QEMU loads a FDT at the base of memory (0x4000_0000) for non-Linux images (e.g. ELFs)
    // passed to -kernel, provided that the image leaves enough space at the base of RAM for the
    // FDT.
    //
    // This does mean that there may not be an FDT at this location in memory. In this case, the
    // pointer is still valid to read from (avoiding UB) but Fdt::from_ptr will fail as the memory
    // (hopefully) does not the FDT magic value.
    //
    // See https://qemu-project.gitlab.io/qemu/system/arm/virt.html#hardware-configuration-information-for-bare-metal-programming.
    let fdt = unsafe { fdt::Fdt::from_ptr(0x4000_0000 as *const u8).unwrap() };

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
