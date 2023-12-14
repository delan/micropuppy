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
        macro_rules! write_regL {
            ($f:expr, $name:expr, $value:expr) => {
                write!($f, "    {:>3}: {:#018x},", $name, $value)
            };
        }

        macro_rules! write_regR {
            ($f:expr, $name:expr, $value:expr) => {
                writeln!($f, " {:>3}: {:#018x},", $name, $value)
            };
        }

        writeln!(f, "Context {{")?;
        write_regL!(f, "x0", self.gprs[0])?;
        write_regR!(f, "x1", self.gprs[1])?;
        write_regL!(f, "x2", self.gprs[2])?;
        write_regR!(f, "x3", self.gprs[3])?;
        write_regL!(f, "x4", self.gprs[4])?;
        write_regR!(f, "x5", self.gprs[5])?;
        write_regL!(f, "x6", self.gprs[6])?;
        write_regR!(f, "x7", self.gprs[7])?;
        write_regL!(f, "x8", self.gprs[8])?;
        write_regR!(f, "x9", self.gprs[9])?;
        write_regL!(f, "x10", self.gprs[10])?;
        write_regR!(f, "x11", self.gprs[11])?;
        write_regL!(f, "x12", self.gprs[12])?;
        write_regR!(f, "x13", self.gprs[13])?;
        write_regL!(f, "x14", self.gprs[14])?;
        write_regR!(f, "x15", self.gprs[15])?;
        write_regL!(f, "x16", self.gprs[16])?;
        write_regR!(f, "x17", self.gprs[17])?;
        write_regL!(f, "x18", self.gprs[18])?;
        write_regR!(f, "x19", self.gprs[19])?;
        write_regL!(f, "x20", self.gprs[20])?;
        write_regR!(f, "x21", self.gprs[21])?;
        write_regL!(f, "x22", self.gprs[22])?;
        write_regR!(f, "x23", self.gprs[23])?;
        write_regL!(f, "x24", self.gprs[24])?;
        write_regR!(f, "x25", self.gprs[25])?;
        write_regL!(f, "x26", self.gprs[26])?;
        write_regR!(f, "x27", self.gprs[27])?;
        write_regL!(f, "x28", self.gprs[28])?;
        write_regR!(f, "x29", self.gprs[29])?;
        write_regL!(f, "x30", self.gprs[30])?;
        write_regR!(f, "psr", self.psr)?;
        write_regL!(f, "pc", self.pc as usize)?;
        write_regR!(f, "sp", self.sp as usize)?;
        writeln!(f, "}}")?;

        Ok(())
    }
}
