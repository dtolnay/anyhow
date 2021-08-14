#[derive(Debug)]
struct OhNoError;
impl std::fmt::Display for OhNoError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("oh no")
    }
}
impl std::error::Error for OhNoError {}

fn produce_anyhow_error() -> anyhow::Error {
    anyhow::Error::new(OhNoError).context("context 1")
}

#[allow(clippy::vtable_address_comparisons)]
#[test]
fn test_into_error_source() {
    let anyhow_error = produce_anyhow_error();
    let std_error = produce_anyhow_error().into_error();

    let mut anyhow_source_chain = anyhow_error.chain();

    let mut current_std_error = &std_error as &dyn std::error::Error;
    while let Some(std_source) = current_std_error.source() {
        let anyhow_source = anyhow_source_chain
            .next()
            .expect("std source chain is longer than anyhow one");

        // Strip vtable by casting to *const (). Vtable may be different for two pointers even if
        // they point at the same memory.
        assert_eq!(
            anyhow_source as *const _ as *const (),
            std_source as *const _ as *const (),
        );

        current_std_error = std_source;
    }

    assert!(
        anyhow_source_chain.next().is_none(),
        "anyhow source chain is longer than std one",
    );
}

#[test]
fn test_into_error_display() {
    let anyhow_error = produce_anyhow_error();
    let std_error = produce_anyhow_error().into_error();

    assert_eq!(format!("{}", anyhow_error), format!("{}", std_error));

    // We cannot test Debug because the backtraces mismatch
    // assert_eq!(format!("{:?}", anyhow_error), format!("{:?}", std_error));
}
