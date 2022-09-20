use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc};
use core::task::Waker;
use crossbeam_queue::ArrayQueue;
use core::task::{Context, Poll};
use alloc::task::Wake;
pub struct Executor {
    //We use a task_queue of task IDs and a BTreeMap named tasks that contains the actual Task instances. 
    tasks: BTreeMap<TaskId, Task>,
    //ArrayQueue of task IDs, wrapped into the Arc type that implements reference counting
    //Using Arc<ArrayQueue>> type because it will be shared between the executor and wakers.
    task_queue: Arc<ArrayQueue<TaskId>>,
    // This map caches the Waker of a task after its creation.
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new() -> Self {
        Executor {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(100)),
            waker_cache: BTreeMap::new(),
        }
    }
}

impl Executor {
    pub fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("task with same ID already in tasks");
        }
        self.task_queue.push(task_id).expect("queue full");
    }

    fn run_ready_tasks(&mut self) {
        // destructure `self` to avoid borrow checker errors
        let Self {
            tasks,
            task_queue,
            waker_cache,
        } = self;

        while let Ok(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            //create waker is it doesn't exist yet
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it and its cached waker
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
    //with waker in place
    pub fn run(&mut self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();   // new
        }
    }
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};
        //avoid the race condition that interrupts happen right between is_empty check 
        //and the call to hlt
        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        }
        else {
            interrupts::enable();
        }
    }
}



//The job of the waker is to push the ID of the woken task to the task_queue of the executor
struct TaskWaker {
    task_id: TaskId,
    //Since the ownership of the task_queue is shared between the executor and wakers, we use the Arc 
    //wrapper type to implement shared reference-counted ownership.
    task_queue: Arc<ArrayQueue<TaskId>>,
}

impl TaskWaker {
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}

impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
