/// Convert an error into `anyhow::Error`.
///
/// Trait can be used when another error type is used, and transparent conversion
/// from both standard error types and from `anyhow::Error` is needed.
///
/// # Example
///
/// ```
/// struct MyError(anyhow::Error);
///
/// impl<E: anyhow::IntoError> From<E> for MyError {
///    fn from(value: E) -> Self {
///       MyError(value.into_error())
///    }
/// }
///
/// impl From<MyError> for anyhow::Error {
///     fn from(value: MyError) -> Self {
///         value.0
///     }
/// }
///
/// fn returns_anyhow_result() -> anyhow::Result<()> {
/// # Ok(())
/// }
///
/// fn returns_io_result() -> std::io::Result<()> {
/// # Ok(())
/// }
///
/// fn returns_my_result() -> Result<(), MyError> {
/// # Ok(())
/// }
///
/// fn foo() -> Result<(), MyError> {
///     returns_anyhow_result()?;
///     returns_io_result()?;
///     returns_my_result()?;
///     Ok(())
/// }
///
/// fn bar() -> anyhow::Result<()> {
///    foo()?;
///    Ok(())
/// }
/// ```
pub trait IntoError {
    /// Convert an error into `anyhow::Error`.
    fn into_error(self) -> crate::Error;
}

impl IntoError for crate::Error {
    #[inline]
    fn into_error(self) -> crate::Error {
        self
    }
}

#[cfg(feature = "std")]
impl<E: std::error::Error + Send + Sync + 'static> IntoError for E {
    #[inline]
    fn into_error(self) -> crate::Error {
        crate::Error::new(self)
    }
}
