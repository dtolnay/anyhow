use anyhow::{bail, Result};
use std::io;

fn bail_literal() -> Result<()> {
    bail!("oh no!");
}

fn bail_fmt() -> Result<()> {
    bail!("{} {}!", "oh", "no");
}

fn bail_error() -> Result<()> {
    bail!(io::Error::new(io::ErrorKind::Other, "oh no!"));
}

#[test]
fn test_messages() {
    assert_eq!("oh no!", bail_literal().unwrap_err().to_string());
    assert_eq!("oh no!", bail_fmt().unwrap_err().to_string());
    assert_eq!("oh no!", bail_error().unwrap_err().to_string());
}

#[test]
fn test_downcast() {
    assert_eq!("oh no!", bail_literal().unwrap_err().downcast::<&str>().unwrap());
    assert_eq!("oh no!", bail_fmt().unwrap_err().downcast::<String>().unwrap());
    assert_eq!("oh no!", bail_error().unwrap_err().downcast::<io::Error>().unwrap().to_string());
}
