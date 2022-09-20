use super::Task;
use alloc::collections::VecDeque;
use core::task::RawWakerVTable;
use core::task::{Waker, RawWaker};
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