use core::{future::Future, pin::Pin};
use alloc::boxed::Box;
use core::task::{Context, Poll};
use core::sync::atomic::{AtomicU64, Ordering};
pub mod simple_executor;
pub mod keyboard;
pub mod executor;
pub struct Task {
    future: Pin<Box<dyn Future<Output = ()>>>,
    id: TaskId, 
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + 'static ) -> Task{
        Task { 
            //pin the future in memory using Box::pin 
            future:Box::pin(future),
            //The new id field makes it possible to uniquely name a task, which is required for waking a specific task.
            id: TaskId::new(),
         }
    }

    //add poll method to allow the executor to poll the stored future
    fn poll(&mut self, context:&mut Context) -> Poll<()>{
        self.future.as_mut().poll(context)
    }
}


//The first step in creating an executor with proper support for waker notifications 
//is to give each task a unique ID. 
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct TaskId(u64);


impl TaskId {
    fn new() -> Self {
        //The function uses a static NEXT_ID variable of type AtomicU64 to ensure that each ID is assigned only once. 
        //The fetch_add method atomically increases the value and returns the previous value in one atomic operation.
        //The Ordering parameter defines whether the compiler is allowed to reorder the fetch_add operation in the instructions stream.
        //Since we only require that ID be unique, the Relaxed ordering with the weakest requirements
        //is enough. 
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}