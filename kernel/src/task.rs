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
    /// General-purpose registers `x0` through `x31`.
    gprs: [u64; 32],
    /// The program counter, from `ELR_EL1`.
    pc: *const (),
    /// The stack pointer, from `SP_EL0`.
    sp: *const (),
    /// The program status register (`PSTATE`), from `SPSR_EL1`.
    psr: u64,
}

impl Context {
    pub fn new(initial_pc: *const (), initial_sp: *const ()) -> Self {
        Self {
            gprs: [0; 32],
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
        writeln!(f, "Context {{")?;

        for pair in 0..16 {
            let index0 = 2 * pair;
            let space0 = if index0 < 10 { " " } else { "" };
            write!(f, "    {}x{}: {:#018x},", space0, index0, self.gprs[index0])?;

            let index1 = 2 * pair + 1;
            let space1 = if index1 < 10 { " " } else { "" };
            writeln!(f, " {}x{}: {:#018x},", space1, index1, self.gprs[index1])?;
        }

        writeln!(
            f,
            "     pc: {:#018x},  sp: {:#018x},",
            self.pc as usize, self.sp as usize
        )?;

        // TODO: decode important fields and display them alongside the PSR
        writeln!(f, "    psr: {:#018x},", self.psr)?;

        writeln!(f, "}}")?;

        Ok(())
    }
}
