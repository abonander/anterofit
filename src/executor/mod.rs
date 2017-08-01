//! Types which can take a boxed closure and execute it, preferably in the background.

#![cfg_attr(feature="clippy", allow(boxed_local))]

pub mod threaded;

pub use mpmc::{Receiver, RecvIter, RecvIntoIter};

/// The default executor which should be suitable for most use-cases.
pub type DefaultExecutor = threaded::SingleThread;

/// A trait describing a type which can execute tasks (in the background or otherwise).
///
/// Invoking `ExecBox` *may* panic, so the executor should
pub trait Executor {
    /// Initialize the executor, polling `recv` for jobs.
    ///
    /// When `Receiver::recv()` returns `None`, the job queue is closed and the executor can quit.
    fn start(self, recv: Receiver);
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
