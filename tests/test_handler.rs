struct CustomHandler {
    msg: &'static str,
}

impl anyhow::ReportHandler for CustomHandler {
    fn report(
        &self,
        _error: &(dyn std::error::Error + 'static),
        f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        write!(f, "{}", self.msg)
    }
}

static EXPECTED: &str = "hook is set!";

#[test]
fn test_custom_hook() {
    let _ = anyhow::set_hook(Box::new(move |_error| {
        Box::new(CustomHandler { msg: EXPECTED })
    }));

    let report = anyhow::anyhow!("heres the message!");
    let actual = format!("{:?}", report);

    assert_eq!(EXPECTED, actual);
}

#[test]
fn test_mutable_hook() {
    let real_expected = "the context was modified!";

    let _ = anyhow::set_hook(Box::new(move |_error| {
        Box::new(CustomHandler { msg: EXPECTED })
    }));

    let mut report = anyhow::anyhow!("heres the message!");
    report
        .handler_mut()
        .downcast_mut::<CustomHandler>()
        .unwrap()
        .msg = real_expected;
    let actual = format!("{:?}", report);

    assert_eq!(real_expected, actual);
}
