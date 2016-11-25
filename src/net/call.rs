use futures::{Future, Oneshot, Async, Poll};
use ::{Result, Error};

use std::mem;

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
pub fn from_oneshot<T>(oneshot: Oneshot<Result<T>>) -> Call<T> {
    Call(Call_::Waiting(oneshot))
}

/// Implementation detail
pub fn immediate<T>(res: Result<T>) -> Call<T> {
    Call(Call_::Immediate(res))
}

fn poll_for_result<T>(oneshot: &mut Oneshot<Result<T>>) -> Poll<T, Error> {
    let ret = match try!(oneshot.poll()) {
        Async::Ready(val) => Async::Ready(try!(val)),
        Async::NotReady => Async::NotReady,
    };

    Ok(ret)
}