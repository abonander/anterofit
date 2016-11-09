extern crate threadpool;

use self::threadpool::ThreadPool;

use super::{ExecBox, Executor};

#[derive(Clone)]
pub struct Pooled {
    pool: ThreadPool,
}

impl Pooled {
    pub fn new(num_threads: usize) -> Self {
        Pooled {
            pool: ThreadPool::new_with_name("anterofit_worker", num_threads)
        }
    }
}

impl Executor for Pooled {
    fn execute(&self, exec: Box<ExecBox>) {
        self.pool.execute(move || exec.exec());
    }
}