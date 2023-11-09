#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod a53;
mod gicv2;
mod logging;
mod reg;

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;

use crate::a53::DAIF;
use crate::logging::Pl011Writer;
use crate::reg::system::Register;

global_asm!(include_str!("start.s"));
global_asm!(include_str!("vectors.s"));
extern "C" {
    fn vectors();
}

macro_rules! dbg {
    ($value:expr) => {{
        let value = $value;
        log::debug!("{:?}", value);
        value
    }};
}

macro_rules! read_special_reg {
    ($special:literal) => {{
        let result: u64;
        unsafe {
            asm!(concat!("mrs {}, ", $special), out(reg) result);
        }
        result
    }};
}

macro_rules! write_special_reg {
    ($special:literal, $value:expr) => {{
        unsafe {
            asm!(concat!("msr ", $special, ", {}"), in(reg) $value);
        }
    }};
}

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

    log::debug!("woof!!!! wraaaooo!!");

    // enable timer interrupts
    write_special_reg!("CNTP_CTL_EL0", 1u64);

    let timer = fdt.find_compatible(&["arm,armv8-timer"]).unwrap();
    let timer_interrupts = timer.property("interrupts").unwrap().value;
    let mut timer_interrupts = gicv2::InterruptSpecifier::interrupts_iter(timer_interrupts);

    let gic = fdt.find_compatible(&["arm,cortex-a15-gic"]).unwrap();
    let mut gic = gic.reg().unwrap();
    let mut gicd = gicv2::Distributor::new(gic.next().unwrap().starting_address);
    gicd.enable();

    // TODO document this, is it the virt or the non-secure phys?
    // https://github.com/torvalds/linux/blob/90b0c2b2edd1adff742c621e246562fbefa11b70/Documentation/devicetree/bindings/timer/arm%2Carch_timer.yaml#L44-L58
    gicd.enable_interrupt(timer_interrupts.nth(1).unwrap().interrupt_id().unwrap());

    let mut gicc = gicv2::CpuInterface::new(gic.next().unwrap().starting_address);
    gicc.enable();

    unsafe {
        // set up vector table base address
        asm!("msr VBAR_EL1, {}", in(reg) vectors);
        // unmask interrupts
        Register::<DAIF>::new().write_default(|w| w.i(false));
    }

    log::debug!("CNTP_CTL_EL0 = {:016X}h", read_special_reg!("CNTP_CTL_EL0"));

    loop {}
}
