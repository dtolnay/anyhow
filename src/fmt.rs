use crate::error::ErrorImpl;
use std::fmt::{self, Debug};

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
        if f.alternate() {
            return Debug::fmt(self.error(), f);
        }

        writeln!(f, "{}", self.error())?;

        let mut chain = self.chain().skip(1).enumerate().peekable();
        if let Some((n, error)) = chain.next() {
            write!(f, "\nCaused by:\n    ")?;
            if chain.peek().is_some() {
                write!(f, "{}: ", n)?;
            }
            writeln!(f, "{}", error)?;
            for (n, error) in chain {
                writeln!(f, "    {}: {}", n, error)?;
            }
        }

        #[cfg(backtrace)]
        {
            use std::backtrace::BacktraceStatus;

            let backtrace = self.backtrace();
            match backtrace.status() {
                BacktraceStatus::Captured => {
                    let mut backtrace = backtrace.to_string();
                    if backtrace.starts_with("stack backtrace:") {
                        // Capitalize to match "Caused by:"
                        backtrace.replace_range(0..1, "S");
                    }
                    write!(f, "\n{}", backtrace)?;
                }
                BacktraceStatus::Disabled => {
                    writeln!(
                        f,
                        "\nStack backtrace:\n    Run with RUST_LIB_BACKTRACE=1 env variable to display a backtrace"
                    )?;
                }
                _ => {}
            }
        }

        Ok(())
    }
}
