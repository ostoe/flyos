use core:: {future, pin::Pin};
use std::process::Output;
use alloc::boxed::Box;


pub mod simple_executor;

pub struct Task {
    future: Pin<Box<dyn future<Output = ()>>>,

}

impl Task {

    pub fn new(future: impl future::Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future)
        }
    }

}

impl future::Future for Task {
    fn poll(&mut self, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        self.future.as_mut().poll(cx)
    }
}