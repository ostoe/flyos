use core::task::{RawWaker, Waker, RawWakerVTable};

use alloc::collections::VecDeque;

use super::Task;

pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}

impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor { task_queue: VecDeque::new() }
    }

    pub fn spwan(&mut self, task: Task) {
        self.task_queue.push_back(task);
    }
}

fn dummy_raw_waker() -> RawWaker {
    todo!()
}

fn dymmy_worker() -> Waker {
    unsafe {
        Waker::from_raw(dummy_raw_waker())
    }
}