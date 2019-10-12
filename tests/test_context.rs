use anyhow::{Context, Result};

// https://github.com/dtolnay/anyhow/issues/18
#[test]
fn test_inference() -> Result<()> {
    let x = "1";
    let y: u32 = x.parse().context("...")?;
    assert_eq!(y, 1);
    Ok(())
}
