use core::task::{Waker, Context};

use alloc::{collections::{BTreeMap, VecDeque}, sync::Arc, task::Wake};
use crossbeam_queue::ArrayQueue;

use crate::{interrupts, println};

use super::{Task, TaskId};



/// 满足Wake trait，一个Task一个TaskWaker
struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }

    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task queue full");
    }
}

impl Wake for TaskWaker {
    // 拥有所有权，消费waker
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
    // [可选实现] 引用，性能更好，不消费 waker.
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
     /// 在executor和waker之间共享,只储存taskid，不储存task实例
    task_id_queue: Arc<ArrayQueue<TaskId>>,
    /// 1. 重复利用多次唤醒  2. 确保引用记述不会在中断时释放，不如可能会导致deadlocks
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Executor {
        Executor {
            tasks: BTreeMap::new(),
            task_id_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }

    pub fn spwan(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task_id, task).is_some() {
            panic!("task with the same TaskID in the tasks.")
        }
        self.task_id_queue.push(task_id).expect("task queue full.")
    }

    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }
    /// 并不是没有队列就不执行了，定时中断还是会执行到的
    pub fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_id_queue,
            waker_cache,
        } = self;
        while let Some(task_id) = task_id_queue.pop() {
            if let Some(t) = tasks.get_mut(&task_id) {
                let waker = waker_cache.entry(task_id).or_insert(TaskWaker::new(task_id, task_id_queue.clone()));
                let mut cx = Context::from_waker(&waker);
                match t.poll(&mut cx) {
                    core::task::Poll::Ready(()) => {
                        // task has done.
                        tasks.remove(&task_id);
                        waker_cache.remove(&task_id);
                    },
                    core::task::Poll::Pending => {},
                }
            } else {
                continue;
            }
        }// 无队列 返回 else {
    }

    fn sleep_if_idle(&self) {
        x86_64::instructions::interrupts::disable();
        if self.task_id_queue.is_empty() {
            /// 中断可以在这里发生，这个时候就G了，添加上上句可解决 
            /// 设置了以后，像当于定时中断还是会唤醒看一下的
            x86_64::instructions::interrupts::enable_and_hlt();
        } else {
            x86_64::instructions::interrupts::enable();
        }
    }
}
