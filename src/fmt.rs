use crate::chain::Chain;
use crate::error::ErrorImpl;
use core::fmt::{self, Write};

impl ErrorImpl<()> {
    pub(crate) fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.error())?;

        if f.alternate() {
            for cause in self.chain().skip(1) {
                write!(f, ": {}", cause)?;
            }
        }

        Ok(())
    }

    pub(crate) fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error = self.error();
        let handler = self.handler();

        handler.debug(error, f)
    }
}

impl crate::ReportHandler for crate::DefaultHandler {
    #[cfg(backtrace)]
    fn backtrace<'a>(
        &'a self,
        error: &'a (dyn crate::StdError + 'static),
    ) -> &'a std::backtrace::Backtrace {
        error
            .backtrace()
            .or_else(|| self.backtrace.as_ref())
            .expect("backtrace must have been captured")
    }

    fn debug(
        &self,
        error: &(dyn crate::StdError + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        write!(f, "{}", error)?;

        if let Some(cause) = error.source() {
            write!(f, "\n\nCaused by:")?;
            let multiple = cause.source().is_some();
            for (n, error) in Chain::new(cause).enumerate() {
                writeln!(f)?;
                let mut indented = Indented {
                    inner: f,
                    number: if multiple { Some(n) } else { None },
                    started: false,
                };
                write!(indented, "{}", error)?;
            }
        }

        #[cfg(backtrace)]
        {
            use std::backtrace::BacktraceStatus;

            let backtrace = self.backtrace(error);
            if let BacktraceStatus::Captured = backtrace.status() {
                let mut backtrace = backtrace.to_string();
                write!(f, "\n\n")?;
                if backtrace.starts_with("stack backtrace:") {
                    // Capitalize to match "Caused by:"
                    backtrace.replace_range(0..1, "S");
                } else {
                    // "stack backtrace:" prefix was removed in
                    // https://github.com/rust-lang/backtrace-rs/pull/286
                    writeln!(f, "Stack backtrace:")?;
                }
                backtrace.truncate(backtrace.trim_end().len());
                write!(f, "{}", backtrace)?;
            }
        }

        Ok(())
    }
}

struct Indented<'a, D> {
    inner: &'a mut D,
    number: Option<usize>,
    started: bool,
}

impl<T> Write for Indented<'_, T>
where
    T: Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (i, line) in s.split('\n').enumerate() {
            if !self.started {
                self.started = true;
                match self.number {
                    Some(number) => write!(self.inner, "{: >5}: ", number)?,
                    None => self.inner.write_str("    ")?,
                }
            } else if i > 0 {
                self.inner.write_char('\n')?;
                if self.number.is_some() {
                    self.inner.write_str("       ")?;
                } else {
                    self.inner.write_str("    ")?;
                }
            }

            self.inner.write_str(line)?;
        }

        Ok(())
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn one_digit() {
        let input = "verify\nthis";
        let expected = "    2: verify\n       this";
        let mut output = String::new();

        Indented {
            inner: &mut output,
            number: Some(2),
            started: false,
        }
        .write_str(input)
        .unwrap();

        assert_eq!(expected, output);
    }

    #[test]
    fn two_digits() {
        let input = "verify\nthis";
        let expected = "   12: verify\n       this";
        let mut output = String::new();

        Indented {
            inner: &mut output,
            number: Some(12),
            started: false,
        }
        .write_str(input)
        .unwrap();

        assert_eq!(expected, output);
    }

    #[test]
    fn no_digits() {
        let input = "verify\nthis";
        let expected = "    verify\n    this";
        let mut output = String::new();

        Indented {
            inner: &mut output,
            number: None,
            started: false,
        }
        .write_str(input)
        .unwrap();

        assert_eq!(expected, output);
    }
}
