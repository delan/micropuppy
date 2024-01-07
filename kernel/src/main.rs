#![no_std]
#![no_main]
#![feature(panic_info_message)]
#![deny(clippy::undocumented_unsafe_blocks)]

#[allow(unused_macros)]
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
        ::core::arch::asm!(concat!("mrs {}, ", $special), out(reg) result);
        result
    }};
}

macro_rules! write_special_reg {
    ($special:literal, $value:expr) => {
        ::core::arch::asm!(concat!("msr ", $special, ", {:x}"), in(reg) $value);
    };
}

mod a53;
mod gicv2;
mod logging;
mod reg;
mod scheduler;
mod sync;
mod task;
mod tt;

use core::arch::{asm, global_asm};
use core::fmt::Write;
use core::panic::PanicInfo;
use core::ptr::null;

use allocator::Allocator;
use scheduler::Scheduler;
use task::Context;

use crate::gicv2::InterruptId;
use crate::logging::Pl011Writer;
use crate::sync::OnceCell;
use crate::tt::page::PageBox;
use crate::tt::table::TranslationTable;
use crate::tt::Level0;
// use crate::tt::{PageBox, TranslationTable};

global_asm!(include_str!("entry.s"), options(raw));

extern "C" {
    static VECTORS: [u8; 0x800];
}

// TODO starting with the incorrect values seems bad, is this bad?
static mut TIMER_INTERRUPT: InterruptId = InterruptId::spurious();
static mut GICD: gicv2::Distributor = gicv2::Distributor::new(null());
static mut GICC: gicv2::CpuInterface = gicv2::CpuInterface::new(null());
static mut SCHEDULER: OnceCell<Scheduler> = OnceCell::new();
static mut ALLOCATOR: OnceCell<Allocator> = OnceCell::new();

#[no_mangle]
unsafe extern "C" fn vector_el1_sp0_synchronous() {
    log::trace!("vector_el1_sp0_synchronous");
    panic_on_synchronous_or_serror(b'A');
}

#[no_mangle]
unsafe extern "C" fn vector_el1_sp0_irq() {
    log::trace!("vector_el1_sp0_irq");
}

#[no_mangle]
unsafe extern "C" fn vector_el1_sp0_fiq() {
    log::trace!("vector_el1_sp0_fiq");
}

#[no_mangle]
unsafe extern "C" fn vector_el1_sp0_serror() {
    log::trace!("vector_el1_sp0_serror");
    panic_on_synchronous_or_serror(b'D');
}

#[no_mangle]
unsafe extern "C" fn vector_el1_sp1_synchronous() {
    log::trace!("vector_el1_sp1_synchronous");
    panic_on_synchronous_or_serror(b'E');
}

#[no_mangle]
unsafe extern "C" fn vector_el1_sp1_irq() {
    log::trace!("vector_el1_sp1_irq");
}

#[no_mangle]
unsafe extern "C" fn vector_el1_sp1_fiq() {
    log::trace!("vector_el1_sp1_fiq");
}

#[no_mangle]
unsafe extern "C" fn vector_el1_sp1_serror(_context: *const Context) -> *const Context {
    log::trace!("vector_el1_sp1_serror");
    panic_on_synchronous_or_serror(b'H');
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a64_synchronous(_context: *const Context) -> *const Context {
    log::trace!("vector_el0_a64_synchronous");
    panic_on_synchronous_or_serror(b'I');
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a64_irq(mut context: *const Context) -> *const Context {
    log::trace!("vector_el0_a64_irq");
    log::debug!("{:?}", *context);

    GICC.handle(|cpuid, interrupt_id| {
        log::trace!("elx_irq cpuid = {cpuid}, interrupt_id = {interrupt_id:?}");
        match interrupt_id {
            x if x == TIMER_INTERRUPT => {
                write_special_reg!("CNTP_TVAL_EL0", read_special_reg!("CNTFRQ_EL0") / 10);

                if let Some(scheduler) = SCHEDULER.get_mut() {
                    context = scheduler.schedule().context();
                }
            }
            _ => {}
        }
    });

    context
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a64_fiq(context: *const Context) -> *const Context {
    log::trace!("vector_el0_a64_fiq");

    context
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a64_serror(_context: *const Context) -> *const Context {
    log::trace!("vector_el0_a64_serror");
    panic_on_synchronous_or_serror(b'L');
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a32_synchronous() {
    log::trace!("vector_el0_a32_synchronous");
    panic_on_synchronous_or_serror(b'M');
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a32_irq() {
    log::trace!("vector_el0_a32_irq");
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a32_fiq() {
    log::trace!("vector_el0_a32_fiq");
}

#[no_mangle]
unsafe extern "C" fn vector_el0_a32_serror() {
    log::trace!("vector_el0_a32_serror");
    panic_on_synchronous_or_serror(b'P');
}

fn panic_on_synchronous_or_serror(kind: u8) -> ! {
    // TODO get rid of these kind codes, no need to call this from asm
    let kind = match kind {
        b'A' => "synchronous, SP_EL0",
        b'D' => "SError, SP_EL0",
        b'E' => "synchronous, SP_ELx",
        b'H' => "SError, SP_ELx",
        b'I' => "synchronous, lower64",
        b'L' => "SError, lower64",
        b'M' => "synchronous, lower32",
        b'P' => "SError, lower32",
        _ => unreachable!(),
    };
    // TODO migrate to SystemRegister api
    let syndrome = unsafe { read_special_reg!("ESR_EL1") };
    let exception_class = syndrome >> 26 & 0x3F;
    let reason = match exception_class {
        0x00 => Some("Unknown reason"),
        0x15 => Some("SVC instruction execution in AArch64 state"),
        _ => None,
    };
    if let Some(reason) = reason {
        panic!(
            "Exception ({}): {:016X}h\n    reason {:02X}h = {}",
            kind, syndrome, exception_class, reason
        );
    } else {
        panic!(
            "Exception ({}): {:016X}h\n    reason {:02X}h",
            kind, syndrome, exception_class
        );
    }
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

    extern "C" {
        static _kernel_va: u8;
        static _kernel_pa: u8;
        static _ekernel_va: u8;
    }

    // TODO: PageBox
    let mut tt = PageBox::new(TranslationTable::<Level0>::new());

    // annoying: relocation fails (out of range) when we try and use the PA like we do the VAs below
    let pa: usize;
    unsafe { asm!("ldr {}, =_kernel_pa", out(reg) pa) };

    tt.map_contiguous(
        unsafe { &_kernel_va } as *const _ as usize,
        unsafe { &_ekernel_va } as *const _ as usize,
        pa,
        "rx",
    );

    unsafe {
        asm!("msr TTBR1_EL1, {:x}", "dsb sy", in(reg) tt.addr().addr());
    }

    log::error!("error woof");
    log::warn!("warn woof");
    log::info!("info woof");
    log::debug!("debug woof");
    log::trace!("trace woof");

    log::debug!("woof!!!! wraaaooo!!");

    // enable timer interrupts
    unsafe {
        log::debug!("CNTFRQ_EL0 = {:016X}h", read_special_reg!("CNTFRQ_EL0"));
        write_special_reg!("CNTP_CTL_EL0", 1u64);
    }

    let timer = fdt.find_compatible(&["arm,armv8-timer"]).unwrap();
    let timer_interrupts = timer.property("interrupts").unwrap().value;
    let mut timer_interrupts = gicv2::InterruptSpecifier::interrupts_iter(timer_interrupts);
    unsafe { TIMER_INTERRUPT = timer_interrupts.nth(1).unwrap().interrupt_id().unwrap() };

    let gic = fdt.find_compatible(&["arm,cortex-a15-gic"]).unwrap();
    let mut gic = gic.reg().unwrap();
    unsafe {
        GICD = gicv2::Distributor::new(gic.next().unwrap().starting_address);
        GICD.enable();

        // TODO document this, is it the virt or the non-secure phys?
        // https://github.com/torvalds/linux/blob/90b0c2b2edd1adff742c621e246562fbefa11b70/Documentation/devicetree/bindings/timer/arm%2Carch_timer.yaml#L44-L58
        GICD.enable_interrupt(TIMER_INTERRUPT);

        GICC = gicv2::CpuInterface::new(gic.next().unwrap().starting_address);
        GICC.enable();
    }

    unsafe {
        // set up vector table base address
        asm!("msr VBAR_EL1, {}", in(reg) &VECTORS);

        SCHEDULER.get_or_init(|| Scheduler::new());
    }

    extern "C" {
        // FIXME relocation R_AARCH64_ADR_PREL_PG_HI21 out of range:
        // 281476054814720 is not in [-4294967296, 4294967295]; references '_buddy_alloc_tree_pa'
        // static _buddy_alloc_tree_pa: u8;
        // static _kernel_pa: u8;
        static _buddy_alloc_tree_va: u8;
    }
    let ram = fdt.memory().regions().next().unwrap();
    let allocator_start = unsafe { &_buddy_alloc_tree_va } as *const u8;
    let allocator_start_pa = unsafe { allocator_start.sub(0xffff000000000000 - 0x40000000) };
    let allocator_len = unsafe {
        ram.size.unwrap() - allocator_start_pa.offset_from(ram.starting_address) as usize
    };
    let allocator_end = unsafe { (&_buddy_alloc_tree_va as *const u8).add(allocator_len) };
    unsafe {
        dbg!(ALLOCATOR.get_or_init(|| Allocator::new(allocator_start, allocator_end)));
    }

    // Permanently transfer control to the scheduler.
    // We don’t need to explicitly clear DAIF.I, because the initial task_restore (entry.s) will
    // clear it when ERET copies the task’s SPSR to PSTATE.
    unsafe { SCHEDULER.get_mut() }.unwrap().start();
}
