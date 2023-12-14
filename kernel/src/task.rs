use core::fmt;

#[derive(Debug)]
pub struct Task {
    /// Pointer to the bottom of the task's kernel stack.
    sp_el1: *const (),
}

impl Task {
    pub fn new(sp_el1: *const (), context: Context) -> Self {
        unsafe { Context::from_sp_el1_mut(sp_el1 as *mut _).write(context) }

        Self { sp_el1 }
    }

    pub fn context(&self) -> &Context {
        unsafe { &*Context::from_sp_el1(self.sp_el1) }
    }

    pub fn context_mut(&mut self) -> &mut Context {
        unsafe { &mut *Context::from_sp_el1_mut(self.sp_el1 as *mut _) }
    }

    pub fn start(&self) -> ! {
        extern "C" {
            // defined in entry.s
            fn task_start(context: *const Context) -> !;
        }

        unsafe { task_start(self.context()) }
    }
}

/// The processor state of a task, saved and restored on context switches.
///
/// **This struct MUST be kept in sync with the `task_save` and `task_restore` macros defined in
/// `entry.s`.**
#[repr(C)]
pub struct Context {
    /// General-purpose registers `x0` through `x30`.
    ///
    /// `x31` is `xzr` (the zero register) and is not saved.
    gprs: [u64; 31],
    /// The program status register (`PSTATE`), from `SPSR_EL1`.
    psr: u64,
    /// The program counter, from `ELR_EL1`.
    pc: *const (),
    /// The stack pointer, from `SP_EL0`.
    sp: *const (),
}

impl Context {
    pub fn new(initial_pc: *const (), initial_sp: *const ()) -> Self {
        Self {
            gprs: [0; 31],
            pc: initial_pc,
            psr: 0,
            sp: initial_sp,
        }
    }

    fn from_sp_el1(sp_el1: *const ()) -> *const Context {
        unsafe { (sp_el1 as *const Context).sub(1) }
    }

    fn from_sp_el1_mut(sp_el1: *mut ()) -> *mut Context {
        unsafe { (sp_el1 as *mut Context).sub(1) }
    }
}

impl fmt::Debug for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct R<'a>(&'a str, u64);

        impl fmt::Display for R<'_> {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:>3}: {:#018x}", self.0, self.1)
            }
        }

        struct Blank;

        impl fmt::Display for Blank {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "                       ")
            }
        }

        // not destructuring due to the required `as u64` conversions
        let x = self.gprs;
        let sp = self.sp as u64;
        let pc = self.pc as u64;
        let psr = self.psr;

        writeln!(f, "Context {{")?;
        writeln!(f, "    {}, {},", R("x0", x[0]), R("x1", x[1]))?;
        writeln!(f, "    {}, {},", R("x2", x[2]), R("x3", x[3]))?;
        writeln!(f, "    {}, {},", R("x4", x[4]), R("x5", x[5]))?;
        writeln!(f, "    {}, {},", R("x6", x[6]), R("x6", x[7]))?;
        writeln!(f, "    {}, {},", R("x8", x[8]), R("x9", x[9]))?;
        writeln!(f, "    {}, {},", R("x10", x[10]), R("x11", x[11]))?;
        writeln!(f, "    {}, {},", R("x12", x[12]), R("x13", x[13]))?;
        writeln!(f, "    {}, {},", R("x14", x[14]), R("x15", x[15]))?;
        writeln!(f, "    {}, {},", R("x16", x[16]), R("x16", x[17]))?;
        writeln!(f, "    {}, {},", R("x18", x[18]), R("x19", x[19]))?;
        writeln!(f, "    {}, {},", R("x20", x[20]), R("x21", x[21]))?;
        writeln!(f, "    {}, {},", R("x22", x[22]), R("x23", x[23]))?;
        writeln!(f, "    {}, {},", R("x24", x[24]), R("x25", x[25]))?;
        writeln!(f, "    {}, {},", R("x26", x[26]), R("x26", x[27]))?;
        writeln!(f, "    {}, {},", R("x28", x[28]), R("x29", x[29]))?;
        writeln!(f, "    {}, {},", R("x30", x[30]), R("sp", sp))?;
        writeln!(f, "    {}, {},", R("pc", pc), R("psr", psr))?;
        writeln!(f, "}}")?;

        Ok(())
    }
}
