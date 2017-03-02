//! Executors using background threads

use std::thread::{self, Builder};

use super::{Executor, Receiver};

/// An executor which uses multiple threads to complete jobs.
#[derive(Debug)]
pub struct MultiThread {
    threads: usize,
}

impl MultiThread {
    /// Create a new multithreaded executor using the given thread count.
    ///
    /// The background threads will not be spawned until `Executor::start()` is called.
    pub fn new(threads: usize) -> Self {
        MultiThread {
            threads: threads
        }
    }
}

impl Executor for MultiThread {
    /// Spawn new worker threads to complete jobs. The threads will be named such that they
    /// can easily be associated with Anterofit.
    ///
    /// If a panic occurs on a worker thread, it will be restarted under the same name.
    ///
    /// ## Panics
    /// If a worker thread failed to spawn.
    fn start(self, recv: Receiver) {
        for thread in 0 .. self.threads {
            spawn_thread(thread, recv.clone());
        }
    }
}

/// An executor which uses a single thread to complete jobs.
#[derive(Debug, Default)]
pub struct SingleThread(());

impl SingleThread {
    /// Create a new single-threaded executor.
    ///
    /// The background thread will not be spawned until `Executor::start()` is called.
    pub fn new() -> Self {
        SingleThread(())
    }
}

impl Executor for SingleThread {
    /// Spawn a new worker thread to complete jobs. The thread will be named such that it
    /// can easily be associated with Anterofit.
    ///
    /// If a panic occurs on the worker thread, it will be restarted under the same name.
    ///
    /// ## Panics
    /// If the worker thread failed to spawn.
    fn start(self, recv: Receiver) {
        spawn_thread(0, recv);
    }
}

struct Sentinel {
    thread: usize,
    recv: Receiver
}

impl Drop for Sentinel {
    fn drop(&mut self) {
        if thread::panicking() {
            spawn_thread(self.thread, self.recv.clone());
        }
    }
}

fn spawn_thread(thread: usize, recv: Receiver) {
    let sentinel = Sentinel {
        thread: thread,
        recv: recv
    };

        let _ = Builder::new()
        .name(format!("anterofit_worker_{}", thread))
        .spawn(move ||
            for exec in &sentinel.recv {
                exec.exec();
            }
        )
        .expect("Failed to spawn Anterofit worker thread");
}