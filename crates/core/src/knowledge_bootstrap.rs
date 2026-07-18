//! Knowledge-store facade wiring: the real [`UrlFetcher`](cronus_domain::knowledge_ingest::UrlFetcher)
//! implementation. The domain tier's `UrlFetcher` seam stays I/O-free
//! (`cronus-domain` cannot touch the network, §4.1/§4.3); this crate is where
//! the raw socket lives, matching the `model_bridge`/`loop_bootstrap`
//! facade-wiring precedent.

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

use cronus_domain::knowledge_ingest::UrlFetcher;

const IO_TIMEOUT: Duration = Duration::from_secs(30);

/// A one-shot HTTP/1.1 GET fetcher (KB-5 URL source ingestion), reusing the
/// `model-local` transport's own conventions (`FrameReader`-style: send the
/// request, read the status line, read the body until the server closes the
/// connection — no `Content-Length`/chunked parsing needed, since the request
/// always sends `Connection: close`).
///
/// **Disclosed scope:** `http://` only. `https://` (TLS), `robots.txt`
/// compliance, and rate-limiting (l2-knowledge-store §5.3) are deferred,
/// separately-scoped follow-ups — this proves the fetch mechanics are real
/// against a hermetic local server, not simulated.
#[derive(Debug, Default)]
pub struct HttpUrlFetcher;

impl UrlFetcher for HttpUrlFetcher {
    fn fetch(&self, url: &str) -> Result<String, String> {
        let (host, port, path) = parse_http_url(url)?;
        let mut stream = TcpStream::connect((host.as_str(), port))
            .map_err(|e| format!("connect to {host}:{port} failed: {e}"))?;
        stream
            .set_read_timeout(Some(IO_TIMEOUT))
            .map_err(|e| e.to_string())?;
        stream
            .set_write_timeout(Some(IO_TIMEOUT))
            .map_err(|e| e.to_string())?;

        let request = format!(
            "GET {path} HTTP/1.1\r\nHost: {host}\r\nUser-Agent: cronus-knowledge-store/1\r\nAccept: text/html,text/plain\r\nConnection: close\r\n\r\n"
        );
        stream
            .write_all(request.as_bytes())
            .map_err(|e| format!("write failed: {e}"))?;

        let mut raw = Vec::new();
        stream
            .read_to_end(&mut raw)
            .map_err(|e| format!("read failed: {e}"))?;

        let response = String::from_utf8_lossy(&raw);
        let (head, body) = response
            .split_once("\r\n\r\n")
            .ok_or_else(|| format!("malformed response from {url}: no header terminator"))?;
        let status_line = head
            .lines()
            .next()
            .ok_or_else(|| format!("empty response from {url}"))?;
        let status: u16 = status_line
            .split_whitespace()
            .nth(1)
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| format!("unparseable status line from {url}: {status_line}"))?;
        if !(200..300).contains(&status) {
            return Err(format!("HTTP {status} fetching {url}"));
        }
        Ok(body.to_string())
    }
}

/// Parse an `http://host[:port]/path` URL into its connection parts.
/// `https://` is refused with a clear message rather than silently attempting
/// a plaintext connection to a TLS port (never a confusing hang or garbled
/// read).
fn parse_http_url(url: &str) -> Result<(String, u16, String), String> {
    let rest = url.strip_prefix("http://").ok_or_else(|| {
        if url.starts_with("https://") {
            "https:// URLs are not yet supported (TLS is a deferred follow-up)".to_string()
        } else {
            format!("unsupported URL scheme: {url}")
        }
    })?;
    let (authority, path) = match rest.find('/') {
        Some(i) => (&rest[..i], rest[i..].to_string()),
        None => (rest, "/".to_string()),
    };
    let (host, port) = match authority.rsplit_once(':') {
        Some((h, p)) => (
            h.to_string(),
            p.parse::<u16>().map_err(|_| format!("bad port in {url}"))?,
        ),
        None => (authority.to_string(), 80u16),
    };
    if host.is_empty() {
        return Err(format!("missing host in {url}"));
    }
    Ok((host, port, path))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader};
    use std::net::{Shutdown, TcpListener};
    use std::thread;

    /// Spawn a one-shot hermetic local HTTP server that drains one request
    /// and replies with the fixed `response`, then closes. Returns the base
    /// URL to fetch.
    fn spawn_mock_server(response: &'static str) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
        let addr = listener.local_addr().expect("local_addr");
        thread::spawn(move || {
            if let Ok((stream, _)) = listener.accept() {
                let mut reader = BufReader::new(&stream);
                loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line) {
                        Ok(0) => break,
                        Ok(_) if line == "\r\n" || line == "\n" => break,
                        Ok(_) => continue,
                        Err(_) => break,
                    }
                }
                let mut stream = stream;
                let _ = stream.write_all(response.as_bytes());
                let _ = stream.shutdown(Shutdown::Write);
            }
        });
        format!("http://{addr}")
    }

    #[test]
    fn fetches_a_real_response_over_a_hermetic_local_server() {
        let base = spawn_mock_server(
            "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nConnection: close\r\n\r\n<html><body><p>Hello world.</p></body></html>",
        );
        let body = HttpUrlFetcher.fetch(&base).expect("fetch");
        assert!(body.contains("Hello world."));
    }

    #[test]
    fn a_non_2xx_status_is_reported_as_an_error() {
        let base = spawn_mock_server("HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\nnope");
        let err = HttpUrlFetcher.fetch(&base).expect_err("404 must error");
        assert!(err.contains("404"));
    }

    #[test]
    fn a_connection_refused_is_a_clear_error_not_a_panic() {
        // Port 0 never accepts connections — a deterministic refusal target.
        let err = HttpUrlFetcher
            .fetch("http://127.0.0.1:1")
            .expect_err("nothing listens on port 1");
        assert!(err.contains("connect"));
    }

    #[test]
    fn https_urls_are_refused_with_a_clear_message() {
        let err = HttpUrlFetcher
            .fetch("https://example.test/")
            .expect_err("https refused");
        assert!(err.contains("TLS"));
    }

    #[test]
    fn parse_http_url_extracts_host_port_and_path() {
        assert_eq!(
            parse_http_url("http://example.test/a/b").unwrap(),
            ("example.test".to_string(), 80, "/a/b".to_string())
        );
        assert_eq!(
            parse_http_url("http://localhost:8080").unwrap(),
            ("localhost".to_string(), 8080, "/".to_string())
        );
    }
}
