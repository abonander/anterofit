use futures::{Future, Complete, Oneshot, Async, Poll};
use ::{Result, Error};

use std::mem;

use error::RequestPanicked;

use super::request::RequestHead;

/// A handle representing a pending result to an executed request.
///
/// May be polled for its status (compatible with `futures`) or blocked on.
///
/// Depending on the stage of the request, this may return immediately.
#[must_use = "Result of request is unknown unless polled for"]
//#[derive(Debug)]
pub struct Call<T>(Call_<T>);

enum Call_<T> {
    Waiting(Oneshot<Result<T>>),
    Immediate(Result<T>),
    Taken
}

impl<T> Call<T> {
    /// Ignore the result of this call.
    ///
    /// Equivalent to `let _ = self` but more friendly for method-chaining.
    pub fn ignore(self) {}

    /// Ignore the result of this call, returning `Ok(())` so it can be used
    /// in a `try!()`.
    pub fn ignore_ok(self) -> Result<()> { Ok(()) }

    /// Block on this call until a result is available.
    pub fn wait(self) -> Result<T> {
        <Self as Future>::wait(self)
    }

    /// Poll this call for a result.
    ///
    /// Convenience method for those that don't want to take on the complexity of `futures`.
    ///
    /// Returns `None` in two cases:
    ///
    /// * The result is not ready yet
    /// * The result has already been taken
    pub fn check(&mut self) -> Option<Result<T>> {
        match self.poll() {
            Ok(Async::Ready(val)) => Some(Ok(val)),
            Ok(Async::NotReady) => None,
            Err(Error::ResultTaken) => None,
            Err(e) => Some(Err(e))
        }
    }

    /// Return `true` if a result is immediately available
    /// (a call to `check()` will return the result).
    pub fn is_immediate(&self) -> bool {
        if let Call_::Immediate(_) = self.0 {
            true
        } else {
            false
        }
    }

    /// Returns `true` if the result has already been taken.
    pub fn result_taken(&self) -> bool {
        if let Call_::Taken = self.0 {
            true
        } else {
            false
        }
    }
}

impl<T> Future for Call<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<T, Error> {
        match self.0 {
            Call_::Waiting(ref mut oneshot) => return poll_for_result(oneshot),
            Call_::Taken => return Err(Error::ResultTaken),
            _ => (),
        }

        if let Call_::Immediate(res) = mem::replace(&mut self.0, Call_::Taken) {
            res.map(Async::Ready)
        } else {
            unreachable!();
        }
    }
}

/// Implementation detail
pub fn oneshot<T>(head: Option<RequestHead>) -> (PanicGuard<T>, Call<T>) {
    let (tx, rx) = ::futures::oneshot();

    let guard = PanicGuard {
        head: head,
        tx: Some(tx)
    };

    (guard, Call(Call_::Waiting(rx)))
}

/// Implementation detail
pub fn immediate<T>(res: Result<T>) -> Call<T> {
    Call(Call_::Immediate(res))
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

fn poll_for_result<T>(oneshot: &mut Oneshot<Result<T>>) -> Poll<T, Error> {
    let ret = match try!(oneshot.poll()) {
        Async::Ready(val) => Async::Ready(try!(val)),
        Async::NotReady => Async::NotReady,
    };

    Ok(ret)
}