use clap::ValueEnum;

#[derive(Clone, Copy, Debug, PartialEq, ValueEnum)]
pub enum OutputFormat {
    Text,
    Json,
}

pub struct Context {
    pub format: OutputFormat,
}

impl Context {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    pub fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }
}

/// Escape a string for embedding inside a double-quoted JSON string literal.
///
/// The installation half answers directly rather than through the shared
/// `Outcome` renderer (no other surface projects the `Installation` locus), so
/// its handlers assemble small JSON objects by hand. They must still route every
/// interpolated value through this — an unescaped Windows path (`C:\Users\…`)
/// or a `"` in a name otherwise produces output no JSON parser accepts.
pub(crate) fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::json_escape;

    #[test]
    fn json_escape_makes_windows_paths_and_quotes_safe() {
        assert_eq!(json_escape(r"C:\Users\a\b.json"), r"C:\\Users\\a\\b.json");
        assert_eq!(json_escape(r#"a "quote" here"#), r#"a \"quote\" here"#);
        assert_eq!(json_escape("line\nbreak"), "line\\nbreak");
        // Round-trips through a strict parser as the original string.
        let s = "tab\there \"q\" C:\\x";
        let json = format!("\"{}\"", json_escape(s));
        assert!(json.starts_with('"') && json.ends_with('"'));
    }
}
