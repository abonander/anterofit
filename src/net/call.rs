use futures::{Future, Canceled, Complete, Oneshot, Async, Poll};
use futures::executor::{self, Unpark, Spawn};
use ::{Result, Error};

use std::mem;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use error::RequestPanicked;

use super::request::RequestHead;

/// A handle representing a pending result to an executed request.
///
/// May be polled for its status (compatible with `futures`) or blocked on.
///
/// Depending on the stage of the request, this may return immediately.
#[must_use = "Result of request is unknown unless polled for"]
//#[derive(Debug)]
pub struct Call<T> {
    state: CallState<T>,
    notify: Arc<Notify>,
}

enum CallState<T> {
    Waiting(CallFuture<T>),
    Immediate(Result<T>),
    Taken
}

type CallFuture<T> = Spawn<Oneshot<Result<T>>>;

impl<T> Call<T> {
    /// Ignore the result of this call.
    ///
    /// Equivalent to `let _ = self` but more friendly for method-chaining.
    pub fn ignore(self) {}

    /// Ignore the result of this call, returning `Ok(())` so it can be used
    /// in a `try!()`.
    pub fn ignore_ok(self) -> Result<()> { Ok(()) }

    /// Block on this call until a result is available.
    ///
    /// Depending on the stage of the request, this may return immediately.
    /// Call `is_immediate()` check for this if you want.
    pub fn block(self) -> Result<T> {
        self.wait()
    }

    /// Poll this call for a result.
    ///
    /// Convenience wrapper for `poll_no_task()` which doesn't use types from `futures`.
    ///
    /// Returns `None` in two cases:
    ///
    /// * The result is not ready yet
    /// * The result has already been taken
    pub fn check(&mut self) -> Option<Result<T>> {
        match self.poll_no_task() {
            Ok(Async::Ready(val)) => Some(Ok(val)),
            Ok(Async::NotReady) | Err(Error::ResultTaken) => None,
            Err(e) => Some(Err(e))
        }
    }

    /// Return `true` if a result is available
    /// (a call to `check()` will return the result).
    pub fn is_available(&self) -> bool {
        if let CallState::Immediate(_) = self.state {
            true
        } else {
            self.notify.check()
        }
    }

    /// Returns `true` if the result has already been taken.
    pub fn result_taken(&self) -> bool {
        if let CallState::Taken = self.state {
            true
        } else {
            false
        }
    }

    /// Poll the inner future without requiring a task.
    ///
    /// You can call `is_available()` to check readiness.
    pub fn poll_no_task(&mut self) -> Poll<T, Error> {
        let notify = self.notify.clone();
        self.poll_by(move |fut| fut.poll_future(notify))
    }

    fn poll_by<F>(&mut self, poll: F) -> Poll<T, Error>
    where F: FnOnce(&mut CallFuture<T>) -> Poll<Result<T>, Canceled> {
        match self.state {
            CallState::Waiting(ref mut future) => return map_poll(poll(future)),
            CallState::Taken => return Err(Error::ResultTaken),
            _ => (),
        }

        if let CallState::Immediate(res) = mem::replace(&mut self.state, CallState::Taken) {
            res.map(Async::Ready)
        } else {
            unreachable!();
        }
    }
}

impl<T> Future for Call<T> {
    type Item = T;
    type Error = Error;

    /// ### Panics
    /// If the current thread is not running a futures task.
    ///
    /// Use `poll_no_task()` instead if you want to poll outside of a task.
    fn poll(&mut self) -> Poll<T, Error> {
        self.poll_by(|fut| fut.get_mut().poll())
    }
}

#[derive(Default)]
struct Notify(AtomicBool);

impl Notify {
    fn check(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

impl Unpark for Notify {
    fn unpark(&self) {
        self.0.store(true, Ordering::Relaxed);
    }
}

/// Implementation detail
pub fn oneshot<T>(head: Option<RequestHead>) -> (PanicGuard<T>, Call<T>) {
    let (tx, rx) = ::futures::oneshot();

    let guard = PanicGuard {
        head: head,
        tx: Some(tx)
    };

    (guard, Call {
        state: CallState::Waiting(executor::spawn(rx)),
        notify: Default::default(),
    })
}

/// Implementation detail
pub fn immediate<T>(res: Result<T>) -> Call<T> {
    Call {
        state: CallState::Immediate(res),
        notify: Default::default(),
    }
}

/// Sends the request head on panic.
pub struct PanicGuard<T> {
    head: Option<RequestHead>,
    tx: Option<Complete<Result<T>>>,
}

impl<T> PanicGuard<T> {
    /// Get a mutable reference to the request head.
    pub fn head_mut(&mut self) -> &mut RequestHead {
        self.head.as_mut().expect("PanicGuard::head was None")
    }

    /// Send a result, which will prevent the head being sent on-panic.
    pub fn complete(&mut self, res: Result<T>) {
        if let Some(tx) = self.tx.take() {
            tx.complete(res);
        }
    }
}

impl<T> Drop for PanicGuard<T> {
    fn drop(&mut self) {
        if let Some(head) = self.head.take() {
            self.complete(Err(RequestPanicked(head).into()));
        }
    }
}

fn map_poll<T>(poll: Poll<Result<T>, Canceled>) -> Poll<T, Error> {
    let ret = match try!(poll) {
        Async::Ready(val) => Async::Ready(try!(val)),
        Async::NotReady => Async::NotReady,
    };

    Ok(ret)
}
