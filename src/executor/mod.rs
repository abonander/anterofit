
#[cfg(feature = "pool")]
mod pool;

mod single;

#[cfg(feature = "pool")]
pub use self::pool::Pooled;

pub use self::single::SingleThread;

pub type DefaultExecutor = SingleThread;

pub trait Executor: Send + Clone + 'static {
    fn execute(&self, exec: Box<ExecBox>);
}

#[derive(Clone)]
pub struct SyncExecutor;

impl Executor for SyncExecutor {
    fn execute(&self, exec: Box<ExecBox>) {
        exec.exec();
    }
}

pub trait ExecBox: Send + 'static {
    fn exec(self: Box<Self>);
}

impl ExecBox {
    /// Create a new `ExecBox` which does nothing when called.
    ///
    /// Since it is zero-sized, this call should not allocate.
    pub fn noop() -> Box<Self> {
        Box::new(|| {})
    }
}

impl<F> ExecBox for F where F: FnOnce() + Send + 'static {
    fn exec(self: Box<Self>) {
        (*self)()
    }
}