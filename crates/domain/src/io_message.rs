//! Locale-stable rendering of `std::io::Error`.
//!
//! `std::io::Error`'s `Display` on Windows carries the OS-localised message
//! text ("Не удается найти указанный файл."), which leaks the host locale into
//! otherwise-English CLI diagnostics. [`describe`] renders the same error as a
//! fixed English phrase plus the raw OS error number for support.

/// A fixed English description of an I/O error: an `ErrorKind` phrase and, when
/// present, the `os error N` code. Never the OS's own localised string.
pub fn describe(err: &std::io::Error) -> String {
    use std::io::ErrorKind::*;
    let phrase = match err.kind() {
        NotFound => "not found",
        PermissionDenied => "permission denied",
        AlreadyExists => "already exists",
        InvalidInput => "invalid input",
        InvalidData => "invalid data",
        TimedOut => "timed out",
        WriteZero => "write returned zero",
        Interrupted => "interrupted",
        UnexpectedEof => "unexpected end of file",
        Unsupported => "unsupported operation",
        OutOfMemory => "out of memory",
        _ => "I/O error",
    };
    match err.raw_os_error() {
        Some(code) => format!("{phrase} (os error {code})"),
        None => phrase.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::describe;

    #[test]
    fn describe_is_english_and_carries_the_os_code() {
        let e = std::io::Error::new(std::io::ErrorKind::NotFound, "irrelevant inner text");
        assert_eq!(describe(&e), "not found");

        // A real filesystem miss carries a raw OS code and still reads English.
        let miss = std::fs::read_to_string("definitely-no-such-file-xyzzy").unwrap_err();
        let msg = describe(&miss);
        assert!(msg.starts_with("not found"), "got {msg:?}");
        assert!(msg.contains("os error"), "got {msg:?}");
        assert!(msg.is_ascii(), "message must be ASCII/English, got {msg:?}");
    }
}
