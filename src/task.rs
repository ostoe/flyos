use core:: {future, pin::Pin, sync::atomic::AtomicU64};
use alloc::boxed::Box;


pub mod simple_executor;
pub mod executor;
pub mod keyboard;
 
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, core::sync::atomic::Ordering::Relaxed))
    }
}

pub struct Task {
    future: Pin<Box<dyn future::Future<Output = ()>>>,
    id: TaskId,
}


impl Task {

    pub fn new(future: impl future::Future<Output = ()> + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Box::pin(future)
        }
    }

    fn poll(&mut self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        self.future.as_mut().poll(cx)
    }

    fn id(&self) -> TaskId {
        use core::ops::Deref;
        // let d = &**&self.future;
        // let b = Pin::deref(&self.future);
        // let a =  b as *const _;
        // let c = a as *const ();
        let addr = &**&self.future as *const _ as *const () as u64; // 下面也行的，意思差不多
        // let addr = Pin::deref(&self.future) as *const _ as *const () as u64;
        TaskId(addr)
    }

}

// impl future::Future for Task {
//     type Output = ();
//     fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
//         todo!();
//     }
// }