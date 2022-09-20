use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static ) -> Task{
        Task { 
            //pin the future in memory using Box::pin 
            future:Box::pin(future),
         }
    }

    //add poll method to allow the executor to poll the stored future
    fn poll(&mut self, context:&mut Context) -> Poll<()>{
        self.future.as_mut().poll(context)
    }
}

