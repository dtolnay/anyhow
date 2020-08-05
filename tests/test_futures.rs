#[cfg(feature = "futures")]
#[test]
pub fn test_future_context() {
    use futures::executor::block_on;
    use anyhow::{Result, anyhow, futures::AsyncContext};

    let result: Result<()> = Err(anyhow!("oh no"));
    let fut = futures::future::ready(result).context("context");
    match block_on(fut) {
        Ok(_) => panic!("test failed"),
        Err(_) => {} // test passes
    }
}
