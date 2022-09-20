use super::Task;
use alloc::collections::VecDeque;
use core::task::RawWakerVTable;
use core::task::{Waker, RawWaker};

use core::task::{Context, Poll};
pub struct SimpleExecutor {
    task_queue: VecDeque<Task>,
}
// The idea behind using this type is that we insert new tasks through the spawn method at 
//the end and pop the next task for execution from the front. 
impl SimpleExecutor {
    pub fn new() -> SimpleExecutor {
        SimpleExecutor {
            task_queue: VecDeque::new(),
        }
    }

    pub fn spawn(&mut self, task: Task) {
        self.task_queue.push_back(task)
    }
    pub fn run(&mut self) {
        while let Some(mut task) = self.task_queue.pop_front() {
            //For each task, it first creates a Context type by wrapping a Waker
            // instance returned by our dummy_waker function. 
            let waker = dummy_waker();
            let mut context = Context::from_waker(&waker);
            //Then it invokes the Task::poll method with this context
            match task.poll(&mut context) {
                Poll::Ready(()) => {} // task done
                Poll::Pending => self.task_queue.push_back(task),
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

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}