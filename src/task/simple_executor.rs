use core::task::{RawWaker, Waker, RawWakerVTable, Context, Poll};

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

    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            let waker = dymmy_worker();
            let mut context = Context::from_waker(&waker);
            match task.poll(&mut context) {
                Poll::Pending => {
                    self.task_queue.push_back(task);
                },
                Poll::Ready(()) => {}
            }
        }
    }
}

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }

    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(0 as *const (), vtable)
}

fn dymmy_worker() -> Waker {
    unsafe {
        Waker::from_raw(dummy_raw_waker())
    }
}