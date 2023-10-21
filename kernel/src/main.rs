#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod logging;

use core::arch::global_asm;
use core::fmt::Write;
use core::panic::PanicInfo;

use crate::logging::Pl011Writer;

global_asm!(include_str!("start.s"));

#[panic_handler]
fn on_panic(info: &PanicInfo) -> ! {
    // We've already panicked, so this is our last ditch effort to communicate to the user any
    // relevant information that could be used to debug the issue. As such, if writing fails, we
    // can't do much about it.
    trait ResultExt {
        fn ignore(self);
    }

    impl<T, E> ResultExt for Result<T, E> {
        fn ignore(self) {
            // do nothing
        }
    }

    const RED_BOLD: &str = "\x1b[31m\x1b[1m";
    const BRIGHT_BLACK: &str = "\x1b[38;5;240m";
    const SGR0: &str = "\x1b[0m";

    if let Some(writer) = unsafe { &mut logging::WRITER } {
        write!(writer, "\n\n💣 💥 🐶 {RED_BOLD}panicked{SGR0} 🐶 💥 💣").ignore();
        if let Some(location) = info.location() {
            write!(writer, " {BRIGHT_BLACK}at {location}{SGR0}").ignore();
        }
        writeln!(writer).ignore();

        if let Some(message) = info.message() {
            write!(writer, "{message}").ignore();
        } else if let Some(payload) = info.payload().downcast_ref::<&'static str>() {
            write!(writer, "{payload}").ignore();
        } else {
            write!(writer, "<no message>").ignore();
        }
        write!(writer, "\n\n").ignore();
    }

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
    let uart0 = Pl011Writer::new(uart0.starting_address);
    logging::init(uart0, log::LevelFilter::Trace);

    log::error!("error woof");
    log::warn!("warn woof");
    log::info!("info woof");
    log::debug!("debug woof");
    log::trace!("trace woof");
}
