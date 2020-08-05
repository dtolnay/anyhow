#[cfg(feature = "std")]
#[test]
pub fn test_future_context() {
    use anyhow::{anyhow, futures::AsyncContext, Result};
    use futures::executor::block_on;

    let result: Result<()> = Err(anyhow!("oh no"));
    let fut = futures::future::ready(result).context("context");
    match block_on(fut) {
        Ok(_) => panic!("test failed"),
        Err(_) => {} // test passes
    }
}
