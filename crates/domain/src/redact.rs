//! Output/log redaction (SEC-5): scrub known secret values before rendering.

/// The mask substituted for any secret occurrence.
pub const MASK: &str = "***";

/// Replace every non-empty secret value in `text` with [`MASK`].
///
/// Apply to anything written to logs, the CLI/TUI, or error reports so secret
/// values never leak into rendered output.
pub fn redact(text: &str, secrets: &[&str]) -> String {
    let mut out = text.to_string();
    for secret in secrets {
        if !secret.is_empty() {
            out = out.replace(secret, MASK);
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn masks_known_secrets() {
        let line = "auth=Bearer sk-LIVE-12345 user=alice";
        let red = redact(line, &["sk-LIVE-12345"]);
        assert!(!red.contains("sk-LIVE-12345"));
        assert!(red.contains(MASK));
        assert!(red.contains("user=alice"), "non-secret content preserved");
    }

    #[test]
    fn empty_secret_is_ignored() {
        assert_eq!(redact("unchanged", &[""]), "unchanged");
    }
}
