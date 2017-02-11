use crossbeam::sync::SegQueue;
use parking_lot::{Condvar, Mutex};

use std::iter::IntoIterator;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use executor::ExecBox;

pub fn channel() -> (Sender, Receiver) {
    Arc::new(
        Inner {
            queue: SegQueue::new(),
            mutex: Mutex::new(()),
            cvar: Condvar,
            closed: AtomicBool::new(false)
        }
    )
}

pub struct Sender(Arc<Inner>);

pub struct Receiver(Arc<Inner>);

struct Inner {
    queue: SegQueue<Box<ExecBox>>,
    mutex: Mutex<()>,
    cvar: Condvar,
    closed: AtomicBool,
}

impl Sender {
    pub fn send(&self, exec: Box<ExecBox>) {
        self.0.queue.push(exec);
        self.0.cvar.notify_all();
    }
}

impl Drop for Sender {
    fn drop(&mut self) {
        self.0.closed.store(true, Ordering::Release);
        self.0.cvar.notify_all();
    }
}

impl Receiver {
    fn wait(&self) {
        self.0.cvar.wait(&mut self.0.mutex.lock());
    }

    pub fn recv(&self) -> Option<Box<ExecBox>> {
        loop {
            if let Some(val) = self.0.queue.try_pop() {
                return Some(val);
            }
        }
    }
}