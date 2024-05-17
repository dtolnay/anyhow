use anyhow::{ensure, Result};

fn main() -> Result<()> {
    ensure!("...");

    struct Struct(bool);
    let mut s = Struct(true);
    match &mut s {
        Struct(cond) => ensure!(cond),
    }

    Ok(())
}
