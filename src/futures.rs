use futures::future::{Future, ready};
use futures_util::future::FutureExt;
use std::pin::Pin;
use std::fmt::Display;
use crate::{Context, Error};

pub trait AsyncContext<T, E> {
    fn context<C>(self, context: C) -> Pin<Box<dyn Future<Output=Result<T, Error>>>>
        where
            C: Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Pin<Box<dyn Future<Output=Result<T, Error>>>>
        where
            C: Display + Send + Sync + 'static,
            F: FnOnce() -> C + 'static;
}
impl<T: 'static, E, I: Context<T, E>, Fut: Future<Output=I> + 'static> AsyncContext<T, E> for Fut {
    fn context<C>(self, context: C) -> Pin<Box<dyn Future<Output=Result<T, Error>>>>
        where
            C: Display + Send + Sync + 'static {
        Box::pin(self.then(|result| ready(result.context(context))))
    }

    fn with_context<C, F>(self, f: F) -> Pin<Box<dyn Future<Output=Result<T, Error>>>>
        where
            C: Display + Send + Sync + 'static,
            F: FnOnce() -> C + 'static {
        Box::pin(self.then(|result| ready(result.with_context(f))))
    }
}
