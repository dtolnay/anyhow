use std::fmt::{Display, Formatter};
use std::io;
use std::sync::Arc;

#[derive(Debug)]
struct SharedError(Arc<anyhow::Error>);

impl<E: anyhow::IntoError> From<E> for SharedError {
    fn from(value: E) -> Self {
        SharedError(Arc::new(value.into_error()))
    }
}

impl From<SharedError> for anyhow::Error {
    fn from(value: SharedError) -> Self {
        #[derive(Debug)]
        struct SharedErrorAsError(SharedError);

        impl Display for SharedErrorAsError {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                Display::fmt(&self.0 .0, f)
            }
        }
        impl std::error::Error for SharedErrorAsError {}

        anyhow::Error::new(SharedErrorAsError(value))
    }
}

#[test]
fn test_into_error() {
    let e1 = SharedError::from(anyhow::anyhow!("test"));
    let e2 = SharedError::from(io::Error::new(io::ErrorKind::Other, "test"));
    let _e1: anyhow::Error = anyhow::Error::from(e1);
    let _e2: anyhow::Error = anyhow::Error::from(e2);
}
