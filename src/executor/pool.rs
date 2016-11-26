extern crate threadpool;

use self::threadpool::ThreadPool;

use super::{ExecBox, Executor};

/// An executor wrapped around a thread pool which can execute multiple tasks concurrently.
///
/// Worker threads are automatically restarted when panics occur; subsequent jobs are not
/// affected.
#[derive(Clone)]
pub struct Pooled {
    pool: ThreadPool,
}

impl Pooled {
    /// Spawn a new thread pool with `num_threads` number of threads.
    ///
    /// The threads will be named such that they can be easily identified as workers for this crate.
    pub fn new(num_threads: usize) -> Self {
        Pooled {
            pool: ThreadPool::new_with_name("anterofit_worker".into(), num_threads)
        }
    }
}

impl Executor for Pooled {
    fn execute(&self, exec: Box<ExecBox>) {
        self.pool.execute(move || exec.exec());
    }
}