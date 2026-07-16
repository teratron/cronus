//! End-to-end validation of the model-transport seam (Phase 17 T-17T01).
//!
//! Proves a previously-inert model-consuming path — context compaction, which
//! ships as `NoOpCompactor` returning a fixed placeholder — now produces a
//! real result through the WHOLE seam: facade (`TransportCompactor`) →
//! `contract::InferenceBackend` (a real `EndpointProfile`) → HTTP → a mock
//! provider. And that the no-backend path degrades to `Err` without panicking.
//!
//! Everything is driven through `cronus-core`'s public facade — no direct
//! dependency on the transport, contract, or nodus crates — which is the point:
//! a host wires a backend through the facade and the model surface comes alive.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread::{self, JoinHandle};

use cronus_core::TransportCompactor;
use cronus_core::context_mgmt::{Compactor, ContextEntry};
use cronus_core::model::{Capabilities, EndpointProfile, ProtocolFamily};

/// Spawn a one-shot mock HTTP/SSE server on loopback that streams `tokens`
/// back as OpenAI-shaped `data:` chunks then `[DONE]`. Returns its base URL
/// and the server thread's join handle.
fn spawn_mock_sse(tokens: &'static [&'static str]) -> (String, JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let url = format!(
        "http://127.0.0.1:{}",
        listener.local_addr().expect("addr").port()
    );
    let handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        // Drain the request up to the header terminator.
        let mut acc: Vec<u8> = Vec::new();
        let mut buf = [0u8; 4096];
        loop {
            if acc.windows(4).any(|w| w == b"\r\n\r\n") {
                break;
            }
            match stream.read(&mut buf) {
                Ok(0) | Err(_) => break,
                Ok(n) => acc.extend_from_slice(&buf[..n]),
            }
        }
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\n\r\n")
            .expect("write preamble");
        for t in tokens {
            // The token text here is plain ASCII, so it is JSON-safe inline.
            let payload = format!("{{\"choices\":[{{\"delta\":{{\"content\":\"{t}\"}}}}]}}");
            write!(stream, "data: {payload}\n\n").expect("write token");
        }
        stream.write_all(b"data: [DONE]\n\n").expect("write done");
        let _ = stream.flush();
    });
    (url, handle)
}

fn caps() -> Capabilities {
    Capabilities {
        streaming: true,
        ..Default::default()
    }
}

fn context() -> Vec<ContextEntry> {
    vec![
        ContextEntry::new("user", "wire up the model transport", 8),
        ContextEntry::new("assistant", "streaming + cancellation done", 7),
    ]
}

#[test]
fn compaction_produces_a_real_result_through_the_full_transport_seam() {
    let (url, server) = spawn_mock_sse(&["Summary: ", "transport wired end to end"]);

    // The full seam: facade compactor → InferenceBackend (real EndpointProfile)
    // → HTTP → mock provider.
    let profile = EndpointProfile::new(url, ProtocolFamily::OpenAiCompatible, caps())
        .expect("loopback profile");
    let compactor = TransportCompactor::new(profile, "mock-model");

    let summary = compactor
        .compact(&context(), 500)
        .expect("compaction succeeds");

    server.join().expect("server thread");
    assert_eq!(summary, "Summary: transport wired end to end");
    // Crucially NOT the inert NoOpCompactor placeholder.
    assert_ne!(summary, "[context compacted]");
}

#[test]
fn compaction_degrades_without_panic_when_no_backend_is_reachable() {
    // Bind then drop, so the port is provably not served; the transport fails
    // to reach it and the compactor returns Err — a graceful degrade the
    // caller can fall back from, never a panic.
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind loopback");
    let port = listener.local_addr().expect("addr").port();
    drop(listener);

    let profile = EndpointProfile::new(
        format!("http://127.0.0.1:{port}"),
        ProtocolFamily::OpenAiCompatible,
        caps(),
    )
    .expect("loopback profile");
    let compactor = TransportCompactor::new(profile, "mock-model");

    let result = compactor.compact(&context(), 500);
    assert!(
        result.is_err(),
        "an unreachable backend must degrade to Err, not a summary"
    );
}
