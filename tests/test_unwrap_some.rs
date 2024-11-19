use anyhow::unwrap_some;

#[test]
fn test_some() -> anyhow::Result<()> {
    let maybe_foo: Option<i32> = Some(42);
    let foo = unwrap_some!(maybe_foo);
    assert_eq!(foo, 42);
    Ok(())
}

#[test]
fn test_none() {
    struct Foo {
        maybe: Option<i32>,
    }

    let foo = Foo { maybe: None };
    let result: anyhow::Result<_> = (|| Ok(unwrap_some!(foo.maybe)))();
    assert_eq!(result.unwrap_err().to_string(), "`foo.maybe` is `None`");
}
