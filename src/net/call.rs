use futures::{Future, Oneshot, Canceled, Async};
use ::{Result, Error};

use std::mem;

#[must_use = "Response is being ignored"]
pub enum Call<T> {
    Waiting(Oneshot<Result<T>>),
    Immediate(Result<T>),
    Taken
}

impl<T> Call<T> {
    pub fn ignore(self) {}

    pub fn block(self) -> Result<T> {
        self.wait()
    }

    pub fn check(&mut self) -> Option<Result<T>> {

    }
}

impl<T> Future for Call<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        match *self {
            Call::Waiting(ref mut oneshot) => return try!(oneshot.poll()),
            Call::Taken => return Err(Error::ResultTaken),
        }

        if let Call::Immediate(res) = mem::replace(&mut self, Call::Taken) {
            Ok(Async::Ready(res))
        } else {
            unreachable!();
        }
    }
}

pub fn from_oneshot(oneshot: Oneshot<OurResult<T>>) -> Call<T> {
    Call
}

pub fn immediate(res: OurResult<T>) -> Call<T> {

}

fn poll_to_option(poll: Poll<T, Canceled>) -> Option<T> {
    if let Ok(async) = poll {
        match async {
            Async::Ready(val) => Some(val),
            Async::NotReady => None,
        }
    } else {
        None
    }
}