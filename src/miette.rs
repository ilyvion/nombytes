use crate::NomBytes;
use miette::SourceCode;

impl SourceCode for NomBytes {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<std::boxed::Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        self.to_str()
            .read_span(span, context_lines_before, context_lines_after)
    }
}
