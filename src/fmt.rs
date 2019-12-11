use crate::chain::Chain;
use crate::error::ErrorImpl;
use core::fmt::{self, Debug};
use std::fmt::Write;

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

        if f.alternate() {
            return Debug::fmt(error, f);
        }

        write!(f, "{}", error)?;

        if let Some(cause) = error.source() {
            writeln!(f, "\n\nCaused by:")?;
            let multiple = cause.source().is_some();
            for (n, error) in Chain::new(cause).enumerate() {
                let mut f2 = Numbered {
                    inner: &mut *f,
                    ind: Some(n).filter(|_| multiple),
                    started: false,
                };
                write!(f2, "{}", error)?;
                writeln!(f)?;
            }
        }

        #[cfg(backtrace)]
        {
            use std::backtrace::BacktraceStatus;

            let backtrace = self.backtrace();
            if let BacktraceStatus::Captured = backtrace.status() {
                let mut backtrace = backtrace.to_string();
                if backtrace.starts_with("stack backtrace:") {
                    // Capitalize to match "Caused by:"
                    backtrace.replace_range(0..1, "S");
                }
                backtrace.truncate(backtrace.trim_end().len());
                write!(f, "\n{}", backtrace)?;
            }
        }

        Ok(())
    }
}

struct Numbered<D> {
    inner: D,
    ind: Option<usize>,
    started: bool,
}

impl<T> fmt::Write for Numbered<T>
where
    T: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for line in s.split_terminator('\n') {
            match (self.ind.as_ref(), self.started) {
                // Numbered Cases
                (Some(ind), false) => {
                    self.started = true;
                    self.inner.write_fmt(format_args!("{: >4}: ", ind))?;
                }
                (Some(_), true) => self.inner.write_fmt(format_args!("\n      "))?,
                // Unnumbered Cases
                (None, false) => {
                    self.started = true;
                    self.inner.write_fmt(format_args!("    "))?;
                }
                _ => self.inner.write_fmt(format_args!("\n    "))?,
            }

            self.inner.write_str(line)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_digit() {
        let input = "verify\nthis";
        let expected = "   2: verify\n      this";
        let mut output = String::new();

        Numbered {
            inner: &mut output,
            ind: Some(2),
            started: false,
        }
        .write_str(input)
        .unwrap();

        assert_eq!(expected, output);
    }

    #[test]
    fn two_digits() {
        let input = "verify\nthis";
        let expected = "  12: verify\n      this";
        let mut output = String::new();

        Numbered {
            inner: &mut output,
            ind: Some(12),
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

        Numbered {
            inner: &mut output,
            ind: None,
            started: false,
        }
        .write_str(input)
        .unwrap();

        assert_eq!(expected, output);
    }
}
