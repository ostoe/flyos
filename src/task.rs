use core:: {future, pin::Pin};
use alloc::boxed::Box;


pub mod simple_executor;
pub mod keyboard;
 
pub struct Task {
    future: Pin<Box<dyn future::Future<Output = ()>>>,

}

impl Task {

    pub fn new(future: impl future::Future<Output = ()> + 'static) -> Task {
        Task {
            future: Box::pin(future)
        }
    }

    fn poll(&mut self, cx: &mut core::task::Context<'_>) -> core::task::Poll<()> {
        self.future.as_mut().poll(cx)
    }

}

impl future::Future for Task {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut core::task::Context<'_>) -> core::task::Poll<Self::Output> {
        todo!();
    }
}