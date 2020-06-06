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

#[test]
fn test_custom_hook() {
    let expected = "hook is set!";
    anyhow::set_hook(Box::new(move |_error| {
        Box::new(CustomHandler { msg: expected })
    }))
    .unwrap();

    let report = anyhow::anyhow!("heres the message!");
    let actual = format!("{:?}", report);

    assert_eq!(expected, actual);
}

#[test]
fn test_mutable_hook() {
    let fake_expected = "hook is set!";
    let real_expected = "the context was modified!";

    anyhow::set_hook(Box::new(move |_error| {
        Box::new(CustomHandler { msg: fake_expected })
    }))
    .unwrap();

    let mut report = anyhow::anyhow!("heres the message!");
    report.handler_mut().downcast_mut::<CustomHandler>().msg = real_expected;
    let actual = format!("{:?}", report);

    assert_eq!(real_expected, actual);
}
