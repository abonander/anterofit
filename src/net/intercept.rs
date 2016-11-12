use super::RequestHead;

pub trait Interceptor: Send + Sync + 'static {
    fn intercept(&self, req: &mut RequestHead);
}

impl Interceptor for () {
    fn intercept(&self, _req: &mut RequestHead) {}
}

pub struct Chain<I1, I2>(I1, I2);

impl<I1, I2> Chain<I1, I2> {
    pub fn new(one: I1, two: I2) -> Self {
        Chain(one, two)
    }
}

impl<I1: Interceptor, I2: Interceptor> Interceptor for Chain<I1, I2> {
    fn intercept(&self, req: &mut RequestHead) {
        self.0.intercept(req);
        self.1.intercept(req);
    }
}