#[derive(Debug, Clone)]
pub(crate) struct Location {
    file: &'static str,
    line: u32,
    column: u32,
}

impl Location {
    #[track_caller]
    pub(crate) fn capture() -> Self {
        let loc = core::panic::Location::caller();
        Self {
            file: loc.file(),
            line: loc.line(),
            column: loc.column(),
        }
    }
}

impl core::fmt::Display for Location {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}:{}:{}", self.file, self.line, self.column)
    }
}
