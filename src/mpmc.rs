use crossbeam::sync::SegQueue;
use parking_lot::{Condvar, Mutex};

use std::iter::IntoIterator;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use executor::ExecBox;

pub fn channel() -> (Sender, Receiver) {
    let inner = Arc::new(Inner {
        queue: SegQueue::new(),
        mutex: Mutex::new(()),
        cvar: Condvar::new(),
        closed: AtomicBool::new(false),
    });

    let inner_ = inner.clone();

    (Sender(inner), Receiver(inner_))
}

pub struct Sender(Arc<Inner>);

/// The receiver half of an MPMC queue of executor jobs.
///
/// Poll with `recv()`, when it returns `None` the job queue is closed.
pub struct Receiver(Arc<Inner>);

struct Inner {
    queue: SegQueue<Box<dyn ExecBox>>,
    mutex: Mutex<()>,
    cvar: Condvar,
    closed: AtomicBool,
}

impl Sender {
    pub fn send(&self, exec: Box<dyn ExecBox>) {
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
        // RFC: should this have a timeout?
        self.0.cvar.wait(&mut self.0.mutex.lock());
    }

    /// Poll the queue, blocking if it is empty.
    ///
    /// Returns `None` when the sending half of the queue is closed.
    pub fn recv(&self) -> Option<Box<dyn ExecBox>> {
        loop {
            if let Some(val) = self.0.queue.try_pop() {
                // Wake another thread so it can check if there's more work in the queue
                self.0.cvar.notify_one();
                return Some(val);
            }

            if self.0.closed.load(Ordering::Acquire) {
                // Wake any remaining blocked threads so they can observe the closed status
                self.0.cvar.notify_all();
                return None;
            }

            self.wait();
        }
    }

    /// Get a blocking iterator that yields `None` when the queue is closed.
    ///
    /// `IntoIter` is also implemented for `&Receiver`.
    pub fn iter(&self) -> RecvIter {
        RecvIter(self)
    }
}

impl Clone for Receiver {
    fn clone(&self) -> Self {
        Receiver(self.0.clone())
    }
}

impl<'a> IntoIterator for &'a Receiver {
    type Item = Box<dyn ExecBox>;
    type IntoIter = RecvIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl IntoIterator for Receiver {
    type Item = Box<dyn ExecBox>;
    type IntoIter = RecvIntoIter;

    fn into_iter(self) -> Self::IntoIter {
        RecvIntoIter(self)
    }
}

/// Blocking owned iterator type.
pub struct RecvIntoIter(Receiver);

impl Iterator for RecvIntoIter {
    type Item = Box<dyn ExecBox>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.recv()
    }
}

/// Blocking shared iterator type.
pub struct RecvIter<'a>(&'a Receiver);

impl<'a> Iterator for RecvIter<'a> {
    type Item = Box<dyn ExecBox>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.recv()
    }
}
