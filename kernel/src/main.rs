#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![deny(clippy::undocumented_unsafe_blocks)]

mod logging;

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr;

use crate::logging::Pl011Writer;

global_asm!(include_str!("start.s"));
global_asm!(include_str!("vectors.s"));
extern "C" {
    fn vectors();
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

    let gicd = 0x8000000 as *mut u32;
    let gicd_ctlr = unsafe { gicd.add(0) };
    let gicd_isenabler = |n: usize| unsafe { gicd.add(64 + n) };
    // let gicd_itargetsr = |n: usize| unsafe { gicd.add(512 + n) };

    unsafe {
        // enable group 0 interrupts
        ptr::write_volatile(gicd_ctlr, 1);
        // enable 0xd ppi = 0x1d interrupt
        ptr::write_volatile(gicd_isenabler(0x1e / 32), 1 << (0x1e % 32));
        // make core 0 the target
        // ptr::write_volatile(gicd_itargetsr(0x1e / 4), 0x01 << (0x1e % 4));
    }
    
    let gicc = 0x8010000 as *mut u32;
    let gicc_ctlr = unsafe { gicc.add(0) };
    let gicc_pmr = unsafe { gicc.add(1) };
    let gicc_bpr = unsafe { gicc.add(2) };

    unsafe {
        // enable group 0 interrupts
        ptr::write_volatile(gicc_ctlr, 1);
        // configure priority
        ptr::write_volatile(gicc_pmr, 0xff);
        // ptr::write_volatile(gicc_bpr, 0);
    }

    unsafe {
        // set up vector table base address
        asm!("msr VBAR_EL1, {}", in(reg) vectors);
        // unmask all interrupts
        asm!("msr DAIF, {:x}", in(reg) 0);
    }

    log::debug!("CNTP_CTL_EL0 = {:016X}h", read_special_reg!("CNTP_CTL_EL0"));

    loop {}

    // log::debug!("EL = {:X}h", read_special_reg!("CurrentEL") >> 2);

    // log::debug!("VBAR_EL1 = {:016X}h", read_special_reg!("VBAR_EL1"));
    // unsafe {
    //     asm!("msr VBAR_EL1, {}", in(reg) vectors);
    // }
    // log::debug!("VBAR_EL1 = {:016X}h", read_special_reg!("VBAR_EL1"));

    // log::debug!("DAIF = {:X}h", read_special_reg!("DAIF") >> 6);
    // let daif = 0u64;
    // unsafe {
    //     asm!("msr DAIF, {}", in(reg) daif);
    // }
    // log::debug!("DAIF = {:X}h", read_special_reg!("DAIF") >> 6);

    // log::debug!("CNTKCTL_EL1 = {:016X}h", read_special_reg!("CNTKCTL_EL1"));
    // log::debug!("CNTV_CTL_EL0 = {:016X}h", read_special_reg!("CNTV_CTL_EL0"));
    // log::debug!(
    //     "CNTP_TVAL_EL0 = {:016X}h",
    //     read_special_reg!("CNTP_TVAL_EL0")
    // );
    // log::debug!("CNTP_CTL_EL0 = {:016X}h", read_special_reg!("CNTP_CTL_EL0"));

    // let gicc_ctlr = 0x8010000 as *mut u32;
    // unsafe {
    //     ptr::write_volatile(gicc_ctlr, 1);
    // }

    // let gicd_ctlr = 0x8000000 as *mut u32;
    // unsafe {
    //     ptr::write_volatile(gicd_ctlr, 1);
    // }

    // let gicc_pmr = 0x8000004 as *mut u32;
    // unsafe {
    //     ptr::write_volatile(gicc_pmr, 0xff);
    // }

    // let gicc_bpr = 0x8000008 as *mut u32;
    // unsafe {
    //     ptr::write_volatile(gicc_bpr, 0);
    // }

    // let gicd_itargetsr6 = (0x8000000 + 0x800 + 6 * 0x4) as *mut u32;
    // let gicd_itargetsr7 = (0x8000000 + 0x800 + 7 * 0x4) as *mut u32;
    // unsafe {
    //     ptr::write_volatile(gicd_itargetsr7, 1 << 8);
    //     ptr::write_volatile(gicd_itargetsr7, 1 << 16);
    //     ptr::write_volatile(gicd_itargetsr6, 1 << 16);
    //     ptr::write_volatile(gicd_itargetsr6, 1 << 24);
    // }

    // for i in 0..16 {
    //     unsafe {
    //         ptr::write_volatile((0x8000000 + 0x800 + i* 4) as *mut u32, 0x01010101);
    //     }
    // }

    // for i in 0..16 {
    //     unsafe {
    //         ptr::write_volatile((0x8000100 + i* 4) as *mut u32, 0xffffffff);
    //     }
    // }

    // let gicd_isenabler0 = 0x8000100 as *mut u32;
    // unsafe {
    //     ptr::write_volatile(gicd_isenabler0, 1 << (16 + 0xd));
    //     ptr::write_volatile(gicd_isenabler0, 1 << (16 + 0xe));
    //     ptr::write_volatile(gicd_isenabler0, 1 << (16 + 0xb));
    //     ptr::write_volatile(gicd_isenabler0, 1 << (16 + 0xa));
    // }

    // log::debug!("CNTP_CTL_EL0 = {:016X}h", read_special_reg!("CNTP_CTL_EL0"));

    // // enable timer interrupts
    // write_special_reg!("CNTP_CTL_EL0", 1u64);
    

    // log::debug!("CNTP_CTL_EL0 = {:016X}h", read_special_reg!("CNTP_CTL_EL0"));

    // log::debug!("CNTFRQ_EL0 = {:016X}h", read_special_reg!("CNTFRQ_EL0"));

    // // unsafe { asm!("svc #0"); }
    // // unsafe { asm!("svc #0"); }
    // // unsafe { asm!("svc #0"); }

    // // return;
    // loop {
    //     // log::debug!("CNTP_CTL_EL0 = {:016X}h", read_special_reg!("CNTP_CTL_EL0"));

    //     // return;
    //     // log::debug!("CNTP_CTL_EL0 = {:016X}h, CNTP_TVAL_EL0 = {:016X}h", read_special_reg!("CNTP_CTL_EL0"), read_special_reg!("CNTP_TVAL_EL0"));
    //     // let cntpct_el0 = read_special_reg!("CNTPCT_EL0");
    //     // log::debug!("{:016X}h ({})", cntpct_el0, cntpct_el0);
    //     // let cntvct_el0 = read_special_reg!("CNTVCT_EL0");
    //     // log::debug!("{:016X}h ({})", cntvct_el0, cntvct_el0);
    //     // break;
    // }
}
