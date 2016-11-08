use futures::{Future, Oneshot};
use ::{Result as OurResult, Error};

#[must_use = "Response is being ignored"]
pub struct Call<T>(Oneshot<OurResult<T>>);

impl<T> Call<T> {
    pub fn ignore(self) {}

    pub fn wait(self) -> Result<T, Error> {}
}

impl<T> Future for Call<T> {
    type Item = T;
    type Error = Error;

    fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
        try!(self.0.poll())
    }
}

pub fn from_oneshot(oneshot: Oneshot<OurResult<T>>) -> Call<T> {
    Call
}