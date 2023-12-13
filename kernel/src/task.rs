// use core::arch::asm;
// use core::num::NonZeroUsize;

// pub struct Scheduler {
//     tasks: [Task; 2],
//     current: Option<NonZeroUsize>,
//     next: Option<NonZeroUsize>,
// }

// impl Scheduler {
//     pub fn new() -> Self {
//         let tasks = [
//             Task::new(task1 as *const _, unsafe { &TASK1_INITIAL_SP as *const _ }),
//             Task::new(task2 as *const _, unsafe { &TASK2_INITIAL_SP as *const _ }),
//         ];
//         Self {
//             tasks,
//             current: None,
//             next: NonZeroUsize::new(1),
//         }
//     }

//     pub fn current_task(&self) -> Option<NonZeroUsize> {
//         self.current
//     }

//     pub fn get(&self, task: NonZeroUsize) -> Option<&Task> {
//         self.tasks.get(task.get() - 1)
//     }

//     fn get_mut(&mut self, task: NonZeroUsize) -> Option<&mut Task> {
//         self.tasks.get_mut(task.get() - 1)
//     }

//     pub fn save(&mut self) {
//         let task = self.current.take().expect("No current task!");
//         let task = self.get_mut(task).expect("No such task!");
//         task.program_counter = read_special_reg!("ELR_EL1");
//         task.stack_pointer = read_special_reg!("SP_EL0");
//         task.pstate = read_special_reg!("SPSR_EL1");
//     }

//     pub fn run(&mut self) -> ! {
//         if let Some(task) = self.current {
//             panic!("Task {} is still running!", task);
//         }
//         let task = self.next.take().expect("No next task!");
//         self.current = Some(task);
//         self.next = NonZeroUsize::new(task.get() % self.tasks.len() + 1);
//         let task = self.get(task).expect("No such task!");

//         write_special_reg!("ELR_EL1", task.program_counter);
//         write_special_reg!("SPSR_EL1", task.pstate);
//         write_special_reg!("SP_EL0", task.stack_pointer);
//         write_special_reg!("SPSel", 0);
//         unsafe {
//             asm!("eret");
//         }
//         unreachable!();
//     }
// }

pub struct Scheduler {
    tasks: [Task; 2],
    idx: usize,
}

impl Scheduler {
    pub fn new() -> Self {
        extern "C" {
            static TASK1_INITIAL_SP: ();
            static TASK1_KERNEL_INITIAL_SP: ();
            static TASK2_INITIAL_SP: ();
            static TASK2_KERNEL_INITIAL_SP: ();
        }

        let task_context =
            Context::new(task1 as *const _, unsafe { &TASK1_INITIAL_SP } as *const _);
        let task1 = Task::new(unsafe { &TASK1_KERNEL_INITIAL_SP }, task_context);
        let task_context =
            Context::new(task2 as *const _, unsafe { &TASK2_INITIAL_SP } as *const _);
        let task2 = Task::new(unsafe { &TASK2_KERNEL_INITIAL_SP }, task_context);

        Self {
            tasks: [task1, task2],
            idx: 0,
        }
    }

    pub fn do_your_work(&mut self) -> *const Context {
        self.idx += 1;
        self.idx %= 2;
        unsafe { (self.tasks[self.idx].sp_el1 as *mut Context).sub(1) as *const _ }
    }

    pub fn start(&mut self) -> ! {
        extern "C" {
            fn scheduler_start(task: *const ()) -> !;
        }

        unsafe {
            scheduler_start((self.tasks[self.idx].sp_el1 as *mut Context).sub(1) as *const _);
        }
    }
}

#[derive(Debug)]
pub struct Task {
    /// Pointer to the task's kernel stack.
    sp_el1: *const (),
}

#[derive(Debug)]
#[repr(C)]
pub struct Context {
    pub x: [u64; 32],
    pub pc: *const (),
    pub sp: *const (),
    pub pstate: u64,
}

impl Task {
    pub fn new(sp_el1: *const (), context: Context) -> Self {
        unsafe {
            let sp_el1_context = (sp_el1 as *mut Context).sub(1);
            *sp_el1_context = context;
        }

        Self { sp_el1 }
    }
}

impl Context {
    pub fn new(initial_pc: *const (), initial_sp: *const ()) -> Self {
        Self {
            x: [0; 32],
            pc: initial_pc,
            pstate: 0,
            sp: initial_sp,
        }
    }
}

#[no_mangle]
fn task1() {
    log::trace!("task1 start");
    loop {
        log::trace!("task1");
        for _ in 0..1000000 {}
    }
}

#[no_mangle]
fn task2() {
    log::trace!("task2 start");
    loop {
        log::trace!("task2");
        for _ in 0..1000000 {}
    }
}
