/// Compiles a host of things depending on the verb used:
/// - `selector` compiles a `scraper::Selector`
/// - `regex` compiles a `regex::Regex`
///
/// # Usage
/// ```
/// let regex = compile!{ regex ".*" };
/// let selector = compile!{ selector "html" }
/// ```
macro_rules! compile {
    {selector $s: literal} => {
        scraper::Selector::parse($s).map_err(|e| crate::extractor::ExtractionError::SelectorParseError(e))
    };
    {regex $s: literal } => {
        regex::Regex::new($s).map_err(|e| crate::parser::ParseError::RegexParseError(e))
    };
}
