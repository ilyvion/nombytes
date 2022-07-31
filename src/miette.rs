use crate::NomBytes;
use miette::SourceCode;
use nom::AsBytes;

impl SourceCode for NomBytes {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<std::boxed::Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        let bytes = self.as_bytes();
        bytes.read_span(span, context_lines_before, context_lines_after)
    }
}

#[cfg(test)]
mod tests {
    use crate::NomBytes;
    use bytes::Bytes;
    use miette::{Diagnostic, SourceSpan};
    use std::error::Error;
    use std::fmt::Display;

    #[derive(Debug, Diagnostic)]
    struct ErrorWithSpan {
        #[source_code]
        src: NomBytes,

        #[label = "span"]
        err_span: SourceSpan,
    }
    impl Display for ErrorWithSpan {
        fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
            write!(f, "Error")
        }
    }
    impl Error for ErrorWithSpan {}

    #[test]
    fn works_with_source_code_and_span() {
        let src = NomBytes::new(Bytes::from("Hello, world!"));
        let error = ErrorWithSpan {
            src,
            err_span: (0, 5).into(),
        };
        let report: miette::Result<()> = Err(error.into());
        let output = format!("{report:?}");

        // Let's simply check that the generated error output at the very least
        // contains the text from the span, which suggests that the `read_span`
        // worked like it should.
        assert!(output.contains("Hello"), "{output}");
    }
}
