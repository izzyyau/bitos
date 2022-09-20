use conquer_once::spin::OnceCell;
use crossbeam_queue::ArrayQueue;
use crate::println;

use core::{pin::Pin, task::{Poll, Context}};
use futures_util::stream::Stream;
use futures_util::task::AtomicWaker;


use futures_util::stream::StreamExt;
use pc_keyboard::{layouts, DecodedKey, HandleControl, Keyboard, ScancodeSet1};
use crate::print;

//Using the ArrayQueue type, we can now create a global scancode queue in a new task::keyboard module:
static SCANCODE_QUEUE: OnceCell<ArrayQueue<u8>> = OnceCell::uninit();

//To wake the stored Waker, we add a call to WAKER.wake() in the add_scancode function:
pub(crate) fn add_scancode(scancode: u8) {
    if let Ok(queue) = SCANCODE_QUEUE.try_get() {
        if let Err(_) = queue.push(scancode) {
            println!("WARNING: scancode queue full; dropping keyboard input");
        } else {
            // we call wake only after pushing to the queue because otherwise the task might
            // be woken too early while the queue is still empty.
            WAKER.wake(); // new
        }
    } else {
        println!("WARNING: scancode queue uninitialized");
    }
}

pub struct ScancodeStream {
    _private: (),
}

impl ScancodeStream {
    pub fn new() -> Self {
        SCANCODE_QUEUE.try_init_once(|| ArrayQueue::new(100))
            .expect("ScancodeStream::new should only be called once");
        ScancodeStream { _private: () }
    }
}


impl Stream for ScancodeStream {
    type Item = u8;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<u8>> {
        //We first use the OnceCell::try_get method to get a reference to the initialized scancode queue. 
        let queue = SCANCODE_QUEUE
            .try_get()
            .expect("scancode queue not initialized");

        // Next, we use the ArrayQueue::pop method to try to get the next element from the queue. 
        if let Ok(scancode) = queue.pop() {
            //If it succeeds, we return the scancode wrapped in Poll::Ready(Some(…))
            return Poll::Ready(Some(scancode));
        }

        //In order to avoid race condition caused by interrupr handler, we register the 
        //Waker static before the second check.
        WAKER.register(&cx.waker());
        match queue.pop() {
            Ok(scancode) => {
                //The task has been finished, remove the waker associated with it.
                WAKER.take();
                Poll::Ready(Some(scancode))
            }
            Err(crossbeam_queue::PopError) => Poll::Pending,
        }
    }
}


static WAKER: AtomicWaker = AtomicWaker::new();

//we first create a new Scancode stream and then repeatedly use the next method provided by the StreamExt trait to get a Future that resolves to the next element in the stream. By using the await operator on it, 
//we asynchronously wait for the result of the future.
pub async fn print_keypresses() {
    let mut scancodes = ScancodeStream::new();
    let mut keyboard = Keyboard::new(layouts::Us104Key, ScancodeSet1,
        HandleControl::Ignore);

    while let Some(scancode) = scancodes.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}