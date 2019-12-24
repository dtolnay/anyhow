use crate::chain::Chain;
use crate::error::ErrorImpl;
use core::fmt::{self, Debug, Write};

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
            write!(f, "\n\nCaused by:")?;
            let multiple = cause.source().is_some();
            for (n, error) in Chain::new(cause).enumerate() {
                writeln!(f)?;
                write!(
                    if multiple {
                        Indented::numbered(f, n)
                    } else {
                        Indented::new(f)
                    },
                    "{}",
                    error
                )?;
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
                write!(f, "\n\n{}", backtrace)?;
            }
        }

        Ok(())
    }
}

struct Indented<'a, D> {
    inner: &'a mut D,
    ind: Option<usize>,
    started: bool,
}

impl<'a, D> Indented<'a, D> {
    fn numbered(inner: &'a mut D, ind: usize) -> Self {
        Self {
            inner,
            ind: Some(ind),
            started: false,
        }
    }

    fn new(inner: &'a mut D) -> Self {
        Self {
            inner,
            ind: None,
            started: false,
        }
    }
}

impl<T> fmt::Write for Indented<'_, T>
where
    T: fmt::Write,
{
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for (ind, mut line) in s.split('\n').enumerate() {
            if !self.started {
                // trim first line to ensure it lines up with the number nicely
                line = line.trim();
                // Don't render the first line unless its actually got text on it
                if line.is_empty() {
                    continue;
                }

                self.started = true;
                match self.ind {
                    Some(ind) => self.inner.write_fmt(format_args!("{: >5}: ", ind))?,
                    None => self.inner.write_fmt(format_args!("    "))?,
                }
            } else if ind > 0 {
                self.inner.write_char('\n')?;
                if self.ind.is_some() {
                    self.inner.write_fmt(format_args!("       "))?;
                } else {
                    self.inner.write_fmt(format_args!("    "))?;
                }
            }

            self.inner.write_fmt(format_args!("{}", line))?;
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
        let expected = "    2: verify\n       this";
        let mut output = String::new();

        Indented {
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
        let expected = "   12: verify\n       this";
        let mut output = String::new();

        Indented {
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

        Indented {
            inner: &mut output,
            ind: None,
            started: false,
        }
        .write_str(input)
        .unwrap();

        assert_eq!(expected, output);
    }
}
