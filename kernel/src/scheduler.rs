use crate::task::{Context, Task};

pub struct Scheduler {
    tasks: [Task; 2],
    current_index: usize,
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
            current_index: 0,
        }
    }

    pub fn schedule(&mut self) -> &Task {
        self.current_index += 1;
        self.current_index %= 4;

        &self.tasks[self.current_index >> 1]
    }

    pub fn start(&mut self) -> ! {
        self.tasks[self.current_index >> 1].start();
    }
}

fn task1() {
    log::trace!("task1 start");

    loop {
        log::trace!("task1");
        for _ in 0..500000 {}
    }
}

fn task2() {
    log::trace!("task2 start");

    let mut x = 0;
    loop {
        log::trace!("task2");
        for _ in 0..1000000 {}
        x += 1;
        if x > 10 {
            unsafe { core::arch::asm!("svc #0") }
        }
    }
}
