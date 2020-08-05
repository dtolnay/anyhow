use crate::{Context, Error};
use std::future::Future;
use std::fmt::Display;
use std::pin::Pin;

pub trait AsyncContext<T, E> {
    fn context<C>(self, context: C) -> Pin<Box<dyn Future<Output = Result<T, Error>>>>
    where
        C: Display + Send + Sync + 'static;

    fn with_context<C, F>(self, f: F) -> Pin<Box<dyn Future<Output = Result<T, Error>>>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C + 'static;
}

impl<T: 'static, E, I: Context<T, E>, Fut: Future<Output = I> + 'static> AsyncContext<T, E>
    for Fut
{
    fn context<C>(self, context: C) -> Pin<Box<dyn Future<Output = Result<T, Error>>>>
    where
        C: Display + Send + Sync + 'static,
    {
        Box::pin(async { self.await.context(context) })
    }

    fn with_context<C, F>(self, f: F) -> Pin<Box<dyn Future<Output = Result<T, Error>>>>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C + 'static,
    {
        Box::pin(async { self.await.with_context(f) })
    }
}
