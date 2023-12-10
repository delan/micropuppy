use core::{arch::asm, fmt::Write, num::NonZeroUsize};

use crate::{
    a53::{nzcv::NZCV, spsr::SPSR_EL1},
    reg::system::Register as SystemRegister,
};

pub struct Scheduler {
    tasks: [Task; 2],
    current: Option<NonZeroUsize>,
    next: Option<NonZeroUsize>,
}

#[derive(Debug)]
pub struct Task {
    program_counter: u64,
    stack_pointer: u64,
    pstate: u64,
}

impl Scheduler {
    pub fn new() -> Self {
        let tasks = [
            Task::new(task1 as *const _, unsafe { &TASK1_INITIAL_SP as *const _ }),
            Task::new(task2 as *const _, unsafe { &TASK2_INITIAL_SP as *const _ }),
        ];
        Self {
            tasks,
            current: None,
            next: NonZeroUsize::new(1),
        }
    }

    pub fn current_task(&self) -> Option<NonZeroUsize> {
        self.current
    }

    pub fn get(&self, task: NonZeroUsize) -> Option<&Task> {
        self.tasks.get(task.get() - 1)
    }

    fn get_mut(&mut self, task: NonZeroUsize) -> Option<&mut Task> {
        self.tasks.get_mut(task.get() - 1)
    }

    pub fn save(&mut self) {
        let task = self.current.take().expect("No current task!");
        let task = self.get_mut(task).expect("No such task!");
        task.program_counter = read_special_reg!("ELR_EL1");
        task.stack_pointer = read_special_reg!("SP_EL0");
        task.pstate = read_special_reg!("SPSR_EL1");
    }

    pub fn run(&mut self) -> ! {
        if let Some(task) = self.current {
            panic!("Task {} is still running!", task);
        }
        let task = self.next.take().expect("No next task!");
        self.current = Some(task);
        self.next = NonZeroUsize::new(task.get() % self.tasks.len() + 1);
        let task = self.get(task).expect("No such task!");

        write_special_reg!("ELR_EL1", task.program_counter);
        write_special_reg!("SPSR_EL1", task.pstate);
        write_special_reg!("SP_EL0", task.stack_pointer);
        write_special_reg!("SPSel", 0);
        unsafe {
            asm!("eret");
        }
        unreachable!();
    }
}

impl Task {
    fn new(initial_pc: *const (), initial_sp: *const ()) -> Self {
        Self {
            program_counter: initial_pc as u64,
            stack_pointer: initial_sp as u64,
            pstate: 0,
        }
    }
}

extern "C" {
    static TASK1_INITIAL_SP: ();
    static TASK2_INITIAL_SP: ();
}

#[no_mangle]
fn task1() {
    log::trace!("task1");
    loop {
        // for _ in 0..1000000 {}
        // log::trace!("task1");
    }
}

#[no_mangle]
fn task2() {
    log::trace!("task2");
    loop {
        // for _ in 0..1000000 {}
        // log::trace!("task2");
    }
}
