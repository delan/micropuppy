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
#[derive(Debug)]
#[repr(C)]
pub struct Context {
    /// General-purpose registers `x0` through `x31`.
    x: [u64; 32],
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
            x: [0; 32],
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
