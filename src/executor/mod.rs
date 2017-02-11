//! Types which can take a boxed closure and execute it, preferably in the background.

use std::sync::Arc;

#[cfg(feature = "pool")]
mod pool;

mod single;

#[cfg(feature = "pool")]
pub use self::pool::Pooled;

pub use self::single::SingleThread;

pub use mpmc::Queue;

/// The default executor which should be suitable for most use-cases.
pub type DefaultExecutor = SingleThread;

/// A trait describing a type which can execute tasks (in the background or otherwise).
///
/// It is up to the implementing type to decide how to handle panics.
pub trait Executor: Send + Clone + 'static {
    /// Execute `exec` on this executor.
    ///
    /// This may or may not block the current thread, but documenting this behavior is preferable.
    fn execute(&self, exec: Box<ExecBox>);
}

/// An executor which executes all tasks immediately on the current thread (blocking).
///
/// Does not allocate or spawn threads.
///
/// Panics are allowed to unwind back into calling code.
#[derive(Clone)]
pub struct Blocking;

impl Executor for Blocking {
    /// ## Blocks
    /// Executes `exec` on the current thread, blocking until it returns.
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
