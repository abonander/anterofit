use futures::{Future, Oneshot, Async, Poll};
use ::{Result, Error};

use std::mem;

#[must_use = "Response is being ignored"]
//#[derive(Debug)]
pub struct Call<T>(Call_<T>);

enum Call_<T> {
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
        match self.poll() {
            Ok(Async::Ready(val)) => Some(Ok(val)),
            Ok(Async::NotReady) => None,
            Err(Error::ResultTaken) => None,
            Err(e) => Some(Err(e))
        }
    }

    pub fn is_immediate(&self) -> bool {
        if let Call_::Immediate(_) = self.0 {
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
            unreachable!()
        }
    }
}

pub fn from_oneshot<T>(oneshot: Oneshot<Result<T>>) -> Call<T> {
    Call(Call_::Waiting(oneshot))
}

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