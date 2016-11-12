
#[cfg(feature = "pool")]
mod pool;

mod single;

#[cfg(feature = "pool")]
pub use self::pool::Pooled;

pub use self::single::SingleThread;

/// The default executor which should be suitable for most use-cases.
pub type DefaultExecutor = SingleThread;

/// A trait describing a type which can execute tasks (in the background or otherwise).
pub trait Executor: Send + Clone + 'static {
    fn execute(&self, exec: Box<ExecBox>);
}

/// An executor which executes all tasks immediately on the current thread (blocking).
///
/// Does not allocate or spawn threads.
#[derive(Clone)]
pub struct SyncExecutor;

impl Executor for SyncExecutor {
    fn execute(&self, exec: Box<ExecBox>) {
        exec.exec();
    }
}

/// A wrapper for `FnOnce() + Send + 'static` which can be invoked from a `Box`.
pub trait ExecBox: Send + 'static {
    /// Invoke the contained closure.
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